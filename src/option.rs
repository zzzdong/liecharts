use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};
use std::collections::HashMap;
use std::fmt;

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
pub enum Position {
    Preset(PositionPreset),
    Pixel(f64),
    Percent(f64),
}

impl Position {
    pub fn auto() -> Self {
        Position::Preset(PositionPreset::Auto)
    }
    pub fn center() -> Self {
        Position::Preset(PositionPreset::Center)
    }
    pub fn left() -> Self {
        Position::Preset(PositionPreset::Left)
    }
    pub fn right() -> Self {
        Position::Preset(PositionPreset::Right)
    }
    pub fn top() -> Self {
        Position::Preset(PositionPreset::Top)
    }
    pub fn bottom() -> Self {
        Position::Preset(PositionPreset::Bottom)
    }
    pub fn px(value: f64) -> Self {
        Position::Pixel(value)
    }
    pub fn percent(value: f64) -> Self {
        Position::Percent(value)
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::Preset(PositionPreset::Auto)
    }
}

impl Serialize for Position {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Position::Preset(p) => {
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
            Position::Pixel(v) => serializer.serialize_f64(*v),
            Position::Percent(v) => serializer.serialize_str(&format!("{}%", v)),
        }
    }
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PositionVisitor;

        impl<'de> Visitor<'de> for PositionVisitor {
            type Value = Position;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a position value: preset string, number, or percentage string")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Position, E> {
                if value.ends_with('%') {
                    let v = value
                        .trim_end_matches('%')
                        .parse::<f64>()
                        .map_err(|_| de::Error::custom(format!("invalid percentage: {}", value)))?;
                    Ok(Position::Percent(v))
                } else {
                    match value {
                        "auto" => Ok(Position::Preset(PositionPreset::Auto)),
                        "center" => Ok(Position::Preset(PositionPreset::Center)),
                        "left" => Ok(Position::Preset(PositionPreset::Left)),
                        "right" => Ok(Position::Preset(PositionPreset::Right)),
                        "top" => Ok(Position::Preset(PositionPreset::Top)),
                        "bottom" => Ok(Position::Preset(PositionPreset::Bottom)),
                        _ => {
                            if let Ok(v) = value.parse::<f64>() {
                                Ok(Position::Pixel(v))
                            } else {
                                Err(de::Error::custom(format!("invalid position: {}", value)))
                            }
                        }
                    }
                }
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Position, E> {
                Ok(Position::Pixel(value))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Position, E> {
                Ok(Position::Pixel(value as f64))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Position, E> {
                Ok(Position::Pixel(value as f64))
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

    fn to_hex_string(&self) -> String {
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
        serializer.serialize_str(&self.to_hex_string())
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


// ============================================================
// 配置结构体
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct LieChartOption {
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


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleOption {
    pub text: Option<String>,
    pub subtext: Option<String>,
    pub left: Option<Position>,
    pub top: Option<Position>,
    pub text_style: Option<TextStyleOption>,
    pub subtext_style: Option<TextStyleOption>,
}

impl Default for TitleOption {
    fn default() -> Self {
        Self {
            text: None,
            subtext: None,
            left: Some(Position::center()),
            top: Some(Position::auto()),
            text_style: None,
            subtext_style: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LegendOption {
    pub show: Option<bool>,
    pub data: Option<Vec<String>>,
    pub left: Option<Position>,
    pub top: Option<Position>,
    pub orient: Option<Orient>,
    pub text_style: Option<TextStyleOption>,
    pub item_width: Option<f64>,
    pub item_height: Option<f64>,
    pub symbol_size: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridOption {
    pub left: Option<Position>,
    pub right: Option<Position>,
    pub top: Option<Position>,
    pub bottom: Option<Position>,
    pub contain_label: Option<bool>,
}

impl Default for GridOption {
    fn default() -> Self {
        Self {
            left: Some(Position::percent(10.0)),
            right: Some(Position::percent(10.0)),
            top: Some(Position::percent(15.0)),
            bottom: Some(Position::percent(15.0)),
            contain_label: Some(true),
        }
    }
}

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
        Self {
            padding: Some(8.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
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
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatterSeriesOption {
    pub name: Option<String>,
    pub data: Vec<DataPoint>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    pub symbol_size: Option<f64>,
    pub item_style: Option<ItemStyleOption>,
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
            data: vec![GaugeDataPoint { value: 0.0, name: None }],
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataPoint {
    Number(f64),
    Array(Vec<serde_json::Value>),
    Object(HashMap<String, serde_json::Value>),
}

impl DataPoint {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            DataPoint::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<serde_json::Value>> {
        match self {
            DataPoint::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, serde_json::Value>> {
        match self {
            DataPoint::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

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