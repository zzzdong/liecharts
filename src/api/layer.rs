use crate::{
    pipeline::dataframe::DataFrame,
    visual::Color,
};

/// A chart layer that maps a DataFrame to visual elements.
///
/// Each layer represents one visual series (line, bar, pie, etc.)
/// with a DataFrame and column name mappings.
#[derive(Debug, Clone)]
pub enum LayerSpec {
    Line(LineLayer),
    Bar(BarLayer),
    Pie(PieLayer),
    Scatter(ScatterLayer),
    Bubble(BubbleLayer),
    Candlestick(CandlestickLayer),
    Radar(RadarLayer),
    PolarBar(PolarBarLayer),
    PolarScatter(PolarScatterLayer),
    Gauge(GaugeLayer),
    Table(TableLayer),
}

// ──────────────────────────────────────────────
// LineLayer
// ──────────────────────────────────────────────

/// A line chart layer.
#[derive(Debug, Clone)]
pub struct LineLayer {
    pub name: String,
    pub data: DataFrame,
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

impl LineLayer {
    pub fn new(data: DataFrame) -> Self {
        Self {
            name: String::new(),
            data,
            x: "x".into(),
            y: "y".into(),
            smooth: false,
            stack: None,
            symbol: SymbolType::Circle,
            symbol_size: 4.0,
            area: false,
            color: None,
            y_axis_index: 0,
            grid_index: 0,
        }
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
}

// ──────────────────────────────────────────────
// BarLayer
// ──────────────────────────────────────────────

/// A bar/column chart layer.
#[derive(Debug, Clone)]
pub struct BarLayer {
    pub name: String,
    pub data: DataFrame,
    pub x: String,
    pub y: String,
    pub stack: Option<String>,
    pub group_index: Option<usize>,
    pub color: Option<Color>,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl BarLayer {
    pub fn new(data: DataFrame) -> Self {
        Self {
            name: String::new(),
            data,
            x: "x".into(),
            y: "y".into(),
            stack: None,
            group_index: None,
            color: None,
            y_axis_index: 0,
            grid_index: 0,
        }
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
}

// ──────────────────────────────────────────────
// PieLayer
// ──────────────────────────────────────────────

/// A pie/doughnut chart layer.
#[derive(Debug, Clone)]
pub struct PieLayer {
    pub name: String,
    pub data: DataFrame,
    pub category: String,
    pub value: String,
    pub radius: (f64, f64),
    pub center: (f64, f64),
}

impl PieLayer {
    pub fn new(data: DataFrame) -> Self {
        Self {
            name: String::new(),
            data,
            category: "category".into(),
            value: "value".into(),
            radius: (0.0, 75.0),
            center: (50.0, 50.0),
        }
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

    pub fn radius(mut self, inner: f64, outer: f64) -> Self {
        self.radius = (inner, outer);
        self
    }

    pub fn center(mut self, x: f64, y: f64) -> Self {
        self.center = (x, y);
        self
    }
}

// ──────────────────────────────────────────────
// ScatterLayer
// ──────────────────────────────────────────────

/// A scatter chart layer.
#[derive(Debug, Clone)]
pub struct ScatterLayer {
    pub name: String,
    pub data: DataFrame,
    pub x: String,
    pub y: String,
    pub symbol_size: f64,
    pub color: Option<Color>,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl ScatterLayer {
    pub fn new(data: DataFrame) -> Self {
        Self {
            name: String::new(),
            data,
            x: "x".into(),
            y: "y".into(),
            symbol_size: 10.0,
            color: None,
            y_axis_index: 0,
            grid_index: 0,
        }
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
}

// ──────────────────────────────────────────────
// BubbleLayer
// ──────────────────────────────────────────────

/// A bubble chart layer.
#[derive(Debug, Clone)]
pub struct BubbleLayer {
    pub name: String,
    pub data: DataFrame,
    pub name_col: Option<String>,
    pub color: Option<Color>,
    pub symbol_size_scale: f64,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl BubbleLayer {
    pub fn new(data: DataFrame) -> Self {
        Self {
            name: String::new(),
            data,
            name_col: None,
            color: None,
            symbol_size_scale: 1.0,
            y_axis_index: 0,
            grid_index: 0,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
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
}

// ──────────────────────────────────────────────
// CandlestickLayer
// ──────────────────────────────────────────────

/// A candlestick (K-line) chart layer.
///
/// The DataFrame must have columns for open, close, low, high values,
/// and optionally a name/category column.
#[derive(Debug, Clone)]
pub struct CandlestickLayer {
    pub name: String,
    pub data: DataFrame,
    pub category: String,
    pub open: String,
    pub close: String,
    pub low: String,
    pub high: String,
    pub y_axis_index: usize,
    pub grid_index: usize,
}

impl CandlestickLayer {
    pub fn new(data: DataFrame) -> Self {
        Self {
            name: String::new(),
            data,
            category: "category".into(),
            open: "open".into(),
            close: "close".into(),
            low: "low".into(),
            high: "high".into(),
            y_axis_index: 0,
            grid_index: 0,
        }
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
}

// ──────────────────────────────────────────────
// RadarLayer
// ──────────────────────────────────────────────

/// A radar chart layer.
///
/// The DataFrame must contain a column with comma-separated values (one per indicator),
/// or a `values` column of type `Vec<f64>`.
#[derive(Debug, Clone)]
pub struct RadarLayer {
    pub name: String,
    pub data: DataFrame,
    pub values: String,
    pub indicators: Vec<String>,
    pub color: Option<Color>,
}

impl RadarLayer {
    pub fn new(data: DataFrame, indicators: Vec<String>) -> Self {
        Self {
            name: String::new(),
            data,
            values: "value".into(),
            indicators,
            color: None,
        }
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

// ──────────────────────────────────────────────
// PolarBarLayer
// ──────────────────────────────────────────────

/// A polar bar chart layer.
#[derive(Debug, Clone)]
pub struct PolarBarLayer {
    pub name: String,
    pub data: DataFrame,
    pub angle: String,
    pub radius: String,
    pub color: Option<Vec<Color>>,
    pub pad_angle: f64,
    pub start_angle: f64,
}

impl PolarBarLayer {
    pub fn new(data: DataFrame) -> Self {
        Self {
            name: String::new(),
            data,
            angle: "angle".into(),
            radius: "radius".into(),
            color: None,
            pad_angle: 2.0,
            start_angle: 0.0,
        }
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

// ──────────────────────────────────────────────
// PolarScatterLayer
// ──────────────────────────────────────────────

/// A polar scatter chart layer.
#[derive(Debug, Clone)]
pub struct PolarScatterLayer {
    pub name: String,
    pub data: DataFrame,
    pub angle: String,
    pub radius: String,
    pub symbol_size: Option<f64>,
}

impl PolarScatterLayer {
    pub fn new(data: DataFrame) -> Self {
        Self {
            name: String::new(),
            data,
            angle: "angle".into(),
            radius: "radius".into(),
            symbol_size: None,
        }
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

// ──────────────────────────────────────────────
// GaugeLayer
// ──────────────────────────────────────────────

/// A gauge chart layer.
#[derive(Debug, Clone)]
pub struct GaugeLayer {
    pub name: String,
    pub data: DataFrame,
    pub value: String,
    pub min: f64,
    pub max: f64,
    pub center: (f64, f64),
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub split_number: usize,
}

impl GaugeLayer {
    pub fn new(data: DataFrame) -> Self {
        Self {
            name: String::new(),
            data,
            value: "value".into(),
            min: 0.0,
            max: 100.0,
            center: (50.0, 50.0),
            radius: 75.0,
            start_angle: -225.0,
            end_angle: 45.0,
            split_number: 10,
        }
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

    pub fn center(mut self, x: f64, y: f64) -> Self {
        self.center = (x, y);
        self
    }

    pub fn radius(mut self, r: f64) -> Self {
        self.radius = r;
        self
    }
}

// ──────────────────────────────────────────────
// TableLayer
// ──────────────────────────────────────────────

/// A table layer.
#[derive(Debug, Clone)]
pub struct TableLayer {
    pub name: String,
    pub data: DataFrame,
}

impl TableLayer {
    pub fn new(data: DataFrame) -> Self {
        Self {
            name: String::new(),
            data,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

// ──────────────────────────────────────────────
// Shared enums
// ──────────────────────────────────────────────

/// Symbol/marker type for data points.
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