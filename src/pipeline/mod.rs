pub mod axis_binding_resolver;
pub mod axis_label;
pub mod axis_renderer;
pub mod builder;
pub mod chart_pipeline;
pub mod color_assigner;
pub mod compat;
pub mod data_processor;
pub mod dataframe;
pub mod decorator;
pub mod grid_planner;
pub mod materializer;
pub mod typed_series;
pub mod types;

pub use axis_binding_resolver::AxisBindingResolver;
pub use builder::{SeriesBuilder, build_typed_series};
pub use chart_pipeline::{build_chart, build_chart_from_spec, build_chart_with_theme};
pub use color_assigner::ColorAssigner;
pub use grid_planner::GridPlanner;
pub use materializer::{SeriesMaterializer, materialize_all};
pub use typed_series::{
    BarGroupType, BarRect, BarSeries, BarSubSeries, BoxplotRect, BoxplotSeries, Bubble,
    BubbleSeries, CandleRect, CandlestickSeries, GaugeSeries, GroupedBarRow, GroupedBarSeries,
    HeatmapCell, HeatmapSeries, LabelPosition, LineSeries, PieSeries, PieSlice, PolarBarPoint,
    PolarBarSeries, PolarPoint, PolarScatterSeries, RadarSeries, RenderContext, ScatterSeries,
    SymbolType, TableSeries, TypedSeries,
};
pub use types::*;
