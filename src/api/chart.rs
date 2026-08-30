use super::layer::{
    Bar, Boxplot, Bubble, Candlestick, Gauge, Heatmap, LayerSpec, Line, Pie, PolarBar,
    PolarScatter, Radar, Scatter, SymbolType as LayerSymbol, Table,
};
use crate::{
    error::Result,
    pipeline::dataframe::DataFrame,
    theme::Theme,
    visual::{Color, VisualElement},
};

// ── Macros ──

macro_rules! add_layer_method {
    ($method:ident, $layer:ty, $variant:ident) => {
        pub fn $method(mut self, layer: $layer) -> Self {
            self.layers.push(LayerSpec::$variant(layer));
            self
        }
    };
}

macro_rules! impl_from_layer {
    ($layer:ty, $variant:ident) => {
        impl From<$layer> for LayerSpec {
            fn from(l: $layer) -> Self {
                LayerSpec::$variant(l)
            }
        }
    };
}

// ── GridBuilder ──

/// A context for configuring a single grid and its contents.
///
/// Created by [`Chart::sub_grid`]. All layers and axes added through
/// this builder are automatically linked to the correct grid index,
/// eliminating the need for manual `grid_index()` calls.
///
/// # Example
///
/// ```ignore
/// .sub_grid(Grid::new(), |g| g
///     .x_axis(Axis::category().data(["A", "B"]))
///     .y_axis(Axis::value())
///     .add_layer(Bar::new().data(df).x("cat").y("val"))
/// )
/// ```
#[derive(Debug, Clone)]
pub struct GridBuilder {
    grid_index: usize,
    x_axes: Vec<Axis>,
    y_axes: Vec<Axis>,
    layers: Vec<LayerSpec>,
}

impl GridBuilder {
    fn new(grid_index: usize) -> Self {
        Self {
            grid_index,
            x_axes: Vec::new(),
            y_axes: Vec::new(),
            layers: Vec::new(),
        }
    }

    /// Add an x-axis linked to this grid.
    pub fn x_axis(mut self, mut axis: Axis) -> Self {
        axis.grid_index = self.grid_index;
        self.x_axes.push(axis);
        self
    }

    /// Add a y-axis linked to this grid.
    /// The first y-axis is positioned on the left, the second on the right.
    pub fn y_axis(mut self, mut axis: Axis) -> Self {
        axis.grid_index = self.grid_index;
        // Auto-set position based on index: first -> Left, second -> Right
        let y_axis_idx = self.y_axes.len();
        if y_axis_idx == 0 {
            axis.position = AxisPosition::Left;
        } else {
            axis.position = AxisPosition::Right;
        }
        self.y_axes.push(axis);
        self
    }

    /// Add a layer (Line, Bar, Pie, etc.) linked to this grid.
    ///
    /// Cartesian layers (Line, Bar, Scatter, Bubble, Candlestick)
    /// get their `grid_index` set automatically. Non-cartesian layers
    /// (Pie, Radar, Gauge, etc.) ignore the grid index.
    pub fn add_layer(mut self, layer: impl Into<LayerSpec>) -> Self {
        let mut spec: LayerSpec = layer.into();
        spec.set_grid_index(self.grid_index);
        self.layers.push(spec);
        self
    }
}

// ── Position / Orient / Axis enums ──

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

    /// 将 Position 转换为像素值（对 Percent 根据容器尺寸换算）
    pub(crate) fn to_pixels(self, container_size: f64) -> f64 {
        match self {
            Position::Pixel(v) => v,
            Position::Percent(v) => container_size * v / 100.0,
            Position::Auto | Position::Center => container_size / 2.0,
            Position::Left | Position::Top => 0.0,
            Position::Right => container_size,
            Position::Bottom => container_size,
        }
    }
}

/// A size value that can be specified in pixels or as a percentage of the container.
///
/// Used for dimensions like radius, symbol sizes, etc.
/// Analogous to `Position` but without directional semantics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Pixel(f64),
    Percent(f64),
}

impl Size {
    pub fn px(v: f64) -> Self {
        Size::Pixel(v)
    }
    pub fn pct(v: f64) -> Self {
        Size::Percent(v)
    }

    /// Convert to a percentage value (0-100) relative to the container size.
    pub(crate) fn to_percent(self, container_size: f64) -> f64 {
        match self {
            Size::Percent(v) => v,
            Size::Pixel(v) => {
                if container_size > 0.0 {
                    v / container_size * 100.0
                } else {
                    0.0
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orient {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisType {
    Category,
    Value,
    Time,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisPosition {
    Top,
    Bottom,
    Left,
    Right,
}

// ── Title ──

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

// ── Legend ──

#[derive(Debug, Clone)]
pub struct Legend {
    pub show: bool,
    pub data: Vec<String>,
    pub left: Position,
    pub top: Position,
    pub orient: Orient,
    /// 图例文本模板（支持 `{name}`/`{a}`/`{b}`）
    pub formatter: Option<String>,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            show: true,
            data: Vec::new(),
            left: Position::Center,
            top: Position::Auto,
            orient: Orient::Horizontal,
            formatter: None,
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
    pub fn formatter(mut self, formatter: impl Into<String>) -> Self {
        self.formatter = Some(formatter.into());
        self
    }
}

// ── Grid ──

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

// ── Axis ──

#[derive(Debug, Clone)]
pub struct Axis {
    pub position: AxisPosition,
    pub axis_type: AxisType,
    pub data: Vec<String>,
    pub name: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub boundary_gap: bool,
    pub grid_index: usize,
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
            grid_index: 0,
        }
    }
}

impl Axis {
    pub fn category() -> Self {
        Self {
            axis_type: AxisType::Category,
            ..Default::default()
        }
    }
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
    pub fn grid_index(mut self, idx: usize) -> Self {
        self.grid_index = idx;
        self
    }
}

/// A DataFrame-centric chart builder.
///
/// This is the main entry point for the new API.
/// Construct it with dimensions, configure layers and options,
/// then call `render_svg()` or `render_png()`.
///
/// # Examples
///
/// ```no_run
/// use liecharts::api::*;
///
/// let df = dataframe!(
///     "category" => ["A", "B", "C"],
///     "value" => [10.0, 20.0, 30.0],
/// );
///
/// let svg = Chart::new(800, 600)
///     .data(dataframe!(
///         "category" => ["A", "B", "C"],
///         "value" => [10.0, 20.0, 30.0],
///     ))
///     .add_bar(Bar::new().name("My Bar").x("category").y("value"))
///     .render_svg()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Chart {
    width: u32,
    height: u32,
    title: Option<Title>,
    legend: Option<Legend>,
    grids: Vec<Grid>,
    x_axes: Vec<Axis>,
    y_axes: Vec<Axis>,
    layers: Vec<LayerSpec>,
    data: Option<DataFrame>,
    background_color: Option<Color>,
    theme_name: Option<String>,
}

impl Chart {
    /// Create a new chart with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            title: None,
            legend: None,
            grids: Vec::new(),
            x_axes: Vec::new(),
            y_axes: Vec::new(),
            layers: Vec::new(),
            data: None,
            background_color: None,
            theme_name: None,
        }
    }

    // ── Configuration ──

    /// Set shared data for all layers. Layers without their own data will use this.
    pub fn data(mut self, data: DataFrame) -> Self {
        self.data = Some(data);
        self
    }

    pub fn title(mut self, title: impl Into<Title>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn legend(mut self, legend: impl Into<Option<Legend>>) -> Self {
        self.legend = legend.into();
        self
    }

    pub fn grid(mut self, grid: Grid) -> Self {
        self.grids.push(grid);
        self
    }

    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axes.push(axis);
        self
    }

    pub fn y_axis(mut self, mut axis: Axis) -> Self {
        // Auto-set position based on per-subplot index: first -> Left, second -> Right
        let grid_idx = axis.grid_index;
        let per_subplot_idx = self
            .y_axes
            .iter()
            .filter(|a| a.grid_index == grid_idx)
            .count();
        if per_subplot_idx == 0 {
            axis.position = AxisPosition::Left;
        } else {
            axis.position = AxisPosition::Right;
        }
        self.y_axes.push(axis);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn theme(mut self, name: impl Into<String>) -> Self {
        self.theme_name = Some(name.into());
        self
    }

    // ── Layers ──

    add_layer_method!(add_line, Line, Line);
    add_layer_method!(add_bar, Bar, Bar);
    add_layer_method!(add_pie, Pie, Pie);
    add_layer_method!(add_scatter, Scatter, Scatter);
    add_layer_method!(add_bubble, Bubble, Bubble);
    add_layer_method!(add_candlestick, Candlestick, Candlestick);
    add_layer_method!(add_boxplot, Boxplot, Boxplot);
    add_layer_method!(add_heatmap, Heatmap, Heatmap);
    add_layer_method!(add_radar, Radar, Radar);
    add_layer_method!(add_polar_bar, PolarBar, PolarBar);
    add_layer_method!(add_polar_scatter, PolarScatter, PolarScatter);
    add_layer_method!(add_gauge, Gauge, Gauge);
    add_layer_method!(add_table, Table, Table);

    pub fn add_layer(mut self, layer: impl Into<LayerSpec>) -> Self {
        self.layers.push(layer.into());
        self
    }

    /// Add a grid with a single layer. Shortcut for multi-grid layouts.
    /// Creates a default grid and assigns the layer to it.
    pub fn with_grid(mut self, layer: impl Into<LayerSpec>) -> Self {
        let idx = self.grids.len();
        self.grids.push(Grid::default());
        let mut spec = layer.into();
        spec.set_grid_index(idx);
        self.layers.push(spec);
        self
    }

    /// Define a grid and its contents in a closure.
    ///
    /// All axes and layers added inside the closure are automatically
    /// linked to this grid, eliminating manual `grid_index()` calls.
    ///
    /// # Example
    ///
    /// ```ignore
    /// Chart::new(1000, 900)
    ///     .sub_grid(
    ///         Grid::new().left(Position::pct(3.0)).top(Position::pct(12.0)),
    ///         |g| g
    ///             .x_axis(Axis::category().data(["A", "B", "C"]))
    ///             .y_axis(Axis::value())
    ///             .add_layer(Bar::new().data(sales).x("cat").y("val"))
    ///             .add_layer(Line::new().data(trend).x("cat").y("val")),
    ///     )
    ///     .add_pie(Pie::new().data(segments).category("name").value("pct"))
    /// ```
    pub fn sub_grid(mut self, grid: Grid, f: impl FnOnce(GridBuilder) -> GridBuilder) -> Self {
        let idx = self.grids.len();
        self.grids.push(grid);
        let gb = f(GridBuilder::new(idx));
        self.x_axes.extend(gb.x_axes);
        self.y_axes.extend(gb.y_axes);
        self.layers.extend(gb.layers);
        self
    }

    // ── Rendering ──

    /// Build the chart and collect visual elements.
    pub fn build(&self) -> Result<Vec<VisualElement>> {
        let spec = self.to_chart_spec();
        let theme = match self.theme_name.as_deref() {
            Some("dark") => Theme::dark(),
            _ => Theme::echarts(),
        };
        crate::pipeline::chart_pipeline::build_chart_from_spec(&spec, &theme)
    }

    /// Render to SVG string.
    pub fn render_svg(&self) -> Result<String> {
        let elements = self.build()?;
        let renderer = crate::render::SvgRenderer::new();
        renderer.render(&elements, self.width, self.height)
    }

    /// Render to PNG bytes.
    pub fn render_png(&self) -> Result<Vec<u8>> {
        let elements = self.build()?;
        let renderer = crate::render::PixmapRenderer::new(self.width, self.height);
        let pixmap = renderer.render(&elements)?;
        let data: Vec<u8> = pixmap
            .data()
            .iter()
            .flat_map(|p| vec![p.r, p.g, p.b, p.a])
            .collect();
        let width = pixmap.width() as u32;
        let height = pixmap.height() as u32;
        let image = image::RgbaImage::from_raw(width, height, data).ok_or_else(|| {
            crate::error::ChartError::RenderError("Failed to create PNG image".to_string())
        })?;
        let mut buf = Vec::new();
        image.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
        Ok(buf)
    }

    /// Render to an image file.
    pub fn render_to_image(&self, path: &str) -> Result<()> {
        let elements = self.build()?;
        let renderer = crate::render::PixmapRenderer::new(self.width, self.height);
        let pixmap = renderer.render(&elements)?;
        let data: Vec<u8> = pixmap
            .data()
            .iter()
            .flat_map(|p| vec![p.r, p.g, p.b, p.a])
            .collect();
        let image = image::RgbaImage::from_raw(pixmap.width() as u32, pixmap.height() as u32, data)
            .ok_or_else(|| {
                crate::error::ChartError::RenderError("Failed to create image".to_string())
            })?;
        image.save(path)?;
        Ok(())
    }

    /// Render to an SVG file.
    pub fn render_to_svg(&self, path: &str) -> Result<()> {
        let svg = self.render_svg()?;
        std::fs::write(path, svg)?;
        Ok(())
    }
}

impl Chart {
    /// 直接转换为 ChartSpec（新管线入口）
    pub(crate) fn to_chart_spec(&self) -> crate::pipeline::types::ChartSpec {
        use crate::pipeline::types::{
            AxisSpec, BarConfig, BoxplotConfig, BubbleConfig, CandlestickConfig, ChartSpec,
            GaugeConfig, GridSpec, HeatmapConfig, ItemStyleSpec, LegendSpec, LineConfig, PieConfig,
            PolarBarConfig, PolarScatterConfig, RadarConfig, ScatterConfig, SeriesConfig,
            SeriesSpec, SymbolType, TableConfig, TitleSpec,
        };

        // Grids
        let grids = if self.grids.is_empty() {
            vec![GridSpec {
                left: None,
                right: None,
                top: None,
                bottom: None,
                contain_label: false,
            }]
        } else {
            self.grids
                .iter()
                .map(|g| GridSpec {
                    left: Some(g.left.to_pixels(self.width as f64)),
                    right: Some(g.right.to_pixels(self.width as f64)),
                    top: Some(g.top.to_pixels(self.height as f64)),
                    bottom: Some(g.bottom.to_pixels(self.height as f64)),
                    contain_label: g.contain_label,
                })
                .collect()
        };

        // 从层数据中提取 X 轴类别（用于默认 X 轴）
        let mut default_categories: Vec<String> = vec![];
        for layer in &self.layers {
            let (data, x_col) = match layer {
                LayerSpec::Line(l) => (l.data.clone().or_else(|| self.data.clone()), l.x.clone()),
                LayerSpec::Bar(l) => (l.data.clone().or_else(|| self.data.clone()), l.x.clone()),
                LayerSpec::Scatter(l) => {
                    (l.data.clone().or_else(|| self.data.clone()), l.x.clone())
                }
                LayerSpec::Candlestick(l) => (
                    l.data.clone().or_else(|| self.data.clone()),
                    l.category.clone(),
                ),
                LayerSpec::Boxplot(l) => (
                    l.data.clone().or_else(|| self.data.clone()),
                    l.category.clone(),
                ),
                LayerSpec::Heatmap(l) => {
                    (l.data.clone().or_else(|| self.data.clone()), l.x.clone())
                }
                _ => (None, String::new()),
            };
            if let Some(df) = data
                && let Some(col) = df.get_column(&x_col)
            {
                // 提取类别（字符串列）
                let cats: Vec<String> = (0..col.len()).filter_map(|i| col.as_string(i)).collect();
                if !cats.is_empty() {
                    default_categories = cats;
                    break;
                }
            }
        }

        // X Axes
        let x_axes: Vec<AxisSpec> = if self.x_axes.is_empty() {
            vec![AxisSpec {
                axis_type: crate::pipeline::types::AxisType::Category,
                position: crate::pipeline::types::AxisPosition::Bottom,
                grid_index: 0,
                min: None,
                max: None,
                name: None,
                name_location: None,
                categories: default_categories.clone(),
                boundary_gap: true,
                inverse: false,
                split_number: None,
                label_show: true,
                label_formatter: None,
                label_rotate: None,
                axis_line_show: true,
                split_line_show: true,
                z: None,
            }]
        } else {
            self.x_axes
                .iter()
                .map(|a| AxisSpec {
                    axis_type: match a.axis_type {
                        crate::api::AxisType::Value => crate::pipeline::types::AxisType::Value,
                        crate::api::AxisType::Category => {
                            crate::pipeline::types::AxisType::Category
                        }
                        crate::api::AxisType::Time => crate::pipeline::types::AxisType::Time,
                        crate::api::AxisType::Log => crate::pipeline::types::AxisType::Log,
                    },
                    position: match a.position {
                        crate::api::AxisPosition::Left => {
                            crate::pipeline::types::AxisPosition::Left
                        }
                        crate::api::AxisPosition::Right => {
                            crate::pipeline::types::AxisPosition::Right
                        }
                        crate::api::AxisPosition::Bottom => {
                            crate::pipeline::types::AxisPosition::Bottom
                        }
                        crate::api::AxisPosition::Top => crate::pipeline::types::AxisPosition::Top,
                    },
                    grid_index: a.grid_index,
                    min: a.min,
                    max: a.max,
                    name: a.name.clone(),
                    name_location: None,
                    categories: if a.data.is_empty() {
                        default_categories.clone()
                    } else {
                        a.data.clone()
                    },
                    boundary_gap: a.boundary_gap,
                    inverse: false,
                    split_number: None,
                    label_show: true,
                    label_formatter: None,
                    label_rotate: None,
                    axis_line_show: true,
                    split_line_show: true,
                    z: None,
                })
                .collect()
        };

        // Y Axes
        let y_axes: Vec<AxisSpec> = if self.y_axes.is_empty() {
            vec![AxisSpec {
                axis_type: crate::pipeline::types::AxisType::Value,
                position: crate::pipeline::types::AxisPosition::Left,
                grid_index: 0,
                min: None,
                max: None,
                name: None,
                name_location: None,
                categories: vec![],
                boundary_gap: true,
                inverse: false,
                split_number: None,
                label_show: true,
                label_formatter: None,
                label_rotate: None,
                axis_line_show: true,
                split_line_show: true,
                z: None,
            }]
        } else {
            self.y_axes
                .iter()
                .map(|a| AxisSpec {
                    axis_type: match a.axis_type {
                        crate::api::AxisType::Value => crate::pipeline::types::AxisType::Value,
                        crate::api::AxisType::Category => {
                            crate::pipeline::types::AxisType::Category
                        }
                        crate::api::AxisType::Time => crate::pipeline::types::AxisType::Time,
                        crate::api::AxisType::Log => crate::pipeline::types::AxisType::Log,
                    },
                    position: match a.position {
                        crate::api::AxisPosition::Left => {
                            crate::pipeline::types::AxisPosition::Left
                        }
                        crate::api::AxisPosition::Right => {
                            crate::pipeline::types::AxisPosition::Right
                        }
                        crate::api::AxisPosition::Bottom => {
                            crate::pipeline::types::AxisPosition::Bottom
                        }
                        crate::api::AxisPosition::Top => crate::pipeline::types::AxisPosition::Top,
                    },
                    grid_index: a.grid_index,
                    min: a.min,
                    max: a.max,
                    name: a.name.clone(),
                    name_location: None,
                    categories: a.data.clone(),
                    boundary_gap: a.boundary_gap,
                    inverse: false,
                    split_number: None,
                    label_show: true,
                    label_formatter: None,
                    label_rotate: None,
                    axis_line_show: true,
                    split_line_show: true,
                    z: None,
                })
                .collect()
        };

        // Series
        let shared_data = self.data.as_ref();
        let series: Vec<SeriesSpec> = self
            .layers
            .iter()
            .map(|layer| {
                let config: SeriesConfig = match layer {
                    LayerSpec::Line(l) => {
                        let sym = match l.symbol {
                            LayerSymbol::Circle => SymbolType::Circle,
                            LayerSymbol::EmptyCircle => SymbolType::EmptyCircle,
                            LayerSymbol::Rect => SymbolType::Rect,
                            LayerSymbol::RoundRect => SymbolType::RoundRect,
                            LayerSymbol::Triangle => SymbolType::Triangle,
                            LayerSymbol::Diamond => SymbolType::Diamond,
                            LayerSymbol::Pin => SymbolType::Pin,
                            LayerSymbol::Arrow => SymbolType::Arrow,
                            LayerSymbol::None => SymbolType::None,
                        };
                        // 面积填充：使用系列颜色（由 ColorAssigner 分配），用户可指定颜色
                        let area = l.area;
                        let area_color = l.color; // 用户指定的颜色，None 时使用系列颜色
                        SeriesConfig::Line(LineConfig {
                            x_col: l.x.clone(),
                            y_col: l.y.clone(),
                            smooth: l.smooth,
                            step: l.step.map(|s| match s {
                                crate::api::layer::StepType::Start => {
                                    crate::pipeline::types::StepType::Start
                                }
                                crate::api::layer::StepType::Middle => {
                                    crate::pipeline::types::StepType::Middle
                                }
                                crate::api::layer::StepType::End => {
                                    crate::pipeline::types::StepType::End
                                }
                            }),
                            line_width: 2.0,
                            area,
                            area_color,
                            area_opacity: 0.5,
                            symbol_type: sym,
                            symbol_size: l.symbol_size,
                            label_show: l.label_show,
                            label_font_size: l.label_font_size,
                            label_formatter: None,
                            mark_line: Vec::new(),
                        })
                    }
                    LayerSpec::Bar(l) => {
                        let y_axis_idx = l.y_axis_index;
                        let is_horizontal = y_axes
                            .get(y_axis_idx)
                            .map(|a| {
                                matches!(a.axis_type, crate::pipeline::types::AxisType::Category)
                            })
                            .unwrap_or(false);
                        let (x_col, y_col) = if is_horizontal {
                            (l.y.clone(), l.x.clone())
                        } else {
                            (l.x.clone(), l.y.clone())
                        };
                        SeriesConfig::Bar(BarConfig {
                            x_col,
                            y_col,
                            bar_width: l.bar_width.map_or(0.6, |bw| match bw {
                                crate::api::Size::Percent(p) => p / 100.0,
                                crate::api::Size::Pixel(p) => p / 100.0,
                            }),
                            label_show: l.label_show,
                            label_font_size: l.label_font_size,
                            label_formatter: None,
                            mark_line: Vec::new(),
                        })
                    }
                    LayerSpec::Scatter(l) => SeriesConfig::Scatter(ScatterConfig {
                        x_col: l.x.clone(),
                        y_col: l.y.clone(),
                        symbol_size: l.symbol_size,
                    }),
                    LayerSpec::Bubble(l) => SeriesConfig::Bubble(BubbleConfig {
                        x_col: "x".into(),
                        y_col: "y".into(),
                        size_col: l.size_col.clone(),
                        name_col: l.name_col.clone(),
                        symbol_size_scale: l.symbol_size_scale,
                    }),
                    LayerSpec::Candlestick(l) => SeriesConfig::Candlestick(CandlestickConfig {
                        category_col: l.category.clone(),
                        open_col: l.open.clone(),
                        close_col: l.close.clone(),
                        low_col: l.low.clone(),
                        high_col: l.high.clone(),
                    }),
                    LayerSpec::Boxplot(l) => SeriesConfig::Boxplot(BoxplotConfig {
                        category_col: l.category.clone(),
                        min_col: l.min.clone(),
                        q1_col: l.q1.clone(),
                        median_col: l.median.clone(),
                        q3_col: l.q3.clone(),
                        max_col: l.max.clone(),
                    }),
                    LayerSpec::Heatmap(l) => SeriesConfig::Heatmap(HeatmapConfig {
                        x_col: l.x.clone(),
                        y_col: l.y.clone(),
                        value_col: l.value.clone(),
                        min: l.min,
                        max: l.max,
                        colors: l.colors.clone().unwrap_or_else(|| {
                            vec![
                                Color::rgb(80, 163, 186),
                                Color::rgb(234, 199, 54),
                                Color::rgb(217, 78, 93),
                            ]
                        }),
                        border_color: l.border_color,
                        border_width: l.border_width,
                        label_show: l.label_show,
                        label_font_size: l.label_font_size,
                    }),
                    LayerSpec::Pie(l) => {
                        let min_dim = self.width.min(self.height) as f64 * 0.5;
                        SeriesConfig::Pie(PieConfig {
                            category_col: l.category.clone(),
                            value_col: l.value.clone(),
                            center: (
                                l.center.0.to_percent(self.width as f64),
                                l.center.1.to_percent(self.height as f64),
                            ),
                            radius: (
                                l.radius.0.to_percent(min_dim),
                                l.radius.1.to_percent(min_dim),
                            ),
                            label_show: l.label_show,
                            label_position: l.label_position,
                            label_font_size: 12.0,
                            label_formatter: None,
                        })
                    }
                    LayerSpec::Radar(l) => SeriesConfig::Radar(RadarConfig {
                        value_col: l.values.clone(),
                        indicators: l.indicators.clone(),
                    }),
                    LayerSpec::PolarBar(l) => SeriesConfig::PolarBar(PolarBarConfig {
                        angle_col: l.angle.clone(),
                        radius_col: l.radius.clone(),
                        category_col: None,
                        pad_angle: l.pad_angle,
                        start_angle: l.start_angle,
                    }),
                    LayerSpec::PolarScatter(l) => SeriesConfig::PolarScatter(PolarScatterConfig {
                        angle_col: l.angle.clone(),
                        radius_col: l.radius.clone(),
                        symbol_size: l.symbol_size.unwrap_or(8.0),
                    }),
                    LayerSpec::Gauge(l) => {
                        let min_dim = self.width.min(self.height) as f64 * 0.5;
                        SeriesConfig::Gauge(GaugeConfig {
                            value_col: l.value.clone(),
                            min: l.min,
                            max: l.max,
                            center: (
                                l.center.0.to_percent(self.width as f64),
                                l.center.1.to_percent(self.height as f64),
                            ),
                            radius: l.radius.to_percent(min_dim),
                            start_angle: l.start_angle,
                            end_angle: l.end_angle,
                            split_number: l.split_number,
                        })
                    }
                    LayerSpec::Table(_) => SeriesConfig::Table(TableConfig),
                };

                let data = match layer {
                    LayerSpec::Line(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::Bar(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::Scatter(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::Bubble(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::Candlestick(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::Boxplot(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::Heatmap(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::Pie(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::Radar(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::PolarBar(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::PolarScatter(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::Gauge(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                    LayerSpec::Table(l) => l
                        .data
                        .clone()
                        .or_else(|| shared_data.cloned())
                        .unwrap_or_default(),
                };

                let name = match layer {
                    LayerSpec::Line(l) => l.name.clone(),
                    LayerSpec::Bar(l) => l.name.clone(),
                    LayerSpec::Scatter(l) => l.name.clone(),
                    LayerSpec::Bubble(l) => l.name.clone(),
                    LayerSpec::Candlestick(l) => l.name.clone(),
                    LayerSpec::Boxplot(l) => l.name.clone(),
                    LayerSpec::Heatmap(l) => l.name.clone(),
                    LayerSpec::Pie(l) => l.name.clone(),
                    LayerSpec::Radar(l) => l.name.clone(),
                    LayerSpec::PolarBar(l) => l.name.clone(),
                    LayerSpec::PolarScatter(l) => l.name.clone(),
                    LayerSpec::Gauge(l) => l.name.clone(),
                    LayerSpec::Table(l) => l.name.clone(),
                };

                let grid_idx = match layer {
                    LayerSpec::Line(l) => l.grid_index,
                    LayerSpec::Bar(l) => l.grid_index,
                    LayerSpec::Scatter(l) => l.grid_index,
                    LayerSpec::Bubble(l) => l.grid_index,
                    LayerSpec::Candlestick(l) => l.grid_index,
                    LayerSpec::Boxplot(l) => l.grid_index,
                    LayerSpec::Heatmap(l) => l.grid_index,
                    LayerSpec::Pie(_) => 0,
                    LayerSpec::Radar(_) => 0,
                    LayerSpec::PolarBar(_) => 0,
                    LayerSpec::PolarScatter(_) => 0,
                    LayerSpec::Gauge(_) => 0,
                    LayerSpec::Table(_) => 0,
                };

                let y_axis_idx = match layer {
                    LayerSpec::Line(l) => l.y_axis_index,
                    LayerSpec::Bar(l) => l.y_axis_index,
                    LayerSpec::Scatter(l) => l.y_axis_index,
                    LayerSpec::Bubble(l) => l.y_axis_index,
                    LayerSpec::Candlestick(l) => l.y_axis_index,
                    LayerSpec::Boxplot(l) => l.y_axis_index,
                    LayerSpec::Heatmap(l) => l.y_axis_index,
                    _ => 0,
                };

                let stack = match layer {
                    LayerSpec::Line(l) => l.stack.clone(),
                    LayerSpec::Bar(l) => l.stack.clone(),
                    _ => None,
                };

                let group_index = match layer {
                    LayerSpec::Bar(l) => l.group_index.unwrap_or(0),
                    _ => 0,
                };

                let sampling = match &layer {
                    LayerSpec::Line(l) => l.sampling.as_ref().map(|s| {
                        let ty = match s {
                            crate::api::layer::Sampling::Lttb(_) => {
                                crate::sampling::SamplingType::Lttb
                            }
                            crate::api::layer::Sampling::Average(_) => {
                                crate::sampling::SamplingType::Average
                            }
                            crate::api::layer::Sampling::Max(_) => {
                                crate::sampling::SamplingType::Max
                            }
                            crate::api::layer::Sampling::Min(_) => {
                                crate::sampling::SamplingType::Min
                            }
                        };
                        let threshold = match s {
                            crate::api::layer::Sampling::Lttb(n)
                            | crate::api::layer::Sampling::Average(n)
                            | crate::api::layer::Sampling::Max(n)
                            | crate::api::layer::Sampling::Min(n) => *n,
                        };
                        (ty, threshold)
                    }),
                    _ => None,
                };

                SeriesSpec {
                    name,
                    data,
                    grid_index: grid_idx,
                    x_axis_index: 0,
                    y_axis_index: y_axis_idx,
                    stack,
                    group_index,
                    sampling,
                    item_style: ItemStyleSpec::default(),
                    config,
                }
            })
            .collect();

        ChartSpec {
            width: self.width,
            height: self.height,
            grids,
            x_axes,
            y_axes,
            series,
            title: self.title.as_ref().map(|t| TitleSpec {
                text: Some(t.text.clone()),
                subtext: t.subtext.clone(),
                font_size: None,
                subfont_size: None,
                color: None,
                subcolor: None,
            }),
            legend: self.legend.as_ref().map(|l| LegendSpec {
                show: l.show,
                data: l.data.clone(),
                symbol_size: 10.0,
                item_gap: 10.0,
                formatter: l.formatter.clone(),
            }),
            background: self.background_color.unwrap_or(Color::rgb(255, 255, 255)),
            palette: vec![],
            theme_name: self.theme_name.clone(),
        }
    }
}

// ── From impls for Into<LayerSpec> ──

impl_from_layer!(Line, Line);
impl_from_layer!(Bar, Bar);
impl_from_layer!(Pie, Pie);
impl_from_layer!(Scatter, Scatter);
impl_from_layer!(Bubble, Bubble);
impl_from_layer!(Candlestick, Candlestick);
impl_from_layer!(Boxplot, Boxplot);
impl_from_layer!(Heatmap, Heatmap);
impl_from_layer!(Radar, Radar);
impl_from_layer!(PolarBar, PolarBar);
impl_from_layer!(PolarScatter, PolarScatter);
impl_from_layer!(Gauge, Gauge);
impl_from_layer!(Table, Table);

// ── Into<Title> for &str ──

impl From<String> for Title {
    fn from(text: String) -> Self {
        Title::new(text)
    }
}

impl From<&str> for Title {
    fn from(text: &str) -> Self {
        Title::new(text.to_string())
    }
}

impl From<Title> for Option<Legend> {
    fn from(_: Title) -> Self {
        None
    }
}
