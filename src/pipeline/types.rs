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
    pub name_location: Option<String>,
    pub categories: Vec<String>, // Category 轴的标签
    pub boundary_gap: bool,
    pub inverse: bool,
    pub split_number: Option<usize>,
    pub label_show: bool,
    pub label_formatter: Option<String>,
    pub label_rotate: Option<f64>,
    pub axis_line_show: bool,
    pub split_line_show: bool,
    pub z: Option<f64>,
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
    Boxplot,
    Heatmap,
    Radar,
    PolarBar,
    PolarScatter,
    Gauge,
    Table,
}

#[derive(Debug, Clone)]
pub struct SeriesSpec {
    pub name: String,
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
    Boxplot(BoxplotConfig),
    Heatmap(HeatmapConfig),
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
            SeriesConfig::Boxplot(_) => ChartType::Boxplot,
            SeriesConfig::Heatmap(_) => ChartType::Heatmap,
            SeriesConfig::Radar(_) => ChartType::Radar,
            SeriesConfig::PolarBar(_) => ChartType::PolarBar,
            SeriesConfig::PolarScatter(_) => ChartType::PolarScatter,
            SeriesConfig::Gauge(_) => ChartType::Gauge,
            SeriesConfig::Table(_) => ChartType::Table,
        }
    }
}

// ── StepType ──

/// Step line style for line charts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepType {
    Start,
    Middle,
    End,
}

// ── MarkLine ──

/// 标注线类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarkLineType {
    /// 平均值
    Average,
    /// 最小值
    Min,
    /// 最大值
    Max,
}

/// 标注线配置（来自 `series.markLine`）
#[derive(Debug, Clone)]
pub struct MarkLineSpec {
    pub data_type: MarkLineType,
    pub name: Option<String>,
}

// ── LineConfig ──

#[derive(Debug, Clone)]
pub struct LineConfig {
    pub x_col: String,
    pub y_col: String,
    pub smooth: bool,
    pub step: Option<StepType>,
    pub line_width: f64,
    /// 是否显示面积填充
    pub area: bool,
    /// 面积填充颜色（None 时使用系列颜色）
    pub area_color: Option<Color>,
    pub area_opacity: f64,
    pub symbol_type: SymbolType,
    pub symbol_size: f64,
    /// 是否显示值标签
    pub label_show: bool,
    pub label_font_size: f64,
    /// 值标签模板（支持 `{a}`/`{b}`/`{c}`/`{value}`）
    pub label_formatter: Option<String>,
    /// 标注线配置
    pub mark_line: Vec<MarkLineSpec>,
}

impl Default for LineConfig {
    fn default() -> Self {
        Self {
            x_col: "x".into(),
            y_col: "y".into(),
            smooth: false,
            step: None,
            line_width: 2.0,
            area: false,
            area_color: None,
            area_opacity: 0.5,
            symbol_type: SymbolType::EmptyCircle,
            symbol_size: 4.0,
            label_show: false,
            label_font_size: 12.0,
            label_formatter: None,
            mark_line: Vec::new(),
        }
    }
}

// ── BarConfig ──

#[derive(Debug, Clone)]
pub struct BarConfig {
    pub x_col: String,
    pub y_col: String,
    pub bar_width: f64, // 0.0~1.0 ratio
    /// 是否显示值标签
    pub label_show: bool,
    pub label_font_size: f64,
    /// 值标签模板（支持 `{a}`/`{b}`/`{c}`/`{value}`）
    pub label_formatter: Option<String>,
    /// 标注线配置
    pub mark_line: Vec<MarkLineSpec>,
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            x_col: "x".into(),
            y_col: "y".into(),
            bar_width: 0.6,
            label_show: false,
            label_font_size: 12.0,
            label_formatter: None,
            mark_line: Vec::new(),
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
    /// 标签格式化模板，支持 `{b}`（名称）、`{c}`（数值）、`{d}`（百分比）
    pub label_formatter: Option<String>,
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
            label_formatter: None,
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

// ── BoxplotConfig ──

#[derive(Debug, Clone)]
pub struct BoxplotConfig {
    pub category_col: String,
    pub min_col: String,
    pub q1_col: String,
    pub median_col: String,
    pub q3_col: String,
    pub max_col: String,
}

impl Default for BoxplotConfig {
    fn default() -> Self {
        Self {
            category_col: "category".into(),
            min_col: "min".into(),
            q1_col: "q1".into(),
            median_col: "median".into(),
            q3_col: "q3".into(),
            max_col: "max".into(),
        }
    }
}

// ── HeatmapConfig ──

/// 热力图配置：`[x, y, value]` 三元组 + visualMap 颜色映射。
#[derive(Debug, Clone)]
pub struct HeatmapConfig {
    pub x_col: String,
    pub y_col: String,
    pub value_col: String,
    /// visualMap 最小值；None 时由数据自动推断
    pub min: Option<f64>,
    /// visualMap 最大值；None 时由数据自动推断
    pub max: Option<f64>,
    /// visualMap 连续渐变颜色（按值从低到高插值）
    pub colors: Vec<Color>,
    /// 单元格描边颜色（来自 itemStyle）
    pub border_color: Option<Color>,
    pub border_width: f64,
    /// 是否显示单元格数值标签
    pub label_show: bool,
    pub label_font_size: f64,
}

impl Default for HeatmapConfig {
    fn default() -> Self {
        Self {
            x_col: "x".into(),
            y_col: "y".into(),
            value_col: "value".into(),
            min: None,
            max: None,
            colors: vec![
                Color::rgb(80, 163, 186), // #50a3ba
                Color::rgb(234, 199, 54), // #eac736
                Color::rgb(217, 78, 93),  // #d94e5d
            ],
            border_color: None,
            border_width: 0.0,
            label_show: false,
            label_font_size: 12.0,
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
    /// 类目名（数据项名称）列。`None` 时 materializer 自动探测
    /// `label` / `category` / `name` 列，找不到则回退 `Item {i}`。
    pub category_col: Option<String>,
    pub pad_angle: f64,
    pub start_angle: f64,
}

impl Default for PolarBarConfig {
    fn default() -> Self {
        Self {
            angle_col: "angle".into(),
            radius_col: "radius".into(),
            category_col: None,
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SymbolType {
    #[default]
    Circle,
    EmptyCircle,
    Rect,
    RoundRect,
    Triangle,
    Diamond,
    Pin,
    Arrow,
    None,
}

// ── SeriesSpec helpers ──

impl Default for SeriesSpec {
    fn default() -> Self {
        Self {
            name: String::new(),
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
    /// 获取当前图表类型（从 config 推导）
    pub fn chart_type(&self) -> ChartType {
        self.config.chart_type()
    }

    /// 获取 Y 列的全部数值（用于轴范围计算）
    pub fn y_values(&self) -> Vec<f64> {
        // K 线图和箱线图需要包含所有极值/分位数来计算轴范围
        if matches!(self.chart_type(), super::ChartType::Candlestick) {
            let mut all = Vec::new();
            let cols = ["open", "close", "low", "high"];
            for col_name in &cols {
                if let Some(col) = self.data.get_column(col_name) {
                    for v in &col.data {
                        if let crate::pipeline::dataframe::DataValue::Float(f) = v {
                            all.push(*f);
                        } else if let crate::pipeline::dataframe::DataValue::Integer(i) = v {
                            all.push(*i as f64);
                        }
                    }
                }
            }
            return all;
        }

        if matches!(self.chart_type(), super::ChartType::Boxplot) {
            let mut all = Vec::new();
            let cols = ["min", "q1", "median", "q3", "max"];
            for col_name in &cols {
                if let Some(col) = self.data.get_column(col_name) {
                    for v in &col.data {
                        if let crate::pipeline::dataframe::DataValue::Float(f) = v {
                            all.push(*f);
                        } else if let crate::pipeline::dataframe::DataValue::Integer(i) = v {
                            all.push(*i as f64);
                        }
                    }
                }
            }
            return all;
        }

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
            SeriesConfig::Boxplot(c) => &c.category_col,
            SeriesConfig::Heatmap(c) => &c.x_col,
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
            SeriesConfig::Boxplot(_) => "median",
            SeriesConfig::Heatmap(c) => &c.value_col,
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
    pub font_size: Option<f64>,
    pub subfont_size: Option<f64>,
    pub color: Option<Color>,
    pub subcolor: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct LegendSpec {
    pub show: bool,
    pub data: Vec<String>,
    pub symbol_size: f64,
    pub item_gap: f64,
    /// 图例文本模板（支持 `{name}`/`{a}`/`{b}`），None 时直接显示名称
    pub formatter: Option<String>,
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
    pub categories: Vec<String>,
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
            background: Color::rgb(255, 255, 255),
            series_colors: Vec::new(),
            axis_line_color: Color::rgb(200, 200, 200),
            axis_label_color: Color::rgb(50, 50, 50),
            grid_line_color: Color::rgb(230, 230, 230),
            border_color: Color::rgb(255, 255, 255),
            text_color: Color::rgb(51, 51, 51),
            text_secondary_color: Color::rgb(102, 102, 102),
            up_color: Color::rgb(234, 85, 67),
            down_color: Color::rgb(80, 170, 94),
            table_header_bg: Color::rgb(220, 220, 220),
            table_row_even_bg: Color::rgb(248, 248, 248),
            table_row_odd_bg: Color::rgb(255, 255, 255),
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
                    Color::rgb(80, 112, 221), // 蓝色
                    Color::rgb(182, 214, 52), // 绿色
                    Color::rgb(234, 85, 67),  // 红色
                    Color::rgb(255, 193, 7),  // 黄色
                    Color::rgb(156, 39, 176), // 紫色
                    Color::rgb(0, 188, 212),  // 青色
                    Color::rgb(255, 87, 34),  // 橙色
                    Color::rgb(96, 125, 139), // 蓝灰色
                ];
                default_colors
                    .get(index % default_colors.len())
                    .copied()
                    .unwrap_or(Color::rgb(80, 112, 221))
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
            .unwrap_or(Color::rgb(80, 112, 221))
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
        (layout.width, layout.height)
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
        (layout.width, layout.height)
    }
}
