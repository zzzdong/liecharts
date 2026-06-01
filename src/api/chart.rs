use super::layer::{
    Bar, Bubble, Candlestick, Gauge, LayerSpec, Line, Pie,
    PolarBar, PolarScatter, Radar, Scatter, Table,
};
use crate::{
    error::Result,
    option::{
        self, AxisOption, AxisType as InternalAxisType, BarSeriesOption, BubbleDataPoint,
        BubbleSeriesOption, CandlestickDataPoint, CandlestickSeriesOption, ChartOption,
        ColorOption, DataPoint, GaugeDataPoint, GaugeSeriesOption, GridOption, ItemStyleOption,
        LegendOption, LineSeriesOption, PieSeriesOption, PolarBarSeriesOption,
        PolarScatterDataPoint, PolarScatterSeriesOption, PositionOption, PositionPreset,
        RadarDataOption, RadarIndicatorOption, RadarOption, RadarSeriesOption, ScatterSeriesOption,
        SeriesOption, TableSeriesOption,
    },
    pipeline::{
        dataframe::{DataFrame, DataValue},
        pipeline::build_chart_with_theme,
    },
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

    pub fn y_axis(mut self, axis: Axis) -> Self {
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

    // ── Rendering ──

    /// Build the chart and collect visual elements.
    pub fn build(&self) -> Result<Vec<VisualElement>> {
        let option = self.to_chart_option();
        let theme = match self.theme_name.as_deref() {
            Some("dark") => Theme::dark(),
            _ => Theme::echarts(),
        };
        build_chart_with_theme(&option, self.width, self.height, &theme)
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

// ── Conversion to ChartOption ──

impl Chart {
    fn to_chart_option(&self) -> ChartOption {
        let mut option = ChartOption::default();

        // Title
        if let Some(title) = &self.title {
            option.title = Some(option::TitleOption {
                text: Some(title.text.clone()),
                subtext: title.subtext.clone(),
                left: Some(convert_position(title.left)),
                top: Some(convert_position(title.top)),
                text_style: None,
                subtext_style: None,
            });
        }

        // Legend
        if let Some(legend) = &self.legend {
            if !legend.data.is_empty() || legend.show {
                option.legend = Some(LegendOption {
                    show: Some(legend.show),
                    data: Some(legend.data.clone()),
                    left: Some(convert_position(legend.left)),
                    top: Some(convert_position(legend.top)),
                    orient: Some(convert_orient(legend.orient)),
                    ..Default::default()
                });
            }
        }

        // Grids
        if self.grids.is_empty() {
            option.grid.push(GridOption::default());
        } else {
            for grid in &self.grids {
                option.grid.push(GridOption {
                    left: Some(convert_position(grid.left)),
                    right: Some(convert_position(grid.right)),
                    top: Some(convert_position(grid.top)),
                    bottom: Some(convert_position(grid.bottom)),
                    contain_label: Some(grid.contain_label),
                });
            }
        }

        // X Axes
        if self.x_axes.is_empty() && self.has_cartesian_layers() {
            option.x_axis.push(AxisOption::category());
        } else {
            for axis in &self.x_axes {
                option.x_axis.push(convert_axis(axis));
            }
        }

        // Y Axes
        if self.y_axes.is_empty() && self.has_cartesian_layers() {
            option.y_axis.push(AxisOption::value());
        } else {
            for axis in &self.y_axes {
                option.y_axis.push(convert_axis(axis));
            }
        }

        // Background color
        if let Some(color) = &self.background_color {
            option.background_color = Some(ColorOption::new(color.r, color.g, color.b));
        }

        // Series / Layers
        for layer in &self.layers {
            let series = convert_layer(layer, self.data.as_ref());
            option.series.push(series);
        }

        // Auto-detect category labels from data if no x_axis data set
        if let Some(series) = option.series.first() {
            let cat_data = extract_category_names(&option, series);
            if let Some(cat_names) = cat_data {
                if let Some(x_axis) = option.x_axis.first_mut() {
                    if x_axis.data.is_none()
                        || x_axis.data.as_ref().map(|d| d.is_empty()).unwrap_or(true)
                    {
                        x_axis.data = Some(cat_names);
                    }
                }
            }
        }

        // Radar indicators from layers
        for layer in &self.layers {
            if let LayerSpec::Radar(rl) = layer {
                let indicators: Vec<RadarIndicatorOption> = rl
                    .indicators
                    .iter()
                    .map(|name| RadarIndicatorOption {
                        name: Some(name.clone()),
                        max: None,
                    })
                    .collect();
                option.radar = Some(RadarOption {
                    indicator: Some(indicators),
                    ..Default::default()
                });
                break;
            }
        }

        option
    }

    fn has_cartesian_layers(&self) -> bool {
        self.layers.iter().any(|l| {
            matches!(
                l,
                LayerSpec::Line(_)
                    | LayerSpec::Bar(_)
                    | LayerSpec::Scatter(_)
                    | LayerSpec::Bubble(_)
                    | LayerSpec::Candlestick(_)
            )
        })
    }
}

// ── Conversion helpers ──

fn convert_position(pos: Position) -> PositionOption {
    match pos {
        Position::Auto => PositionOption::Preset(PositionPreset::Auto),
        Position::Center => PositionOption::Preset(PositionPreset::Center),
        Position::Left => PositionOption::Preset(PositionPreset::Left),
        Position::Right => PositionOption::Preset(PositionPreset::Right),
        Position::Top => PositionOption::Preset(PositionPreset::Top),
        Position::Bottom => PositionOption::Preset(PositionPreset::Bottom),
        Position::Pixel(v) => PositionOption::Pixel(v),
        Position::Percent(v) => PositionOption::Percent(v),
    }
}

fn convert_orient(orient: Orient) -> option::Orient {
    match orient {
        Orient::Horizontal => option::Orient::Horizontal,
        Orient::Vertical => option::Orient::Vertical,
    }
}

fn convert_axis(axis: &Axis) -> AxisOption {
    let axis_type = match axis.axis_type {
        AxisType::Category => InternalAxisType::Category,
        AxisType::Value => InternalAxisType::Value,
    };

    let pos = match axis.position {
        AxisPosition::Top => crate::option::AxisPosition::Top,
        AxisPosition::Bottom => crate::option::AxisPosition::Bottom,
        AxisPosition::Left => crate::option::AxisPosition::Left,
        AxisPosition::Right => crate::option::AxisPosition::Right,
    };

    AxisOption {
        axis_type: Some(axis_type),
        data: if axis.data.is_empty() {
            None
        } else {
            Some(axis.data.clone())
        },
        name: axis.name.clone(),
        min: axis.min,
        max: axis.max,
        boundary_gap: Some(axis.boundary_gap),
        position: Some(pos),
        grid_index: Some(axis.grid_index),
        ..Default::default()
    }
}

fn convert_layer(layer: &LayerSpec, chart_data: Option<&DataFrame>) -> SeriesOption {
    match layer {
        LayerSpec::Line(l) => convert_line_layer(l, chart_data),
        LayerSpec::Bar(b) => convert_bar_layer(b, chart_data),
        LayerSpec::Pie(p) => convert_pie_layer(p, chart_data),
        LayerSpec::Scatter(s) => convert_scatter_layer(s, chart_data),
        LayerSpec::Bubble(b) => convert_bubble_layer(b, chart_data),
        LayerSpec::Candlestick(c) => convert_candlestick_layer(c, chart_data),
        LayerSpec::Radar(r) => convert_radar_layer(r, chart_data),
        LayerSpec::PolarBar(p) => convert_polar_bar_layer(p, chart_data),
        LayerSpec::PolarScatter(p) => convert_polar_scatter_layer(p, chart_data),
        LayerSpec::Gauge(g) => convert_gauge_layer(g, chart_data),
        LayerSpec::Table(t) => convert_table_layer(t, chart_data),
    }
}

fn resolve_data<'a>(layer_data: &'a Option<DataFrame>, chart_data: Option<&'a DataFrame>) -> &'a DataFrame {
    layer_data.as_ref().or(chart_data).expect("Layer must have data either from layer.data() or Chart.data()")
}

fn convert_line_layer(l: &Line, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&l.data, chart_data);
    let data = extract_xy_data(df, &l.x, &l.y);
    let mut item_style = l.color.map(|c| ItemStyleOption {
        color: Some(ColorOption::new(c.r, c.g, c.b)),
        ..Default::default()
    });

    let area_style = if l.area {
        Some(option::AreaStyleOption {
            color: item_style
                .as_ref()
                .and_then(|s| s.color)
                .or_else(|| Some(ColorOption::new(80, 112, 221))),
            opacity: Some(0.3),
        })
    } else {
        None
    };

    if item_style.is_none() && area_style.is_some() {
        item_style = Some(ItemStyleOption {
            color: Some(ColorOption::new(80, 112, 221)),
            ..Default::default()
        });
    }

    let mut opt = LineSeriesOption::new(l.name.clone(), data);
    opt.smooth = Some(l.smooth);
    opt.stack = l.stack.clone();
    opt.symbol = Some(l.symbol.into());
    opt.symbol_size = Some(l.symbol_size);
    opt.item_style = item_style;
    opt.area_style = area_style;
    opt.y_axis_index = Some(l.y_axis_index);
    opt.grid_index = Some(l.grid_index);

    SeriesOption::Line(opt)
}

fn convert_bar_layer(b: &Bar, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&b.data, chart_data);
    let data = extract_xy_data(df, &b.x, &b.y);
    let item_style = b.color.map(|c| ItemStyleOption {
        color: Some(ColorOption::new(c.r, c.g, c.b)),
        ..Default::default()
    });

    let mut opt = BarSeriesOption::new(b.name.clone(), data);
    opt.stack = b.stack.clone();
    opt.group_index = b.group_index;
    opt.item_style = item_style;
    opt.y_axis_index = Some(b.y_axis_index);
    opt.grid_index = Some(b.grid_index);

    SeriesOption::Bar(opt)
}

fn convert_pie_layer(p: &Pie, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&p.data, chart_data);
    let data: Vec<DataPoint> = (0..df.row_count())
        .map(|i| {
            let name = df
                .get_column(&p.category)
                .and_then(|c| c.as_string(i))
                .unwrap_or_default();
            let value = df
                .get_column(&p.value)
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            DataPoint::Named(name, value)
        })
        .collect();

    let mut opt = PieSeriesOption::new(p.name.clone(), data);
    opt.radius = Some(vec![format!("{}%", p.radius.0), format!("{}%", p.radius.1)]);
    opt.center = Some(vec![format!("{}%", p.center.0), format!("{}%", p.center.1)]);

    SeriesOption::Pie(opt)
}

fn convert_scatter_layer(s: &Scatter, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&s.data, chart_data);
    let data = extract_xy_data(df, &s.x, &s.y);
    let item_style = s.color.map(|c| ItemStyleOption {
        color: Some(ColorOption::new(c.r, c.g, c.b)),
        ..Default::default()
    });

    let mut opt = ScatterSeriesOption::new(s.name.clone(), data);
    opt.symbol_size = Some(s.symbol_size);
    opt.item_style = item_style;
    opt.y_axis_index = Some(s.y_axis_index);
    opt.grid_index = Some(s.grid_index);

    SeriesOption::Scatter(opt)
}

fn convert_bubble_layer(b: &Bubble, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&b.data, chart_data);
    let data: Vec<BubbleDataPoint> = (0..df.row_count())
        .map(|i| {
            let x = df
                .get_column("x")
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let y = df
                .get_column("y")
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let size = df.get_column("size").and_then(|c| c.as_f64(i));
            let name = b
                .name_col
                .as_ref()
                .and_then(|col| df.get_column(col).and_then(|c| c.as_string(i)));
            BubbleDataPoint { x, y, size, name }
        })
        .collect();

    let opt = BubbleSeriesOption {
        name: Some(b.name.clone()),
        data,
        y_axis_index: Some(b.y_axis_index),
        grid_index: Some(b.grid_index),
        symbol_size_scale: Some(b.symbol_size_scale),
        item_style: b.color.map(|c| ItemStyleOption {
            color: Some(ColorOption::new(c.r, c.g, c.b)),
            ..Default::default()
        }),
    };

    SeriesOption::Bubble(opt)
}

fn convert_candlestick_layer(c: &Candlestick, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&c.data, chart_data);
    let data: Vec<CandlestickDataPoint> = (0..df.row_count())
        .map(|i| {
            let open = df
                .get_column(&c.open)
                .and_then(|col| col.as_f64(i))
                .unwrap_or(0.0);
            let close = df
                .get_column(&c.close)
                .and_then(|col| col.as_f64(i))
                .unwrap_or(0.0);
            let low = df
                .get_column(&c.low)
                .and_then(|col| col.as_f64(i))
                .unwrap_or(0.0);
            let high = df
                .get_column(&c.high)
                .and_then(|col| col.as_f64(i))
                .unwrap_or(0.0);
            let name = df
                .get_column(&c.category)
                .and_then(|col| col.as_string(i));
            CandlestickDataPoint {
                open,
                close,
                low,
                high,
                name,
            }
        })
        .collect();

    let mut opt = CandlestickSeriesOption::default();
    opt.name = Some(c.name.clone());
    opt.data = data;
    opt.y_axis_index = Some(c.y_axis_index);
    opt.grid_index = Some(c.grid_index);

    SeriesOption::Candlestick(opt)
}

fn convert_radar_layer(r: &Radar, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&r.data, chart_data);
    let data: Vec<RadarDataOption> = (0..df.row_count())
        .map(|i| {
            let value_str = df
                .get_column(&r.values)
                .and_then(|c| c.as_string(i))
                .unwrap_or_default();
            let values: Vec<f64> = value_str
                .split(',')
                .filter_map(|s| s.trim().parse::<f64>().ok())
                .collect();
            let name = df.get_column("name").and_then(|c| c.as_string(i));
            RadarDataOption {
                value: values,
                name,
            }
        })
        .collect();

    let mut opt = RadarSeriesOption::default();
    opt.name = Some(r.name.clone());
    opt.data = data;

    SeriesOption::Radar(opt)
}

fn convert_polar_bar_layer(p: &PolarBar, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&p.data, chart_data);
    let data: Vec<DataPoint> = (0..df.row_count())
        .map(|i| {
            let angle = df
                .get_column(&p.angle)
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let radius = df
                .get_column(&p.radius)
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            DataPoint::XY(angle, radius)
        })
        .collect();

    let mut opt = PolarBarSeriesOption::new(p.name.clone(), data);
    opt.pad_angle = Some(p.pad_angle);
    opt.start_angle = Some(p.start_angle);

    SeriesOption::PolarBar(opt)
}

fn convert_polar_scatter_layer(p: &PolarScatter, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&p.data, chart_data);
    let data: Vec<PolarScatterDataPoint> = (0..df.row_count())
        .map(|i| {
            let angle = df
                .get_column(&p.angle)
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let radius = df
                .get_column(&p.radius)
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            PolarScatterDataPoint {
                angle,
                radius,
                symbol_size: p.symbol_size,
                name: None,
            }
        })
        .collect();

    let mut opt = PolarScatterSeriesOption::default();
    opt.name = Some(p.name.clone());
    opt.data = data;

    SeriesOption::PolarScatter(opt)
}

fn convert_gauge_layer(g: &Gauge, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&g.data, chart_data);
    let data: Vec<GaugeDataPoint> = (0..df.row_count())
        .map(|i| {
            let value = df
                .get_column(&g.value)
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let name = df.get_column("name").and_then(|c| c.as_string(i));
            GaugeDataPoint { value, name }
        })
        .collect();

    let mut opt = GaugeSeriesOption::default();
    opt.name = Some(g.name.clone());
    opt.data = data;
    opt.min = Some(g.min);
    opt.max = Some(g.max);
    opt.center = Some(vec![format!("{}%", g.center.0), format!("{}%", g.center.1)]);
    opt.radius = Some(format!("{}%", g.radius));
    opt.start_angle = Some(g.start_angle);
    opt.end_angle = Some(g.end_angle);
    opt.split_number = Some(g.split_number);

    SeriesOption::Gauge(opt)
}

fn convert_table_layer(t: &Table, chart_data: Option<&DataFrame>) -> SeriesOption {
    let df = resolve_data(&t.data, chart_data);
    // Convert DataFrame to a grid of strings for the table renderer
    let col_names: Vec<String> = df.column_names().to_vec();
    let row_count = df.row_count();

    let mut grid_data: Vec<Vec<serde_json::Value>> = Vec::new();

    for i in 0..row_count {
        let mut row = Vec::new();
        for col_name in &col_names {
            let val = df.get_column(col_name).and_then(|c| c.as_string(i));
            let json_val: serde_json::Value = val
                .map(|s| {
                    // Try to parse as number first
                    s.parse::<f64>()
                        .map(|n| serde_json::Value::from(n))
                        .unwrap_or_else(|_| serde_json::Value::String(s))
                })
                .unwrap_or(serde_json::Value::Null);
            row.push(json_val);
        }
        grid_data.push(row);
    }

    let mut opt = TableSeriesOption::default();
    opt.name = Some(t.name.clone());
    opt.columns = Some(col_names);
    opt.data = Some(grid_data);

    SeriesOption::Table(opt)
}

/// Extract (x, y) DataPoints from a DataFrame, detecting whether x is category or numeric.
fn extract_xy_data(df: &DataFrame, x_col: &str, y_col: &str) -> Vec<DataPoint> {
    let has_category = df
        .get_column(x_col)
        .and_then(|c| c.data.first())
        .map(|v| matches!(v, DataValue::String(_)))
        .unwrap_or(false);

    (0..df.row_count())
        .map(|i| {
            let y_val = df
                .get_column(y_col)
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);

            if has_category {
                let name = df
                    .get_column(x_col)
                    .and_then(|c| c.as_string(i))
                    .unwrap_or_default();
                DataPoint::Named(name, y_val)
            } else {
                let x_val = df
                    .get_column(x_col)
                    .and_then(|c| c.as_f64(i))
                    .unwrap_or(i as f64);
                DataPoint::XY(x_val, y_val)
            }
        })
        .collect()
}

/// Extract unique category names from a layer's data for populating x_axis labels.
fn extract_category_names(_option: &ChartOption, series: &SeriesOption) -> Option<Vec<String>> {
    let data = match series {
        SeriesOption::Line(l) => Some(&l.data),
        SeriesOption::Bar(b) => Some(&b.data),
        SeriesOption::Scatter(s) => Some(&s.data),
        _ => None,
    }?;

    let names: Vec<String> = data
        .iter()
        .filter_map(|dp| match dp {
            DataPoint::Named(n, _) => Some(n.clone()),
            _ => None,
        })
        .collect();

    if names.is_empty() { None } else { Some(names) }
}

// ── From impls for Into<LayerSpec> ──

impl_from_layer!(Line, Line);
impl_from_layer!(Bar, Bar);
impl_from_layer!(Pie, Pie);
impl_from_layer!(Scatter, Scatter);
impl_from_layer!(Bubble, Bubble);
impl_from_layer!(Candlestick, Candlestick);
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
