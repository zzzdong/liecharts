//! 共享的 IR 输出断言基础设施。
//!
//! 测试套件的核心思路（参考 echarts 的 JSON-option 数据驱动 + 结构化输出断言）：
//! 从 `site/examples/*.json` 读取 ECharts 兼容的 JSON option，渲染为 lievisual 的
//! 结构化场景 IR（`Vec<SceneNode>`），再对 IR 中的几何坐标、样式、元素数量做精确断言。
//! 这比解析 SVG 字符串更可靠（不受字体/文本排版/坐标格式化影响），也比像素截图更稳定。
//!
//! 注意：`common` 模块会被每个 `tests/*_output_test.rs` 文件独立编译，因此不同
//! 测试文件只会用到部分 helper。这里统一用 `#![allow(dead_code)]` 抑制"某测试文件
//! 未使用某个 helper"的跨文件编译警告，这是共享测试基建的标准做法。

#![allow(dead_code)]

use liecharts::option::ChartOption;
use liecharts::visual::{Color, SceneNode};
use lievisual::scene::Element;
use vello_cpu::kurbo::{Point, Rect, Shape};

/// 从 `site/examples/{name}.json` 读取并解析 ECharts 兼容 option。
pub fn option_from_example(name: &str) -> ChartOption {
    let path = format!("site/examples/{}.json", name);
    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("无法读取示例 JSON {}: {}", path, e));
    serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("无法解析示例 JSON {}: {}", name, e))
}

/// 用默认 echarts 主题渲染 option，得到结构化场景 IR。
pub fn render(name: &str, width: u32, height: u32) -> Vec<SceneNode> {
    let option = option_from_example(name);
    let chart = liecharts::chart::Chart::new(option, liecharts::theme::Theme::echarts(), width, height);
    chart
        .collect_visual_elements()
        .unwrap_or_else(|e| panic!("渲染 {} 失败: {:?}", name, e))
}

// ── 元素提取 ────────────────────────────────────────────────────────────

/// 扁平化遍历场景：递归展开 Group，产出所有叶子节点。
///
/// 同时保留节点的 z_index（用于区分网格/系列/轴/标签层级）。
pub fn flatten<'a>(nodes: &'a [SceneNode], out: &mut Vec<(&'a Element, i32)>) {
    for n in nodes {
        match &n.element {
            Element::Group { children } => flatten(children, out),
            other => out.push((other, n.z_index)),
        }
    }
}

/// 提取所有特定类型的元素。
pub fn elements_of<'a>(nodes: &'a [SceneNode], f: impl Fn(&Element) -> bool) -> Vec<(&'a Element, i32)> {
    let mut all = Vec::new();
    flatten(nodes, &mut all);
    all.into_iter().filter(|(e, _)| f(e)).collect()
}

/// 提取所有 Rect 元素（含 Group 内）。返回 (rect, style)。
pub fn rects<'a>(nodes: &'a [SceneNode]) -> Vec<(Rect, &'a liecharts::visual::FillStrokeStyle)> {
    elements_of(nodes, |e| matches!(e, Element::Rect { .. }))
        .into_iter()
        .map(|(e, _)| match e {
            Element::Rect { rect, style } => (*rect, style),
            _ => unreachable!(),
        })
        .collect()
}

/// 提取所有 Circle 元素。返回 (center, radius, style)。
pub fn circles<'a>(
    nodes: &'a [SceneNode],
) -> Vec<(Point, f64, &'a liecharts::visual::FillStrokeStyle)> {
    elements_of(nodes, |e| matches!(e, Element::Circle { .. }))
        .into_iter()
        .map(|(e, _)| match e {
            Element::Circle {
                center,
                radius,
                style,
            } => (*center, *radius, style),
            _ => unreachable!(),
        })
        .collect()
}

/// 提取所有 Path 元素。返回 (path, style, closed)。
pub fn paths<'a>(
    nodes: &'a [SceneNode],
) -> Vec<(&'a vello_cpu::kurbo::BezPath, &'a liecharts::visual::FillStrokeStyle, bool)> {
    elements_of(nodes, |e| matches!(e, Element::Path { .. }))
        .into_iter()
        .map(|(e, _)| match e {
            Element::Path { path, style, closed } => (path, style, *closed),
            _ => unreachable!(),
        })
        .collect()
}

/// 提取所有"填充色为实心色"的 Path（用于饼图扇区等实心区域）。
/// 返回 (边界框, 填充色)。
pub fn solid_filled_paths<'a>(
    nodes: &'a [SceneNode],
) -> Vec<(Rect, String)> {
    use liecharts::visual::Fill;
    paths(nodes)
        .into_iter()
        .filter_map(|(path, style, _)| {
            let color = style.fill.as_ref().and_then(|f| match f {
                Fill::Solid(c) => Some(solid_color(c)),
                _ => None,
            })?;
            let bb = path.bounding_box();
            Some((bb, color))
        })
        .collect()
}

/// 提取所有文本及其位置。返回 (text, x, y)。
pub fn texts<'a>(nodes: &'a [SceneNode]) -> Vec<(String, f64, f64)> {
    elements_of(nodes, |e| matches!(e, Element::Text { .. }))
        .into_iter()
        .map(|(e, _)| match e {
            Element::Text {
                spans, position, ..
            } => {
                let content: String = spans.iter().map(|s| s.text.clone()).collect();
                (content, position.x, position.y)
            }
            _ => unreachable!(),
        })
        .collect()
}

// ── 断言 helper ─────────────────────────────────────────────────────────

/// 断言所有给定点都在画布范围内（可容忍小误差）。
///
/// 这是最关键的基础断言，能捕获坐标轴刻度/网格线飞出画布的回归
/// （例如历史 bug：value 轴刻度位置被放大到上万像素）。
pub fn assert_all_points_in_canvas(points: &[(f64, f64)], w: f64, h: f64, tol: f64) {
    for (x, y) in points {
        assert!(
            x.is_finite() && y.is_finite(),
            "坐标必须为有限值，实际 ({}, {})",
            x,
            y
        );
        assert!(
            *x >= -tol && *x <= w + tol,
            "x={} 超出画布宽度 [0, {}]",
            x,
            w
        );
        assert!(
            *y >= -tol && *y <= h + tol,
            "y={} 超出画布高度 [0, {}]",
            y,
            h
        );
    }
}

/// 将 Color 归一化为 `#rrggbb`（忽略 alpha，便于与主题色对比）。
pub fn solid_color(c: &Color) -> String {
    Color::rgb(
        (c.r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.b.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
    .to_hex()
}
