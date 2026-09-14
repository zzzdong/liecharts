use lievisual::text::{RichSpan, measure_text};
use vello_cpu::kurbo::Rect;

use crate::{Color, SceneNode, TextStyle, theme::DEFAULT_FONT_STACK};

// ═══════════════════════════════════════════════════════════════════
// NEW: ChartSpec — pipeline 的统一输入类型
// ═══════════════════════════════════════════════════════════════════

/// Pipeline 的统一输入规格。可从新 API (Chart) 或旧 option (ChartOption) 转换而来。
/// 画布尺寸语义（P1/P4 引入，见 docs/布局自适应改造计划.md §五）
///
/// - [`FitMode::Fixed`]（默认）：`width`/`height` 是刚性画布，空间不足时
///   按既有策略向内收缩/旋转/抽稀/压缩（历史行为，逐字节兼容）。
/// - [`FitMode::Hug`]：`width`/`height` 是**期望尺寸**。布局组件上报的
///   空间需求（轴标签、图例换行、表格最小行高）会通过迭代求解把画布
///   **按需长大**（信息零损失优先），字号与线宽恒定、不做整体缩放。
/// - [`FitMode::HugMax`]：同 [`FitMode::Hug`]，但 `width`/`height` 同时是
///   **上限**：内容长大超过上限时，渲染阶段整体等比缩放回上限内
///   （`lievisual::fit::fit_scene`，对齐 liemermaid 的 `fit_options` 语义）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FitMode {
    #[default]
    Fixed,
    Hug,
    HugMax,
}

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
    /// 画布尺寸语义，默认 [`FitMode::Fixed`]
    pub fit_mode: FitMode,
}

/// grid 边距的原始语义（P2b 引入，见 docs/布局自适应改造计划.md P2b）
///
/// 延迟到像素布局阶段解析：`Pct` 相对画布对应维度，画布变化（Hug）时
/// 随比例缩放，不再被 api/compat 层提前固化成像素。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridEdge {
    /// 绝对像素
    Px(f64),
    /// 相对画布宽/高的百分比
    Pct(f64),
}

/// P2b 起边距保留原始语义（百分比 / 像素），由 `GridPlanner` 在布局阶段解析
#[derive(Debug, Clone)]
pub struct GridSpec {
    pub left: Option<GridEdge>, // None = auto（按 contain_label 选默认）
    pub right: Option<GridEdge>,
    pub top: Option<GridEdge>,
    pub bottom: Option<GridEdge>,
    /// 显式绘图区宽度（ECharts `grid.width`）。`Some` 时覆盖 `right`。
    pub width: Option<GridEdge>,
    /// 显式绘图区高度（ECharts `grid.height`）。`Some` 时覆盖 `bottom`。
    pub height: Option<GridEdge>,
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

/// 坐标轴装饰样式（ECharts `axisLine` / `axisTick` / `splitLine` / `splitArea` 的子集）。
///
/// 默认值与 ECharts 默认语义一致：轴线/刻度/分隔线都在，但**类目轴的
/// `splitLine` 默认关闭**（ECharts 文档：分隔线「默认数值轴显示，类目轴不显示」），
/// 且类目刻度默认对齐**带边界**而非类目中心（`axisTick.alignWithLabel` 默认 false）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AxisDecor {
    /// `axisLine.show`
    pub line_show: bool,
    pub line_color: Option<Color>,
    pub line_width: f64,
    /// `axisTick.show`
    pub tick_show: bool,
    pub tick_length: f64,
    /// `axisTick.alignWithLabel`：true 时刻度落在类目中心，false 落在带边界
    pub align_with_label: bool,
    /// `splitLine.show`
    pub split_line_show: bool,
    pub split_line_color: Option<Color>,
    /// `splitLine.lineStyle.type` 展开的虚线段长（空 = 实线）
    pub split_line_dash: Vec<f64>,
    /// `splitArea.show`：沿轴绘制交替色带
    pub split_area_show: bool,
    pub split_area_colors: Vec<Color>,
}

impl AxisDecor {
    /// ECharts 语义的默认值（不用 `derive(Default)`，因为布尔默认并非全 false）
    pub fn echant_defaults() -> Self {
        Self {
            line_show: true,
            line_color: None,
            line_width: 1.0,
            tick_show: true,
            tick_length: 5.0,
            align_with_label: false,
            split_line_show: true,
            split_line_color: None,
            split_line_dash: Vec::new(),
            split_area_show: false,
            split_area_colors: Vec::new(),
        }
    }

    /// 按轴类型套用 ECharts 默认显隐：类目轴默认不画分隔线。
    pub fn for_axis_type(axis_type: AxisType) -> Self {
        let mut d = Self::echant_defaults();
        if axis_type == AxisType::Category {
            d.split_line_show = false;
        }
        d
    }
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
    /// `nameGap`：轴名与轴线的距离（None = 默认）
    pub name_gap: Option<f64>,
    pub categories: Vec<String>, // Category 轴的标签
    pub boundary_gap: bool,
    pub inverse: bool,
    pub split_number: Option<usize>,
    pub label_show: bool,
    pub label_formatter: Option<String>,
    pub label_rotate: Option<f64>,
    /// `axisLabel.interval`：显式刻度抽稀步长（None / 非正数 = 自动）
    pub label_interval: Option<f64>,
    /// 轴线 / 刻度 / 分隔线 / 色带的样式与显隐
    pub decor: AxisDecor,
    pub z: Option<f64>,
}

impl AxisSpec {
    /// Category 轴的类目总数（刻度标签的来源）。
    pub fn category_count(&self) -> usize {
        self.categories.len()
    }

    /// 分类轴第 `i` 个类目的归一化位置（见 [`category_norm`]）。
    ///
    /// 已含 `boundary_gap` 与 `inverse` 的处理：`inverse` 时类目顺序反转
    /// （类目 0 落到轴的另一端），与数据侧（像素映射）的口径一致。
    pub fn category_norm(&self, i: usize) -> f64 {
        category_norm(i, self.category_count(), self.boundary_gap, self.inverse)
    }
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

    /// 该系列在指定维度上承载「类目名」的列名（`is_x` = X 轴维度）。
    ///
    /// 用于类目轴缺 `xAxis.data` 时从系列数据反推类目名（ECharts 语义：
    /// `xAxis:{type:'category'}` 配合 `dataset`/`series.data` 即可自动生成类目）。
    pub fn axis_col(&self, is_x: bool) -> Option<&str> {
        match self {
            SeriesConfig::Line(c) => Some(if is_x { &c.x_col } else { &c.y_col }),
            SeriesConfig::Bar(c) => Some(if is_x { &c.x_col } else { &c.y_col }),
            SeriesConfig::Scatter(c) => Some(if is_x { &c.x_col } else { &c.y_col }),
            SeriesConfig::Bubble(c) => Some(if is_x { &c.x_col } else { &c.y_col }),
            SeriesConfig::Heatmap(c) => Some(if is_x { &c.x_col } else { &c.y_col }),
            SeriesConfig::Candlestick(c) => Some(if is_x { &c.category_col } else { &c.close_col }),
            SeriesConfig::Boxplot(c) => Some(if is_x { &c.category_col } else { &c.median_col }),
            _ => None,
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

// ── MarkPoint ──

/// 标注点类型（ECharts `markPoint.data[].type`）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarkPointType {
    Average,
    Min,
    Max,
}

/// 标注点配置（来自 `series.markPoint`）
#[derive(Debug, Clone)]
pub struct MarkPointSpec {
    pub data_type: MarkPointType,
    pub name: Option<String>,
    /// 显式覆盖值（`markPoint.data[].value`），`None` 时按类型统计
    pub value: Option<f64>,
}

// ── LineConfig ──

#[derive(Debug, Clone)]
pub struct LineConfig {
    pub x_col: String,
    pub y_col: String,
    pub smooth: bool,
    pub step: Option<StepType>,
    pub line_width: f64,
    /// 虚线段长（`lineStyle.type: dashed/dotted` 展开）。空 = 实线。
    pub line_dash: Vec<f64>,
    /// `lineStyle.color`：显式线色（优先于调色板）
    pub line_color: Option<Color>,
    /// `connectNulls`：true 时跨越 null 数据点连成一条线（ECharts 默认 **false**，
    /// 即在 null 处断开）。
    pub connect_nulls: bool,
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
    /// 值标签位置（None = Top，ECharts 默认）
    pub label_position: Option<ValueLabelPos>,
    /// 值标签颜色（None = 跟随系列色，ECharts 默认）
    pub label_color: Option<Color>,
    /// 标注线配置
    pub mark_line: Vec<MarkLineSpec>,
    /// 标注点配置
    pub mark_point: Vec<MarkPointSpec>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValueLabelPos {
    /// 数据点/柱顶外侧（默认）
    #[default]
    Top,
    /// 数据点下方（折线）
    Bottom,
    /// 柱内（柱状图：内顶部白字；折线图回退 Top）
    Inside,
}

impl Default for LineConfig {
    fn default() -> Self {
        Self {
            x_col: "x".into(),
            y_col: "y".into(),
            smooth: false,
            step: None,
            line_width: 2.0,
            line_dash: Vec::new(),
            line_color: None,
            connect_nulls: false,
            area: false,
            area_color: None,
            area_opacity: 0.5,
            symbol_type: SymbolType::EmptyCircle,
            symbol_size: 4.0,
            label_show: false,
            label_font_size: 12.0,
            label_formatter: None,
            label_position: None,
            label_color: None,
            mark_line: Vec::new(),
            mark_point: Vec::new(),
        }
    }
}

// ── BarConfig ──

/// 柱宽尺寸：比例（相对类目槽宽）或绝对像素。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarSize {
    /// 相对类目槽宽的比例（`"60%"`）
    Ratio(f64),
    /// 绝对像素（`60`）
    Px(f64),
}

/// 柱状图几何布局（ECharts `barWidth` / `barGap` / `barCategoryGap` / `barMaxWidth` / `barMinWidth`）。
///
/// ECharts 默认：`barCategoryGap:'20%'`（整组占槽宽 80%）、`barGap:'30%'`（组内柱间距
/// 为柱宽的 30%）。此前 liecharts 固定「带宽 60% 且组内紧贴」，与 ECharts 输出不一致。
#[derive(Debug, Clone, PartialEq)]
pub struct BarLayout {
    /// 显式 `barWidth`；None = 由 `category_gap` 推导
    pub width: Option<BarSize>,
    /// `barGap`：组内相邻柱间距
    pub gap: BarSize,
    /// `barCategoryGap`：类目槽两侧的总留白
    pub category_gap: BarSize,
    pub max_width: Option<BarSize>,
    pub min_width: Option<BarSize>,
    /// `barMinHeight`：值方向的像素下限（保证极小值也可见）
    pub min_height: Option<f64>,
    /// `showBackground` 时的背景柱颜色（None = 不画）
    pub background: Option<Color>,
}

impl Default for BarLayout {
    fn default() -> Self {
        Self {
            width: None,
            gap: BarSize::Ratio(0.3),
            category_gap: BarSize::Ratio(0.2),
            max_width: None,
            min_width: None,
            min_height: None,
            background: None,
        }
    }
}

impl BarLayout {
    /// 计算单根柱的像素宽度与组内间距。
    ///
    /// `slot` 为类目槽宽（像素），`series_count` 为**同组并排**柱数（堆叠视为 1）。
    pub fn band(&self, slot: f64, series_count: usize) -> (f64, f64) {
        let n = series_count.max(1) as f64;
        let gap_of = |w: f64| match self.gap {
            BarSize::Ratio(r) => w * r.max(0.0),
            BarSize::Px(px) => px.max(0.0),
        };

        let mut w = if let Some(width) = self.width {
            match width {
                BarSize::Ratio(r) => slot * r,
                BarSize::Px(px) => px,
            }
        } else {
            let total = match self.category_gap {
                BarSize::Ratio(r) => slot * (1.0 - r.clamp(0.0, 1.0)),
                BarSize::Px(px) => (slot - px).max(0.0),
            };
            match self.gap {
                BarSize::Ratio(r) => total / (n + (n - 1.0) * r.max(0.0)),
                BarSize::Px(px) => ((total - (n - 1.0) * px) / n).max(0.0),
            }
        };

        if let Some(mx) = self.max_width {
            let v = match mx {
                BarSize::Ratio(r) => slot * r,
                BarSize::Px(px) => px,
            };
            w = w.min(v);
        }
        if let Some(mn) = self.min_width {
            let v = match mn {
                BarSize::Ratio(r) => slot * r,
                BarSize::Px(px) => px,
            };
            w = w.max(v);
        }
        (w, gap_of(w))
    }
}

#[derive(Debug, Clone)]
pub struct BarConfig {
    pub x_col: String,
    pub y_col: String,
    /// 柱体几何（ECharts barWidth/barGap/barCategoryGap/…）
    pub layout: BarLayout,
    /// 是否显示值标签
    pub label_show: bool,
    pub label_font_size: f64,
    /// 值标签模板（支持 `{a}`/`{b}`/`{c}`/`{value}`）
    pub label_formatter: Option<String>,
    /// 值标签位置（None = Top，ECharts 默认；柱状图 Top = 值端外侧）
    pub label_position: Option<ValueLabelPos>,
    /// 值标签颜色（None = Top 跟随系列色 / Inside 白字，ECharts 默认）
    pub label_color: Option<Color>,
    /// 标注线配置
    pub mark_line: Vec<MarkLineSpec>,
    /// 标注点配置
    pub mark_point: Vec<MarkPointSpec>,
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            x_col: "x".into(),
            y_col: "y".into(),
            layout: BarLayout::default(),
            label_show: false,
            label_font_size: 12.0,
            label_formatter: None,
            label_position: None,
            label_color: None,
            mark_line: Vec::new(),
            mark_point: Vec::new(),
        }
    }
}

// ── PieConfig ──

/// 玫瑰图模式（ECharts `series-pie.roseType`）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieRoseType {
    /// 半径映射数值
    Radius,
    /// 面积（半径按 √value）映射数值
    Area,
}

#[derive(Debug, Clone)]
pub struct PieConfig {
    pub category_col: String,
    pub value_col: String,
    pub center: (f64, f64),
    /// 内外半径，**绝对像素**（P2a 起由 api/compat 层以「画布 min/2」为基准折算，
    /// 见 docs/布局自适应改造计划.md P2a）。
    ///
    /// 渲染前会经 `builder::resolve_radius` 收口：`> 0` 时再 clamp 到绘图区内接
    /// 半径（防止多 subplot / 紧边距越界）；`<= 0` 表示**未指定**，按内接半径
    /// 自适应（内径 0、外径 75%，见 docs 同篇 P5）。
    pub radius: (f64, f64),
    pub label_show: bool,
    pub label_position: LabelPosition,
    pub label_font_size: f64,
    /// 标签格式化模板，支持 `{b}`（名称）、`{c}`（数值）、`{d}`（百分比）
    pub label_formatter: Option<String>,
    /// `labelLine.show`：外部标签的引导线（默认 true）
    pub label_line_show: bool,
    /// `clockwise`：扇区按顺时针排布（ECharts 默认 true）
    pub clockwise: bool,
    /// `roseType`：玫瑰图模式
    pub rose_type: Option<PieRoseType>,
    /// `padAngle`：扇区之间的间隔角（弧度）
    pub pad_angle: f64,
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
            // <=0 = 未指定 → 渲染时按绘图区内接半径自适应（内 0 / 外 75%）；
            // 直接构造 `ChartSpec` 时不会退化成固定 75px 的小饼
            radius: (0.0, 0.0),
            label_show: false,
            label_position: LabelPosition::Outside,
            label_font_size: 12.0,
            label_formatter: None,
            label_line_show: true,
            clockwise: true,
            rose_type: None,
            pad_angle: 0.0,
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
    /// `itemStyle.color`：阳线（涨）颜色。None = 主题色
    pub up_color: Option<Color>,
    /// `itemStyle.color0`：阴线（跌）颜色。None = 主题色
    pub down_color: Option<Color>,
    /// `itemStyle.border_color`：阳线描边色。None = 主题色
    pub border_color: Option<Color>,
}

impl Default for CandlestickConfig {
    fn default() -> Self {
        Self {
            category_col: "category".into(),
            open_col: "open".into(),
            close_col: "close".into(),
            low_col: "low".into(),
            high_col: "high".into(),
            up_color: None,
            down_color: None,
            border_color: None,
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
    /// 每个指标的最大值（`radar.indicator[].max`，缺省 100）。
    /// 顶点半径 = `radius × value / max`，与雷达网格口径一致。
    pub maxes: Vec<f64>,
    /// 是否显示各顶点的数值标签（`series[].label.show`）
    pub label_show: bool,
    pub label_font_size: f64,
}

impl Default for RadarConfig {
    fn default() -> Self {
        Self {
            value_col: "value".into(),
            indicators: vec![],
            maxes: vec![],
            label_show: false,
            label_font_size: 12.0,
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
    /// 外半径，**绝对像素**（P2a 起由 api/compat 层以「画布 min/2」为基准折算）。
    ///
    /// 渲染前经 `builder::resolve_radius` 收口：`<= 0` 表示未指定，按绘图区
    /// 内接半径的 75% 自适应（见 docs/布局自适应改造计划.md P5）。
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
            // <=0 = 未指定 → 渲染时按绘图区内接半径的 75% 自适应
            radius: 0.0,
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
    /// 圆角半径（柱状图生效）
    pub border_radius: Option<f64>,
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
    /// `title.show`（false 时不渲染）
    pub show: bool,
    /// `left`：`"center"` / `"left"` / `"right"` / `"20%"` / `20`（原始字符串，渲染期解析）
    pub left: Option<String>,
    /// `right`：同 `left` 的取值形式
    pub right: Option<String>,
    /// `top`：`"top"` / `"middle"` / `"bottom"` / `"20%"` / `20`
    pub top: Option<String>,
    /// `textAlign`：`"auto" | "left" | "center" | "right"`
    pub text_align: Option<String>,
    /// `itemGap`：主/副标题间距
    pub item_gap: Option<f64>,
}

impl Default for TitleSpec {
    fn default() -> Self {
        Self {
            text: None,
            subtext: None,
            font_size: None,
            subfont_size: None,
            color: None,
            subcolor: None,
            show: true,
            left: None,
            right: None,
            top: None,
            text_align: None,
            item_gap: None,
        }
    }
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

/// 分类轴第 `i` 个类目（共 `n` 个）的类目下标；`inverse` 时反转顺序。
fn category_index(i: usize, n: usize, inverse: bool) -> usize {
    if inverse { n.saturating_sub(1 + i) } else { i }
}

/// 分类轴第 `i` 个类目（共 `n` 个）的归一化位置 `t ∈ [0,1]`
///（X 轴像素 = `x0 + t·W`，Y 轴 = `y1 − t·H`）。
///
/// 口径与轴范围解析（`axis_binding_resolver`）保持一致：
/// - `boundary_gap = true`：解析范围 `[0, n]`，类目落在带中心 `(i+0.5)/n`；
/// - `boundary_gap = false`：解析范围 `[0, n-1]`，类目落在数据点 `i/(n-1)`；
/// - `n <= 1` 且无留白：范围退化为 `[0, 0]`，统一取 0。
///
/// `inverse = true`（ECharts `axis.inverse`）时类目顺序反转。
pub fn category_norm(i: usize, n: usize, boundary_gap: bool, inverse: bool) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let j = category_index(i, n, inverse);
    if boundary_gap {
        (j as f64 + 0.5) / n as f64
    } else if n <= 1 {
        0.0
    } else {
        j as f64 / (n - 1) as f64
    }
}

/// 分类轴第 `i` 个类目在轴**数据空间**中的坐标，可直接交给
/// `map_x_to_pixel` / `map_y_to_pixel`。
///
/// 与 [`category_norm`] 同源，区别是不做归一化（留白时返回带中心 `i+0.5`）。
/// 该坐标恒落在 `[min, max]` 内，调用方无需再 clamp。
///
/// 注意：这里**不**处理 `inverse` —— 类目顺序反转统一由像素映射
/// （`map_x_to_pixel` / `map_y_to_pixel`）实现，保证折线、散点等按原始数据坐标
/// 映射的系列与柱状图族行为一致。
pub fn category_value(i: usize, n: usize, boundary_gap: bool) -> f64 {
    if n == 0 {
        return 0.0;
    }
    i as f64 + if boundary_gap { 0.5 } else { 0.0 }
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
    /// 坐标轴反向（`AxisSpec.inverse`）。像素方向翻转，但 `min`/`max` 的先后不交换：
    /// - X 轴：`min` 在右端、`max` 在左端；
    /// - Y 轴：`min` 在顶部、`max` 在底部；
    /// - Category 轴：类目顺序反转（`category_value`）。
    ///
    /// 该标志同时被像素映射（`map_x_to_pixel` / `map_y_to_pixel`）与刻度绘制读取，
    /// 二者口径一致。
    pub inverse: bool,
    /// 分类轴是否在两端留白（`AxisSpec.boundary_gap`）。
    ///
    /// 决定类目落在**带中心**（`[0, n]`，留白）还是**数据点**（`[0, n-1]`，无留白）。
    /// 此前各 materializer 靠 `(max - min) >= n` 反推该标志，在多系列行数不等时会
    /// 误判并导致系列间错位，故改为随解析结果直传。
    pub boundary_gap: bool,
}

impl ResolvedAxisRange {
    pub fn is_y_axis(&self) -> bool {
        matches!(self.position, AxisPosition::Left | AxisPosition::Right)
    }

    /// Category 轴的类目总数；非 Category 轴返回 0。
    ///
    /// 优先取轴声明的 [`Self::categories`]（与刻度标签同源），否则由解析范围反推：
    /// 留白时范围 `[0, n]` → `n = span`；无留白时 `[0, n-1]` → `n = span + 1`。
    pub fn category_count(&self) -> usize {
        if self.axis_type != AxisType::Category {
            return 0;
        }
        if !self.categories.is_empty() {
            return self.categories.len();
        }
        let span = (self.max - self.min).max(0.0);
        let n = if self.boundary_gap { span } else { span + 1.0 };
        n.round().max(1.0) as usize
    }

    /// 分类轴第 `i` 个类目的数据空间坐标（见 [`category_value`]）。
    ///
    /// 数据坐标始终按**升序**（类目 0 对应 `min` 端）排列；`inverse` 由像素映射
    /// （`map_x_to_pixel` / `map_y_to_pixel`）统一翻转，与折线等按数据坐标映射的
    /// 系列保持一致。
    pub fn category_value(&self, i: usize) -> f64 {
        category_value(i, self.category_count(), self.boundary_gap)
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
    pub series_elements: Vec<SceneNode>,
    pub axis_elements: Vec<SceneNode>,
    pub grid_lines: Vec<SceneNode>,
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
        let mut lv_style = style.clone();
        if lv_style.font_family.trim().is_empty()
            || lv_style
                .font_family
                .trim()
                .eq_ignore_ascii_case("sans-serif")
        {
            lv_style.font_family = DEFAULT_FONT_STACK.to_string();
        }
        let layout =
            (*measure_text(&[RichSpan::new(text.to_string(), lv_style)], None).layout).clone();
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
        let mut lv_style = style.clone();
        if lv_style.font_family.trim().is_empty()
            || lv_style
                .font_family
                .trim()
                .eq_ignore_ascii_case("sans-serif")
        {
            lv_style.font_family = DEFAULT_FONT_STACK.to_string();
        }
        let layout = (*measure_text(
            &[RichSpan::new(text.to_string(), lv_style)],
            Some(max_width),
        )
        .layout)
            .clone();
        (layout.width, layout.height)
    }
}
