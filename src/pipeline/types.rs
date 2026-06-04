use vello_cpu::kurbo::Rect;

use crate::{
    text::create_text_layout,
    visual::{Color, TextStyle, VisualElement},
};

// ═══════════════════════════════════════════════════════════════════
// NEW: ChartSpec — pipeline 的统一输入类型
// ═══════════════════════════════════════════════════════════════════

/// Pipeline 的统一输入规格。可从新 API (Chart) 或旧 option (ChartOption) 转换而来。
#[derive(Debug, Clone)]
pub struct ChartSpec {
    pub width: u32,
    pub height: u32,
    pub grids: Vec<GridSpec>,
    pub x_axes: Vec<AxisSpec>,
    pub y_axes: Vec<AxisSpec>,
    pub series: Vec<SeriesSpec>,
    pub title: Option<TitleSpec>,
    pub legend: Option<LegendSpec>,
    pub background: Color,
    pub palette: Vec<Color>,
    pub theme_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GridSpec {
    pub left: Option<f64>, // pixels, None = auto
    pub right: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
    pub contain_label: bool,
}

/// 坐标轴类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisType {
    Category,
    Value,
    Time,
    Log,
}

/// 坐标轴位置
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct AxisSpec {
    pub axis_type: AxisType,
    pub position: AxisPosition,
    pub grid_index: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub name: Option<String>,
    pub categories: Vec<String>, // Category 轴的标签
    pub boundary_gap: bool,
}

/// 图表类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Scatter,
    Bubble,
    Candlestick,
    Radar,
    PolarBar,
    PolarScatter,
    Gauge,
    Table,
}

#[derive(Debug, Clone)]
pub struct SeriesSpec {
    pub name: String,
    pub chart_type: ChartType,
    pub data: crate::pipeline::dataframe::DataFrame,
    pub grid_index: usize,
    pub x_axis_index: usize,
    pub y_axis_index: usize,
    pub stack: Option<String>,
    pub group_index: usize,
    pub sampling: Option<(crate::sampling::SamplingType, usize)>,
    pub item_style: ItemStyleSpec,
    /// Type-specific configuration (no Option fields — values are always deterministic)
    pub config: SeriesConfig,
}

// ═══════════════════════════════════════════════════════════════════
// SeriesConfig — 每种图表类型拥有独立的、确定值的配置
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum SeriesConfig {
    Line(LineConfig),
    Bar(BarConfig),
    Pie(PieConfig),
    Scatter(ScatterConfig),
    Bubble(BubbleConfig),
    Candlestick(CandlestickConfig),
    Radar(RadarConfig),
    PolarBar(PolarBarConfig),
    PolarScatter(PolarScatterConfig),
    Gauge(GaugeConfig),
    Table(TableConfig),
}

impl SeriesConfig {
    pub fn chart_type(&self) -> ChartType {
        match self {
            SeriesConfig::Line(_) => ChartType::Line,
            SeriesConfig::Bar(_) => ChartType::Bar,
            SeriesConfig::Pie(_) => ChartType::Pie,
            SeriesConfig::Scatter(_) => ChartType::Scatter,
            SeriesConfig::Bubble(_) => ChartType::Bubble,
            SeriesConfig::Candlestick(_) => ChartType::Candlestick,
            SeriesConfig::Radar(_) => ChartType::Radar,
            SeriesConfig::PolarBar(_) => ChartType::PolarBar,
            SeriesConfig::PolarScatter(_) => ChartType::PolarScatter,
            SeriesConfig::Gauge(_) => ChartType::Gauge,
            SeriesConfig::Table(_) => ChartType::Table,
        }
    }
}

// ── LineConfig ──

#[derive(Debug, Clone)]
pub struct LineConfig {
    pub x_col: String,
    pub y_col: String,
    pub smooth: bool,
    pub line_width: f64,
    pub area_color: Option<Color>,
    pub area_opacity: f64,
    pub symbol_type: SymbolType,
    pub symbol_size: f64,
}

impl Default for LineConfig {
    fn default() -> Self {
        Self {
            x_col: "x".into(),
            y_col: "y".into(),
            smooth: false,
            line_width: 2.0,
            area_color: None,
            area_opacity: 0.5,
            symbol_type: SymbolType::Circle,
            symbol_size: 4.0,
        }
    }
}

// ── BarConfig ──

#[derive(Debug, Clone)]
pub struct BarConfig {
    pub x_col: String,
    pub y_col: String,
    pub bar_width: f64, // 0.0~1.0 ratio
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            x_col: "x".into(),
            y_col: "y".into(),
            bar_width: 0.6,
        }
    }
}

// ── PieConfig ──

#[derive(Debug, Clone)]
pub struct PieConfig {
    pub category_col: String,
    pub value_col: String,
    pub center: (f64, f64),
    pub radius: (f64, f64),
    pub label_show: bool,
    pub label_position: LabelPosition,
    pub label_font_size: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelPosition {
    Outside,
    Inside,
}

impl Default for PieConfig {
    fn default() -> Self {
        Self {
            category_col: "category".into(),
            value_col: "value".into(),
            center: (50.0, 50.0),
            radius: (0.0, 75.0),
            label_show: false,
            label_position: LabelPosition::Outside,
            label_font_size: 12.0,
        }
    }
}

// ── ScatterConfig ──

#[derive(Debug, Clone)]
pub struct ScatterConfig {
    pub x_col: String,
    pub y_col: String,
    pub symbol_size: f64,
}

impl Default for ScatterConfig {
    fn default() -> Self {
        Self {
            x_col: "x".into(),
            y_col: "y".into(),
            symbol_size: 10.0,
        }
    }
}

// ── BubbleConfig ──

#[derive(Debug, Clone)]
pub struct BubbleConfig {
    pub x_col: String,
    pub y_col: String,
    pub size_col: Option<String>,
    pub name_col: Option<String>,
    pub symbol_size_scale: f64,
}

impl Default for BubbleConfig {
    fn default() -> Self {
        Self {
            x_col: "x".into(),
            y_col: "y".into(),
            size_col: None,
            name_col: None,
            symbol_size_scale: 1.0,
        }
    }
}

// ── CandlestickConfig ──

#[derive(Debug, Clone)]
pub struct CandlestickConfig {
    pub category_col: String,
    pub open_col: String,
    pub close_col: String,
    pub low_col: String,
    pub high_col: String,
}

impl Default for CandlestickConfig {
    fn default() -> Self {
        Self {
            category_col: "category".into(),
            open_col: "open".into(),
            close_col: "close".into(),
            low_col: "low".into(),
            high_col: "high".into(),
        }
    }
}

// ── RadarConfig ──

#[derive(Debug, Clone)]
pub struct RadarConfig {
    pub value_col: String,
    pub indicators: Vec<String>,
}

impl Default for RadarConfig {
    fn default() -> Self {
        Self {
            value_col: "value".into(),
            indicators: vec![],
        }
    }
}

// ── PolarBarConfig ──

#[derive(Debug, Clone)]
pub struct PolarBarConfig {
    pub angle_col: String,
    pub radius_col: String,
    pub pad_angle: f64,
    pub start_angle: f64,
}

impl Default for PolarBarConfig {
    fn default() -> Self {
        Self {
            angle_col: "angle".into(),
            radius_col: "radius".into(),
            pad_angle: 2.0,
            start_angle: 0.0,
        }
    }
}

// ── PolarScatterConfig ──

#[derive(Debug, Clone)]
pub struct PolarScatterConfig {
    pub angle_col: String,
    pub radius_col: String,
    pub symbol_size: f64,
}

impl Default for PolarScatterConfig {
    fn default() -> Self {
        Self {
            angle_col: "angle".into(),
            radius_col: "radius".into(),
            symbol_size: 8.0,
        }
    }
}

// ── GaugeConfig ──

#[derive(Debug, Clone)]
pub struct GaugeConfig {
    pub value_col: String,
    pub min: f64,
    pub max: f64,
    pub center: (f64, f64),
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub split_number: usize,
}

impl Default for GaugeConfig {
    fn default() -> Self {
        Self {
            value_col: "value".into(),
            min: 0.0,
            max: 100.0,
            center: (50.0, 75.0),
            radius: 75.0,
            start_angle: -225.0,
            end_angle: 45.0,
            split_number: 10,
        }
    }
}

// ── TableConfig ──

#[derive(Debug, Clone, Default)]
pub struct TableConfig;

// ── SymbolType ──

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
    Circle,
    Rect,
    RoundRect,
    Triangle,
    Diamond,
    Pin,
    Arrow,
    None,
}

impl Default for SymbolType {
    fn default() -> Self {
        SymbolType::Circle
    }
}

// ── SeriesSpec helpers ──

impl Default for SeriesSpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            chart_type: ChartType::Line,
            data: crate::pipeline::dataframe::DataFrame::new(),
            grid_index: 0,
            x_axis_index: 0,
            y_axis_index: 0,
            stack: None,
            group_index: 0,
            sampling: None,
            item_style: ItemStyleSpec::default(),
            config: SeriesConfig::Line(LineConfig::default()),
        }
    }
}

impl SeriesSpec {
    /// 获取 Y 列的全部数值（用于轴范围计算）
    pub fn y_values(&self) -> Vec<f64> {
        let y_col = self.config.y_col_name();
        self.data
            .get_column(y_col)
            .map(|s| {
                s.data
                    .iter()
                    .filter_map(|v| match v {
                        crate::pipeline::dataframe::DataValue::Float(f) => Some(*f),
                        crate::pipeline::dataframe::DataValue::Integer(i) => Some(*i as f64),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取 X 列的全部数值（用于轴范围计算）
    pub fn x_values(&self) -> Vec<f64> {
        let x_col = self.config.x_col_name();
        self.data
            .get_column(x_col)
            .map(|s| {
                s.data
                    .iter()
                    .filter_map(|v| match v {
                        crate::pipeline::dataframe::DataValue::Float(f) => Some(*f),
                        crate::pipeline::dataframe::DataValue::Integer(i) => Some(*i as f64),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl SeriesConfig {
    pub fn x_col_name(&self) -> &str {
        match self {
            SeriesConfig::Line(c) => &c.x_col,
            SeriesConfig::Bar(c) => &c.x_col,
            SeriesConfig::Scatter(c) => &c.x_col,
            SeriesConfig::Bubble(c) => &c.x_col,
            SeriesConfig::Pie(c) => &c.category_col,
            SeriesConfig::Candlestick(c) => &c.category_col,
            SeriesConfig::Radar(_) => "indicator",
            SeriesConfig::PolarBar(c) => &c.angle_col,
            SeriesConfig::PolarScatter(c) => &c.angle_col,
            SeriesConfig::Gauge(_) => "name",
            SeriesConfig::Table(_) => "",
        }
    }

    pub fn y_col_name(&self) -> &str {
        match self {
            SeriesConfig::Line(c) => &c.y_col,
            SeriesConfig::Bar(c) => &c.y_col,
            SeriesConfig::Scatter(c) => &c.y_col,
            SeriesConfig::Bubble(c) => &c.y_col,
            SeriesConfig::Pie(c) => &c.value_col,
            SeriesConfig::Candlestick(_) => "close",
            SeriesConfig::Radar(c) => &c.value_col,
            SeriesConfig::PolarBar(c) => &c.radius_col,
            SeriesConfig::PolarScatter(c) => &c.radius_col,
            SeriesConfig::Gauge(c) => &c.value_col,
            SeriesConfig::Table(_) => "",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ItemStyleSpec {
    pub color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<f64>,
    pub opacity: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TitleSpec {
    pub text: Option<String>,
    pub subtext: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LegendSpec {
    pub show: bool,
    pub data: Vec<String>,
    pub symbol_size: f64,
}

// ═══════════════════════════════════════════════════════════════════
// 原有类型不变（SubplotSpec, ResolvedAxisRange, ColorContext 等）
// ═══════════════════════════════════════════════════════════════════

/// GridPlanner 的输出：一个 subplot 的完整分配信息
#[derive(Debug, Clone)]
pub struct SubplotSpec {
    pub id: usize,
    pub bounds: Rect,
    pub series_indices: Vec<usize>,
    pub x_axis_indices: Vec<usize>,
    pub y_axis_indices: Vec<usize>,
}

/// AxisBindingResolver 的输出：单个轴实例的解析结果
#[derive(Debug, Clone)]
pub struct ResolvedAxisRange {
    pub axis_index: usize,
    pub position: AxisPosition,
    pub axis_type: AxisType,
    pub min: f64,
    pub max: f64,
    pub is_user_defined: bool,
    pub tick_count_hint: Option<usize>,
}

impl ResolvedAxisRange {
    pub fn is_y_axis(&self) -> bool {
        matches!(self.position, AxisPosition::Left | AxisPosition::Right)
    }
}

/// AxisBindingResolver 的输出：所有轴的解析结果集合
#[derive(Debug, Clone)]
pub struct ResolvedAxisRanges {
    pub ranges: Vec<ResolvedAxisRange>,
}

impl ResolvedAxisRanges {
    pub fn get_x_range(&self, axis_index: usize) -> Option<&ResolvedAxisRange> {
        self.ranges
            .iter()
            .find(|r| !r.is_y_axis() && r.axis_index == axis_index)
    }

    pub fn get_y_range(&self, axis_index: usize) -> Option<&ResolvedAxisRange> {
        self.ranges
            .iter()
            .find(|r| r.is_y_axis() && r.axis_index == axis_index)
    }
}

/// ColorAssigner 的输出：颜色上下文
#[derive(Debug, Clone)]
pub struct ColorContext {
    pub palette: Vec<Color>,
    pub background: Color,
    pub series_colors: Vec<Color>,
    pub axis_line_color: Color,
    pub axis_label_color: Color,
    pub grid_line_color: Color,
    // 新增颜色字段
    pub border_color: Color,         // 边框/描边颜色
    pub text_color: Color,           // 主要文字颜色
    pub text_secondary_color: Color, // 次要文字颜色
    pub up_color: Color,             // 涨/正值颜色（K线图等）
    pub down_color: Color,           // 跌/负值颜色（K线图等）
    pub table_header_bg: Color,      // 表格表头背景
    pub table_row_even_bg: Color,    // 表格偶数行背景
    pub table_row_odd_bg: Color,     // 表格奇数行背景
}

impl Default for ColorContext {
    fn default() -> Self {
        Self {
            palette: Vec::new(),
            background: Color::new(255, 255, 255),
            series_colors: Vec::new(),
            axis_line_color: Color::new(200, 200, 200),
            axis_label_color: Color::new(50, 50, 50),
            grid_line_color: Color::new(230, 230, 230),
            border_color: Color::new(255, 255, 255),
            text_color: Color::new(51, 51, 51),
            text_secondary_color: Color::new(102, 102, 102),
            up_color: Color::new(234, 85, 67),
            down_color: Color::new(80, 170, 94),
            table_header_bg: Color::new(220, 220, 220),
            table_row_even_bg: Color::new(248, 248, 248),
            table_row_odd_bg: Color::new(255, 255, 255),
        }
    }
}

impl ColorContext {
    /// 获取指定索引的系列颜色，支持回退到 palette
    pub fn get_series_color(&self, index: usize) -> Color {
        self.series_colors
            .get(index)
            .copied()
            .or_else(|| self.palette.get(index).copied())
            .unwrap_or_else(|| {
                // 回退到默认调色板
                let default_colors = [
                    Color::new(80, 112, 221), // 蓝色
                    Color::new(182, 214, 52), // 绿色
                    Color::new(234, 85, 67),  // 红色
                    Color::new(255, 193, 7),  // 黄色
                    Color::new(156, 39, 176), // 紫色
                    Color::new(0, 188, 212),  // 青色
                    Color::new(255, 87, 34),  // 橙色
                    Color::new(96, 125, 139), // 蓝灰色
                ];
                default_colors
                    .get(index % default_colors.len())
                    .copied()
                    .unwrap_or(Color::new(80, 112, 221))
            })
    }

    /// 获取数据点颜色（用于饼图、散点等按数据点着色的图表）
    pub fn get_data_color(&self, index: usize) -> Color {
        self.palette
            .get(index)
            .copied()
            .unwrap_or_else(|| self.get_series_color(index))
    }

    /// 获取默认颜色（第一个系列颜色，或第一个调色板颜色）
    pub fn get_default_color(&self) -> Color {
        self.series_colors
            .first()
            .copied()
            .or_else(|| self.palette.first().copied())
            .unwrap_or(Color::new(80, 112, 221))
    }
}

/// DataProcessor 的输出
#[derive(Debug, Clone)]
pub struct SubplotVisualData {
    pub series_elements: Vec<VisualElement>,
    pub axis_elements: Vec<VisualElement>,
    pub grid_lines: Vec<VisualElement>,
}

/// 文本测量工具
///
/// 使用 parley 进行真实的文本排版和测量。
/// 注意：LayoutContext 内部已有缓存机制，无需额外缓存。
#[derive(Debug, Clone, Default)]
pub struct TextMeasurer;

impl TextMeasurer {
    pub fn new() -> Self {
        Self
    }

    /// 测量指定文本在给定字体样式下的宽度和高度
    ///
    /// 使用 parley 进行真实的文本排版，而非简单估算。
    pub fn measure(&mut self, text: &str, style: &TextStyle) -> (f64, f64) {
        let layout = create_text_layout(text, style, None);
        (layout.width() as f64, layout.height() as f64)
    }

    /// 测量文本，支持最大宽度限制（自动换行）
    ///
    /// LayoutContext 内部会缓存布局结果。
    pub fn measure_with_max_width(
        &mut self,
        text: &str,
        style: &TextStyle,
        max_width: f64,
    ) -> (f64, f64) {
        let layout = create_text_layout(text, style, Some(max_width));
        (layout.width() as f64, layout.height() as f64)
    }
}
