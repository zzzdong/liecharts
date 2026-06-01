use crate::{pipeline::dataframe::DataFrame, visual::Color};

/// A chart layer that maps a DataFrame to visual elements.
#[derive(Debug, Clone)]
pub enum LayerSpec {
    Line(Line),
    Bar(Bar),
    Pie(Pie),
    Scatter(Scatter),
    Bubble(Bubble),
    Candlestick(Candlestick),
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
    pub stack: Option<String>,
    pub symbol: SymbolType,
    pub symbol_size: f64,
    pub area: bool,
    pub color: Option<Color>,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl Line {
    pub fn new() -> Self {
        Self {
            name: String::new(), data: None, x: "x".into(), y: "y".into(),
            smooth: false, stack: None, symbol: SymbolType::Circle, symbol_size: 4.0,
            area: false, color: None, y_axis_index: 0, grid_index: 0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn x(mut self, col: impl Into<String>) -> Self { self.x = col.into(); self }
    pub fn y(mut self, col: impl Into<String>) -> Self { self.y = col.into(); self }
    pub fn smooth(mut self, val: bool) -> Self { self.smooth = val; self }
    pub fn stack(mut self, name: impl Into<String>) -> Self { self.stack = Some(name.into()); self }
    pub fn symbol(mut self, symbol: SymbolType) -> Self { self.symbol = symbol; self }
    pub fn symbol_size(mut self, size: f64) -> Self { self.symbol_size = size; self }
    pub fn area(mut self, val: bool) -> Self { self.area = val; self }
    pub fn color(mut self, color: Color) -> Self { self.color = Some(color); self }
    pub fn y_axis_index(mut self, idx: usize) -> Self { self.y_axis_index = idx; self }
    pub fn grid_index(mut self, idx: usize) -> Self { self.grid_index = idx; self }
}

impl Default for Line { fn default() -> Self { Self::new() } }

// ── Bar ──

#[derive(Debug, Clone)]
pub struct Bar {
    pub name: String,
    pub data: Option<DataFrame>,
    pub x: String,
    pub y: String,
    pub stack: Option<String>,
    pub group_index: Option<usize>,
    pub color: Option<Color>,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl Bar {
    pub fn new() -> Self {
        Self {
            name: String::new(), data: None, x: "x".into(), y: "y".into(),
            stack: None, group_index: None, color: None, y_axis_index: 0, grid_index: 0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn x(mut self, col: impl Into<String>) -> Self { self.x = col.into(); self }
    pub fn y(mut self, col: impl Into<String>) -> Self { self.y = col.into(); self }
    pub fn stack(mut self, name: impl Into<String>) -> Self { self.stack = Some(name.into()); self }
    pub fn group_index(mut self, idx: usize) -> Self { self.group_index = Some(idx); self }
    pub fn color(mut self, color: Color) -> Self { self.color = Some(color); self }
    pub fn y_axis_index(mut self, idx: usize) -> Self { self.y_axis_index = idx; self }
    pub fn grid_index(mut self, idx: usize) -> Self { self.grid_index = idx; self }
}

impl Default for Bar { fn default() -> Self { Self::new() } }

// ── Pie ──

#[derive(Debug, Clone)]
pub struct Pie {
    pub name: String,
    pub data: Option<DataFrame>,
    pub category: String,
    pub value: String,
    pub radius: (f64, f64),
    pub center: (f64, f64),
}

impl Pie {
    pub fn new() -> Self {
        Self {
            name: String::new(), data: None,
            category: "category".into(), value: "value".into(),
            radius: (0.0, 75.0), center: (50.0, 50.0),
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn category(mut self, col: impl Into<String>) -> Self { self.category = col.into(); self }
    pub fn value(mut self, col: impl Into<String>) -> Self { self.value = col.into(); self }
    pub fn radius(mut self, inner: f64, outer: f64) -> Self { self.radius = (inner, outer); self }
    pub fn center(mut self, x: f64, y: f64) -> Self { self.center = (x, y); self }
}

impl Default for Pie { fn default() -> Self { Self::new() } }

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
            name: String::new(), data: None, x: "x".into(), y: "y".into(),
            symbol_size: 10.0, color: None, y_axis_index: 0, grid_index: 0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn x(mut self, col: impl Into<String>) -> Self { self.x = col.into(); self }
    pub fn y(mut self, col: impl Into<String>) -> Self { self.y = col.into(); self }
    pub fn symbol_size(mut self, size: f64) -> Self { self.symbol_size = size; self }
    pub fn color(mut self, color: Color) -> Self { self.color = Some(color); self }
    pub fn y_axis_index(mut self, idx: usize) -> Self { self.y_axis_index = idx; self }
    pub fn grid_index(mut self, idx: usize) -> Self { self.grid_index = idx; self }
}

impl Default for Scatter { fn default() -> Self { Self::new() } }

// ── Bubble ──

#[derive(Debug, Clone)]
pub struct Bubble {
    pub name: String,
    pub data: Option<DataFrame>,
    pub name_col: Option<String>,
    pub color: Option<Color>,
    pub symbol_size_scale: f64,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl Bubble {
    pub fn new() -> Self {
        Self {
            name: String::new(), data: None, name_col: None,
            color: None, symbol_size_scale: 1.0, y_axis_index: 0, grid_index: 0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn name_col(mut self, col: impl Into<String>) -> Self { self.name_col = Some(col.into()); self }
    pub fn color(mut self, color: Color) -> Self { self.color = Some(color); self }
    pub fn symbol_size_scale(mut self, scale: f64) -> Self { self.symbol_size_scale = scale; self }
    pub fn y_axis_index(mut self, idx: usize) -> Self { self.y_axis_index = idx; self }
    pub fn grid_index(mut self, idx: usize) -> Self { self.grid_index = idx; self }
}

impl Default for Bubble { fn default() -> Self { Self::new() } }

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
            name: String::new(), data: None,
            category: "category".into(), open: "open".into(), close: "close".into(),
            low: "low".into(), high: "high".into(), y_axis_index: 0, grid_index: 0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn category(mut self, col: impl Into<String>) -> Self { self.category = col.into(); self }
    pub fn open(mut self, col: impl Into<String>) -> Self { self.open = col.into(); self }
    pub fn close(mut self, col: impl Into<String>) -> Self { self.close = col.into(); self }
    pub fn low(mut self, col: impl Into<String>) -> Self { self.low = col.into(); self }
    pub fn high(mut self, col: impl Into<String>) -> Self { self.high = col.into(); self }
    pub fn y_axis_index(mut self, idx: usize) -> Self { self.y_axis_index = idx; self }
    pub fn grid_index(mut self, idx: usize) -> Self { self.grid_index = idx; self }
}

impl Default for Candlestick { fn default() -> Self { Self::new() } }

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
            name: String::new(), data: None,
            values: "value".into(), indicators, color: None,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn values(mut self, col: impl Into<String>) -> Self { self.values = col.into(); self }
    pub fn color(mut self, color: Color) -> Self { self.color = Some(color); self }
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
            name: String::new(), data: None,
            angle: "angle".into(), radius: "radius".into(),
            color: None, pad_angle: 2.0, start_angle: 0.0,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn angle(mut self, col: impl Into<String>) -> Self { self.angle = col.into(); self }
    pub fn radius(mut self, col: impl Into<String>) -> Self { self.radius = col.into(); self }
    pub fn pad_angle(mut self, angle: f64) -> Self { self.pad_angle = angle; self }
    pub fn start_angle(mut self, angle: f64) -> Self { self.start_angle = angle; self }
}

impl Default for PolarBar { fn default() -> Self { Self::new() } }

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
            name: String::new(), data: None,
            angle: "angle".into(), radius: "radius".into(), symbol_size: None,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn angle(mut self, col: impl Into<String>) -> Self { self.angle = col.into(); self }
    pub fn radius(mut self, col: impl Into<String>) -> Self { self.radius = col.into(); self }
    pub fn symbol_size(mut self, size: f64) -> Self { self.symbol_size = Some(size); self }
}

impl Default for PolarScatter { fn default() -> Self { Self::new() } }

// ── Gauge ──

#[derive(Debug, Clone)]
pub struct Gauge {
    pub name: String,
    pub data: Option<DataFrame>,
    pub value: String,
    pub min: f64,
    pub max: f64,
    pub center: (f64, f64),
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub split_number: usize,
}

impl Gauge {
    pub fn new() -> Self {
        Self {
            name: String::new(), data: None, value: "value".into(),
            min: 0.0, max: 100.0, center: (50.0, 75.0), radius: 75.0,
            start_angle: -225.0, end_angle: 45.0, split_number: 10,
        }
    }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
    pub fn value(mut self, col: impl Into<String>) -> Self { self.value = col.into(); self }
    pub fn min(mut self, val: f64) -> Self { self.min = val; self }
    pub fn max(mut self, val: f64) -> Self { self.max = val; self }
    pub fn range(mut self, min: f64, max: f64) -> Self { self.min = min; self.max = max; self }
    pub fn center(mut self, x: f64, y: f64) -> Self { self.center = (x, y); self }
    pub fn radius(mut self, r: f64) -> Self { self.radius = r; self }
}

impl Default for Gauge { fn default() -> Self { Self::new() } }

// ── Table ──

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub data: Option<DataFrame>,
}

impl Table {
    pub fn new() -> Self { Self { name: String::new(), data: None } }
    pub fn data(mut self, data: DataFrame) -> Self { self.data = Some(data); self }
    pub fn name(mut self, name: impl Into<String>) -> Self { self.name = name.into(); self }
}

impl Default for Table { fn default() -> Self { Self::new() } }

// ── SymbolType ──

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
    Circle, Rect, RoundRect, Triangle, Diamond, Pin, Arrow, None,
}

impl From<SymbolType> for crate::option::SymbolType {
    fn from(s: SymbolType) -> Self {
        match s {
            SymbolType::Circle => crate::option::SymbolType::Circle,
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