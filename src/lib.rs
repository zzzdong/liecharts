pub mod chart;
pub mod component;
pub mod error;
pub mod layout;
pub mod model;
pub mod option;
pub mod pipeline;
pub mod render;
pub mod text;
pub mod theme;
pub mod visual;

pub use chart::LieChart;
pub use error::ChartError;
pub use model::ResolvedOption;
pub use option::{
    AreaStyleOption, AxisOption, AxisPosition, AxisType, BarSeriesOption, BubbleDataPoint,
    BubbleSeriesOption, CandlestickDataPoint, CandlestickItemStyleOption, CandlestickSeriesOption,
    ColorOption, DataPoint, FontWeight, FontWeightNamed, GaugeAxisLabelOption, GaugeAxisLineOption,
    GaugeAxisTickOption, GaugeDataPoint, GaugeDetailOption, GaugePointerOption, GaugeSeriesOption,
    GaugeSplitLineOption, GaugeTitleOption, GridOption, ItemStyleOption, LabelAlign, LabelOption,
    LabelPosition, LabelVerticalAlign, LegendOption, LieChartOption, LineSeriesOption,
    LineStyleOption, LineType, NameLocation, Orient, PieSeriesOption, PolarBarSeriesOption,
    PolarScatterDataPoint, PolarScatterSeriesOption, Position, PositionPreset, RadarDataOption,
    RadarIndicatorOption, RadarNameOption, RadarOption, RadarSeriesOption, ScatterSeriesOption,
    SeriesOption, SplitLineOption, SymbolType, TableBodyOption, TableCellStyleOption,
    TableHeaderOption, TableRowStyleOption, TableSeriesOption, TextAlignOption, TextStyleOption,
    TitleOption,
};
pub use render::{PixmapRenderer, Renderer, SvgRenderer};
pub use text::{FontSource, register_font};
pub use theme::{
    BorderTokens, ColorTokens, DesignTokens, EffectTokens, SpacingTokens, TextTokens, Theme,
    ThemeRegistry,
};
pub use visual::{TextAlign, VisualElement};
