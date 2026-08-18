//! Builder 阶段：将 TypedSeries 组装为 lievisual 的 `SceneNode`
//!
//! 职责：
//! - 接收 `&LineSeries` + `&RenderContext`
//! - 产生 `Vec<lievisual::SceneNode>`
//! - 不包含任何坐标映射、颜色解析或字段提取

use lievisual::geometry::Color;
use lievisual::scene::{Element, Fill, FillStrokeStyle, LinearGradient, SceneNode, Stroke};

/// 便捷构造函数：矩形节点。
pub fn rect(rect: vello_cpu::kurbo::Rect, style: FillStrokeStyle, z: i32) -> SceneNode {
    SceneNode::new(Element::rect(rect, style)).with_z(z)
}

/// 便捷构造函数：圆节点。
pub fn circle(center: vello_cpu::kurbo::Point, radius: f64, style: FillStrokeStyle, z: i32) -> SceneNode {
    SceneNode::new(Element::circle(center, radius, style)).with_z(z)
}

/// 便捷构造函数：线段节点。
pub fn line(start: vello_cpu::kurbo::Point, end: vello_cpu::kurbo::Point, style: Stroke, z: i32) -> SceneNode {
    SceneNode::new(Element::line(start, end, style)).with_z(z)
}

/// 便捷构造函数：折线节点。
pub fn poly(points: Vec<vello_cpu::kurbo::Point>, style: Stroke, z: i32) -> SceneNode {
    SceneNode::new(Element::poly(points, style)).with_z(z)
}

/// 便捷构造函数：路径节点。
pub fn path(path: vello_cpu::kurbo::BezPath, style: FillStrokeStyle, closed: bool, z: i32) -> SceneNode {
    SceneNode::new(Element::Path { path, style, closed }).with_z(z)
}

/// 便捷构造函数：渐变路径节点。
pub fn gradient_path(
    path: vello_cpu::kurbo::BezPath,
    gradient: LinearGradient,
    stroke: Option<Stroke>,
    z: i32,
) -> SceneNode {
    SceneNode::new(Element::GradientPath { path, gradient, stroke }).with_z(z)
}

/// 便捷构造函数：分组节点。
pub fn group(children: Vec<SceneNode>, z: i32) -> SceneNode {
    SceneNode::group(children).with_z(z)
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
                r: ((n >> 24) & 0xff) as f64 / 255.0,
                g: ((n >> 16) & 0xff) as f64 / 255.0,
                b: ((n >> 8) & 0xff) as f64 / 255.0,
                a: (n & 0xff) as f64 / 255.0,
            }
        } else {
            Color {
                r: ((n >> 16) & 0xff) as f64 / 255.0,
                g: ((n >> 8) & 0xff) as f64 / 255.0,
                b: (n & 0xff) as f64 / 255.0,
                a: 1.0,
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
    use lievisual::geometry::Point;
    use lievisual::text::{RichSpan, TextAlign, TextBaseline, TextStyle};

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
