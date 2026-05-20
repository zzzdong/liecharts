use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
};

use crate::sampling::SamplingOption;

/// Root chart configuration.
///
/// This is the raw option struct that mirrors the ECharts JSON schema. In most cases
/// you should use [`ChartBuilder`](crate::ChartBuilder) instead of constructing this directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct ChartOption {
    pub title: Option<TitleOption>,
    pub legend: Option<LegendOption>,
    #[serde(default)]
    pub grid: Vec<GridOption>,
    pub radar: Option<RadarOption>,
    #[serde(default)]
    pub x_axis: Vec<AxisOption>,
    #[serde(default)]
    pub y_axis: Vec<AxisOption>,
    #[serde(default)]
    pub series: Vec<SeriesOption>,
    pub color: Option<Vec<ColorOption>>,
    pub background_color: Option<ColorOption>,
    pub theme: Option<String>,
    pub text_style: Option<TextStyleOption>,
}

/// Chart title configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleOption {
    pub text: Option<String>,
    pub subtext: Option<String>,
    pub left: Option<PositionOption>,
    pub top: Option<PositionOption>,
    pub text_style: Option<TextStyleOption>,
    pub subtext_style: Option<TextStyleOption>,
}

impl Default for TitleOption {
    fn default() -> Self {
        Self {
            text: None,
            subtext: None,
            left: Some(PositionOption::center()),
            top: Some(PositionOption::auto()),
            text_style: None,
            subtext_style: None,
        }
    }
}

impl TitleOption {
    /// 创建一个标题，text 为必填
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }

    /// Sets the subtitle.
    pub fn subtext(mut self, subtext: impl Into<String>) -> Self {
        self.subtext = Some(subtext.into());
        self
    }

    pub fn left(mut self, left: PositionOption) -> Self {
        self.left = Some(left);
        self
    }

    pub fn top(mut self, top: PositionOption) -> Self {
        self.top = Some(top);
        self
    }

    pub fn text_style(mut self, style: TextStyleOption) -> Self {
        self.text_style = Some(style);
        self
    }

    pub fn subtext_style(mut self, style: TextStyleOption) -> Self {
        self.subtext_style = Some(style);
        self
    }
}

/// Legend configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LegendOption {
    pub show: Option<bool>,
    pub data: Option<Vec<String>>,
    pub left: Option<PositionOption>,
    pub top: Option<PositionOption>,
    pub orient: Option<Orient>,
    pub text_style: Option<TextStyleOption>,
    pub item_width: Option<f64>,
    pub item_height: Option<f64>,
    pub symbol_size: Option<f64>,
}

impl LegendOption {
    pub fn data(mut self, data: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data = Some(data.into_iter().map(Into::into).collect());
        self
    }

    pub fn left(mut self, left: PositionOption) -> Self {
        self.left = Some(left);
        self
    }

    pub fn top(mut self, top: PositionOption) -> Self {
        self.top = Some(top);
        self
    }

    pub fn orient(mut self, orient: Orient) -> Self {
        self.orient = Some(orient);
        self
    }

    pub fn show(mut self, show: bool) -> Self {
        self.show = Some(show);
        self
    }
}

/// Grid region configuration for multi-layout charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridOption {
    pub left: Option<PositionOption>,
    pub right: Option<PositionOption>,
    pub top: Option<PositionOption>,
    pub bottom: Option<PositionOption>,
    pub contain_label: Option<bool>,
}

impl Default for GridOption {
    fn default() -> Self {
        Self {
            left: Some(PositionOption::percent(10.0)),
            right: Some(PositionOption::percent(10.0)),
            top: Some(PositionOption::percent(15.0)),
            bottom: Some(PositionOption::percent(15.0)),
            contain_label: Some(true),
        }
    }
}

impl GridOption {
    pub fn left(mut self, left: PositionOption) -> Self {
        self.left = Some(left);
        self
    }

    pub fn right(mut self, right: PositionOption) -> Self {
        self.right = Some(right);
        self
    }

    pub fn top(mut self, top: PositionOption) -> Self {
        self.top = Some(top);
        self
    }

    pub fn bottom(mut self, bottom: PositionOption) -> Self {
        self.bottom = Some(bottom);
        self
    }

    pub fn contain_label(mut self, contain: bool) -> Self {
        self.contain_label = Some(contain);
        self
    }
}

/// Axis configuration (category, value, or time).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisOption {
    #[serde(rename = "type")]
    pub axis_type: Option<AxisType>,
    pub data: Option<Vec<String>>,
    pub name: Option<String>,
    pub name_location: Option<NameLocation>,
    pub name_text_style: Option<TextStyleOption>,
    pub axis_label: Option<AxisLabelOption>,
    pub axis_line: Option<AxisLineOption>,
    pub axis_tick: Option<AxisTickOption>,
    pub split_line: Option<SplitLineOption>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub boundary_gap: Option<bool>,
    pub position: Option<AxisPosition>,
    pub grid_index: Option<usize>,
}

impl Default for AxisOption {
    fn default() -> Self {
        Self {
            axis_type: Some(AxisType::Category),
            data: None,
            name: None,
            name_location: Some(NameLocation::End),
            name_text_style: None,
            axis_label: None,
            axis_line: None,
            axis_tick: None,
            grid_index: None,
            split_line: None,
            min: None,
            max: None,
            boundary_gap: Some(true),
            position: None,
        }
    }
}

impl AxisOption {
    /// 创建类目轴
    pub fn category() -> Self {
        Self {
            axis_type: Some(AxisType::Category),
            ..Default::default()
        }
    }

    /// 创建数值轴
    pub fn value() -> Self {
        Self {
            axis_type: Some(AxisType::Value),
            ..Default::default()
        }
    }

    pub fn data(mut self, data: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data = Some(data.into_iter().map(Into::into).collect());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn position(mut self, position: AxisPosition) -> Self {
        self.position = Some(position);
        self
    }

    pub fn grid_index(mut self, index: usize) -> Self {
        self.grid_index = Some(index);
        self
    }

    pub fn boundary_gap(mut self, gap: bool) -> Self {
        self.boundary_gap = Some(gap);
        self
    }

    pub fn axis_label(mut self, label: AxisLabelOption) -> Self {
        self.axis_label = Some(label);
        self
    }

    pub fn split_line(mut self, split: SplitLineOption) -> Self {
        self.split_line = Some(split);
        self
    }
}

/// Axis label configuration (axis label style).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisLabelOption {
    pub show: Option<bool>,
    pub rotate: Option<f64>,
    pub formatter: Option<String>,
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub align: Option<LabelAlign>,
    pub vertical_align: Option<LabelVerticalAlign>,
    pub margin: Option<f64>,
}

impl Default for AxisLabelOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            rotate: None,
            formatter: None,
            color: None,
            font_size: Some(12.0),
            font_family: None,
            font_weight: None,
            align: None,
            vertical_align: None,
            margin: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisLineOption {
    pub show: Option<bool>,
    pub line_style: Option<LineStyleOption>,
}

impl Default for AxisLineOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            line_style: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisTickOption {
    pub show: Option<bool>,
    pub align_with_label: Option<bool>,
    pub line_style: Option<LineStyleOption>,
}

impl Default for AxisTickOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            align_with_label: None,
            line_style: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitLineOption {
    pub show: Option<bool>,
    pub line_style: Option<LineStyleOption>,
}

impl Default for SplitLineOption {
    fn default() -> Self {
        Self {
            show: Some(false),
            line_style: None,
        }
    }
}

/// Line style configuration (color, width, dash, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineStyleOption {
    pub color: Option<ColorOption>,
    pub width: Option<f64>,
    #[serde(rename = "type")]
    pub line_type: Option<LineType>,
}

impl Default for LineStyleOption {
    fn default() -> Self {
        Self {
            color: None,
            width: Some(2.0),
            line_type: Some(LineType::Solid),
        }
    }
}

/// Global text style applied when no per-item style is specified.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextStyleOption {
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub align: Option<TextAlignOption>,
    pub vertical_align: Option<LabelVerticalAlign>,
}

impl Default for TextStyleOption {
    fn default() -> Self {
        Self {
            color: None,
            font_size: Some(12.0),
            font_family: None,
            font_weight: Some(FontWeight::Named(FontWeightNamed::Normal)),
            font_style: None,
            align: None,
            vertical_align: None,
        }
    }
}

impl TextStyleOption {
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn color(mut self, color: ColorOption) -> Self {
        self.color = Some(color);
        self
    }

    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    pub fn align(mut self, align: TextAlignOption) -> Self {
        self.align = Some(align);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSeriesOption {
    pub name: Option<String>,
    pub data: Option<Vec<Vec<serde_json::Value>>>,
    pub columns: Option<Vec<String>>,
    pub header: Option<TableHeaderOption>,
    pub body: Option<TableBodyOption>,
    pub row_style: Option<TableRowStyleOption>,
    pub cell_style: Option<TableCellStyleOption>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub left: Option<f64>,
    pub top: Option<f64>,
    /// 表格所属的 grid 索引，默认 0
    pub grid_index: Option<usize>,
    /// 是否自动调整 grid 大小以适应表格内容
    pub auto_fit_grid: Option<bool>,
}

impl Default for TableSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: None,
            columns: None,
            header: Some(TableHeaderOption::default()),
            body: Some(TableBodyOption::default()),
            row_style: Some(TableRowStyleOption::default()),
            cell_style: Some(TableCellStyleOption::default()),
            width: None,
            height: None,
            left: None,
            top: None,
            grid_index: Some(0),
            auto_fit_grid: Some(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableHeaderOption {
    pub show: Option<bool>,
    pub height: Option<f64>,
    pub style: Option<TextStyleOption>,
    pub background_color: Option<ColorOption>,
    pub align: Option<TextAlignOption>,
}

impl Default for TableHeaderOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            height: Some(40.0),
            style: Some(TextStyleOption {
                color: Some(ColorOption::new(51, 51, 51)),
                font_size: Some(14.0),
                font_family: Some("Arial, sans-serif".to_string()),
                font_weight: Some(FontWeight::Named(FontWeightNamed::Bold)),
                font_style: None,
                align: None,
                vertical_align: None,
            }),
            background_color: Some(ColorOption::new(248, 248, 248)),
            align: Some(TextAlignOption::Center),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBodyOption {
    pub show: Option<bool>,
    pub style: Option<TextStyleOption>,
    pub row_height: Option<f64>,
    pub even_row_background_color: Option<ColorOption>,
    pub odd_row_background_color: Option<ColorOption>,
    pub align: Option<TextAlignOption>,
}

impl Default for TableBodyOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            style: Some(TextStyleOption {
                color: Some(ColorOption::new(51, 51, 51)),
                font_size: Some(12.0),
                font_family: Some("Arial, sans-serif".to_string()),
                font_weight: Some(FontWeight::Named(FontWeightNamed::Normal)),
                font_style: None,
                align: None,
                vertical_align: None,
            }),
            row_height: Some(32.0),
            even_row_background_color: Some(ColorOption::new(255, 255, 255)),
            odd_row_background_color: Some(ColorOption::new(250, 250, 250)),
            align: Some(TextAlignOption::Center),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRowStyleOption {
    pub border_color: Option<ColorOption>,
    pub border_width: Option<f64>,
}

impl Default for TableRowStyleOption {
    fn default() -> Self {
        Self {
            border_color: Some(ColorOption::new(220, 220, 220)),
            border_width: Some(1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCellStyleOption {
    pub padding: Option<f64>,
}

impl Default for TableCellStyleOption {
    fn default() -> Self {
        Self { padding: Some(8.0) }
    }
}

/// Abstract series type that dispatches to concrete variants (bar, line, pie, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SeriesOption {
    #[serde(rename = "line")]
    Line(LineSeriesOption),
    #[serde(rename = "bar")]
    Bar(BarSeriesOption),
    #[serde(rename = "candlestick")]
    Candlestick(CandlestickSeriesOption),
    #[serde(rename = "pie")]
    Pie(PieSeriesOption),
    #[serde(rename = "scatter")]
    Scatter(ScatterSeriesOption),
    #[serde(rename = "radar")]
    Radar(RadarSeriesOption),
    #[serde(rename = "polarBar")]
    PolarBar(PolarBarSeriesOption),
    #[serde(rename = "polarScatter")]
    PolarScatter(PolarScatterSeriesOption),
    #[serde(rename = "bubble")]
    Bubble(BubbleSeriesOption),
    #[serde(rename = "gauge")]
    Gauge(GaugeSeriesOption),
    #[serde(rename = "table")]
    Table(TableSeriesOption),
}

/// Line series configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineSeriesOption {
    pub name: Option<String>,
    pub data: Vec<DataPoint>,
    pub stack: Option<String>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    pub smooth: Option<bool>,
    pub symbol: Option<SymbolType>,
    pub symbol_size: Option<f64>,
    pub line_style: Option<LineStyleOption>,
    pub item_style: Option<ItemStyleOption>,
    pub area_style: Option<AreaStyleOption>,
    /// Optional data sampling for large datasets.
    pub sampling: Option<SamplingOption>,
}

impl Default for LineSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            stack: None,
            y_axis_index: None,
            grid_index: None,
            smooth: Some(false),
            symbol: Some(SymbolType::Circle),
            symbol_size: Some(4.0),
            line_style: None,
            item_style: None,
            area_style: None,
            sampling: None,
        }
    }
}

impl LineSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<DataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    pub fn smooth(mut self, smooth: bool) -> Self {
        self.smooth = Some(smooth);
        self
    }

    pub fn stack(mut self, stack: impl Into<String>) -> Self {
        self.stack = Some(stack.into());
        self
    }

    pub fn area_style(mut self, style: AreaStyleOption) -> Self {
        self.area_style = Some(style);
        self
    }

    pub fn sampling(mut self, sampling: SamplingOption) -> Self {
        self.sampling = Some(sampling);
        self
    }
}

/// Bar/column series configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct BarSeriesOption {
    pub name: Option<String>,
    pub data: Vec<DataPoint>,
    pub stack: Option<String>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    pub bar_width: Option<String>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    /// 分组索引，自动分组时无需设置
    pub group_index: Option<usize>,
    /// Optional data sampling for large datasets.
    pub sampling: Option<SamplingOption>,
}

impl BarSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<DataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    pub fn stack(mut self, stack: impl Into<String>) -> Self {
        self.stack = Some(stack.into());
        self
    }

    pub fn sampling(mut self, sampling: SamplingOption) -> Self {
        self.sampling = Some(sampling);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct CandlestickSeriesOption {
    pub name: Option<String>,
    pub data: Vec<CandlestickDataPoint>,
    pub x_axis_index: Option<usize>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    pub item_style: Option<CandlestickItemStyleOption>,
    pub label: Option<LabelOption>,
    /// Optional data sampling for large datasets.
    pub sampling: Option<SamplingOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandlestickDataPoint {
    pub open: f64,
    pub close: f64,
    pub low: f64,
    pub high: f64,
    pub name: Option<String>,
}

impl CandlestickDataPoint {
    pub fn new(open: f64, close: f64, low: f64, high: f64) -> Self {
        Self {
            open,
            close,
            low,
            high,
            name: None,
        }
    }

    pub fn is_up(&self) -> bool {
        self.close >= self.open
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct CandlestickItemStyleOption {
    pub color: Option<ColorOption>,
    pub color0: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_color0: Option<ColorOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PieSeriesOption {
    pub name: Option<String>,
    pub data: Vec<DataPoint>,
    pub radius: Option<Vec<String>>,
    pub center: Option<Vec<String>>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    pub grid_index: Option<usize>,
}

impl Default for PieSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            radius: Some(vec!["0%".to_string(), "75%".to_string()]),
            center: Some(vec!["50%".to_string(), "50%".to_string()]),
            item_style: None,
            label: None,
            grid_index: None,
        }
    }
}

impl PieSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<DataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

/// Scatter series configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatterSeriesOption {
    pub name: Option<String>,
    pub data: Vec<DataPoint>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    pub symbol_size: Option<f64>,
    pub item_style: Option<ItemStyleOption>,
    /// Optional data sampling for large datasets.
    pub sampling: Option<SamplingOption>,
}

impl ScatterSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<DataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    pub fn sampling(mut self, sampling: SamplingOption) -> Self {
        self.sampling = Some(sampling);
        self
    }
}

impl Default for ScatterSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            y_axis_index: None,
            grid_index: None,
            symbol_size: Some(10.0),
            item_style: None,
            sampling: None,
        }
    }
}

// ============================================================
// RadarOption - 雷达图配置
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct RadarIndicatorOption {
    pub name: Option<String>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarNameOption {
    pub show: Option<bool>,
    pub formatter: Option<String>,
    pub text_style: Option<TextStyleOption>,
}

impl Default for RadarNameOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            formatter: None,
            text_style: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarOption {
    pub indicator: Option<Vec<RadarIndicatorOption>>,
    pub center: Option<Vec<String>>,
    pub radius: Option<Vec<String>>,
    pub split_number: Option<usize>,
    pub name: Option<RadarNameOption>,
}

impl Default for RadarOption {
    fn default() -> Self {
        Self {
            indicator: None,
            center: Some(vec!["50%".to_string(), "50%".to_string()]),
            radius: Some(vec!["0%".to_string(), "75%".to_string()]),
            split_number: Some(5),
            name: None,
        }
    }
}

// ============================================================
// RadarSeriesOption - 雷达系列
// ============================================================

/// Radar series configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarSeriesOption {
    pub name: Option<String>,
    pub data: Vec<RadarDataOption>,
    pub item_style: Option<ItemStyleOption>,
    pub line_style: Option<LineStyleOption>,
    pub area_style: Option<AreaStyleOption>,
    pub symbol: Option<SymbolType>,
    pub symbol_size: Option<f64>,
}

impl Default for RadarSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            item_style: None,
            line_style: None,
            area_style: None,
            symbol: Some(SymbolType::Circle),
            symbol_size: Some(4.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarDataOption {
    pub value: Vec<f64>,
    pub name: Option<String>,
}

// ============================================================
// PolarBarSeriesOption - 极坐标柱状图系列
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolarBarSeriesOption {
    pub name: Option<String>,
    pub data: Vec<DataPoint>,
    pub item_style: Option<ItemStyleOption>,
    /// 每个扇区的颜色，按数据索引
    pub color: Option<Vec<ColorOption>>,
    /// 扇区之间的间隔（角度，单位：度）
    pub pad_angle: Option<f64>,
    /// 起始角度（单位：度，0表示12点钟方向）
    pub start_angle: Option<f64>,
}

impl Default for PolarBarSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            item_style: None,
            color: None,
            pad_angle: Some(2.0),
            start_angle: Some(0.0),
        }
    }
}

impl PolarBarSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<DataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

// ============================================================
// PolarScatterSeriesOption - 极坐标散点图系列
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolarScatterSeriesOption {
    pub name: Option<String>,
    /// 数据格式：[角度, 半径] 或 [角度, 半径, 大小]
    pub data: Vec<PolarScatterDataPoint>,
    pub item_style: Option<ItemStyleOption>,
    pub symbol: Option<SymbolType>,
    /// 默认符号大小
    pub symbol_size: Option<f64>,
}

impl Default for PolarScatterSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            item_style: None,
            symbol: Some(SymbolType::Circle),
            symbol_size: Some(10.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolarScatterDataPoint {
    /// 角度（单位：度，0表示12点钟方向，顺时针）
    pub angle: f64,
    /// 半径值
    pub radius: f64,
    /// 可选的符号大小（覆盖 series 的 symbol_size）
    pub symbol_size: Option<f64>,
    /// 可选的名称
    pub name: Option<String>,
}

// ============================================================
// BubbleSeriesOption - 气泡图系列
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BubbleSeriesOption {
    pub name: Option<String>,
    /// 数据格式：[x, y, size] 或 [x, y]
    pub data: Vec<BubbleDataPoint>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    /// 气泡大小缩放因子
    pub symbol_size_scale: Option<f64>,
    pub item_style: Option<ItemStyleOption>,
}

impl Default for BubbleSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            y_axis_index: None,
            grid_index: None,
            symbol_size_scale: Some(1.0),
            item_style: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleDataPoint {
    pub x: f64,
    pub y: f64,
    /// 气泡大小（可选，默认使用固定大小）
    pub size: Option<f64>,
    /// 可选的名称
    pub name: Option<String>,
}

// ============================================================
// GaugeSeriesOption - 仪表盘系列
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeSeriesOption {
    pub name: Option<String>,
    /// 当前值
    pub data: Vec<GaugeDataPoint>,
    /// 最小值
    pub min: Option<f64>,
    /// 最大值
    pub max: Option<f64>,
    /// 中心位置（百分比）
    pub center: Option<Vec<String>>,
    /// 半径（百分比）
    pub radius: Option<String>,
    /// 起始角度（默认-225度，即7:30方向）
    pub start_angle: Option<f64>,
    /// 结束角度（默认45度，即4:30方向）
    pub end_angle: Option<f64>,
    /// 分割段数
    pub split_number: Option<usize>,
    /// 轴线样式
    pub axis_line: Option<GaugeAxisLineOption>,
    /// 指针样式
    pub pointer: Option<GaugePointerOption>,
    /// 刻度样式
    pub axis_tick: Option<GaugeAxisTickOption>,
    /// 刻度标签
    pub axis_label: Option<GaugeAxisLabelOption>,
    /// 分隔线
    pub split_line: Option<GaugeSplitLineOption>,
    /// 标题
    pub title: Option<GaugeTitleOption>,
    /// 详情（数值显示）
    pub detail: Option<GaugeDetailOption>,
    /// 渐变色配置
    pub gradient_colors: Option<Vec<GradientColorStopOption>>,
}

impl Default for GaugeSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: vec![GaugeDataPoint {
                value: 0.0,
                name: None,
            }],
            min: Some(0.0),
            max: Some(100.0),
            center: Some(vec!["50%".to_string(), "50%".to_string()]),
            radius: Some("75%".to_string()),
            start_angle: Some(-225.0),
            end_angle: Some(45.0),
            split_number: Some(10),
            axis_line: None,
            pointer: None,
            axis_tick: None,
            axis_label: None,
            split_line: None,
            title: None,
            detail: None,
            gradient_colors: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeDataPoint {
    pub value: f64,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeAxisLineOption {
    pub show: Option<bool>,
    pub line_style: Option<LineStyleOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GaugePointerOption {
    pub show: Option<bool>,
    pub length: Option<String>,
    pub width: Option<f64>,
    pub item_style: Option<ItemStyleOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeAxisTickOption {
    pub show: Option<bool>,
    pub length: Option<f64>,
    pub line_style: Option<LineStyleOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeAxisLabelOption {
    pub show: Option<bool>,
    pub distance: Option<f64>,
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeSplitLineOption {
    pub show: Option<bool>,
    pub length: Option<f64>,
    pub line_style: Option<LineStyleOption>,
}

/// Gauge title label options.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GaugeTitleOption {
    pub show: Option<bool>,
    pub offset_center: Option<Vec<String>>,
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GaugeDetailOption {
    pub show: Option<bool>,
    pub formatter: Option<String>,
    pub offset_center: Option<Vec<String>>,
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradientColorStopOption {
    pub offset: f64,
    pub color: String,
}

// ============================================================
// DataPoint
// ============================================================

/// A resolved data point in one of three forms:
///
/// | Variant | Meaning | JSON repr |
/// |---------|---------|-----------|
/// | `Value(v)` | value only, key from category index | `1.0` |
/// | `Named(n, v)` | named data point | `["label", 1.0]` |
/// | `XY(x, v)` | x-y data point | `[-1.0, 1.0]` |
///
/// Use [`Into<DataPoint>`] conversions to create instances conveniently:
/// ```
/// # use liecharts::option::DataPoint;
/// let a: DataPoint = 42.0.into();           // Value
/// let b: DataPoint = ("Jan", 30.0).into();  // Named
/// let c: DataPoint = (-1.0, 1.0).into();    // XY
/// ```
#[derive(Debug, Clone)]
pub enum DataPoint {
    /// Value-only data point; x position is derived from category index.
    Value(f64),
    /// Named data point: `(category_name, value)`.
    Named(String, f64),
    /// X-Y data point: `(x, y)` for value-axis plots.
    XY(f64, f64),
}

// ── helper accessors ──

impl DataPoint {
    /// If this is a `Value`, returns the contained number.
    pub fn as_value(&self) -> Option<f64> {
        match self {
            DataPoint::Value(n) => Some(*n),
            _ => None,
        }
    }

    /// If this is a `Named`, returns the name and value.
    pub fn as_named(&self) -> Option<(&str, f64)> {
        match self {
            DataPoint::Named(n, v) => Some((n.as_str(), *v)),
            _ => None,
        }
    }

    /// If this is an `XY`, returns the x and y.
    pub fn as_xy(&self) -> Option<(f64, f64)> {
        match self {
            DataPoint::XY(x, y) => Some((*x, *y)),
            _ => None,
        }
    }
}

// ── From impls ──

impl From<f64> for DataPoint {
    fn from(v: f64) -> Self {
        DataPoint::Value(v)
    }
}

impl From<(f64, f64)> for DataPoint {
    fn from((x, y): (f64, f64)) -> Self {
        DataPoint::XY(x, y)
    }
}

impl From<(&str, f64)> for DataPoint {
    fn from((name, value): (&str, f64)) -> Self {
        DataPoint::Named(name.to_string(), value)
    }
}

impl From<(String, f64)> for DataPoint {
    fn from((name, value): (String, f64)) -> Self {
        DataPoint::Named(name, value)
    }
}

// ── Serialize ──

impl Serialize for DataPoint {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            DataPoint::Value(v) => v.serialize(serializer),
            DataPoint::Named(n, v) => (n.as_str(), v).serialize(serializer),
            DataPoint::XY(x, y) => (x, y).serialize(serializer),
        }
    }
}

// ── Deserialize ──

struct DataPointVisitor;

impl<'de> Visitor<'de> for DataPointVisitor {
    type Value = DataPoint;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a number, a [key, value] array, or a {name, value} object")
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<DataPoint, E> {
        Ok(DataPoint::Value(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<DataPoint, E> {
        Ok(DataPoint::Value(v as f64))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<DataPoint, E> {
        Ok(DataPoint::Value(v as f64))
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<DataPoint, A::Error> {
        use serde::de::Error;
        let first = seq
            .next_element::<serde_json::Value>()?
            .ok_or_else(|| A::Error::custom("expected at least 2 elements"))?;
        let second = seq
            .next_element::<serde_json::Value>()?
            .ok_or_else(|| A::Error::custom("expected at least 2 elements"))?;

        match first {
            serde_json::Value::String(s) => {
                let value = second
                    .as_f64()
                    .ok_or_else(|| A::Error::custom("second element must be a number"))?;
                Ok(DataPoint::Named(s, value))
            }
            serde_json::Value::Number(n) => {
                let x = n
                    .as_f64()
                    .ok_or_else(|| A::Error::custom("first element must be a valid number"))?;
                let value = second
                    .as_f64()
                    .ok_or_else(|| A::Error::custom("second element must be a number"))?;
                Ok(DataPoint::XY(x, value))
            }
            other => Err(A::Error::custom(format!(
                "unexpected array element type: {:?}",
                other
            ))),
        }
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<DataPoint, A::Error> {
        use serde::de::Error;
        let mut name: Option<String> = None;
        let mut value: Option<f64> = None;
        let mut x: Option<f64> = None;

        while let Some(k) = map.next_key::<String>()? {
            match k.as_str() {
                "name" => name = Some(map.next_value()?),
                "value" => value = Some(map.next_value()?),
                "x" => x = Some(map.next_value()?),
                _ => {
                    let _: serde_json::Value = map.next_value()?;
                }
            }
        }

        let value = value.ok_or_else(|| A::Error::custom("missing 'value' field"))?;
        match x {
            Some(xv) => Ok(DataPoint::XY(xv, value)),
            None => Ok(DataPoint::Named(name.unwrap_or_default(), value)),
        }
    }
}

impl<'de> Deserialize<'de> for DataPoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(DataPointVisitor)
    }
}

/// Data label configuration for series items.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LabelOption {
    pub show: Option<bool>,
    pub position: Option<LabelPosition>,
    pub formatter: Option<String>,
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
}

/// Item style configuration (fill color, stroke color, opacity, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemStyleOption {
    pub color: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<f64>,
    pub opacity: Option<f64>,
}

impl Default for ItemStyleOption {
    fn default() -> Self {
        Self {
            color: None,
            border_color: None,
            border_width: None,
            opacity: Some(1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaStyleOption {
    pub color: Option<ColorOption>,
    pub opacity: Option<f64>,
}

impl Default for AreaStyleOption {
    fn default() -> Self {
        Self {
            color: None,
            opacity: Some(0.5),
        }
    }
}

/// 文本对齐配置 - 用于 option 层的序列化
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum TextAlignOption {
    #[default]
    Left,
    Center,
    Right,
}

impl From<TextAlignOption> for crate::visual::TextAlign {
    fn from(option: TextAlignOption) -> Self {
        match option {
            TextAlignOption::Left => crate::visual::TextAlign::Left,
            TextAlignOption::Center => crate::visual::TextAlign::Center,
            TextAlignOption::Right => crate::visual::TextAlign::Right,
        }
    }
}

// ============================================================
// Position 枚举 - 支持预设值、像素值、百分比值
// ============================================================

/// 预设位置值
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PositionPreset {
    Auto,
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

/// 位置枚举 - 支持预设值、像素值或百分比值
#[derive(Debug, Clone, PartialEq)]
pub enum PositionOption {
    Preset(PositionPreset),
    Pixel(f64),
    Percent(f64),
}

impl PositionOption {
    pub fn auto() -> Self {
        PositionOption::Preset(PositionPreset::Auto)
    }
    pub fn center() -> Self {
        PositionOption::Preset(PositionPreset::Center)
    }
    pub fn left() -> Self {
        PositionOption::Preset(PositionPreset::Left)
    }
    pub fn right() -> Self {
        PositionOption::Preset(PositionPreset::Right)
    }
    pub fn top() -> Self {
        PositionOption::Preset(PositionPreset::Top)
    }
    pub fn bottom() -> Self {
        PositionOption::Preset(PositionPreset::Bottom)
    }
    pub fn px(value: f64) -> Self {
        PositionOption::Pixel(value)
    }
    pub fn percent(value: f64) -> Self {
        PositionOption::Percent(value)
    }
}

impl Default for PositionOption {
    fn default() -> Self {
        PositionOption::Preset(PositionPreset::Auto)
    }
}

impl Serialize for PositionOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PositionOption::Preset(p) => {
                let s = match p {
                    PositionPreset::Auto => "auto",
                    PositionPreset::Center => "center",
                    PositionPreset::Left => "left",
                    PositionPreset::Right => "right",
                    PositionPreset::Top => "top",
                    PositionPreset::Bottom => "bottom",
                };
                serializer.serialize_str(s)
            }
            PositionOption::Pixel(v) => serializer.serialize_f64(*v),
            PositionOption::Percent(v) => serializer.serialize_str(&format!("{}%", v)),
        }
    }
}

impl<'de> Deserialize<'de> for PositionOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PositionVisitor;

        impl<'de> Visitor<'de> for PositionVisitor {
            type Value = PositionOption;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a position value: preset string, number, or percentage string")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<PositionOption, E> {
                if value.ends_with('%') {
                    let v = value
                        .trim_end_matches('%')
                        .parse::<f64>()
                        .map_err(|_| de::Error::custom(format!("invalid percentage: {}", value)))?;
                    Ok(PositionOption::Percent(v))
                } else {
                    match value {
                        "auto" => Ok(PositionOption::Preset(PositionPreset::Auto)),
                        "center" => Ok(PositionOption::Preset(PositionPreset::Center)),
                        "left" => Ok(PositionOption::Preset(PositionPreset::Left)),
                        "right" => Ok(PositionOption::Preset(PositionPreset::Right)),
                        "top" => Ok(PositionOption::Preset(PositionPreset::Top)),
                        "bottom" => Ok(PositionOption::Preset(PositionPreset::Bottom)),
                        _ => {
                            if let Ok(v) = value.parse::<f64>() {
                                Ok(PositionOption::Pixel(v))
                            } else {
                                Err(de::Error::custom(format!("invalid position: {}", value)))
                            }
                        }
                    }
                }
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<PositionOption, E> {
                Ok(PositionOption::Pixel(value))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<PositionOption, E> {
                Ok(PositionOption::Pixel(value as f64))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<PositionOption, E> {
                Ok(PositionOption::Pixel(value as f64))
            }
        }

        deserializer.deserialize_any(PositionVisitor)
    }
}

// ============================================================
// ColorOption - 颜色类型，支持从 "#RRGGBB" / "#RRGGBBAA" 解析
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorOption {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorOption {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                Some(Self::new(r * 17, g * 17, b * 17))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::new(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::with_alpha(r, g, b, a))
            }
            _ => None,
        }
    }

    fn hex_string(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

impl Default for ColorOption {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl Serialize for ColorOption {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.hex_string())
    }
}

impl<'de> Deserialize<'de> for ColorOption {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = ColorOption;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hex color string like #RRGGBB or #RRGGBBAA")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<ColorOption, E> {
                ColorOption::from_hex(value)
                    .ok_or_else(|| de::Error::custom(format!("invalid color: {}", value)))
            }
        }

        deserializer.deserialize_str(ColorVisitor)
    }
}

// ============================================================
// NameLocation 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum NameLocation {
    Start,
    Middle,
    Center,
    #[default]
    End,
}

// ============================================================
// Orient 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum Orient {
    #[default]
    Horizontal,
    Vertical,
}

// ============================================================
// LineType 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum LineType {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

// ============================================================
// FontWeight 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FontWeightNamed {
    Normal,
    Bold,
    Bolder,
    Lighter,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FontWeight {
    Named(FontWeightNamed),
    Numeric(u16),
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Named(FontWeightNamed::Normal)
    }
}

// ============================================================
// SymbolType 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum SymbolType {
    #[default]
    Circle,
    Rect,
    RoundRect,
    Triangle,
    Diamond,
    Pin,
    Arrow,
    None,
}

// ============================================================
// LabelPosition 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum LabelPosition {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
    Inside,
    Outside,
    Center,
}

// ============================================================
// AxisType 枚举
// ============================================================

/// Axis type: category, value, or time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum AxisType {
    #[default]
    Category,
    Value,
    Time,
    Log,
}

// ============================================================
// AxisPosition 枚举 - 坐标轴位置
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum AxisPosition {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

// ============================================================
// FontStyle 枚举 - 字体风格
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

// ============================================================
// LabelAlign 枚举 - 标签水平对齐
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum LabelAlign {
    Left,
    #[default]
    Center,
    Right,
}

// ============================================================
// LabelVerticalAlign 枚举 - 标签垂直对齐
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum LabelVerticalAlign {
    Top,
    #[default]
    Middle,
    Bottom,
}
