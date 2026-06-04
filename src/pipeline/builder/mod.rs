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

pub mod line;
pub mod bar;
pub mod scatter;
pub mod pie;
pub mod bubble;
pub mod candlestick;
pub mod radar;
pub mod polar_bar;
pub mod polar_scatter;
pub mod gauge;
pub mod table;
pub mod grouped_bar;

pub use line::LineBuilder;
pub use bar::BarBuilder;
pub use scatter::ScatterBuilder;
pub use pie::PieBuilder;
pub use bubble::BubbleBuilder;
pub use candlestick::CandlestickBuilder;
pub use radar::RadarBuilder;
pub use polar_bar::PolarBarBuilder;
pub use polar_scatter::PolarScatterBuilder;
pub use gauge::GaugeBuilder;
pub use table::TableBuilder;
pub use grouped_bar::GroupedBarBuilder;

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
