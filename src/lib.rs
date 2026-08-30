pub mod api;
pub mod builder;
pub mod chart;

pub mod error;
pub mod option;
pub mod pipeline;
pub mod prelude;
pub mod render;
pub mod sampling;
pub mod theme;

pub use lievisual::{
    Color, Fill, FillStrokeStyle, FontStyle, Point, Rect, Scene, SceneNode, Stroke, TextAlign,
    TextBaseline, TextStyle, Transform, Vec2,
    text::{
        FontSource, TextLayout, compute_text_offset, parse_generic_family, register_font,
        register_font_generic,
    },
};

pub const Z_BACKGROUND: i32 = 0;
pub const Z_GRID: i32 = 10;
pub const Z_SERIES: i32 = 20;
pub const Z_SERIES_FILL: i32 = 20;
pub const Z_SERIES_LINE: i32 = 21;
pub const Z_SERIES_POINT: i32 = 22;
pub const Z_AXIS: i32 = 30;
pub const Z_LABEL: i32 = 40;
pub const Z_TITLE: i32 = 50;

pub use builder::ChartBuilder;
pub use chart::Chart;
pub use error::ChartError;
pub use option::{
    AnimationOption, AreaStyleOption, AxisLabelOption, AxisLineOption, AxisOption,
    AxisPointerOption, AxisPosition, AxisTickOption, AxisType, BarSeriesOption, BoxplotDataPoint,
    BoxplotItemStyleOption, BoxplotSeriesOption, BrushOption, BubbleDataPoint, BubbleSeriesOption,
    CandlestickDataPoint, CandlestickItemStyleOption, CandlestickSeriesOption, ChartOption,
    ColorOption, DataPoint, DataZoomOption, DatasetOption, EasingFunction, FontWeight,
    FontWeightNamed, GaugeAxisLabelOption, GaugeAxisLineOption, GaugeAxisTickOption,
    GaugeDataPoint, GaugeDetailOption, GaugePointerOption, GaugeProgressOption, GaugeSeriesOption,
    GaugeSplitLineOption, GaugeTitleOption, GradientColorStopOption, GridConfig, GridOption,
    HeatmapDataPoint, HeatmapSeriesOption, ItemStyleOption, LabelAlign, LabelLineOption,
    LabelOption, LabelPosition, LabelVerticalAlign, LegendOption, LineSeriesOption,
    LineStyleOption, LineType, MarkAreaOption, MarkLineOption, MarkPointOption, NameLocation,
    OneOrMany, Orient, PieSeriesOption, PolarBarSeriesOption, PolarScatterDataPoint,
    PolarScatterSeriesOption, PositionOption, PositionPreset, RadarDataOption,
    RadarIndicatorOption, RadarNameOption, RadarOption, RadarSeriesOption, ScatterSeriesOption,
    SeriesEncodeOption, SeriesOption, ShadowStyleOption, SplitAreaOption, SplitLineOption,
    SymbolType, TableBodyOption, TableCellStyleOption, TableHeaderOption, TableRowStyleOption,
    TableSeriesOption, TextAlignOption, TextStyleOption, TitleOption, TooltipOption,
    TooltipTrigger, VisualMapOption, VisualMapType,
};
pub use pipeline::{
    AxisBindingResolver, ColorAssigner, ColorContext, GridPlanner, ResolvedAxisRange,
    ResolvedAxisRanges, SubplotSpec, TextMeasurer, build_chart, build_chart_with_theme,
};
pub use sampling::{SamplingOption, SamplingType};
pub use theme::{Theme, ThemeRegistry};
