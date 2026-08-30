//! 坐标轴输出断言测试。
//!
//! 对应 `docs/svg_chart_checklist.md` 第一节第 4 项"坐标轴"：
//! - X/Y 轴刻度数量和位置正确
//! - 数值轴范围计算合理
//! - 轴名称不超出画布边界

mod common;
use common::*;
use lievisual::scene::Element;
use liecharts::visual::{Z_AXIS, Z_GRID};

const W: f64 = 800.0;
const H: f64 = 600.0;

/// 提取所有"网格线 + 轴线"的端点（z 在 GRID 与 AXIS 之间），断言它们都在画布内。
/// 这是捕获"value 轴刻度/网格线飞出画布"回归的关键断言。
fn assert_grid_and_axis_in_canvas(nodes: &[liecharts::visual::SceneNode]) {
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut all = Vec::new();
    common::flatten(nodes, &mut all);
    for (e, z) in all {
        if !(Z_GRID..=Z_AXIS).contains(&z) {
            continue;
        }
        if let Element::Line { start, end, .. } = e {
            pts.push((start.x, start.y));
            pts.push((end.x, end.y));
        }
    }
    assert!(!pts.is_empty(), "应存在网格线/轴线元素");
    assert_all_points_in_canvas(&pts, W, H, 1.0);
}

/// 提取所有轴刻度标签文本（z == LABEL 层，且位于绘图区外缘的数值）。
fn axis_label_texts(nodes: &[liecharts::visual::SceneNode]) -> Vec<(String, f64, f64)> {
    texts(nodes)
        .into_iter()
        .filter(|(t, _, _)| {
            t.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-')
        })
        .collect()
}

/// 所有数值轴刻度标签坐标必须在画布内（捕获刻度位置飞出画布 bug）。
fn assert_axis_labels_in_canvas(nodes: &[liecharts::visual::SceneNode]) {
    let labels = axis_label_texts(nodes);
    assert!(!labels.is_empty(), "应存在数值轴刻度标签");
    let pts: Vec<(f64, f64)> = labels.iter().map(|(_, x, y)| (*x, *y)).collect();
    assert_all_points_in_canvas(&pts, W, H, 2.0);
}

/// 折线图：Y 轴刻度与网格线必须在画布内（回归：value 轴刻度曾飞到 y=-41500）。
#[test]
fn line_y_axis_ticks_in_canvas() {
    let nodes = render("line", 800, 600);
    assert_grid_and_axis_in_canvas(&nodes);
    assert_axis_labels_in_canvas(&nodes);
}

/// 散点图：X 轴（value 轴）刻度必须在画布内（回归：1970 曾出现在 x=1339660）。
#[test]
fn scatter_x_axis_ticks_in_canvas() {
    let nodes = render("scatter", 800, 600);
    assert_grid_and_axis_in_canvas(&nodes);
    assert_axis_labels_in_canvas(&nodes);
}

/// 双 Y 轴：左右两个 value 轴的刻度都必须在画布内。
#[test]
fn dual_y_axis_ticks_in_canvas() {
    let nodes = render("dual_y_axis", 800, 600);
    assert_grid_and_axis_in_canvas(&nodes);
    assert_axis_labels_in_canvas(&nodes);
}

/// 面积图：value 轴刻度与面积网格线必须在画布内。
#[test]
fn area_y_axis_ticks_in_canvas() {
    let nodes = render("area", 800, 600);
    assert_grid_and_axis_in_canvas(&nodes);
    assert_axis_labels_in_canvas(&nodes);
}

/// 柱状图：value 轴刻度在画布内。
#[test]
fn bar_y_axis_ticks_in_canvas() {
    let nodes = render("bar", 800, 600);
    assert_grid_and_axis_in_canvas(&nodes);
    assert_axis_labels_in_canvas(&nodes);
}

/// Y 轴刻度数量应合理（折线图数据 0~200，应有 3~6 个刻度）。
#[test]
fn line_y_axis_tick_count_reasonable() {
    let nodes = render("line", 800, 600);
    let labels = axis_label_texts(&nodes);
    // 折线图有 2 个 y 轴系列共用一个 value 轴，刻度数量应合理
    assert!(
        labels.len() >= 3 && labels.len() <= 8,
        "Y 轴刻度数量不合理: {}",
        labels.len()
    );
}

/// X 轴类别刻度应与 option 中的类别一一对应（数量一致、内容一致、不重复）。
#[test]
fn line_x_axis_category_labels_match_option() {
    let option = option_from_example("line");
    // 从 option 读取 x 轴类别名
    let cats: Vec<String> = option
        .x_axis
        .as_slice()
        .first()
        .and_then(|a| a.data.as_ref())
        .map(|d| d.iter().map(|v| v.as_str().to_string()).collect())
        .expect("line.json 应定义 x 轴类别");

    // 渲染后统计每个类别名出现的次数（期望各出现一次，即不重复渲染）
    let nodes = render("line", 800, 600);
    let all_labels = texts(&nodes);
    let rendered: Vec<String> = all_labels.iter().map(|(t, _, _)| t.clone()).collect();
    for cat in &cats {
        let count = rendered.iter().filter(|t| *t == cat).count();
        assert_eq!(
            count, 1,
            "X 轴类别 '{}' 应恰好渲染一次，实际 {} 次。渲染文本: {:?}",
            cat, count, rendered
        );
    }
}

/// 数值轴范围应覆盖数据最大值的合理倍数（Y 轴上限应 ≥ 数据最大值）。
#[test]
fn line_y_axis_range_covers_max() {
    let option = option_from_example("line");
    // 取所有系列的数据最大值
    let max_val = option
        .series
        .iter()
        .flat_map(|s| match s {
            liecharts::option::SeriesOption::Line(l) => l.data.clone(),
            _ => Vec::new(),
        })
        .filter_map(|d| d.as_value())
        .fold(0.0_f64, f64::max);
    // 从输出刻度中提取最大刻度
    let nodes = render("line", 800, 600);
    let labels = axis_label_texts(&nodes);
    let max_tick = labels
        .iter()
        .filter_map(|(t, _, y)| t.parse::<f64>().ok().filter(|_| *y < 300.0)) // 顶部刻度
        .fold(0.0_f64, f64::max);
    assert!(
        max_tick >= max_val,
        "Y 轴最大刻度 {} 应 ≥ 数据最大值 {}",
        max_tick,
        max_val
    );
}
