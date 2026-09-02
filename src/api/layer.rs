use crate::{
    Color,
    api::Size,
    pipeline::{
        dataframe::DataFrame,
        types::{LabelPosition, ValueLabelPos},
    },
};

// ── StepType ──

/// Step line style for line charts.
///
/// Controls whether the step appears at the start, middle, or end of each segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepType {
    Start,
    Middle,
    End,
}

// ── Sampling ──

/// Data sampling strategy for reducing the number of data points
/// while preserving visual features.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sampling {
    Lttb(usize),
    Average(usize),
    Max(usize),
    Min(usize),
}

impl Sampling {
    pub fn threshold(&self) -> usize {
        match self {
            Sampling::Lttb(n) | Sampling::Average(n) | Sampling::Max(n) | Sampling::Min(n) => *n,
        }
    }
}

impl From<Sampling> for (crate::sampling::SamplingType, usize) {
    fn from(s: Sampling) -> Self {
        match s {
            Sampling::Lttb(n) => (crate::sampling::SamplingType::Lttb, n),
            Sampling::Average(n) => (crate::sampling::SamplingType::Average, n),
            Sampling::Max(n) => (crate::sampling::SamplingType::Max, n),
            Sampling::Min(n) => (crate::sampling::SamplingType::Min, n),
        }
    }
}

/// A chart layer that maps a DataFrame to visual elements.
#[derive(Debug, Clone)]
pub enum LayerSpec {
    Line(Line),
    Bar(Bar),
    Pie(Pie),
    Scatter(Scatter),
    Bubble(Bubble),
    Candlestick(Candlestick),
    Boxplot(Boxplot),
    Heatmap(Heatmap),
    Radar(Radar),
    PolarBar(PolarBar),
    PolarScatter(PolarScatter),
    Gauge(Gauge),
    Table(Table),
}

impl LayerSpec {
    pub fn set_grid_index(&mut self, idx: usize) {
        match self {
            LayerSpec::Line(l) => l.grid_index = idx,
            LayerSpec::Bar(l) => l.grid_index = idx,
            LayerSpec::Scatter(l) => l.grid_index = idx,
            LayerSpec::Bubble(l) => l.grid_index = idx,
            LayerSpec::Candlestick(l) => l.grid_index = idx,
            LayerSpec::Boxplot(l) => l.grid_index = idx,
            LayerSpec::Heatmap(l) => l.grid_index = idx,
            _ => {}
        }
    }
}

// ── Line ──

#[derive(Debug, Clone)]
pub struct Line {
    pub name: String,
    pub data: Option<DataFrame>,
    pub x: String,
    pub y: String,
    pub smooth: bool,
    pub step: Option<StepType>,
    pub stack: Option<String>,
    pub symbol: SymbolType,
    pub symbol_size: f64,
    pub area: bool,
    pub color: Option<Color>,
    pub y_axis_index: usize,
    pub grid_index: usize,
    pub sampling: Option<Sampling>,
    pub label_show: bool,
    pub label_font_size: f64,
    /// 值标签位置，None = Top（数据点上方）
    pub label_position: Option<ValueLabelPos>,
    /// 值标签颜色，None = 跟随系列色
    pub label_color: Option<Color>,
    /// 值标签模板（`{a}`/`{b}`/`{c}`），None = 直接显示数值
    pub label_formatter: Option<String>,
}

impl Line {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            x: "x".into(),
            y: "y".into(),
            smooth: false,
            step: None,
            stack: None,
            symbol: SymbolType::EmptyCircle,
            symbol_size: 4.0,
            area: false,
            color: None,
            y_axis_index: 0,
            grid_index: 0,
            sampling: None,
            label_show: false,
            label_font_size: 12.0,
            label_position: None,
            label_color: None,
            label_formatter: None,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn x(mut self, col: impl Into<String>) -> Self {
        self.x = col.into();
        self
    }
    pub fn y(mut self, col: impl Into<String>) -> Self {
        self.y = col.into();
        self
    }
    pub fn smooth(mut self, val: bool) -> Self {
        self.smooth = val;
        self
    }
    pub fn step(mut self, val: StepType) -> Self {
        self.step = Some(val);
        self
    }
    pub fn stack(mut self, name: impl Into<String>) -> Self {
        self.stack = Some(name.into());
        self
    }
    pub fn symbol(mut self, symbol: SymbolType) -> Self {
        self.symbol = symbol;
        self
    }
    pub fn symbol_size(mut self, size: f64) -> Self {
        self.symbol_size = size;
        self
    }
    pub fn area(mut self, val: bool) -> Self {
        self.area = val;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    pub fn y_axis_index(mut self, idx: usize) -> Self {
        self.y_axis_index = idx;
        self
    }
    pub fn grid_index(mut self, idx: usize) -> Self {
        self.grid_index = idx;
        self
    }
    /// Shortcut for `y_axis_index(1)`: bind this series to the right y-axis.
    pub fn right_axis(mut self) -> Self {
        self.y_axis_index = 1;
        self
    }
    /// Set data sampling strategy for large datasets.
    pub fn sampling(mut self, sampling: Sampling) -> Self {
        self.sampling = Some(sampling);
        self
    }
    pub fn label_show(mut self, val: bool) -> Self {
        self.label_show = val;
        self
    }
    pub fn label_font_size(mut self, size: f64) -> Self {
        self.label_font_size = size;
        self
    }
    /// 值标签位置（`Top` 上方 / `Bottom` 下方；`Inside` 在折线图回退为 `Top`）。
    pub fn label_position(mut self, position: ValueLabelPos) -> Self {
        self.label_position = Some(position);
        self
    }
    /// 值标签颜色。默认跟随系列色。
    pub fn label_color(mut self, color: Color) -> Self {
        self.label_color = Some(color);
        self
    }
    /// 值标签模板，支持 `{a}`（系列名）/`{b}`（名称）/`{c}`（数值）。
    pub fn label_formatter(mut self, formatter: impl Into<String>) -> Self {
        self.label_formatter = Some(formatter.into());
        self
    }
}

impl Default for Line {
    fn default() -> Self {
        Self::new()
    }
}

// ── Bar ──

#[derive(Debug, Clone)]
pub struct Bar {
    pub name: String,
    pub data: Option<DataFrame>,
    pub x: String,
    pub y: String,
    pub bar_width: Option<Size>,
    pub stack: Option<String>,
    pub group_index: Option<usize>,
    pub color: Option<Color>,
    pub y_axis_index: usize,
    pub grid_index: usize,
    pub label_show: bool,
    pub label_font_size: f64,
    /// 值标签位置，None = Top（柱顶外侧；负值柱在柱底外侧）
    pub label_position: Option<ValueLabelPos>,
    /// 值标签颜色，None = 柱外跟随系列色 / 柱内白字
    pub label_color: Option<Color>,
    /// 值标签模板（`{a}`/`{b}`/`{c}`），None = 直接显示数值
    pub label_formatter: Option<String>,
}

impl Bar {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            x: "x".into(),
            y: "y".into(),
            bar_width: None,
            stack: None,
            group_index: None,
            color: None,
            y_axis_index: 0,
            grid_index: 0,
            label_show: false,
            label_font_size: 12.0,
            label_position: None,
            label_color: None,
            label_formatter: None,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn x(mut self, col: impl Into<String>) -> Self {
        self.x = col.into();
        self
    }
    pub fn y(mut self, col: impl Into<String>) -> Self {
        self.y = col.into();
        self
    }
    pub fn bar_width(mut self, width: Size) -> Self {
        self.bar_width = Some(width);
        self
    }
    pub fn stack(mut self, name: impl Into<String>) -> Self {
        self.stack = Some(name.into());
        self
    }
    pub fn group_index(mut self, idx: usize) -> Self {
        self.group_index = Some(idx);
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    pub fn y_axis_index(mut self, idx: usize) -> Self {
        self.y_axis_index = idx;
        self
    }
    pub fn grid_index(mut self, idx: usize) -> Self {
        self.grid_index = idx;
        self
    }
    /// Shortcut for `y_axis_index(1)`: bind this series to the right y-axis.
    pub fn right_axis(mut self) -> Self {
        self.y_axis_index = 1;
        self
    }
    pub fn label_show(mut self, val: bool) -> Self {
        self.label_show = val;
        self
    }
    pub fn label_font_size(mut self, size: f64) -> Self {
        self.label_font_size = size;
        self
    }
    /// 值标签位置（`Top` 柱外值端 / `Inside` 柱内值端；`Bottom` 在柱状图回退为 `Top`）。
    ///
    /// `Inside` 在柱体高度不足容纳文字时自动回退到柱外，避免文字溢出。
    pub fn label_position(mut self, position: ValueLabelPos) -> Self {
        self.label_position = Some(position);
        self
    }
    /// 值标签颜色。默认柱外跟随系列色、柱内白字。
    pub fn label_color(mut self, color: Color) -> Self {
        self.label_color = Some(color);
        self
    }
    /// 值标签模板，支持 `{a}`（系列名）/`{b}`（类目名）/`{c}`（数值）。
    pub fn label_formatter(mut self, formatter: impl Into<String>) -> Self {
        self.label_formatter = Some(formatter.into());
        self
    }
}

impl Default for Bar {
    fn default() -> Self {
        Self::new()
    }
}

// ── Pie ──

#[derive(Debug, Clone)]
pub struct Pie {
    pub name: String,
    pub data: Option<DataFrame>,
    pub category: String,
    pub value: String,
    pub radius: (Size, Size),
    pub center: (Size, Size),
    pub label_show: bool,
    pub label_position: LabelPosition,
}

impl Pie {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            category: "category".into(),
            value: "value".into(),
            radius: (Size::Percent(0.0), Size::Percent(75.0)),
            center: (Size::Percent(50.0), Size::Percent(50.0)),
            label_show: false,
            label_position: LabelPosition::Outside,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn category(mut self, col: impl Into<String>) -> Self {
        self.category = col.into();
        self
    }
    pub fn value(mut self, col: impl Into<String>) -> Self {
        self.value = col.into();
        self
    }
    pub fn radius(mut self, inner: Size, outer: Size) -> Self {
        self.radius = (inner, outer);
        self
    }
    pub fn center(mut self, x: Size, y: Size) -> Self {
        self.center = (x, y);
        self
    }
    pub fn label(mut self, show: bool) -> Self {
        self.label_show = show;
        self
    }
    pub fn label_position(mut self, position: LabelPosition) -> Self {
        self.label_position = position;
        self
    }
}

impl Default for Pie {
    fn default() -> Self {
        Self::new()
    }
}

// ── Scatter ──

#[derive(Debug, Clone)]
pub struct Scatter {
    pub name: String,
    pub data: Option<DataFrame>,
    pub x: String,
    pub y: String,
    pub symbol_size: f64,
    pub color: Option<Color>,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl Scatter {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            x: "x".into(),
            y: "y".into(),
            symbol_size: 10.0,
            color: None,
            y_axis_index: 0,
            grid_index: 0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn x(mut self, col: impl Into<String>) -> Self {
        self.x = col.into();
        self
    }
    pub fn y(mut self, col: impl Into<String>) -> Self {
        self.y = col.into();
        self
    }
    pub fn symbol_size(mut self, size: f64) -> Self {
        self.symbol_size = size;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    pub fn y_axis_index(mut self, idx: usize) -> Self {
        self.y_axis_index = idx;
        self
    }
    pub fn grid_index(mut self, idx: usize) -> Self {
        self.grid_index = idx;
        self
    }
    /// Shortcut for `y_axis_index(1)`: bind this series to the right y-axis.
    pub fn right_axis(mut self) -> Self {
        self.y_axis_index = 1;
        self
    }
}

impl Default for Scatter {
    fn default() -> Self {
        Self::new()
    }
}

// ── Bubble ──

#[derive(Debug, Clone)]
pub struct Bubble {
    pub name: String,
    pub data: Option<DataFrame>,
    pub size_col: Option<String>,
    pub name_col: Option<String>,
    pub color: Option<Color>,
    pub symbol_size_scale: f64,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl Bubble {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            size_col: None,
            name_col: None,
            color: None,
            symbol_size_scale: 1.0,
            y_axis_index: 0,
            grid_index: 0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn size_col(mut self, col: impl Into<String>) -> Self {
        self.size_col = Some(col.into());
        self
    }
    pub fn name_col(mut self, col: impl Into<String>) -> Self {
        self.name_col = Some(col.into());
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    pub fn symbol_size_scale(mut self, scale: f64) -> Self {
        self.symbol_size_scale = scale;
        self
    }
    pub fn y_axis_index(mut self, idx: usize) -> Self {
        self.y_axis_index = idx;
        self
    }
    pub fn grid_index(mut self, idx: usize) -> Self {
        self.grid_index = idx;
        self
    }
    /// Shortcut for `y_axis_index(1)`: bind this series to the right y-axis.
    pub fn right_axis(mut self) -> Self {
        self.y_axis_index = 1;
        self
    }
}

impl Default for Bubble {
    fn default() -> Self {
        Self::new()
    }
}

// ── Candlestick ──

#[derive(Debug, Clone)]
pub struct Candlestick {
    pub name: String,
    pub data: Option<DataFrame>,
    pub category: String,
    pub open: String,
    pub close: String,
    pub low: String,
    pub high: String,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl Candlestick {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            category: "category".into(),
            open: "open".into(),
            close: "close".into(),
            low: "low".into(),
            high: "high".into(),
            y_axis_index: 0,
            grid_index: 0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn category(mut self, col: impl Into<String>) -> Self {
        self.category = col.into();
        self
    }
    pub fn open(mut self, col: impl Into<String>) -> Self {
        self.open = col.into();
        self
    }
    pub fn close(mut self, col: impl Into<String>) -> Self {
        self.close = col.into();
        self
    }
    pub fn low(mut self, col: impl Into<String>) -> Self {
        self.low = col.into();
        self
    }
    pub fn high(mut self, col: impl Into<String>) -> Self {
        self.high = col.into();
        self
    }
    pub fn y_axis_index(mut self, idx: usize) -> Self {
        self.y_axis_index = idx;
        self
    }
    pub fn grid_index(mut self, idx: usize) -> Self {
        self.grid_index = idx;
        self
    }
    /// Shortcut for `y_axis_index(1)`: bind this series to the right y-axis.
    pub fn right_axis(mut self) -> Self {
        self.y_axis_index = 1;
        self
    }
}

impl Default for Candlestick {
    fn default() -> Self {
        Self::new()
    }
}

// ── Boxplot ──

#[derive(Debug, Clone)]
pub struct Boxplot {
    pub name: String,
    pub data: Option<DataFrame>,
    pub category: String,
    pub min: String,
    pub q1: String,
    pub median: String,
    pub q3: String,
    pub max: String,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl Boxplot {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            category: "category".into(),
            min: "min".into(),
            q1: "q1".into(),
            median: "median".into(),
            q3: "q3".into(),
            max: "max".into(),
            y_axis_index: 0,
            grid_index: 0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn category(mut self, col: impl Into<String>) -> Self {
        self.category = col.into();
        self
    }
    pub fn min(mut self, col: impl Into<String>) -> Self {
        self.min = col.into();
        self
    }
    pub fn q1(mut self, col: impl Into<String>) -> Self {
        self.q1 = col.into();
        self
    }
    pub fn median(mut self, col: impl Into<String>) -> Self {
        self.median = col.into();
        self
    }
    pub fn q3(mut self, col: impl Into<String>) -> Self {
        self.q3 = col.into();
        self
    }
    pub fn max(mut self, col: impl Into<String>) -> Self {
        self.max = col.into();
        self
    }
    pub fn y_axis_index(mut self, idx: usize) -> Self {
        self.y_axis_index = idx;
        self
    }
    pub fn grid_index(mut self, idx: usize) -> Self {
        self.grid_index = idx;
        self
    }
    /// Shortcut for `y_axis_index(1)`: bind this series to the right y-axis.
    pub fn right_axis(mut self) -> Self {
        self.y_axis_index = 1;
        self
    }
}

impl Default for Boxplot {
    fn default() -> Self {
        Self::new()
    }
}

// ── Heatmap ──

#[derive(Debug, Clone)]
pub struct Heatmap {
    pub name: String,
    pub data: Option<DataFrame>,
    pub x: String,
    pub y: String,
    pub value: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub colors: Option<Vec<Color>>,
    pub border_color: Option<Color>,
    pub border_width: f64,
    pub label_show: bool,
    pub label_font_size: f64,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl Heatmap {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            x: "x".into(),
            y: "y".into(),
            value: "value".into(),
            min: None,
            max: None,
            colors: None,
            border_color: None,
            border_width: 0.0,
            label_show: false,
            label_font_size: 12.0,
            y_axis_index: 0,
            grid_index: 0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn x(mut self, col: impl Into<String>) -> Self {
        self.x = col.into();
        self
    }
    pub fn y(mut self, col: impl Into<String>) -> Self {
        self.y = col.into();
        self
    }
    pub fn value(mut self, col: impl Into<String>) -> Self {
        self.value = col.into();
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
    pub fn colors(mut self, colors: impl IntoIterator<Item = Color>) -> Self {
        self.colors = Some(colors.into_iter().collect());
        self
    }
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }
    pub fn border_width(mut self, width: f64) -> Self {
        self.border_width = width;
        self
    }
    pub fn label_show(mut self, show: bool) -> Self {
        self.label_show = show;
        self
    }
    pub fn y_axis_index(mut self, idx: usize) -> Self {
        self.y_axis_index = idx;
        self
    }
    pub fn grid_index(mut self, idx: usize) -> Self {
        self.grid_index = idx;
        self
    }
    /// Shortcut for `y_axis_index(1)`: bind this series to the right y-axis.
    pub fn right_axis(mut self) -> Self {
        self.y_axis_index = 1;
        self
    }
}

impl Default for Heatmap {
    fn default() -> Self {
        Self::new()
    }
}

// ── Radar ──

#[derive(Debug, Clone)]
pub struct Radar {
    pub name: String,
    pub data: Option<DataFrame>,
    pub values: String,
    pub indicators: Vec<String>,
    pub color: Option<Color>,
}

impl Radar {
    pub fn new(indicators: Vec<String>) -> Self {
        Self {
            name: String::new(),
            data: None,
            values: "value".into(),
            indicators,
            color: None,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn values(mut self, col: impl Into<String>) -> Self {
        self.values = col.into();
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

// ── PolarBar ──

#[derive(Debug, Clone)]
pub struct PolarBar {
    pub name: String,
    pub data: Option<DataFrame>,
    pub angle: String,
    pub radius: String,
    pub color: Option<Vec<Color>>,
    pub pad_angle: f64,
    pub start_angle: f64,
}

impl PolarBar {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            angle: "angle".into(),
            radius: "radius".into(),
            color: None,
            pad_angle: 2.0,
            start_angle: 0.0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn angle(mut self, col: impl Into<String>) -> Self {
        self.angle = col.into();
        self
    }
    pub fn radius(mut self, col: impl Into<String>) -> Self {
        self.radius = col.into();
        self
    }
    pub fn pad_angle(mut self, angle: f64) -> Self {
        self.pad_angle = angle;
        self
    }
    pub fn start_angle(mut self, angle: f64) -> Self {
        self.start_angle = angle;
        self
    }
}

impl Default for PolarBar {
    fn default() -> Self {
        Self::new()
    }
}

// ── PolarScatter ──

#[derive(Debug, Clone)]
pub struct PolarScatter {
    pub name: String,
    pub data: Option<DataFrame>,
    pub angle: String,
    pub radius: String,
    pub symbol_size: Option<f64>,
}

impl PolarScatter {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            angle: "angle".into(),
            radius: "radius".into(),
            symbol_size: None,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn angle(mut self, col: impl Into<String>) -> Self {
        self.angle = col.into();
        self
    }
    pub fn radius(mut self, col: impl Into<String>) -> Self {
        self.radius = col.into();
        self
    }
    pub fn symbol_size(mut self, size: f64) -> Self {
        self.symbol_size = Some(size);
        self
    }
}

impl Default for PolarScatter {
    fn default() -> Self {
        Self::new()
    }
}

// ── Gauge ──

#[derive(Debug, Clone)]
pub struct Gauge {
    pub name: String,
    pub data: Option<DataFrame>,
    pub value: String,
    pub min: f64,
    pub max: f64,
    pub center: (Size, Size),
    pub radius: Size,
    pub start_angle: f64,
    pub end_angle: f64,
    pub split_number: usize,
}

impl Gauge {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
            value: "value".into(),
            min: 0.0,
            max: 100.0,
            center: (Size::Percent(50.0), Size::Percent(75.0)),
            radius: Size::Percent(75.0),
            start_angle: -225.0,
            end_angle: 45.0,
            split_number: 10,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn value(mut self, col: impl Into<String>) -> Self {
        self.value = col.into();
        self
    }
    pub fn min(mut self, val: f64) -> Self {
        self.min = val;
        self
    }
    pub fn max(mut self, val: f64) -> Self {
        self.max = val;
        self
    }
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn center(mut self, x: Size, y: Size) -> Self {
        self.center = (x, y);
        self
    }
    pub fn radius(mut self, r: Size) -> Self {
        self.radius = r;
        self
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

// ── Table ──

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub data: Option<DataFrame>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            data: None,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

// ── SymbolType ──

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
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

impl From<SymbolType> for crate::option::SymbolType {
    fn from(s: SymbolType) -> Self {
        match s {
            SymbolType::Circle => crate::option::SymbolType::Circle,
            SymbolType::EmptyCircle => crate::option::SymbolType::EmptyCircle,
            SymbolType::Rect => crate::option::SymbolType::Rect,
            SymbolType::RoundRect => crate::option::SymbolType::RoundRect,
            SymbolType::Triangle => crate::option::SymbolType::Triangle,
            SymbolType::Diamond => crate::option::SymbolType::Diamond,
            SymbolType::Pin => crate::option::SymbolType::Pin,
            SymbolType::Arrow => crate::option::SymbolType::Arrow,
            SymbolType::None => crate::option::SymbolType::None,
        }
    }
}

// ── Backward-compatible aliases ──
pub type LineLayer = Line;
pub type BarLayer = Bar;
pub type PieLayer = Pie;
pub type ScatterLayer = Scatter;
pub type BubbleLayer = Bubble;
pub type CandlestickLayer = Candlestick;
pub type RadarLayer = Radar;
pub type PolarBarLayer = PolarBar;
pub type PolarScatterLayer = PolarScatter;
pub type GaugeLayer = Gauge;
pub type TableLayer = Table;
