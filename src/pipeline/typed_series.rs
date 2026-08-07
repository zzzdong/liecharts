//! TypedSeries: 管线中间产物，所有字段已完全解析为像素空间和具体值
//!
//! 这是 Materialize 阶段的输出，Builder 阶段的输入。
//! 所有坐标已经是像素空间，渲染器无需再做任何计算或字段提取。

use vello_cpu::kurbo::{Point, Rect};

use crate::visual::Color;

/// 管线中间产物：所有字段已完全解析为像素空间和具体值
/// 渲染器无需再做任何计算或字段提取
#[derive(Debug, Clone)]
pub enum TypedSeries {
    Line(LineSeries),
    Bar(BarSeries),
    GroupedBar(GroupedBarSeries),
    Scatter(ScatterSeries),
    Bubble(BubbleSeries),
    Candlestick(CandlestickSeries),
    Boxplot(BoxplotSeries),
    Heatmap(HeatmapSeries),
    Pie(PieSeries),
    Radar(RadarSeries),
    PolarBar(PolarBarSeries),
    PolarScatter(PolarScatterSeries),
    Gauge(GaugeSeries),
    Table(TableSeries),
}

/// 渲染上下文
pub struct RenderContext<'a> {
    /// 颜色上下文（仅用于轴/网格线/边框等装饰元素）
    pub colors: &'a crate::pipeline::types::ColorContext,
    /// 主题（文本样式等）
    pub theme: &'a crate::theme::Theme,
    /// 画布边界（用于将百分比坐标转换为像素坐标）
    pub bounds: vello_cpu::kurbo::Rect,
}

// ═══════════════════════════════════════════════════════════════════
// LineSeries
// ═══════════════════════════════════════════════════════════════════

/// Step line style for line charts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepType {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone)]
pub struct LineSeries {
    pub name: String,
    /// 样式（已解析）
    pub color: Color,
    pub line_width: f64,
    pub smooth: bool,
    pub step: Option<StepType>,
    pub area_color: Option<Color>,
    pub area_opacity: f64,
    pub symbol_type: SymbolType,
    pub symbol_size: f64,
    /// 数据点（像素空间坐标，无需再映射）
    pub points: Vec<Point>,
    /// 面积填充的基线 Y（像素空间）
    pub baseline_y: f64,
    /// 堆叠面积图的底部轮廓点（像素空间）
    /// - None: 以 baseline_y 为底（最底层的面积图）
    /// - Some(points): 以此前的系列线条为上界的轮廓（堆叠面积图）
    pub baseline_points: Option<Vec<Point>>,
    /// 数据点的原始数值（用于 label 显示）
    pub values: Vec<f64>,
    /// 标签配置
    pub label: Option<SeriesLabelConfig>,
    /// 标注线（像素空间，横向贯穿整个绘图区）
    pub mark_lines: Vec<MarkLineRender>,
}

/// 渲染用标注线
#[derive(Debug, Clone)]
pub struct MarkLineRender {
    /// 线的 Y 像素坐标（横向标注线）
    pub y: f64,
    /// 标注文本
    pub label: String,
    /// 标注线颜色
    pub color: Color,
}

// ═══════════════════════════════════════════════════════════════════
// BarSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct BarSeries {
    pub name: String,
    /// 样式
    pub color: Color,
    /// 数据点（像素空间：每个条目已经算好了像素矩形）
    pub bars: Vec<BarRect>,
    /// 标签配置
    pub label: Option<SeriesLabelConfig>,
    /// 标注线（像素空间，横向贯穿整个绘图区）
    pub mark_lines: Vec<MarkLineRender>,
}

#[derive(Debug, Clone)]
pub struct BarRect {
    pub rect: Rect,       // 像素空间的矩形
    pub category: String, // 类别名（用于 label）
    pub value: f64,       // 原始值（用于 label）
}

// ═══════════════════════════════════════════════════════════════════
// GroupedBarSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarGroupType {
    SideBySide,
    Stacked,
}

#[derive(Debug, Clone)]
pub struct GroupedBarSeries {
    /// 每个子系列的名称和颜色
    pub sub_series: Vec<BarSubSeries>,
    pub group_type: BarGroupType,
    /// 数据（像素空间）
    pub rows: Vec<GroupedBarRow>,
    /// 标签配置（None 时不渲染数据标签）
    pub label: Option<SeriesLabelConfig>,
}

#[derive(Debug, Clone)]
pub struct BarSubSeries {
    pub name: String,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct GroupedBarRow {
    pub bar_rect: Rect,        // 像素空间的矩形
    pub sub_series_idx: usize, // 指向 sub_series 的索引
    pub color: Color,          // 子系列颜色
    pub category: String,      // 类别名（用于 label）
    pub value: f64,            // 原始值（用于 label）
}

// ═══════════════════════════════════════════════════════════════════
// ScatterSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ScatterSeries {
    pub name: String,
    pub color: Color,
    pub symbol_size: f64,
    pub points: Vec<Point>, // 像素空间
}

// ═══════════════════════════════════════════════════════════════════
// BubbleSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct BubbleSeries {
    pub name: String,
    pub color: Color,
    pub bubbles: Vec<Bubble>, // 像素空间：center + radius + name
}

#[derive(Debug, Clone)]
pub struct Bubble {
    pub center: Point, // 像素空间中心
    pub radius: f64,   // 像素空间半径
    pub name: String,  // 气泡名（用于 label）
}

// ═══════════════════════════════════════════════════════════════════
// CandlestickSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct CandlestickSeries {
    pub name: String,
    pub up_color: Color,
    pub down_color: Color,
    pub candles: Vec<CandleRect>,
}

#[derive(Debug, Clone)]
pub struct CandleRect {
    pub category: String,
    pub high_line: (Point, Point), // 上影线端点（像素空间）
    pub low_line: (Point, Point),  // 下影线端点（像素空间）
    pub body_rect: Rect,           // 实体矩形（像素空间）
    pub is_up: bool,               // 涨跌
}

// ═══════════════════════════════════════════════════════════════════
// BoxplotSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct BoxplotSeries {
    pub name: String,
    pub color: Color,
    pub border_color: Color,
    pub border_width: f64,
    pub boxes: Vec<BoxplotRect>,
}

#[derive(Debug, Clone)]
pub struct BoxplotRect {
    pub category: String,
    /// 从 min 到 max 的 whisker 垂直线
    pub whisker_line: (Point, Point),
    /// whisker 顶端（max）横线
    pub top_whisker: (Point, Point),
    /// whisker 底端（min）横线
    pub bottom_whisker: (Point, Point),
    /// 箱体矩形：从 Q1 到 Q3
    pub body_rect: Rect,
    /// 中位数横线
    pub median_line: (Point, Point),
}

// ═══════════════════════════════════════════════════════════════════
// HeatmapSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct HeatmapSeries {
    pub name: String,
    /// 单元格（像素空间矩形 + 已解析的映射颜色）
    pub cells: Vec<HeatmapCell>,
}

#[derive(Debug, Clone)]
pub struct HeatmapCell {
    pub rect: Rect,
    pub value: f64,
    pub color: Color,
    pub border_color: Option<Color>,
    pub border_width: f64,
}

// ═══════════════════════════════════════════════════════════════════
// PieSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct PieSeries {
    pub name: String,
    /// 布局参数（已解析）
    pub radius_inner: f64, // 百分比
    pub radius_outer: f64, // 百分比
    pub label_show: bool,
    pub label_position: LabelPosition,
    pub label_font_size: f64,
    /// 标签格式化模板，支持 `{b}`（名称）、`{c}`（数值）、`{d}`（百分比）
    pub label_formatter: Option<String>,
    /// 扇区数据
    pub slices: Vec<PieSlice>,
}

#[derive(Debug, Clone)]
pub struct PieSlice {
    pub name: String,
    pub value: f64,
    pub color: Color,
    pub percent: f64, // 0.0~1.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelPosition {
    Outside,
    Inside,
}

// ═══════════════════════════════════════════════════════════════════
// RadarSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct RadarSeries {
    pub name: String,
    pub color: Color,
    pub indicators: Vec<String>,
    pub values: Vec<f64>,
}

// ═══════════════════════════════════════════════════════════════════
// PolarBarSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct PolarBarSeries {
    pub name: String,
    pub color: Color,
    pub pad_angle: f64,
    pub start_angle: f64,
    pub bars: Vec<PolarBarPoint>, // angle, radius
}

#[derive(Debug, Clone)]
pub struct PolarBarPoint {
    pub angle: f64,  // 角度（度）
    pub radius: f64, // 半径（像素）
    pub value: f64,
    pub name: String,
    pub color: Color, // 每个柱子的颜色
}

// ═══════════════════════════════════════════════════════════════════
// PolarScatterSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct PolarScatterSeries {
    pub name: String,
    pub color: Color,
    pub symbol_size: f64,
    pub points: Vec<PolarPoint>,
}

#[derive(Debug, Clone)]
pub struct PolarPoint {
    pub angle: f64,  // 角度（度）
    pub radius: f64, // 半径（像素）
    pub value: f64,
    pub name: String,
    pub size: f64, // 气泡大小（像素半径）
}

// ═══════════════════════════════════════════════════════════════════
// GaugeSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct GaugeSeries {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub center: (f64, f64), // 百分比
    pub radius: f64,        // 百分比
    pub start_angle: f64,
    pub end_angle: f64,
    pub split_number: usize,
    pub value: f64,
    pub color: Color,
}

// ═══════════════════════════════════════════════════════════════════
// TableSeries
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct TableSeries {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub header_bg: Color,
    pub header_border_color: Color,
    pub row_even_bg: Color,
    pub row_odd_bg: Color,
}

// ═══════════════════════════════════════════════════════════════════
// 共享类型
// ═══════════════════════════════════════════════════════════════════

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

// ── LabelConfig ──

/// 标签显示配置
#[derive(Debug, Clone)]
pub struct SeriesLabelConfig {
    pub show: bool,
    pub position: SeriesLabelPosition,
    pub color: Color,
    pub font_size: f64,
    /// 标签模板（支持 `{a}`/`{b}`/`{c}`/`{d}`/`{value}`），None 时显示数值
    pub formatter: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeriesLabelPosition {
    /// 在柱子上方 / 折线点上方
    Top,
    /// 在柱子内部
    Inside,
}

// ═══════════════════════════════════════════════════════════════════
// TypedSeries 辅助方法
// ═══════════════════════════════════════════════════════════════════

impl TypedSeries {
    /// 获取系列名称
    pub fn name(&self) -> &str {
        match self {
            TypedSeries::Line(s) => &s.name,
            TypedSeries::Bar(s) => &s.name,
            TypedSeries::GroupedBar(s) => {
                // GroupedBar 可能有多个子系列，返回第一个或空字符串
                s.sub_series
                    .first()
                    .map(|ss| ss.name.as_str())
                    .unwrap_or("")
            }
            TypedSeries::Scatter(s) => &s.name,
            TypedSeries::Bubble(s) => &s.name,
            TypedSeries::Candlestick(s) => &s.name,
            TypedSeries::Boxplot(s) => &s.name,
            TypedSeries::Heatmap(s) => &s.name,
            TypedSeries::Pie(s) => &s.name,
            TypedSeries::Radar(s) => &s.name,
            TypedSeries::PolarBar(s) => &s.name,
            TypedSeries::PolarScatter(s) => &s.name,
            TypedSeries::Gauge(s) => &s.name,
            TypedSeries::Table(s) => &s.name,
        }
    }

    /// 获取图表类型名称
    pub fn chart_type_name(&self) -> &'static str {
        match self {
            TypedSeries::Line(_) => "line",
            TypedSeries::Bar(_) => "bar",
            TypedSeries::GroupedBar(_) => "grouped_bar",
            TypedSeries::Scatter(_) => "scatter",
            TypedSeries::Bubble(_) => "bubble",
            TypedSeries::Candlestick(_) => "candlestick",
            TypedSeries::Boxplot(_) => "boxplot",
            TypedSeries::Heatmap(_) => "heatmap",
            TypedSeries::Pie(_) => "pie",
            TypedSeries::Radar(_) => "radar",
            TypedSeries::PolarBar(_) => "polar_bar",
            TypedSeries::PolarScatter(_) => "polar_scatter",
            TypedSeries::Gauge(_) => "gauge",
            TypedSeries::Table(_) => "table",
        }
    }
}
