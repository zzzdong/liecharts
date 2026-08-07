//! Builder 阶段：将 TypedSeries 组装为 VisualElement
//!
//! 职责：
//! - 接收 `&LineSeries` + `&RenderContext`
//! - 产生 `Vec<VisualElement>`
//! - 不包含任何坐标映射、颜色解析或字段提取

use crate::{
    error::Result,
    pipeline::typed_series::{RenderContext, TypedSeries},
    visual::VisualElement,
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

/// 每种 TypedSeries variant 有对应的 VisualElement 构建器
pub trait SeriesBuilder<T> {
    fn build(series: &T, ctx: &RenderContext) -> Result<Vec<VisualElement>>;
}

/// 构建 TypedSeries 为 VisualElement
pub fn build_typed_series(series: &TypedSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
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
pub const Z_AXIS_LINE: i32 = 20;
pub const Z_AXIS_TICK: i32 = 21;
pub const Z_AXIS_LABEL: i32 = 22;
pub const Z_SERIES_FILL: i32 = 30;
pub const Z_SERIES_LINE: i32 = 31;
pub const Z_SERIES_POINT: i32 = 32;
pub const Z_SERIES_LABEL: i32 = 33;
pub const Z_LEGEND: i32 = 40;
pub const Z_TITLE: i32 = 50;

/// 辅助函数：创建填充描边样式
use crate::visual::{Color, FillStrokeStyle, Stroke, StrokeStyle};

pub fn fill_style(color: Color) -> FillStrokeStyle {
    FillStrokeStyle {
        fill: Some(color),
        stroke: None,
    }
}

pub fn stroke_style(color: Color, width: f64) -> StrokeStyle {
    StrokeStyle { color, width }
}

pub fn fill_stroke_style(fill: Color, stroke: Color, stroke_width: f64) -> FillStrokeStyle {
    FillStrokeStyle {
        fill: Some(fill),
        stroke: Some(Stroke {
            color: stroke,
            width: stroke_width,
        }),
    }
}

/// 渲染标注线（markLine）：横向贯穿整个绘图区的线段 + 标签文本。
///
/// 每个 `MarkLineRender` 已解析出 Y 像素坐标与标签文本，这里只负责画线和文字。
pub fn render_mark_lines(
    elements: &mut Vec<VisualElement>,
    mark_lines: &[crate::pipeline::typed_series::MarkLineRender],
    bounds: vello_cpu::kurbo::Rect,
) {
    use vello_cpu::kurbo::Point;

    use crate::visual::{TextAlign, TextBaseline, TextStyle};

    for ml in mark_lines {
        // 标注线：从绘图区左边界到右边界
        elements.push(VisualElement::Path {
            path: {
                let mut path = vello_cpu::kurbo::BezPath::new();
                path.move_to(Point::new(bounds.x0, ml.y));
                path.line_to(Point::new(bounds.x1, ml.y));
                path
            },
            style: FillStrokeStyle {
                fill: None,
                stroke: Some(Stroke {
                    color: ml.color,
                    width: 1.5,
                }),
            },
            z_index: Z_SERIES_LABEL + 1,
        });

        // 标注线标签：放在左边界上方
        elements.push(VisualElement::TextRun {
            text: ml.label.clone(),
            position: Point::new(bounds.x0 + 4.0, ml.y - 4.0),
            style: TextStyle {
                color: ml.color,
                font_size: 11.0,
                align: TextAlign::Left,
                vertical_align: TextBaseline::Bottom,
                ..Default::default()
            },
            rotation: 0.0,
            max_width: None,
            layout: None,
            z_index: Z_SERIES_LABEL + 2,
        });
    }
}
