use crate::option::{
    AxisOption, AxisPosition, ColorOption, DataPoint, GridOption, ItemStyleOption,
    LegendOption, LineStyleOption, TextStyleOption,
    TitleOption, LieChartOption, SeriesOption,
    FontStyle as FontStyleOption, FontWeight, FontWeightNamed, SymbolType as OptionSymbolType, LabelPosition as OptionLabelPosition,
    LabelAlign, LabelVerticalAlign, TextAlignOption,
    RadarOption,
};
use crate::visual::{Color, TextAlign, TextBaseline};
use crate::theme::Theme;
use crate::ChartError;

impl From<ColorOption> for Color {
    fn from(c: ColorOption) -> Self {
        Color::with_alpha(c.r, c.g, c.b, c.a)
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedOption {
    pub title: Option<Title>,
    pub legend: Option<Legend>,
    pub grids: Vec<Grid>,
    pub radar: Option<RadarConfig>,
    pub x_axes: Vec<Axis>,
    pub y_axes: Vec<Axis>,
    pub series: Vec<ResolvedSeries>,
    pub colors: Vec<Color>,
    pub background: Color,
    pub text_style: Option<TextStyle>,
}

#[derive(Debug, Clone)]
pub struct Title {
    pub text: String,
    pub subtext: Option<String>,
    pub left: Position,
    pub top: Position,
    pub text_style: TextStyle,
    pub subtext_style: Option<TextStyle>,
}

#[derive(Debug, Clone)]
pub struct Legend {
    pub show: bool,
    pub data: Vec<String>,
    pub left: Position,
    pub top: Position,
    pub orient: Orient,
    pub text_style: TextStyle,
    pub item_width: f64,
    pub item_height: f64,
    pub symbol_size: f64,
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub left: Position,
    pub right: Position,
    pub top: Position,
    pub bottom: Position,
    pub contain_label: bool,
}

#[derive(Debug, Clone)]
pub struct Axis {
    pub axis_type: AxisType,
    pub data: Option<Vec<String>>,
    pub name: Option<String>,
    pub name_location: NameLocation,
    pub name_text_style: TextStyle,
    pub axis_label: AxisLabel,
    pub axis_line: AxisLine,
    pub axis_tick: AxisTick,
    pub split_line: SplitLine,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub boundary_gap: bool,
    pub position: AxisPosition,
    pub grid_index: usize,
    /// 刻度线长度（默认 5px）
    pub tick_length: f64,
    /// 刻度末端到标签文本的间隙（默认 5px）
    pub label_padding: f64,
    /// 名称与标签区域之间的间隙（默认 5px）
    pub name_gap: f64,
    /// 名称相对于轴线的位置（外/内侧）
    pub name_side: AxisNameSide,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisType {
    Category,
    Value,
    Time,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NameLocation {
    Start,
    Middle,
    End,
}

/// 轴名称相对于轴线的位置（内侧/外侧）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisNameSide {
    Outside, // 轴线外侧（默认）
    Inside,  // 轴线内侧（靠近绘图区）
}

#[derive(Debug, Clone)]
pub struct AxisLabel {
    pub show: bool,
    pub rotate: f64,
    pub formatter: Option<String>,
    pub color: Color,
    pub font_size: f64,
    pub font_family: String,
    pub font_weight: FontWeight,
    pub align: LabelAlign,
    pub vertical_align: LabelVerticalAlign,
    pub margin: f64,
}

#[derive(Debug, Clone)]
pub struct AxisLine {
    pub show: bool,
    pub line_style: LineStyle,
}

#[derive(Debug, Clone)]
pub struct AxisTick {
    pub show: bool,
    pub align_with_label: bool,
    pub line_style: LineStyle,
}

#[derive(Debug, Clone)]
pub struct SplitLine {
    pub show: bool,
    pub line_style: LineStyle,
}

#[derive(Debug, Clone)]
pub struct LineStyle {
    pub color: Color,
    pub width: f64,
    pub line_type: LineType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub color: Color,
    pub font_size: f64,
    pub font_family: String,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub align: Option<TextAlign>,
    pub vertical_align: Option<TextBaseline>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Color::new(0, 0, 0),
            font_size: 12.0,
            font_family: "sans-serif".to_string(),
            font_weight: FontWeight::Named(FontWeightNamed::Normal),
            font_style: FontStyle::Normal,
            align: None,
            vertical_align: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedSeries {
    Line(LineSeries),
    Bar(BarSeries),
    Candlestick(CandlestickSeries),
    Pie(PieSeries),
    Scatter(ScatterSeries),
    Radar(RadarSeries),
    PolarBar(PolarBarSeries),
    PolarScatter(PolarScatterSeries),
    Bubble(BubbleSeries),
    Gauge(GaugeSeries),
    Table(TableSeries),
}

#[derive(Debug, Clone)]
pub struct LineSeries {
    pub name: String,
    pub data: Vec<DataItem>,
    pub stack: Option<String>,
    pub y_axis_index: usize,
    pub grid_index: usize,
    pub smooth: bool,
    pub symbol: Symbol,
    pub symbol_size: f64,
    pub line_style: LineStyle,
    pub item_style: ItemStyle,
    pub area_style: Option<AreaStyle>,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct BarSeries {
    pub name: String,
    pub data: Vec<DataItem>,
    pub stack: Option<String>,
    pub y_axis_index: usize,
    pub grid_index: usize,
    pub bar_width: Option<f64>,
    pub item_style: ItemStyle,
    pub label: Option<Label>,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct CandlestickSeries {
    pub name: String,
    pub data: Vec<CandlestickDataItem>,
    pub x_axis_index: usize,
    pub y_axis_index: usize,
    pub grid_index: usize,
    pub item_style: CandlestickItemStyle,
    pub label: Option<Label>,
    pub color_up: Color,
    pub color_down: Color,
}

#[derive(Debug, Clone)]
pub struct CandlestickDataItem {
    pub open: f64,
    pub close: f64,
    pub low: f64,
    pub high: f64,
    pub name: Option<String>,
}

impl CandlestickDataItem {
    pub fn is_up(&self) -> bool {
        self.close >= self.open
    }
}

#[derive(Debug, Clone)]
pub struct CandlestickItemStyle {
    pub color: Option<Color>,
    pub color0: Option<Color>,
    pub border_color: Option<Color>,
    pub border_color0: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct PieSeries {
    pub name: String,
    pub data: Vec<DataItem>,
    pub radius: (f64, f64),
    pub center: (f64, f64),
    pub item_style: ItemStyle,
    pub label: Option<Label>,
    pub grid_index: usize,
}

#[derive(Debug, Clone)]
pub struct ScatterDataItem {
    pub x: f64,
    pub y: f64,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScatterSeries {
    pub name: String,
    pub data: Vec<ScatterDataItem>,
    pub y_axis_index: usize,
    pub grid_index: usize,
    pub symbol_size: f64,
    pub item_style: ItemStyle,
    pub color: Color,
}

// ============================================================
// Radar 类型
// ============================================================

#[derive(Debug, Clone)]
pub struct RadarIndicator {
    pub name: String,
    pub max: f64,
}

#[derive(Debug, Clone)]
pub struct RadarNameConfig {
    pub show: bool,
    pub formatter: Option<String>,
    pub text_style: TextStyle,
}

#[derive(Debug, Clone)]
pub struct RadarConfig {
    pub indicator: Vec<RadarIndicator>,
    pub center: (f64, f64),
    pub radius: (f64, f64),
    pub split_number: usize,
    pub name: RadarNameConfig,
}

#[derive(Debug, Clone)]
pub struct RadarData {
    pub value: Vec<f64>,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RadarSeries {
    pub name: String,
    pub data: Vec<RadarData>,
    pub item_style: ItemStyle,
    pub line_style: LineStyle,
    pub area_style: Option<AreaStyle>,
    pub symbol: Symbol,
    pub symbol_size: f64,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct DataItem {
    pub name: Option<String>,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct ItemStyle {
    pub color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f64,
    pub opacity: f64,
}

#[derive(Debug, Clone)]
pub struct AreaStyle {
    pub color: Option<Color>,
    pub opacity: f64,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub show: bool,
    pub position: LabelPosition,
    pub formatter: Option<String>,
    pub color: Color,
    pub font_size: f64,
    pub font_family: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelPosition {
    Top,
    Left,
    Right,
    Bottom,
    Inside,
    Outside,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Symbol {
    Circle,
    Rect,
    RoundRect,
    Triangle,
    Diamond,
    Pin,
    Arrow,
    None,
}

#[derive(Debug, Clone)]
pub enum Position {
    Auto,
    Left(f64),
    Right(f64),
    Top(f64),
    Bottom(f64),
    Center,
    Value(f64),
    Percent(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineType {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orient {
    Horizontal,
    Vertical,
}

// ============================================================
// PolarBar 类型 - 极坐标柱状图
// ============================================================

#[derive(Debug, Clone)]
pub struct PolarBarSeries {
    pub name: String,
    pub data: Vec<DataItem>,
    pub item_style: ItemStyle,
    /// 每个扇区的颜色
    pub colors: Vec<Color>,
    /// 扇区之间的间隔（角度，单位：弧度）
    pub pad_angle: f64,
    /// 起始角度（单位：弧度，-PI/2 表示12点钟方向）
    pub start_angle: f64,
}

// ============================================================
// PolarScatter 类型 - 极坐标散点图
// ============================================================

#[derive(Debug, Clone)]
pub struct PolarScatterData {
    /// 角度（单位：弧度，-PI/2 表示12点钟方向）
    pub angle: f64,
    /// 半径值（0-1 之间，相对于最大半径）
    pub radius: f64,
    /// 符号大小
    pub symbol_size: f64,
    /// 名称
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PolarScatterSeries {
    pub name: String,
    pub data: Vec<PolarScatterData>,
    pub item_style: ItemStyle,
    pub symbol: Symbol,
    pub default_symbol_size: f64,
}

// ============================================================
// Bubble 类型 - 气泡图
// ============================================================

#[derive(Debug, Clone)]
pub struct BubbleData {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BubbleSeries {
    pub name: String,
    pub data: Vec<BubbleData>,
    pub y_axis_index: usize,
    pub grid_index: usize,
    pub symbol_size_scale: f64,
    pub item_style: ItemStyle,
    pub color: Color,
}

// ============================================================
// Gauge 类型 - 仪表盘
// ============================================================

#[derive(Debug, Clone)]
pub struct GaugeSeries {
    pub name: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub center: (f64, f64),
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub split_number: usize,
    pub axis_line_show: bool,
    pub axis_line_style: LineStyle,
    pub pointer_show: bool,
    pub pointer_length: f64,
    pub pointer_width: f64,
    pub pointer_color: Color,
    pub axis_tick_show: bool,
    pub axis_tick_length: f64,
    pub axis_tick_style: LineStyle,
    pub axis_label_show: bool,
    pub axis_label_distance: f64,
    pub axis_label_color: Color,
    pub axis_label_font_size: f64,
    pub axis_label_font_family: String,
    pub axis_label_font_weight: FontWeight,
    pub split_line_show: bool,
    pub split_line_length: f64,
    pub split_line_style: LineStyle,
    pub title_show: bool,
    pub title_offset: (f64, f64),
    pub title_color: Color,
    pub title_font_size: f64,
    pub title_font_family: String,
    pub title_font_weight: FontWeight,
    pub detail_show: bool,
    pub detail_formatter: Option<String>,
    pub detail_offset: (f64, f64),
    pub detail_color: Color,
    pub detail_font_size: f64,
    pub detail_font_family: String,
    pub detail_font_weight: FontWeight,
    pub color: Color,
    pub gradient_colors: Vec<(f64, Color)>,
}

#[derive(Debug, Clone)]
pub struct TableSeries {
    pub name: String,
    pub data: Vec<Vec<serde_json::Value>>,
    pub columns: Vec<String>,
    pub header_config: TableHeaderConfig,
    pub body_config: TableBodyConfig,
    /// 表格所属的 grid 索引
    pub grid_index: usize,
    /// 是否自动调整 grid 大小以适应表格内容
    pub auto_fit_grid: bool,
}

#[derive(Debug, Clone)]
pub struct TableHeaderConfig {
    pub show: bool,
    pub height: f64,
    pub style: TextStyle,
    pub background_color: Color,
    pub align: TextAlign,
}

#[derive(Debug, Clone)]
pub struct TableBodyConfig {
    pub show: bool,
    pub row_height: f64,
    pub style: TextStyle,
    pub even_row_background_color: Color,
    pub odd_row_background_color: Color,
    pub align: TextAlign,
}

impl ResolvedOption {
    pub fn merge(option: LieChartOption, theme: Option<Theme>) -> crate::error::Result<Self> {
        let default_theme = Theme::echarts();
        let theme = &theme.unwrap_or(default_theme);

        let colors: Vec<Color> = option
            .color
            .as_ref()
            .map(|c| {
                c.iter()
                    .map(|c| Color::from(*c))
                    .collect::<Vec<_>>()
            })
            .filter(|v: &Vec<Color>| !v.is_empty())
            .unwrap_or_else(|| {
                theme
                    .color
                    .iter()
                    .filter_map(|s| Color::from_hex(s))
                    .collect()
            });

        let background = option
            .background_color
            .map(Color::from)
            .unwrap_or_else(|| theme.get_background_color());

        let title = option.title.map(|t| Self::resolve_title(t, theme));

        let legend = option.legend.map(|l| Self::resolve_legend(l, theme));
        
        // 处理多 grid：如果没有指定 grid，创建一个默认的
        let grids = if option.grid.is_empty() {
            vec![Grid {
                left: Position::Value(60.0),
                right: Position::Value(60.0),
                top: Position::Value(60.0),
                bottom: Position::Value(60.0),
                contain_label: false,
            }]
        } else {
            option.grid.into_iter().map(Self::resolve_grid).collect()
        };

        let x_axes = option
            .x_axis
            .into_iter()
            .map(|a| Self::resolve_axis(a, theme, AxisPosition::Bottom))
            .collect();

        let y_axes = option
            .y_axis
            .into_iter()
            .map(|a| Self::resolve_axis(a, theme, AxisPosition::Left))
            .collect();

        let series = option
            .series
            .into_iter()
            .enumerate()
            .map(|(idx, s)| Self::resolve_series(s, theme, &colors, idx))
            .collect::<crate::error::Result<Vec<_>>>()?;

        let radar = option.radar.map(|r| Self::resolve_radar(r, theme));

        let text_style = option.text_style.map(|s| Self::resolve_text_style(s, theme));

        Ok(Self {
            title,
            legend,
            grids,
            radar,
            x_axes,
            y_axes,
            series,
            colors,
            background,
            text_style,
        })
    }

    fn resolve_title(option: TitleOption, theme: &Theme) -> Title {
        let default_title_style = theme.get_title_text_style();
        let default_subtitle_style = theme.get_subtitle_text_style();
        
        Title {
            text: option.text.unwrap_or_default(),
            subtext: option.subtext,
            left: option
                .left
                .map(Self::convert_position)
                .unwrap_or(Position::Center),
            top: option
                .top
                .map(Self::convert_position)
                .unwrap_or(Position::Top(10.0)),
            text_style: option
                .text_style
                .map(|s| Self::resolve_text_style(s, theme))
                .unwrap_or_else(|| TextStyle {
                    color: Color::from_hex(&default_title_style.color).unwrap(),
                    font_size: default_title_style.font_size,
                    font_family: default_title_style.font_family.clone(),
                    font_weight: FontWeight::Named(FontWeightNamed::Normal),
                    ..Default::default()
                }),
            subtext_style: option
                .subtext_style
                .map(|s| Self::resolve_text_style(s, theme))
                .or_else(|| {
                    Some(TextStyle {
                        color: Color::from_hex(&default_subtitle_style.color).unwrap(),
                        font_size: default_subtitle_style.font_size,
                        font_family: default_subtitle_style.font_family.clone(),
                        font_weight: FontWeight::Named(FontWeightNamed::Normal),
                        ..Default::default()
                    })
                }),
        }
    }

    fn resolve_legend(option: LegendOption, theme: &Theme) -> Legend {
        let default_legend_style = theme.get_legend_text_style();
        let (default_item_width, default_item_height, default_symbol_size) = theme.get_legend_config();
        
        Legend {
            show: option.show.unwrap_or(true),
            data: option.data.unwrap_or_default(),
            left: option
                .left
                .map(Self::convert_position)
                .unwrap_or(Position::Center),
            top: option
                .top
                .map(Self::convert_position)
                .unwrap_or(Position::Auto),
            orient: option
                .orient
                .map(|o| match o {
                    crate::option::Orient::Vertical => Orient::Vertical,
                    crate::option::Orient::Horizontal => Orient::Horizontal,
                })
                .unwrap_or(Orient::Horizontal),
            text_style: option
                .text_style
                .map(|s| Self::resolve_text_style(s, theme))
                .unwrap_or_else(|| TextStyle {
                    color: Color::from_hex(&default_legend_style.color).unwrap(),
                    font_size: default_legend_style.font_size,
                    font_family: default_legend_style.font_family.clone(),
                    font_weight: FontWeight::Named(FontWeightNamed::Normal),
                    ..Default::default()
                }),
            item_width: option.item_width.unwrap_or(default_item_width),
            item_height: option.item_height.unwrap_or(default_item_height),
            symbol_size: option.symbol_size.unwrap_or(default_symbol_size),
        }
    }

    fn resolve_grid(option: GridOption) -> Grid {
        Grid {
            left: option
                .left
                .map(Self::convert_position)
                .unwrap_or(Position::Value(60.0)),
            right: option
                .right
                .map(Self::convert_position)
                .unwrap_or(Position::Value(60.0)),
            top: option
                .top
                .map(Self::convert_position)
                .unwrap_or(Position::Value(60.0)),
            bottom: option
                .bottom
                .map(Self::convert_position)
                .unwrap_or(Position::Value(60.0)),
            contain_label: option.contain_label.unwrap_or(false),
        }
    }

    fn convert_position(pos: crate::option::Position) -> Position {
        match pos {
            crate::option::Position::Preset(crate::option::PositionPreset::Auto) => Position::Auto,
            crate::option::Position::Preset(crate::option::PositionPreset::Center) => Position::Center,
            crate::option::Position::Preset(crate::option::PositionPreset::Left) => Position::Left(0.0),
            crate::option::Position::Preset(crate::option::PositionPreset::Right) => Position::Right(0.0),
            crate::option::Position::Preset(crate::option::PositionPreset::Top) => Position::Top(0.0),
            crate::option::Position::Preset(crate::option::PositionPreset::Bottom) => Position::Bottom(0.0),
            crate::option::Position::Pixel(v) => Position::Value(v),
            crate::option::Position::Percent(v) => Position::Percent(v),
        }
    }

    fn resolve_axis(option: AxisOption, theme: &Theme, default_position: AxisPosition) -> Axis {
        let axis_type = option.axis_type.map(|t| match t {
            crate::option::AxisType::Category => AxisType::Category,
            crate::option::AxisType::Value => AxisType::Value,
            crate::option::AxisType::Time => AxisType::Time,
            crate::option::AxisType::Log => AxisType::Log,
        }).unwrap_or(AxisType::Category);

        let default_axis_label = theme.get_axis_label_style();
        let default_axis_line = theme.get_axis_line_style();
        let default_axis_tick = theme.get_axis_tick_style();
        let default_split_line = theme.get_split_line_style();

        Axis {
            axis_type,
            data: option.data,
            name: option.name,
            name_location: option
                .name_location
                .map(|n| match n {
                    crate::option::NameLocation::Start => NameLocation::Start,
                    crate::option::NameLocation::Middle | crate::option::NameLocation::Center => NameLocation::Middle,
                    crate::option::NameLocation::End => NameLocation::End,
                })
                .unwrap_or(NameLocation::End),
            name_text_style: option
                .name_text_style
                .map(|s| Self::resolve_text_style(s, theme))
                .unwrap_or_else(|| TextStyle {
                    color: Color::from_hex(&default_axis_label.color).unwrap(),
                    font_size: default_axis_label.font_size,
                    font_family: default_axis_label.font_family.clone(),
                    font_weight: FontWeight::Named(FontWeightNamed::Normal),
                    ..Default::default()
                }),
            axis_label: option
                .axis_label
                .map(|l| AxisLabel {
                    show: l.show.unwrap_or(true),
                    rotate: l.rotate.unwrap_or(0.0),
                    formatter: l.formatter,
                    color: l
                        .color
                        .map(Color::from)
                        .unwrap_or_else(|| Color::from_hex(&default_axis_label.color).unwrap()),
                    font_size: l.font_size.unwrap_or(default_axis_label.font_size),
                    font_family: l.font_family.unwrap_or_else(|| default_axis_label.font_family.clone()),
                    font_weight: l.font_weight.unwrap_or(FontWeight::Named(FontWeightNamed::Normal)),
                    align: l.align.unwrap_or(LabelAlign::Center),
                    vertical_align: l.vertical_align.unwrap_or(LabelVerticalAlign::Middle),
                    margin: l.margin.unwrap_or(5.0),
                })
                .unwrap_or_else(|| AxisLabel {
                    show: true,
                    rotate: 0.0,
                    formatter: None,
                    color: Color::from_hex(&default_axis_label.color).unwrap(),
                    font_size: default_axis_label.font_size,
                    font_family: default_axis_label.font_family.clone(),
                    font_weight: FontWeight::Named(FontWeightNamed::Normal),
                    align: LabelAlign::Center,
                    vertical_align: LabelVerticalAlign::Middle,
                    margin: 5.0,
                }),
            axis_line: option
                .axis_line
                .map(|l| AxisLine {
                    show: l.show.unwrap_or(true),
                    line_style: l
                        .line_style
                        .map(|s| Self::resolve_line_style(s, theme, Color::from_hex(&default_axis_line.color).unwrap()))
                        .unwrap_or_else(|| LineStyle {
                            color: Color::from_hex(&default_axis_line.color).unwrap(),
                            width: default_axis_line.width,
                            line_type: LineType::Solid,
                        }),
                })
                .unwrap_or_else(|| AxisLine {
                    show: true,
                    line_style: LineStyle {
                        color: Color::from_hex(&default_axis_line.color).unwrap(),
                        width: default_axis_line.width,
                        line_type: LineType::Solid,
                    },
                }),
            axis_tick: option
                .axis_tick
                .map(|t| AxisTick {
                    show: t.show.unwrap_or(true),
                    align_with_label: t.align_with_label.unwrap_or(false),
                    line_style: t
                        .line_style
                        .map(|s| Self::resolve_line_style(s, theme, Color::from_hex(&default_axis_tick.color).unwrap()))
                        .unwrap_or_else(|| LineStyle {
                            color: Color::from_hex(&default_axis_tick.color).unwrap(),
                            width: default_axis_tick.width,
                            line_type: LineType::Solid,
                        }),
                })
                .unwrap_or_else(|| AxisTick {
                    show: true,
                    align_with_label: false,
                    line_style: LineStyle {
                        color: Color::from_hex(&default_axis_tick.color).unwrap(),
                        width: default_axis_tick.width,
                        line_type: LineType::Solid,
                    },
                }),
            split_line: option
                .split_line
                .map(|l| SplitLine {
                    show: l.show.unwrap_or(true),
                    line_style: l
                        .line_style
                        .map(|s| Self::resolve_line_style(s, theme, Color::from_hex(&default_split_line.color).unwrap()))
                        .unwrap_or_else(|| LineStyle {
                            color: Color::from_hex(&default_split_line.color).unwrap(),
                            width: default_split_line.width,
                            line_type: LineType::Solid,
                        }),
                })
                .unwrap_or_else(|| SplitLine {
                    show: true,
                    line_style: LineStyle {
                        color: Color::from_hex(&default_split_line.color).unwrap(),
                        width: default_split_line.width,
                        line_type: LineType::Solid,
                    },
                }),
            min: option.min,
            max: option.max,
            boundary_gap: option.boundary_gap.unwrap_or(axis_type == AxisType::Category),
            position: option.position.unwrap_or(default_position),
            grid_index: option.grid_index.unwrap_or(0),
            tick_length: 5.0,
            label_padding: 5.0,
            name_gap: 5.0,
            name_side: AxisNameSide::Outside,
        }
    }

    fn resolve_series(
        option: SeriesOption,
        theme: &Theme,
        colors: &[Color],
        index: usize,
    ) -> crate::error::Result<ResolvedSeries> {
        let color = colors.get(index % colors.len()).copied().unwrap_or(Color::new(0, 0, 0));

        match option {
            SeriesOption::Line(opt) => {
                let data = Self::resolve_data(&opt.data)?;
                let series_theme = theme.series.get("line");
                let line_style = opt
                    .line_style
                    .map(|s| Self::resolve_line_style(s, theme, color))
                    .unwrap_or_else(|| {
                        let line_theme = series_theme.and_then(|st| st.line_style.as_ref());
                        LineStyle {
                            color,
                            width: line_theme.map(|lt| lt.width).unwrap_or(2.0),
                            line_type: LineType::Solid,
                        }
                    });
                let series_color = line_style.color; // 使用行样式的有效颜色（用户自定义或主题色）
                Ok(ResolvedSeries::Line(LineSeries {
                    name: opt.name.unwrap_or_default(),
                    data,
                    stack: opt.stack.clone(),
                    y_axis_index: opt.y_axis_index.unwrap_or(0),
                    grid_index: opt.grid_index.unwrap_or(0),
                    smooth: opt.smooth.unwrap_or(false),
                    symbol: opt.symbol.map(Self::convert_symbol).unwrap_or(Symbol::Circle),
                    symbol_size: opt.symbol_size.unwrap_or(4.0),
                    line_style,
                    item_style: opt
                        .item_style
                        .map(|s| Self::resolve_item_style(s, theme))
                        .unwrap_or_else(|| ItemStyle {
                            color: Some(color),
                            border_color: None,
                            border_width: 0.0,
                            opacity: 1.0,
                        }),
                    area_style: opt.area_style.map(|s| AreaStyle {
                        color: s.color.map(Color::from),
                        opacity: s.opacity.unwrap_or(0.7),
                    }),
                    color: series_color,
                }))
            }
            SeriesOption::Bar(opt) => {
                let data = Self::resolve_data(&opt.data)?;
                let bar_color = opt
                    .item_style
                    .as_ref()
                    .and_then(|s| s.color.map(Color::from))
                    .unwrap_or(color);
                Ok(ResolvedSeries::Bar(BarSeries {
                    name: opt.name.unwrap_or_default(),
                    data,
                    stack: opt.stack.clone(),
                    y_axis_index: opt.y_axis_index.unwrap_or(0),
                    grid_index: opt.grid_index.unwrap_or(0),
                    bar_width: opt.bar_width.as_ref().map(|s| Self::parse_percent_or_value(s)),
                    item_style: opt
                        .item_style
                        .map(|s| Self::resolve_item_style(s, theme))
                        .unwrap_or_else(|| ItemStyle {
                            color: Some(bar_color),
                            border_color: None,
                            border_width: 0.0,
                            opacity: 1.0,
                        }),
                    label: opt.label.map(|l| Label {
                        show: l.show.unwrap_or(false),
                        position: l
                            .position
                            .map(Self::convert_label_position)
                            .unwrap_or(LabelPosition::Top),
                        formatter: l.formatter,
                        color: l
                            .color
                            .map(Color::from)
                            .unwrap_or(Color::new(0, 0, 0)),
                        font_size: l.font_size.unwrap_or(12.0),
                        font_family: l.font_family.unwrap_or_else(|| "sans-serif".to_string()),
                    }),
                    color: bar_color,
                }))
            }
            SeriesOption::Candlestick(opt) => {
                let data: Vec<CandlestickDataItem> = opt.data.iter().map(|d| {
                    CandlestickDataItem {
                        open: d.open,
                        close: d.close,
                        low: d.low,
                        high: d.high,
                        name: d.name.clone(),
                    }
                }).collect();

                let default_up_color = theme.get_theme_color(0);
                let default_down_color = Color::new(60, 179, 113);

                let item_style = opt.item_style.as_ref();
                let color_up = match item_style.and_then(|s| s.color) {
                    Some(c) => Color::from(c),
                    None => default_up_color,
                };
                let color_down = match item_style.and_then(|s| s.color0) {
                    Some(c) => Color::from(c),
                    None => default_down_color,
                };

                Ok(ResolvedSeries::Candlestick(CandlestickSeries {
                    name: opt.name.unwrap_or_default(),
                    data,
                    x_axis_index: opt.x_axis_index.unwrap_or(0),
                    y_axis_index: opt.y_axis_index.unwrap_or(0),
                    grid_index: opt.grid_index.unwrap_or(0),
                    item_style: CandlestickItemStyle {
                        color: item_style.and_then(|s| s.color).map(Color::from),
                        color0: item_style.and_then(|s| s.color0).map(Color::from),
                        border_color: item_style.and_then(|s| s.border_color).map(Color::from),
                        border_color0: item_style.and_then(|s| s.border_color0).map(Color::from),
                    },
                    label: opt.label.map(|l| Label {
                        show: l.show.unwrap_or(false),
                        position: l
                            .position
                            .map(Self::convert_label_position)
                            .unwrap_or(LabelPosition::Top),
                        formatter: l.formatter,
                        color: l
                            .color
                            .map(Color::from)
                            .unwrap_or(Color::new(0, 0, 0)),
                        font_size: l.font_size.unwrap_or(12.0),
                        font_family: l.font_family.unwrap_or_else(|| "sans-serif".to_string()),
                    }),
                    color_up,
                    color_down,
                }))
            }
            SeriesOption::Pie(opt) => {
                let data = Self::resolve_data(&opt.data)?;
                Ok(ResolvedSeries::Pie(PieSeries {
                    name: opt.name.unwrap_or_default(),
                    data,
                    radius: opt
                        .radius
                        .map(|r| {
                            if r.len() >= 2 {
                                (
                                    Self::parse_percent_or_value(&r[0]),
                                    Self::parse_percent_or_value(&r[1]),
                                )
                            } else if r.len() == 1 {
                                (0.0, Self::parse_percent_or_value(&r[0]))
                            } else {
                                (0.0, 100.0)
                            }
                        })
                        .unwrap_or((0.0, 100.0)),
                    center: opt
                        .center
                        .map(|c| {
                            if c.len() >= 2 {
                                (
                                    Self::parse_percent_or_value(&c[0]),
                                    Self::parse_percent_or_value(&c[1]),
                                )
                            } else {
                                (50.0, 50.0)
                            }
                        })
                        .unwrap_or((50.0, 50.0)),
                    item_style: opt
                        .item_style
                        .map(|s| Self::resolve_item_style(s, theme))
                        .unwrap_or_else(|| ItemStyle {
                            color: None,
                            border_color: None,
                            border_width: 0.0,
                            opacity: 1.0,
                        }),
                    label: opt.label.map(|l| Label {
                        show: l.show.unwrap_or(true),
                        position: l
                            .position
                            .map(Self::convert_label_position)
                            .unwrap_or(LabelPosition::Outside),
                        formatter: l.formatter,
                        color: l
                            .color
                            .map(Color::from)
                            .unwrap_or(Color::new(0, 0, 0)),
                        font_size: l.font_size.unwrap_or(12.0),
                        font_family: l.font_family.unwrap_or_else(|| "sans-serif".to_string()),
                    }),
                    grid_index: opt.grid_index.unwrap_or(0),
                }))
            }
            SeriesOption::Scatter(opt) => {
                let data = Self::resolve_scatter_data(&opt.data)?;
                let series_color = opt.item_style.as_ref().and_then(|s| s.color.map(Color::from)).unwrap_or(color);
                Ok(ResolvedSeries::Scatter(ScatterSeries {
                    name: opt.name.unwrap_or_default(),
                    data,
                    y_axis_index: opt.y_axis_index.unwrap_or(0),
                    grid_index: opt.grid_index.unwrap_or(0),
                    symbol_size: opt.symbol_size.unwrap_or(10.0),
                    item_style: opt
                        .item_style
                        .map(|s| Self::resolve_item_style(s, theme))
                        .unwrap_or_else(|| ItemStyle {
                            color: Some(color),
                            border_color: None,
                            border_width: 0.0,
                            opacity: 1.0,
                        }),
                    color: series_color,
                }))
            }
            SeriesOption::Radar(opt) => {
                let data = opt.data.into_iter().map(|d| RadarData {
                    value: d.value,
                    name: d.name,
                }).collect();
                Ok(ResolvedSeries::Radar(RadarSeries {
                    name: opt.name.unwrap_or_default(),
                    data,
                    item_style: opt
                        .item_style
                        .map(|s| Self::resolve_item_style(s, theme))
                        .unwrap_or_else(|| ItemStyle {
                            color: Some(color),
                            border_color: None,
                            border_width: 0.0,
                            opacity: 1.0,
                        }),
                    line_style: opt
                        .line_style
                        .map(|s| Self::resolve_line_style(s, theme, color))
                        .unwrap_or_else(|| LineStyle {
                            color,
                            width: 2.0,
                            line_type: LineType::Solid,
                        }),
                    area_style: opt.area_style.map(|s| AreaStyle {
                        color: s.color.map(Color::from),
                        opacity: s.opacity.unwrap_or(0.3),
                    }),
                    symbol: opt.symbol.map(Self::convert_symbol).unwrap_or(Symbol::Circle),
                    symbol_size: opt.symbol_size.unwrap_or(4.0),
                    color,
                }))
            }
            SeriesOption::PolarBar(opt) => {
                let data = Self::resolve_data(&opt.data)?;
                let colors = opt.color.map(|c| {
                    c.iter().map(|c| Color::from(*c)).collect()
                }).unwrap_or_else(|| vec![color]);
                Ok(ResolvedSeries::PolarBar(PolarBarSeries {
                    name: opt.name.unwrap_or_default(),
                    data,
                    item_style: opt.item_style.map(|s| Self::resolve_item_style(s, theme))
                        .unwrap_or_else(|| ItemStyle {
                            color: Some(color),
                            border_color: None,
                            border_width: 0.0,
                            opacity: 1.0,
                        }),
                    colors,
                    pad_angle: opt.pad_angle.unwrap_or(2.0).to_radians(),
                    start_angle: opt.start_angle.unwrap_or(0.0).to_radians() - std::f64::consts::PI / 2.0,
                }))
            }
            SeriesOption::PolarScatter(opt) => {
                let data: Vec<PolarScatterData> = opt.data.into_iter().map(|d| {
                    PolarScatterData {
                        angle: d.angle.to_radians() - std::f64::consts::PI / 2.0,
                        radius: d.radius,
                        symbol_size: d.symbol_size.unwrap_or(opt.symbol_size.unwrap_or(10.0)),
                        name: d.name,
                    }
                }).collect();
                Ok(ResolvedSeries::PolarScatter(PolarScatterSeries {
                    name: opt.name.unwrap_or_default(),
                    data,
                    item_style: opt.item_style.map(|s| Self::resolve_item_style(s, theme))
                        .unwrap_or_else(|| ItemStyle {
                            color: Some(color),
                            border_color: None,
                            border_width: 0.0,
                            opacity: 1.0,
                        }),
                    symbol: opt.symbol.map(Self::convert_symbol).unwrap_or(Symbol::Circle),
                    default_symbol_size: opt.symbol_size.unwrap_or(10.0),
                }))
            }
            SeriesOption::Bubble(opt) => {
                let data: Vec<BubbleData> = opt.data.into_iter().map(|d| BubbleData {
                    x: d.x,
                    y: d.y,
                    size: d.size.unwrap_or(20.0),
                    name: d.name,
                }).collect();
                let series_color = opt.item_style.as_ref().and_then(|s| s.color.map(Color::from)).unwrap_or(color);
                Ok(ResolvedSeries::Bubble(BubbleSeries {
                    name: opt.name.unwrap_or_default(),
                    data,
                    y_axis_index: opt.y_axis_index.unwrap_or(0),
                    grid_index: opt.grid_index.unwrap_or(0),
                    symbol_size_scale: opt.symbol_size_scale.unwrap_or(1.0),
                    item_style: opt
                        .item_style
                        .map(|s| Self::resolve_item_style(s, theme))
                        .unwrap_or_else(|| ItemStyle {
                            color: Some(color),
                            border_color: None,
                            border_width: 0.0,
                            opacity: 0.6,
                        }),
                    color: series_color,
                }))
            }
            SeriesOption::Gauge(opt) => {
                let value = opt.data.first().map(|d| d.value).unwrap_or(0.0);
                let center = opt.center.map(|c| {
                    if c.len() >= 2 {
                        (
                            Self::parse_percent_or_value(&c[0]),
                            Self::parse_percent_or_value(&c[1]),
                        )
                    } else {
                        (50.0, 50.0)
                    }
                }).unwrap_or((50.0, 50.0));
                let radius = opt.radius.map(|r| Self::parse_percent_or_value(&r)).unwrap_or(75.0);
                let pointer = opt.pointer.unwrap_or_default();
                let title = opt.title.unwrap_or_default();
                let detail = opt.detail.unwrap_or_default();
                Ok(ResolvedSeries::Gauge(GaugeSeries {
                    name: opt.name.unwrap_or_default(),
                    value,
                    min: opt.min.unwrap_or(0.0),
                    max: opt.max.unwrap_or(100.0),
                    center,
                    radius,
                    start_angle: opt.start_angle.unwrap_or(-225.0),
                    end_angle: opt.end_angle.unwrap_or(45.0),
                    split_number: opt.split_number.unwrap_or(10),
                    axis_line_show: opt.axis_line.as_ref().and_then(|a| a.show).unwrap_or(true),
                    axis_line_style: opt.axis_line
                        .and_then(|a| a.line_style)
                        .map(|s| Self::resolve_line_style(s, theme, color))
                        .unwrap_or_else(|| LineStyle {
                            color: Color::new(60, 60, 60),
                            width: 10.0,
                            line_type: LineType::Solid,
                        }),
                    pointer_show: pointer.show.unwrap_or(true),
                    pointer_length: pointer.length.map(|l| Self::parse_percent_or_value(&l)).unwrap_or(70.0),
                    pointer_width: pointer.width.unwrap_or(6.0),
                    pointer_color: pointer.item_style
                        .and_then(|s| s.color.map(Color::from))
                        .unwrap_or(color),
                    axis_tick_show: opt.axis_tick.as_ref().and_then(|a| a.show).unwrap_or(true),
                    axis_tick_length: opt.axis_tick.as_ref().and_then(|a| a.length).unwrap_or(8.0),
                    axis_tick_style: opt.axis_tick
                        .and_then(|a| a.line_style)
                        .map(|s| Self::resolve_line_style(s, theme, Color::new(80, 80, 80)))
                        .unwrap_or_else(|| LineStyle {
                            color: Color::new(80, 80, 80),
                            width: 2.0,
                            line_type: LineType::Solid,
                        }),
                    axis_label_show: opt.axis_label.as_ref().and_then(|a| a.show).unwrap_or(true),
                    axis_label_distance: opt.axis_label.as_ref().and_then(|a| a.distance).unwrap_or(15.0),
                    axis_label_color: opt.axis_label.as_ref()
                        .and_then(|a| a.color.map(Color::from))
                        .unwrap_or(Color::new(50, 50, 50)),
                    axis_label_font_size: opt.axis_label.as_ref().and_then(|a| a.font_size).unwrap_or(12.0),
                    axis_label_font_family: opt.axis_label.as_ref()
                        .and_then(|a| a.font_family.clone())
                        .unwrap_or_else(|| "sans-serif".to_string()),
                    axis_label_font_weight: opt.axis_label.as_ref()
                        .and_then(|a| a.font_weight)
                        .unwrap_or(FontWeight::Named(FontWeightNamed::Normal)),
                    split_line_show: opt.split_line.as_ref().and_then(|a| a.show).unwrap_or(true),
                    split_line_length: opt.split_line.as_ref().and_then(|a| a.length).unwrap_or(15.0),
                    split_line_style: opt.split_line
                        .and_then(|a| a.line_style)
                        .map(|s| Self::resolve_line_style(s, theme, Color::new(60, 60, 60)))
                        .unwrap_or_else(|| LineStyle {
                            color: Color::new(60, 60, 60),
                            width: 3.0,
                            line_type: LineType::Solid,
                        }),
                    title_show: title.show.unwrap_or(true),
                    title_offset: title.offset_center.map(|o| {
                        if o.len() >= 2 {
                            (Self::parse_percent_or_value(&o[0]), Self::parse_percent_or_value(&o[1]))
                        } else {
                            (0.0, -20.0)
                        }
                    }).unwrap_or((0.0, -20.0)),
                    title_color: title.color.map(Color::from).unwrap_or(Color::new(80, 80, 80)),
                    title_font_size: title.font_size.unwrap_or(14.0),
                    title_font_family: title.font_family.clone().unwrap_or_else(|| "sans-serif".to_string()),
                    title_font_weight: title.font_weight.unwrap_or(FontWeight::Named(FontWeightNamed::Normal)),
                    detail_show: detail.show.unwrap_or(true),
                    detail_formatter: detail.formatter,
                    detail_offset: detail.offset_center.map(|o| {
                        if o.len() >= 2 {
                            (Self::parse_percent_or_value(&o[0]), Self::parse_percent_or_value(&o[1]))
                        } else {
                            (0.0, 30.0)
                        }
                    }).unwrap_or((0.0, 30.0)),
                    detail_color: detail.color.map(Color::from).unwrap_or(color),
                    detail_font_size: detail.font_size.unwrap_or(24.0),
                    detail_font_family: detail.font_family.clone().unwrap_or_else(|| "sans-serif".to_string()),
                    detail_font_weight: detail.font_weight.unwrap_or(FontWeight::Named(FontWeightNamed::Normal)),
                    color,
                    gradient_colors: opt.gradient_colors.as_ref().map(|stops| {
                        stops.iter().map(|s| {
                            (s.offset, Color::from_hex(&s.color).unwrap_or_else(|| Color::new(100, 100, 100)))
                        }).collect()
                    }).unwrap_or_else(|| vec![
                        (0.0, Color::from_hex("#3fbe95").unwrap()),
                        (0.25, Color::from_hex("#b6d634").unwrap()),
                        (0.5, Color::from_hex("#ffd10a").unwrap()),
                        (0.75, Color::from_hex("#ff994d").unwrap()),
                        (1.0, Color::from_hex("#fb628b").unwrap()),
                    ]),
                }))
            }
            SeriesOption::Table(opt) => {
                let header = opt.header.unwrap_or_default();
                let body = opt.body.unwrap_or_default();
                let header_style = header.style.unwrap_or_default();
                let body_style = body.style.unwrap_or_default();
                Ok(ResolvedSeries::Table(TableSeries {
                    name: opt.name.unwrap_or_default(),
                    data: opt.data.unwrap_or_default(),
                    columns: opt.columns.unwrap_or_default(),
                    header_config: TableHeaderConfig {
                        show: header.show.unwrap_or(true),
                        height: header.height.unwrap_or(40.0),
                        style: Self::resolve_text_style(header_style, theme),
                        background_color: header.background_color
                            .map(Color::from)
                            .unwrap_or_else(|| Color::new(248, 248, 248)),
                        align: header.align
                            .map(Self::convert_text_align)
                            .unwrap_or(TextAlign::Center),
                    },
                    body_config: TableBodyConfig {
                        show: body.show.unwrap_or(true),
                        row_height: body.row_height.unwrap_or(32.0),
                        style: Self::resolve_text_style(body_style, theme),
                        even_row_background_color: body.even_row_background_color
                            .map(Color::from)
                            .unwrap_or_else(|| Color::new(255, 255, 255)),
                        odd_row_background_color: body.odd_row_background_color
                            .map(Color::from)
                            .unwrap_or_else(|| Color::new(250, 250, 250)),
                        align: body.align
                            .map(Self::convert_text_align)
                            .unwrap_or(TextAlign::Center),
                    },
                    grid_index: opt.grid_index.unwrap_or(0),
                    auto_fit_grid: opt.auto_fit_grid.unwrap_or(false),
                }))
            }
        }
    }

    fn resolve_radar(option: RadarOption, theme: &Theme) -> RadarConfig {
        let indicator = option.indicator.unwrap_or_default().into_iter().map(|i| {
            RadarIndicator {
                name: i.name.unwrap_or_default(),
                max: i.max.unwrap_or(100.0),
            }
        }).collect();

        let center = option.center.unwrap_or_else(|| vec!["50%".to_string(), "50%".to_string()]);
        let center_x = Self::parse_percent_or_value(&center.first().cloned().unwrap_or_else(|| "50%".to_string()));
        let center_y = Self::parse_percent_or_value(&center.get(1).cloned().unwrap_or_else(|| "50%".to_string()));

        let radius = option.radius.unwrap_or_else(|| vec!["0%".to_string(), "75%".to_string()]);
        let radius_inner = Self::parse_percent_or_value(&radius.first().cloned().unwrap_or_else(|| "0%".to_string()));
        let radius_outer = Self::parse_percent_or_value(&radius.get(1).cloned().unwrap_or_else(|| "75%".to_string()));

        // 解析 name 配置（指标名称样式）
        let name_config = option.name.unwrap_or_default();
        let name_text_style = name_config.text_style.map(|s| Self::resolve_text_style(s, theme))
            .unwrap_or_else(|| TextStyle {
                color: Color::new(80, 80, 80),
                font_size: 12.0,
                font_family: "sans-serif".to_string(),
                font_weight: FontWeight::Named(FontWeightNamed::Normal),
                ..Default::default()
            });

        RadarConfig {
            indicator,
            center: (center_x, center_y),
            radius: (radius_inner, radius_outer),
            split_number: option.split_number.unwrap_or(5),
            name: RadarNameConfig {
                show: name_config.show.unwrap_or(true),
                formatter: name_config.formatter,
                text_style: name_text_style,
            },
        }
    }

    fn resolve_data(data: &[DataPoint]) -> crate::error::Result<Vec<DataItem>> {
        data.iter()
            .map(|dp| match dp {
                DataPoint::Number(n) => Ok(DataItem { name: None, value: *n }),
                DataPoint::Array(arr) => {
                    if arr.len() >= 2 {
                        let name = arr.first().and_then(|v| v.as_str()).map(|s| s.to_string());
                        let value = arr
                            .get(1)
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| ChartError::InvalidColor("Invalid data value".to_string()))?;
                        Ok(DataItem { name, value })
                    } else {
                        Err(ChartError::InvalidColor("Invalid data array".to_string()))
                    }
                }
                DataPoint::Object(obj) => {
                    let name = obj.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let value = obj
                        .get("value")
                        .and_then(|v| v.as_f64())
                        .ok_or_else(|| ChartError::InvalidColor("Invalid data value".to_string()))?;
                    Ok(DataItem { name, value })
                }
            })
            .collect()
    }

    /// 解析散点图数据，支持 [x, y] 或 {x, y, name?} 格式
    fn resolve_scatter_data(data: &[DataPoint]) -> crate::error::Result<Vec<ScatterDataItem>> {
        data.iter()
            .map(|dp| match dp {
                DataPoint::Number(_) => Err(ChartError::InvalidColor(
                    "Scatter chart data must be [x, y] arrays or {x, y} objects".to_string(),
                )),
                DataPoint::Array(arr) => {
                    if arr.len() >= 2 {
                        let x = arr
                            .get(0)
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| ChartError::InvalidColor("Invalid x value".to_string()))?;
                        let y = arr
                            .get(1)
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| ChartError::InvalidColor("Invalid y value".to_string()))?;
                        let name = arr.get(2).and_then(|v| v.as_str()).map(|s| s.to_string());
                        Ok(ScatterDataItem { x, y, name })
                    } else {
                        Err(ChartError::InvalidColor("Scatter data array must have at least 2 elements [x, y]".to_string()))
                    }
                }
                DataPoint::Object(obj) => {
                    let x = obj
                        .get("x")
                        .and_then(|v| v.as_f64())
                        .ok_or_else(|| ChartError::InvalidColor("Missing or invalid x value".to_string()))?;
                    let y = obj
                        .get("y")
                        .and_then(|v| v.as_f64())
                        .ok_or_else(|| ChartError::InvalidColor("Missing or invalid y value".to_string()))?;
                    let name = obj.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                    Ok(ScatterDataItem { x, y, name })
                }
            })
            .collect()
    }

    fn resolve_text_style(option: TextStyleOption, theme: &Theme) -> TextStyle {
        TextStyle {
            color: option
                .color
                .map(Color::from)
                .unwrap_or_else(|| Color::from_hex(&theme.title.text_style.color).unwrap()),
            font_size: option.font_size.unwrap_or(theme.title.text_style.font_size),
            font_family: option
                .font_family
                .unwrap_or_else(|| theme.title.text_style.font_family.clone()),
            font_weight: option.font_weight.unwrap_or(FontWeight::Named(FontWeightNamed::Normal)),
            font_style: option.font_style.map(|f| match f {
                FontStyleOption::Normal => FontStyle::Normal,
                FontStyleOption::Italic => FontStyle::Italic,
                FontStyleOption::Oblique => FontStyle::Oblique,
            }).unwrap_or(FontStyle::Normal),
            align: option.align.map(|a| match a {
                TextAlignOption::Left => TextAlign::Left,
                TextAlignOption::Center => TextAlign::Center,
                TextAlignOption::Right => TextAlign::Right,
            }),
            vertical_align: option.vertical_align.map(|v| match v {
                LabelVerticalAlign::Top => TextBaseline::Top,
                LabelVerticalAlign::Middle => TextBaseline::Middle,
                LabelVerticalAlign::Bottom => TextBaseline::Bottom,
            }),
        }
    }

    fn resolve_line_style(option: LineStyleOption, _theme: &Theme, default_color: Color) -> LineStyle {
        LineStyle {
            color: option
                .color
                .map(Color::from)
                .unwrap_or(default_color),
            width: option.width.unwrap_or(1.0),
            line_type: option
                .line_type
                .map(|t| match t {
                    crate::option::LineType::Dashed => LineType::Dashed,
                    crate::option::LineType::Dotted => LineType::Dotted,
                    crate::option::LineType::Solid => LineType::Solid,
                })
                .unwrap_or(LineType::Solid),
        }
    }

    fn resolve_item_style(option: ItemStyleOption, _theme: &Theme) -> ItemStyle {
        ItemStyle {
            color: option.color.map(Color::from),
            border_color: option.border_color.map(Color::from),
            border_width: option.border_width.unwrap_or(0.0),
            opacity: option.opacity.unwrap_or(1.0),
        }
    }

    fn convert_symbol(s: OptionSymbolType) -> Symbol {
        match s {
            OptionSymbolType::Circle => Symbol::Circle,
            OptionSymbolType::Rect => Symbol::Rect,
            OptionSymbolType::RoundRect => Symbol::RoundRect,
            OptionSymbolType::Triangle => Symbol::Triangle,
            OptionSymbolType::Diamond => Symbol::Diamond,
            OptionSymbolType::Pin => Symbol::Pin,
            OptionSymbolType::Arrow => Symbol::Arrow,
            OptionSymbolType::None => Symbol::None,
        }
    }

    fn convert_text_align(a: TextAlignOption) -> TextAlign {
        match a {
            TextAlignOption::Left => TextAlign::Left,
            TextAlignOption::Center => TextAlign::Center,
            TextAlignOption::Right => TextAlign::Right,
        }
    }

    fn convert_label_position(p: OptionLabelPosition) -> LabelPosition {
        match p {
            OptionLabelPosition::Top => LabelPosition::Top,
            OptionLabelPosition::Left => LabelPosition::Left,
            OptionLabelPosition::Right => LabelPosition::Right,
            OptionLabelPosition::Bottom => LabelPosition::Bottom,
            OptionLabelPosition::Inside => LabelPosition::Inside,
            OptionLabelPosition::Outside => LabelPosition::Outside,
            OptionLabelPosition::Center => LabelPosition::Center,
        }
    }

    fn parse_percent_or_value(s: &str) -> f64 {
        if s.ends_with('%') {
            s.trim_end_matches('%').parse::<f64>().unwrap_or(0.0)
        } else {
            s.parse::<f64>().unwrap_or(0.0)
        }
    }

    /// 根据系列名称查找其渲染颜色，用于图例等需要与系列颜色一致的地方
    pub fn series_color_by_name(&self, name: &str) -> Option<Color> {
        self.series.iter().find_map(|s| match s {
            ResolvedSeries::Line(ls) if ls.name == name => Some(ls.color),
            ResolvedSeries::Bar(bs) if bs.name == name => Some(bs.color),
            ResolvedSeries::Scatter(ss) if ss.name == name => Some(ss.color),
            ResolvedSeries::Radar(rs) if rs.name == name => Some(rs.color),
            ResolvedSeries::Bubble(bs) if bs.name == name => Some(bs.color),
            ResolvedSeries::Gauge(gs) if gs.name == name => Some(gs.color),
            ResolvedSeries::Candlestick(cs) if cs.name == name => Some(cs.color_up),
            ResolvedSeries::Pie(ps) if ps.name == name => None, // 饼图颜色由 item_style 管理
            ResolvedSeries::Table(ts) if ts.name == name => None,
            _ => None,
        })
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            left: Position::Value(60.0),
            right: Position::Value(60.0),
            top: Position::Value(60.0),
            bottom: Position::Value(60.0),
            contain_label: false,
        }
    }
}