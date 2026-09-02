//! Builder 阶段：将 TypedSeries 组装为 lievisual 的 `SceneNode`
//!
//! 职责：
//! - 接收 `&LineSeries` + `&RenderContext`
//! - 产生 `Vec<lievisual::SceneNode>`
//! - 不包含任何坐标映射、颜色解析或字段提取

use lievisual::{
    geometry::Color,
    scene::{Element, Fill, FillStrokeStyle, LinearGradient, SceneNode, Stroke},
};

/// 便捷构造函数：矩形节点。
pub fn rect(rect: vello_cpu::kurbo::Rect, style: FillStrokeStyle, z: i32) -> SceneNode {
    SceneNode::new(Element::rect(rect, style)).with_z(z)
}

/// 便捷构造函数：圆节点。
pub fn circle(
    center: vello_cpu::kurbo::Point,
    radius: f64,
    style: FillStrokeStyle,
    z: i32,
) -> SceneNode {
    SceneNode::new(Element::circle(center, radius, style)).with_z(z)
}

/// 便捷构造函数：线段节点。
pub fn line(
    start: vello_cpu::kurbo::Point,
    end: vello_cpu::kurbo::Point,
    style: Stroke,
    z: i32,
) -> SceneNode {
    SceneNode::new(Element::line(start, end, style)).with_z(z)
}

/// 便捷构造函数：折线节点。
pub fn poly(points: Vec<vello_cpu::kurbo::Point>, style: Stroke, z: i32) -> SceneNode {
    SceneNode::new(Element::poly(points, style)).with_z(z)
}

/// 便捷构造函数：路径节点。
pub fn path(
    path: vello_cpu::kurbo::BezPath,
    style: FillStrokeStyle,
    closed: bool,
    z: i32,
) -> SceneNode {
    SceneNode::new(Element::Path {
        path,
        style,
        closed,
    })
    .with_z(z)
}

/// 便捷构造函数：渐变路径节点。
pub fn gradient_path(
    path: vello_cpu::kurbo::BezPath,
    gradient: LinearGradient,
    stroke: Option<Stroke>,
    z: i32,
) -> SceneNode {
    SceneNode::new(Element::GradientPath {
        path,
        gradient,
        stroke,
    })
    .with_z(z)
}

/// 便捷构造函数：分组节点。
pub fn group(children: Vec<SceneNode>, z: i32) -> SceneNode {
    SceneNode::group(children).with_z(z)
}

/// 解析极坐标类（饼图/仪表盘）半径为实际像素
///
/// P2a 起 `radius` 为**绝对像素**（api/compat 层以「画布 min/2」为基准折算）。
/// 本函数是 pipeline 侧的统一收口：
/// - `radius > 0`：折算好的绝对像素，再 clamp 到**绘图区内接半径**，避免
///   多 subplot / 紧边距下图形越出 subplot 甚至画布（见 docs/布局自适应改造计划.md P5）。
/// - `radius <= 0`：未指定（如 `PieConfig::default()`）→ 按绘图区内接半径的
///   `default_pct` 自适应，避免 Default 值被当成固定像素画出小图。
pub(crate) fn resolve_radius(
    radius: f64,
    bounds_width: f64,
    bounds_height: f64,
    default_pct: f64,
) -> f64 {
    let max_radius = bounds_width.min(bounds_height) * 0.5;
    if radius > 0.0 {
        radius.min(max_radius)
    } else {
        max_radius * default_pct / 100.0
    }
}

/// 解析饼图（内， 外）半径对（单径语义见 [`resolve_radius`]）。
///
/// 外径被 clamp 时，内径按**同一比例**缩放：若内外径各自独立 clamp，
/// 环形图（inner > 0）会出现 `inner == outer` 的零宽圆环，视觉退化为实心圆。
/// 例：`radius=["40%","75%"]` 在紧边距下 inner 200→75、outer 375→75；
/// 比例传导后 inner = 200 × (75/375) = 40。
pub(crate) fn resolve_pie_radii(
    radius_inner: f64,
    radius_outer: f64,
    bounds_width: f64,
    bounds_height: f64,
    default_outer_pct: f64,
) -> (f64, f64) {
    let outer = resolve_radius(radius_outer, bounds_width, bounds_height, default_outer_pct);
    let inner = if radius_inner > 0.0 {
        // clamp 比例 = clamp 后外径 / 原始外径；外径未指定时以自适应值为基准
        let raw_outer = if radius_outer > 0.0 {
            radius_outer
        } else {
            bounds_width.min(bounds_height) * 0.5 * default_outer_pct / 100.0
        };
        let scale = if raw_outer > 0.0 {
            (outer / raw_outer).min(1.0)
        } else {
            1.0
        };
        (radius_inner * scale).min(outer)
    } else {
        0.0
    };
    (inner, outer)
}

/// `Color` 的 CSS 十六进制解析扩展（lievisual 仅提供 `to_hex`）。
pub trait ColorExt: Sized {
    fn from_hex(hex: &str) -> Option<Self>;
}

impl ColorExt for Color {
    fn from_hex(hex: &str) -> Option<Color> {
        let s = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
        if !(s.len() == 6 || s.len() == 8) || !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let n = u32::from_str_radix(s, 16).ok()?;
        Some(if s.len() == 8 {
            Color {
                r: ((n >> 24) & 0xff) as u8,
                g: ((n >> 16) & 0xff) as u8,
                b: ((n >> 8) & 0xff) as u8,
                a: (n & 0xff) as u8,
            }
        } else {
            Color {
                r: ((n >> 16) & 0xff) as u8,
                g: ((n >> 8) & 0xff) as u8,
                b: (n & 0xff) as u8,
                a: 255,
            }
        })
    }
}

/// 便捷构造函数：文本节点。`style` 已含 `rotation` / `max_width` 等块级属性。
pub fn text_el(
    text: impl Into<String>,
    position: vello_cpu::kurbo::Point,
    style: lievisual::text::TextStyle,
    z: i32,
) -> SceneNode {
    SceneNode::new(Element::Text {
        spans: vec![lievisual::text::RichSpan::new(text, style.clone())],
        position,
        style,
        layout: None,
    })
    .with_z(z)
}

use crate::{
    error::Result,
    pipeline::typed_series::{RenderContext, TypedSeries},
};

pub mod bar;
pub mod boxplot;
pub mod bubble;
pub mod candlestick;
pub mod gauge;
pub mod grouped_bar;
pub mod heatmap;
pub mod line;
pub mod pie;
pub mod polar_bar;
pub mod polar_scatter;
pub mod radar;
pub mod scatter;
pub mod table;

pub use bar::BarBuilder;
pub use boxplot::BoxplotBuilder;
pub use bubble::BubbleBuilder;
pub use candlestick::CandlestickBuilder;
pub use gauge::GaugeBuilder;
pub use grouped_bar::GroupedBarBuilder;
pub use heatmap::HeatmapBuilder;
pub use line::LineBuilder;
pub use pie::PieBuilder;
pub use polar_bar::PolarBarBuilder;
pub use polar_scatter::PolarScatterBuilder;
pub use radar::RadarBuilder;
pub use scatter::ScatterBuilder;
pub use table::TableBuilder;

/// 每种 TypedSeries variant 有对应的 SceneNode 构建器
pub trait SeriesBuilder<T> {
    fn build(series: &T, ctx: &RenderContext) -> Result<Vec<SceneNode>>;
}

/// 构建 TypedSeries 为 lievisual SceneNode
pub fn build_typed_series(series: &TypedSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
    match series {
        TypedSeries::Line(s) => LineBuilder::build(s, ctx),
        TypedSeries::Bar(s) => BarBuilder::build(s, ctx),
        TypedSeries::GroupedBar(s) => GroupedBarBuilder::build(s, ctx),
        TypedSeries::Scatter(s) => ScatterBuilder::build(s, ctx),
        TypedSeries::Bubble(s) => BubbleBuilder::build(s, ctx),
        TypedSeries::Candlestick(s) => CandlestickBuilder::build(s, ctx),
        TypedSeries::Boxplot(s) => BoxplotBuilder::build(s, ctx),
        TypedSeries::Heatmap(s) => HeatmapBuilder::build(s, ctx),
        TypedSeries::Pie(s) => PieBuilder::build(s, ctx),
        TypedSeries::Radar(s) => RadarBuilder::build(s, ctx),
        TypedSeries::PolarBar(s) => PolarBarBuilder::build(s, ctx),
        TypedSeries::PolarScatter(s) => PolarScatterBuilder::build(s, ctx),
        TypedSeries::Gauge(s) => GaugeBuilder::build(s, ctx),
        TypedSeries::Table(s) => TableBuilder::build(s, ctx),
    }
}

/// Z-index 常量
pub const Z_BACKGROUND: i32 = 0;
pub const Z_GRID: i32 = 10;
pub const Z_AXIS: i32 = 30;
pub const Z_AXIS_LINE: i32 = 20;
pub const Z_AXIS_TICK: i32 = 21;
pub const Z_AXIS_LABEL: i32 = 22;
pub const Z_SERIES_FILL: i32 = 30;
pub const Z_SERIES_LINE: i32 = 31;
pub const Z_SERIES_POINT: i32 = 32;
pub const Z_SERIES_LABEL: i32 = 33;
pub const Z_LABEL: i32 = 40;
pub const Z_LEGEND: i32 = 40;
pub const Z_TITLE: i32 = 50;

/// 辅助函数：纯填充样式
pub fn fill_style(color: Color) -> FillStrokeStyle {
    FillStrokeStyle {
        fill: Some(Fill::Solid(color)),
        stroke: None,
    }
}

/// 辅助函数：纯描边样式
pub fn stroke_style(color: Color, width: f64) -> Stroke {
    Stroke::new(color, width)
}

/// 辅助函数：填充 + 描边
pub fn fill_stroke_style(fill: Color, stroke: Color, stroke_width: f64) -> FillStrokeStyle {
    FillStrokeStyle {
        fill: Some(Fill::Solid(fill)),
        stroke: Some(Stroke::new(stroke, stroke_width)),
    }
}

/// 渲染标注线（markLine）：横向贯穿整个绘图区的线段 + 标签文本。
pub fn render_mark_lines(
    elements: &mut Vec<SceneNode>,
    mark_lines: &[crate::pipeline::typed_series::MarkLineRender],
    bounds: vello_cpu::kurbo::Rect,
) {
    use lievisual::{
        geometry::Point,
        text::{RichSpan, TextAlign, TextBaseline, TextStyle},
    };

    for ml in mark_lines {
        // 标注线：从绘图区左边界到右边界
        let mut path = vello_cpu::kurbo::BezPath::new();
        path.move_to(Point::new(bounds.x0, ml.y));
        path.line_to(Point::new(bounds.x1, ml.y));
        elements.push(
            SceneNode::new(Element::Path {
                path,
                style: FillStrokeStyle {
                    fill: None,
                    stroke: Some(Stroke::new(ml.color, 1.5)),
                },
                closed: false,
            })
            .with_z(Z_SERIES_LABEL + 1),
        );

        // 标注线标签：放在左边界上方
        let mut style = TextStyle::new(ml.color, 11.0, "sans-serif");
        style.align = TextAlign::Left;
        style.baseline = TextBaseline::Bottom;
        elements.push(
            SceneNode::new(Element::Text {
                spans: vec![RichSpan::new(ml.label.clone(), style.clone())],
                position: Point::new(bounds.x0 + 4.0, ml.y - 4.0),
                style,
                layout: None,
            })
            .with_z(Z_SERIES_LABEL + 2),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_pie_radii, resolve_radius};

    #[test]
    fn resolve_radius_clamps_to_plot_area() {
        // P5：绝对像素半径超出绘图区内接半径时（多 subplot / 紧边距）必须被 clamp
        let max = 100.0f64; // min(200, 400) / 2
        assert_eq!(resolve_radius(250.0, 200.0, 400.0, 75.0), max);
        // 未超出时保持原值（P2a 的画布基准语义不变）
        assert_eq!(resolve_radius(60.0, 200.0, 400.0, 75.0), 60.0);
    }

    #[test]
    fn resolve_radius_falls_back_when_unspecified() {
        // <=0 视为未指定：按内接半径的 default_pct 自适应
        // （`PieConfig::default()` 的 radius 现在是 (0.0, 0.0)）
        assert_eq!(resolve_radius(0.0, 200.0, 400.0, 75.0), 75.0);
        assert_eq!(resolve_radius(-1.0, 200.0, 400.0, 0.0), 0.0);
    }

    #[test]
    fn resolve_pie_radii_scales_inner_with_clamped_outer() {
        // 外径 375 被 clamp 到 75（比例 0.2）：内径 200 应同比例缩到 40，
        // 而非独立 clamp 成 75（零宽圆环 → 环形图退化为实心圆）
        let (inner, outer) = resolve_pie_radii(200.0, 375.0, 150.0, 150.0, 75.0);
        assert!((outer - 75.0).abs() < 1e-9);
        assert!(
            (inner - 40.0).abs() < 1e-9,
            "inner 应为 200×(75/375)=40，实际 {inner}"
        );
        // 未触发 clamp 时内外径保持原值
        let (inner, outer) = resolve_pie_radii(200.0, 300.0, 1000.0, 1000.0, 75.0);
        assert!((outer - 300.0).abs() < 1e-9);
        assert!((inner - 200.0).abs() < 1e-9);
    }
}
