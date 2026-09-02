//! Gauge Builder：ECharts 风格仪表盘（进度环 + 刻度 + 指针 + 中心锚点）
//!
//! 视觉规格参照 ECharts gauge（官方 gauge-progress 示例 + 默认项；
//! 尺寸单位 = radius/100，与 ECharts 的 width/length/distance 语义一致）：
//! - 轴环厚 17%（灰底 `#E6EBF8` 全环打底，进度色带叠加，外缘 = radius）
//! - 主刻度（splitLine）长 10%、宽 2、`#999`，起于环外缘外 1.5%
//! - 刻度标签**径向对齐**：`textAlign`/`textBaseline` 随角度取值
//!   （`cos→Left/Right/Center`、`sin→Top/Bottom/Middle`，同 ECharts GaugeView）
//! - 指针长 80%、宽 6%，风筝形多边形（替代细线），中心锚点圆环盖于其上
//! - 标题/数值 `offsetCenter [0,'20%'] / [0,'40%']`（中心下方），墨迹盒中心对齐
//!
//! 历史：旧实现环厚固定 18px（radius 的 8%，明显偏细）、指针为 3px 细线、
//! 刻度标签 Center/Middle 锚定（侧向位置视觉脱节）、`series.center` 字段
//! 被硬编码忽略。

use std::f64::consts::PI;

use lievisual::{
    Color,
    scene::{Element, FillStrokeStyle, GradientStop, LinearGradient, SceneNode, Stroke},
    text::{RichSpan, TextAlign, TextBaseline, TextStyle, measure_text},
};
use vello_cpu::kurbo::{Arc, BezPath, PathSeg, Point, Shape};

use crate::{
    error::Result,
    pipeline::{
        builder::{
            SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, Z_SERIES_LINE, circle, gradient_path,
            line, path, resolve_radius,
        },
        typed_series::{GaugeSeries, RenderContext},
    },
};

/// ECharts 视觉常量（单位 = radius/100）
mod spec {
    /// 轴环厚度（ECharts gauge-progress 示例 axisLine.lineStyle.width=18，
    /// 略收至 17 给标签留头部空间）
    pub const RING_WIDTH: f64 = 17.0;
    /// 主刻度（splitLine）与环外缘的间距
    pub const SPLIT_LINE_GAP: f64 = 1.5;
    /// 主刻度长度（示例 splitLine.length=15，收至 10 避让顶部标题区）
    pub const SPLIT_LINE_LENGTH: f64 = 10.0;
    /// 标签与刻度末端的间距（合计 radius+14%，顶标签不与图题副标题重叠）
    pub const LABEL_GAP: f64 = 2.5;
    /// 指针长度（ECharts pointer.length '80%'）
    pub const POINTER_LENGTH: f64 = 80.0;
    /// 指针底宽（ECharts pointer.width 6）
    pub const POINTER_WIDTH: f64 = 6.0;
    /// 指针尾部越过中心的长度
    pub const POINTER_TAIL: f64 = 3.0;
    /// 中心锚点半径（anchor.size 的一半）
    pub const PIN_RADIUS: f64 = 5.5;
    /// 中心锚点描边宽
    pub const PIN_BORDER: f64 = 2.5;
    /// 标题 offsetCenter [0,'20%']
    pub const TITLE_OFFSET: f64 = 20.0;
    /// 数值 offsetCenter [0,'40%']
    pub const DETAIL_OFFSET: f64 = 40.0;
}

pub struct GaugeBuilder;

impl SeriesBuilder<GaugeSeries> for GaugeBuilder {
    fn build(series: &GaugeSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
        let mut elements = Vec::new();

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 中心百分比真正取自 series.center（此前硬编码 50/55 忽略了字段；
        // compat 层默认 (50,50) 与 ECharts 默认一致）
        let center_x = bounds.x0 + width * series.center.0 / 100.0;
        let center_y = bounds.y0 + height * series.center.1 / 100.0;
        let center = Point::new(center_x, center_y);

        // P2a：radius 为**绝对像素**（api/compat 层按「画布 min/2」折算）；
        // P5 起由 `resolve_radius` 统一 clamp 到绘图区内接半径，
        // 未指定时按内接半径的 75% 自适应。
        let radius = resolve_radius(series.radius, width, height, 75.0);
        let unit = radius / 100.0; // ECharts 尺寸单位

        // 转换角度为弧度
        let start_angle = series.start_angle * PI / 180.0;
        let mut end_angle = series.end_angle * PI / 180.0;

        // 修正扫过跨 0°/360° 的角度：当结束角 <= 起始角时（如 225° -> -45°），
        // 实际是顺时针扫过 360°（结束角加一圈），使 sweep 为正、跨越 0° 的弧正确。
        if end_angle <= start_angle {
            end_angle += 2.0 * PI;
        }
        let total_sweep = end_angle - start_angle;

        // 计算数值角度
        let value_ratio = ((series.value - series.min) / (series.max - series.min)).clamp(0.0, 1.0);
        let value_angle = start_angle + total_sweep * value_ratio;

        // 轴环：外缘 = radius，厚 spec::RING_WIDTH%
        let ring_width = spec::RING_WIDTH * unit;
        let outer_r = radius;
        let inner_r = radius - ring_width;

        // 灰底全环（ECharts axisLine 打底：进度叠加其上，value=0/100 也完整）
        let track_path = build_arc_ribbon(center, inner_r, outer_r, start_angle, end_angle);
        elements.push(path(
            track_path,
            FillStrokeStyle {
                fill: Some(lievisual::scene::Fill::Solid(Color::rgb(230, 235, 248))),
                stroke: None,
            },
            true,
            Z_SERIES_FILL,
        ));

        // 绘制进度色带（平滑渐变：绿→黄→红）
        if value_angle > start_angle + 1e-6 {
            let filled_path = build_arc_ribbon(center, inner_r, outer_r, start_angle, value_angle);
            let bbox = filled_path.bounding_box();
            let gradient = LinearGradient {
                start: Point::new(bbox.x0, bbox.y0),
                end: Point::new(bbox.x1, bbox.y1),
                stops: vec![
                    GradientStop {
                        offset: 0.00,
                        color: Color::rgb(80, 180, 80), // 绿
                    },
                    GradientStop {
                        offset: 0.50,
                        color: Color::rgb(255, 200, 50), // 黄
                    },
                    GradientStop {
                        offset: 1.00,
                        color: Color::rgb(220, 80, 80), // 红
                    },
                ],
            };
            elements.push(gradient_path(filled_path, gradient, None, Z_SERIES_FILL));
        }

        // 主刻度（splitLine）与标签
        let split_number = series.split_number.max(1);
        let tick_start_r = outer_r + spec::SPLIT_LINE_GAP * unit;
        let tick_end_r = tick_start_r + spec::SPLIT_LINE_LENGTH * unit;
        let label_r = tick_end_r + spec::LABEL_GAP * unit;

        for i in 0..=split_number {
            let angle = start_angle + total_sweep * i as f64 / split_number as f64;
            let (sin, cos) = angle.sin_cos();

            // 刻度线
            let x1 = center.x + tick_start_r * cos;
            let y1 = center.y + tick_start_r * sin;
            let x2 = center.x + tick_end_r * cos;
            let y2 = center.y + tick_end_r * sin;

            elements.push(line(
                Point::new(x1, y1),
                Point::new(x2, y2),
                crate::pipeline::builder::stroke_style(Color::rgb(153, 153, 153), 2.0),
                Z_SERIES_LINE + 1,
            ));

            // 标签：ECharts GaugeView 的径向对齐——
            // cos>0（右半）→ Left（文字向外/右伸展），cos<0 → Right；
            // sin>0（下半，屏幕坐标 y 向下）→ Top，sin<0 → Bottom。
            let label_val = series.min + (series.max - series.min) * i as f64 / split_number as f64;
            let label_text = if label_val.fract() == 0.0 {
                format!("{:.0}", label_val)
            } else {
                format!("{:.1}", label_val)
            };

            let mut style = TextStyle::new(Color::rgb(84, 85, 90), 12.0, "sans-serif");
            style.align = if cos > 1e-3 {
                TextAlign::Left
            } else if cos < -1e-3 {
                TextAlign::Right
            } else {
                TextAlign::Center
            };
            style.baseline = if sin > 1e-3 {
                TextBaseline::Top
            } else if sin < -1e-3 {
                TextBaseline::Bottom
            } else {
                TextBaseline::Middle
            };
            elements.push(
                SceneNode::new(Element::Text {
                    spans: vec![RichSpan::new(label_text, style.clone())],
                    position: Point::new(center.x + label_r * cos, center.y + label_r * sin),
                    style,
                    layout: None,
                })
                .with_z(Z_SERIES_LABEL),
            );
        }

        // 指针（ECharts 默认：长 80%、宽 6%，风筝形收尖，尾部略越中心）
        let needle_len = spec::POINTER_LENGTH * unit;
        let needle_half_w = spec::POINTER_WIDTH * unit / 2.0;
        let tail_len = spec::POINTER_TAIL * unit;
        let (ns, nc) = value_angle.sin_cos();
        let dir = Point::new(nc, ns);
        let perp = Point::new(-ns, nc);

        let needle = BezPath::from_vec(vec![
            vello_cpu::kurbo::PathEl::MoveTo(Point::new(
                center.x + dir.x * needle_len,
                center.y + dir.y * needle_len,
            )),
            vello_cpu::kurbo::PathEl::LineTo(Point::new(
                center.x + perp.x * needle_half_w,
                center.y + perp.y * needle_half_w,
            )),
            vello_cpu::kurbo::PathEl::LineTo(Point::new(
                center.x - dir.x * tail_len,
                center.y - dir.y * tail_len,
            )),
            vello_cpu::kurbo::PathEl::LineTo(Point::new(
                center.x - perp.x * needle_half_w,
                center.y - perp.y * needle_half_w,
            )),
            vello_cpu::kurbo::PathEl::ClosePath,
        ]);
        elements.push(path(
            needle,
            FillStrokeStyle {
                fill: Some(lievisual::scene::Fill::Solid(series.color)),
                stroke: None,
            },
            true,
            Z_SERIES_LINE + 2,
        ));

        // 中心锚点（ECharts anchor：白底 + 指针色圆环，盖在指针上方）
        elements.push(circle(
            center,
            spec::PIN_RADIUS * unit,
            FillStrokeStyle {
                fill: Some(lievisual::scene::Fill::Solid(Color::rgb(255, 255, 255))),
                stroke: Some(Stroke::new(series.color, spec::PIN_BORDER * unit)),
            },
            Z_SERIES_LINE + 3,
        ));

        // 中心数值显示（detail，offsetCenter [0,'40%']，墨迹盒中心对齐）
        let value_text = format!("{:.1}%", series.value);
        let value_style = TextStyle::new(Color::rgb(60, 60, 65), 30.0, "sans-serif");
        push_ink_centered_text(
            &mut elements,
            &value_text,
            Point::new(center.x, center.y + spec::DETAIL_OFFSET * unit),
            value_style,
            Z_SERIES_LABEL,
        );
        // 标题（title，offsetCenter [0,'20%']，位于数值上方）
        let title_style = TextStyle::new(Color::rgb(132, 132, 138), 16.0, "sans-serif");
        push_ink_centered_text(
            &mut elements,
            &series.name,
            Point::new(center.x, center.y + spec::TITLE_OFFSET * unit),
            title_style,
            Z_SERIES_LABEL,
        );

        Ok(elements)
    }
}

/// 生成墨迹盒中心精确落在 `target` 的单行文本。
///
/// `align=Left`/`baseline=Top` 语义下两个后端的块偏移均为 0（锚点 = 块左上
/// 角），将 `ink_bounds()`（相对块原点）的中心反解进 position 即可，与图例
/// 文本对齐（`decorator/legend.rs`）同一机制。
fn push_ink_centered_text(
    elements: &mut Vec<SceneNode>,
    text: &str,
    target: Point,
    style: TextStyle,
    z: i32,
) {
    // 显式 Top/Left（锚点 = 块左上角，后端块偏移均为 0）——`TextStyle::new`
    // 默认 baseline 是 Alphabetic，直接用会把基线当块顶反解，文本整体偏高
    let mut style = style;
    style.align = TextAlign::Left;
    style.baseline = TextBaseline::Top;
    let layout = measure_text(
        &[RichSpan::new(text.to_string(), style.clone())],
        style.max_width,
    )
    .layout;
    let ink = layout.ink_bounds();
    let position = Point::new(
        target.x - (ink.min_x() + ink.max_x()) / 2.0,
        target.y - (ink.min_y() + ink.max_y()) / 2.0,
    );
    elements.push(
        SceneNode::new(Element::Text {
            spans: vec![RichSpan::new(text.to_string(), style.clone())],
            position,
            style,
            layout: Some(layout),
        })
        .with_z(z),
    );
}

/// 构建一段色带（环形扇区），使用真圆弧
fn build_arc_ribbon(center: Point, inner_r: f64, outer_r: f64, start: f64, end: f64) -> BezPath {
    let sweep = end - start;
    let mut path = BezPath::new();

    // 外圆弧起点
    let outer_start = Point::new(
        center.x + outer_r * start.cos(),
        center.y + outer_r * start.sin(),
    );
    path.move_to(outer_start);

    // 外圆弧
    add_arc(&mut path, center, outer_r, start, sweep);

    // 连接到内圆弧终点
    let inner_end = Point::new(
        center.x + inner_r * end.cos(),
        center.y + inner_r * end.sin(),
    );
    path.line_to(inner_end);

    // 内圆弧（反向）
    add_arc(&mut path, center, inner_r, end, -sweep);

    path.close_path();
    path
}

/// 使用 kurbo::Arc 添加真圆弧段到路径
fn add_arc(path: &mut BezPath, center: Point, radius: f64, start: f64, sweep: f64) {
    let arc = Arc {
        center,
        radii: (radius, radius).into(),
        start_angle: start,
        sweep_angle: sweep,
        x_rotation: 0.0,
    };

    // tolerance 越小越平滑
    arc.to_path(0.1).segments().for_each(|seg| match seg {
        PathSeg::Line(line) => path.line_to(line.p1),
        PathSeg::Quad(quad) => path.quad_to(quad.p1, quad.p2),
        PathSeg::Cubic(cubic) => path.curve_to(cubic.p1, cubic.p2, cubic.p3),
    });
}
