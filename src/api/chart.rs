use crate::{
    error::Result,
    option::{
        self, AxisOption, AxisType as InternalAxisType,
        BarSeriesOption, BubbleDataPoint, BubbleSeriesOption, CandlestickDataPoint,
        CandlestickSeriesOption, ChartOption, ColorOption, DataPoint, GaugeDataPoint,
        GaugeSeriesOption, GridOption, ItemStyleOption, LegendOption, LineSeriesOption,
        PieSeriesOption, PolarBarSeriesOption, PolarScatterDataPoint, PolarScatterSeriesOption,
        PositionOption, PositionPreset, RadarDataOption, RadarIndicatorOption, RadarOption,
        RadarSeriesOption, ScatterSeriesOption, SeriesOption, TableSeriesOption,
    },
    pipeline::{
        dataframe::{DataFrame, DataValue},
        pipeline::build_chart_with_theme,
    },
    theme::Theme,
    visual::{Color, VisualElement},
};

use super::{
    config::{Axis, AxisPosition, AxisType, Grid, Legend, Orient, Position, Title},
    layer::{
        BarLayer, BubbleLayer, CandlestickLayer, GaugeLayer, LayerSpec, LineLayer, PieLayer,
        PolarBarLayer, PolarScatterLayer, RadarLayer, ScatterLayer, TableLayer,
    },
};

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
///     .title("My Chart")
///     .add_bar(BarLayer::new(df).x("category").y("value"))
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
            background_color: None,
            theme_name: None,
        }
    }

    // ── Configuration ──

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

    pub fn add_line(mut self, layer: LineLayer) -> Self {
        self.layers.push(LayerSpec::Line(layer));
        self
    }

    pub fn add_bar(mut self, layer: BarLayer) -> Self {
        self.layers.push(LayerSpec::Bar(layer));
        self
    }

    pub fn add_pie(mut self, layer: PieLayer) -> Self {
        self.layers.push(LayerSpec::Pie(layer));
        self
    }

    pub fn add_scatter(mut self, layer: ScatterLayer) -> Self {
        self.layers.push(LayerSpec::Scatter(layer));
        self
    }

    pub fn add_bubble(mut self, layer: BubbleLayer) -> Self {
        self.layers.push(LayerSpec::Bubble(layer));
        self
    }

    pub fn add_candlestick(mut self, layer: CandlestickLayer) -> Self {
        self.layers.push(LayerSpec::Candlestick(layer));
        self
    }

    pub fn add_radar(mut self, layer: RadarLayer) -> Self {
        self.layers.push(LayerSpec::Radar(layer));
        self
    }

    pub fn add_polar_bar(mut self, layer: PolarBarLayer) -> Self {
        self.layers.push(LayerSpec::PolarBar(layer));
        self
    }

    pub fn add_polar_scatter(mut self, layer: PolarScatterLayer) -> Self {
        self.layers.push(LayerSpec::PolarScatter(layer));
        self
    }

    pub fn add_gauge(mut self, layer: GaugeLayer) -> Self {
        self.layers.push(LayerSpec::Gauge(layer));
        self
    }

    pub fn add_table(mut self, layer: TableLayer) -> Self {
        self.layers.push(LayerSpec::Table(layer));
        self
    }

    pub fn add_layer(mut self, layer: impl Into<LayerSpec>) -> Self {
        self.layers.push(layer.into());
        self
    }

    // ── Rendering ──

    /// Build the chart and collect visual elements.
    pub fn build(&self) -> Result<Vec<VisualElement>> {
        let option = self.to_chart_option();
        let theme = Theme::echarts();
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
        let image = image::RgbaImage::from_raw(
            pixmap.width() as u32,
            pixmap.height() as u32,
            data,
        )
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
            let series = convert_layer(layer);
            option.series.push(series);
        }

        // Auto-detect category labels from data if no x_axis data set
        if let Some(series) = option.series.first() {
            let cat_data = extract_category_names(&option, series);
            if let Some(cat_names) = cat_data {
                if let Some(x_axis) = option.x_axis.first_mut() {
                    if x_axis.data.is_none() || x_axis.data.as_ref().map(|d| d.is_empty()).unwrap_or(true) {
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
        self.layers.iter().any(|l| matches!(l, LayerSpec::Line(_) | LayerSpec::Bar(_) | LayerSpec::Scatter(_) | LayerSpec::Bubble(_) | LayerSpec::Candlestick(_)))
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
        ..Default::default()
    }
}

fn convert_layer(layer: &LayerSpec) -> SeriesOption {
    match layer {
        LayerSpec::Line(l) => convert_line_layer(l),
        LayerSpec::Bar(b) => convert_bar_layer(b),
        LayerSpec::Pie(p) => convert_pie_layer(p),
        LayerSpec::Scatter(s) => convert_scatter_layer(s),
        LayerSpec::Bubble(b) => convert_bubble_layer(b),
        LayerSpec::Candlestick(c) => convert_candlestick_layer(c),
        LayerSpec::Radar(r) => convert_radar_layer(r),
        LayerSpec::PolarBar(p) => convert_polar_bar_layer(p),
        LayerSpec::PolarScatter(p) => convert_polar_scatter_layer(p),
        LayerSpec::Gauge(g) => convert_gauge_layer(g),
        LayerSpec::Table(t) => convert_table_layer(t),
    }
}

fn convert_line_layer(l: &LineLayer) -> SeriesOption {
    let data = extract_xy_data(&l.data, &l.x, &l.y);
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

fn convert_bar_layer(b: &BarLayer) -> SeriesOption {
    let data = extract_xy_data(&b.data, &b.x, &b.y);
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

fn convert_pie_layer(p: &PieLayer) -> SeriesOption {
    let data: Vec<DataPoint> = (0..p.data.row_count())
        .map(|i| {
            let name = p
                .data
                .get_column(&p.category)
                .and_then(|c| c.as_string(i))
                .unwrap_or_default();
            let value = p
                .data
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

fn convert_scatter_layer(s: &ScatterLayer) -> SeriesOption {
    let data = extract_xy_data(&s.data, &s.x, &s.y);
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

fn convert_bubble_layer(b: &BubbleLayer) -> SeriesOption {
    let data: Vec<BubbleDataPoint> = (0..b.data.row_count())
        .map(|i| {
            let x = b.data.get_column("x").and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let y = b.data.get_column("y").and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let size = b.data.get_column("size").and_then(|c| c.as_f64(i));
            let name = b.name_col.as_ref().and_then(|col| {
                b.data.get_column(col).and_then(|c| c.as_string(i))
            });
            BubbleDataPoint {
                x,
                y,
                size,
                name,
            }
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

fn convert_candlestick_layer(c: &CandlestickLayer) -> SeriesOption {
    let data: Vec<CandlestickDataPoint> = (0..c.data.row_count())
        .map(|i| {
            let open = c.data.get_column(&c.open).and_then(|col| col.as_f64(i)).unwrap_or(0.0);
            let close = c.data.get_column(&c.close).and_then(|col| col.as_f64(i)).unwrap_or(0.0);
            let low = c.data.get_column(&c.low).and_then(|col| col.as_f64(i)).unwrap_or(0.0);
            let high = c.data.get_column(&c.high).and_then(|col| col.as_f64(i)).unwrap_or(0.0);
            let name = c.data.get_column(&c.category).and_then(|col| col.as_string(i));
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

fn convert_radar_layer(r: &RadarLayer) -> SeriesOption {
    // The data values come from the `values` column, parsed as comma-separated or as a Vec<f64>
    let data: Vec<RadarDataOption> = (0..r.data.row_count())
        .map(|i| {
            let value_str = r
                .data
                .get_column(&r.values)
                .and_then(|c| c.as_string(i))
                .unwrap_or_default();
            let values: Vec<f64> = value_str
                .split(',')
                .filter_map(|s| s.trim().parse::<f64>().ok())
                .collect();
            let name = r.data.get_column("name").and_then(|c| c.as_string(i));
            RadarDataOption { value: values, name }
        })
        .collect();

    let mut opt = RadarSeriesOption::default();
    opt.name = Some(r.name.clone());
    opt.data = data;

    SeriesOption::Radar(opt)
}

fn convert_polar_bar_layer(p: &PolarBarLayer) -> SeriesOption {
    let data: Vec<DataPoint> = (0..p.data.row_count())
        .map(|i| {
            let angle = p.data.get_column(&p.angle).and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let radius = p.data.get_column(&p.radius).and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            DataPoint::XY(angle, radius)
        })
        .collect();

    let mut opt = PolarBarSeriesOption::new(p.name.clone(), data);
    opt.pad_angle = Some(p.pad_angle);
    opt.start_angle = Some(p.start_angle);

    SeriesOption::PolarBar(opt)
}

fn convert_polar_scatter_layer(p: &PolarScatterLayer) -> SeriesOption {
    let data: Vec<PolarScatterDataPoint> = (0..p.data.row_count())
        .map(|i| {
            let angle = p.data.get_column(&p.angle).and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let radius = p.data.get_column(&p.radius).and_then(|c| c.as_f64(i)).unwrap_or(0.0);
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

fn convert_gauge_layer(g: &GaugeLayer) -> SeriesOption {
    let data: Vec<GaugeDataPoint> = (0..g.data.row_count())
        .map(|i| {
            let value = g.data.get_column(&g.value).and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let name = g.data.get_column("name").and_then(|c| c.as_string(i));
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

fn convert_table_layer(t: &TableLayer) -> SeriesOption {
    // Convert DataFrame to a grid of strings for the table renderer
    let col_names: Vec<String> = t.data.column_names().to_vec();
    let row_count = t.data.row_count();

    let mut grid_data: Vec<Vec<serde_json::Value>> = Vec::new();

    for i in 0..row_count {
        let mut row = Vec::new();
        for col_name in &col_names {
            let val = t.data.get_column(col_name).and_then(|c| c.as_string(i));
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

    if names.is_empty() {
        None
    } else {
        Some(names)
    }
}

// ── From impls for Into<LayerSpec> ──

impl From<LineLayer> for LayerSpec {
    fn from(l: LineLayer) -> Self {
        LayerSpec::Line(l)
    }
}

impl From<BarLayer> for LayerSpec {
    fn from(b: BarLayer) -> Self {
        LayerSpec::Bar(b)
    }
}

impl From<PieLayer> for LayerSpec {
    fn from(p: PieLayer) -> Self {
        LayerSpec::Pie(p)
    }
}

impl From<ScatterLayer> for LayerSpec {
    fn from(s: ScatterLayer) -> Self {
        LayerSpec::Scatter(s)
    }
}

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