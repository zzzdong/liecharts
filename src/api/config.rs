/// Title configuration.
#[derive(Debug, Clone)]
pub struct Title {
    pub text: String,
    pub subtext: Option<String>,
    pub left: Position,
    pub top: Position,
}

impl Title {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            subtext: None,
            left: Position::Center,
            top: Position::Auto,
        }
    }

    pub fn subtext(mut self, text: impl Into<String>) -> Self {
        self.subtext = Some(text.into());
        self
    }

    pub fn left(mut self, pos: Position) -> Self {
        self.left = pos;
        self
    }

    pub fn top(mut self, pos: Position) -> Self {
        self.top = pos;
        self
    }
}

/// Legend configuration.
#[derive(Debug, Clone)]
pub struct Legend {
    pub show: bool,
    pub data: Vec<String>,
    pub left: Position,
    pub top: Position,
    pub orient: Orient,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            show: true,
            data: Vec::new(),
            left: Position::Center,
            top: Position::Auto,
            orient: Orient::Horizontal,
        }
    }
}

impl Legend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data(mut self, data: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data = data.into_iter().map(Into::into).collect();
        self
    }

    pub fn left(mut self, pos: Position) -> Self {
        self.left = pos;
        self
    }

    pub fn top(mut self, pos: Position) -> Self {
        self.top = pos;
        self
    }

    pub fn orient(mut self, orient: Orient) -> Self {
        self.orient = orient;
        self
    }
}

/// Grid region for multi-layout charts.
#[derive(Debug, Clone)]
pub struct Grid {
    pub left: Position,
    pub right: Position,
    pub top: Position,
    pub bottom: Position,
    pub contain_label: bool,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            left: Position::Percent(10.0),
            right: Position::Percent(10.0),
            top: Position::Percent(15.0),
            bottom: Position::Percent(15.0),
            contain_label: true,
        }
    }
}

impl Grid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn left(mut self, pos: Position) -> Self {
        self.left = pos;
        self
    }

    pub fn right(mut self, pos: Position) -> Self {
        self.right = pos;
        self
    }

    pub fn top(mut self, pos: Position) -> Self {
        self.top = pos;
        self
    }

    pub fn bottom(mut self, pos: Position) -> Self {
        self.bottom = pos;
        self
    }

    pub fn contain_label(mut self, val: bool) -> Self {
        self.contain_label = val;
        self
    }
}

/// Axis configuration.
#[derive(Debug, Clone)]
pub struct Axis {
    pub position: AxisPosition,
    pub axis_type: AxisType,
    pub data: Vec<String>,
    pub name: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub boundary_gap: bool,
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            position: AxisPosition::Bottom,
            axis_type: AxisType::Category,
            data: Vec::new(),
            name: None,
            min: None,
            max: None,
            boundary_gap: true,
        }
    }
}

impl Axis {
    /// Create a category axis.
    pub fn category() -> Self {
        Self {
            axis_type: AxisType::Category,
            ..Default::default()
        }
    }

    /// Create a value axis.
    pub fn value() -> Self {
        Self {
            axis_type: AxisType::Value,
            ..Default::default()
        }
    }

    pub fn data(mut self, data: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data = data.into_iter().map(Into::into).collect();
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

    pub fn position(mut self, pos: AxisPosition) -> Self {
        self.position = pos;
        self
    }

    pub fn boundary_gap(mut self, gap: bool) -> Self {
        self.boundary_gap = gap;
        self
    }
}

/// Position enum for layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Auto,
    Center,
    Left,
    Right,
    Top,
    Bottom,
    Pixel(f64),
    Percent(f64),
}

impl Position {
    pub fn px(v: f64) -> Self {
        Position::Pixel(v)
    }

    pub fn pct(v: f64) -> Self {
        Position::Percent(v)
    }
}

/// Orientation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orient {
    Horizontal,
    Vertical,
}

/// Axis type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisType {
    Category,
    Value,
}

/// Axis position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisPosition {
    Top,
    Bottom,
    Left,
    Right,
}