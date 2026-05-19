use crate::{
    chart::Chart,
    error::{ChartError, Result},
    model::ChartModel,
    option::{
        AxisOption, ChartOption, ColorOption, DataPoint, GridOption, LegendOption,
        LineSeriesOption, RadarOption, SeriesOption, TextStyleOption, TitleOption,
    },
    theme::{Theme, ThemeRegistry},
};

/// A fluent builder for creating charts.
///
/// Configure chart options via method chaining, then call [`build`](ChartBuilder::build)
/// to produce a [`Chart`] ready for rendering.
///
/// # Examples
///
/// ```no_run
/// use liecharts::{ChartBuilder, TitleOption, AxisOption, SeriesOption, BarSeriesOption};
///
/// ChartBuilder::new()
///     .with_title(TitleOption::new("My Chart"))
///     .with_x_axis(AxisOption::category().data(["Q1", "Q2", "Q3"]))
///     .with_y_axis(AxisOption::value())
///     .with_series(SeriesOption::Bar(BarSeriesOption::new(
///         "Sales",
///         vec![100.0, 200.0, 150.0],
///     )))
///     .build(800, 600)
///     .unwrap()
///     .render_to_image("chart.png")
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ChartBuilder {
    theme_registry: ThemeRegistry,
    option: ChartOption,
}

impl Default for ChartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartBuilder {
    /// Creates a new `ChartBuilder` with default options.
    pub fn new() -> Self {
        Self {
            theme_registry: ThemeRegistry::new(),
            option: ChartOption::default(),
        }
    }

    /// Creates a `ChartBuilder` from an existing [`ChartOption`].
    pub fn from_option(option: ChartOption) -> Self {
        Self {
            theme_registry: ThemeRegistry::new(),
            option,
        }
    }

    /// Creates a `ChartBuilder` from a JSON string of [`ChartOption`].
    pub fn from_option_json(option: &str) -> Result<Self> {
        Ok(Self {
            theme_registry: ThemeRegistry::new(),
            option: serde_json::from_str(option)?,
        })
    }

    /// Registers a custom [`Theme`] for reuse.
    pub fn register_theme(mut self, theme: Theme) -> Self {
        self.theme_registry.register(theme);
        self
    }

    /// Sets the chart theme by name.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.option.theme = Some(theme.name.clone());
        self.theme_registry.register(theme);
        self
    }

    /// Sets the chart title.
    pub fn with_title(mut self, title: TitleOption) -> Self {
        self.option.title = Some(title);
        self
    }

    /// Sets the chart legend.
    pub fn with_legend(mut self, legend: LegendOption) -> Self {
        self.option.legend = Some(legend);
        self
    }

    /// Adds a grid region for multi-layout charts.
    pub fn with_grid(mut self, grid: GridOption) -> Self {
        self.option.grid.push(grid);
        self
    }

    /// Adds an X-axis.
    pub fn with_x_axis(mut self, axis: AxisOption) -> Self {
        self.option.x_axis.push(axis);
        self
    }

    /// Adds a Y-axis.
    pub fn with_y_axis(mut self, axis: AxisOption) -> Self {
        self.option.y_axis.push(axis);
        self
    }

    /// Adds a data series (line, bar, pie, etc.).
    pub fn with_series(mut self, series: SeriesOption) -> Self {
        self.option.series.push(series);
        self
    }

    /// Convenience method to plot a mathematical function as a smooth line series.
    ///
    /// Computes `f(x)` evenly across `range` with `steps` intervals,
    /// then wraps it as a [`LineSeriesOption`] with smoothing enabled.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use liecharts::ChartBuilder;
    ///
    /// ChartBuilder::new()
    ///     .add_function("y = x²", -1.0..=1.0, 100, |x| x * x)
    ///     .build(800, 600)
    ///     .unwrap()
    ///     .render_to_svg("function.svg")
    ///     .unwrap();
    /// ```
    pub fn add_function(
        mut self,
        name: impl Into<String>,
        range: std::ops::RangeInclusive<f64>,
        steps: usize,
        f: impl Fn(f64) -> f64,
    ) -> Self {
        let data = function_data(range, steps, f);
        self.option.series.push(SeriesOption::Line(
            LineSeriesOption::new(name, data).smooth(true),
        ));
        self
    }

    /// Sets the radar configuration.
    pub fn with_radar(mut self, radar: RadarOption) -> Self {
        self.option.radar = Some(radar);
        self
    }

    /// Sets the color palette.
    pub fn with_color(mut self, colors: Vec<ColorOption>) -> Self {
        self.option.color = Some(colors);
        self
    }

    pub fn with_background_color(mut self, color: ColorOption) -> Self {
        self.option.background_color = Some(color);
        self
    }

    /// Sets the default text style.
    pub fn with_text_style(mut self, style: TextStyleOption) -> Self {
        self.option.text_style = Some(style);
        self
    }

    /// Resolves all options and builds a [`ChartModel`] (data only, no layout or rendering).
    pub fn build_model(self) -> Result<ChartModel> {
        let theme =
            match self.option.theme.as_deref() {
                Some(name) => self.theme_registry.get(name).cloned().ok_or_else(|| {
                    ChartError::ThemeNotFound(format!("Theme not found: {}", name))
                })?,
                None => Theme::echarts(),
            };
        ChartModel::new(self.option, theme)
    }

    /// Builds a [`Chart`] bound to the given dimensions, ready for rendering.
    pub fn build(self, width: u32, height: u32) -> Result<Chart> {
        let model = self.build_model()?;
        Ok(Chart::new(model, width, height))
    }

    /// Builds and renders to a PNG/JPEG image file in one step.
    pub fn render_to_image(self, width: u32, height: u32, path: &str) -> Result<()> {
        self.build(width, height)?.render_to_image(path)
    }

    /// Builds and renders to an SVG file in one step.
    pub fn render_to_svg(self, width: u32, height: u32, path: &str) -> Result<()> {
        self.build(width, height)?.render_to_svg(path)
    }

    /// Builds and renders to PNG bytes in one step.
    pub fn render_png(self, width: u32, height: u32) -> Result<Vec<u8>> {
        self.build(width, height)?.render_png()
    }

    /// Builds and renders to an SVG string in one step.
    pub fn render_svg(self, width: u32, height: u32) -> Result<String> {
        self.build(width, height)?.render_svg()
    }
}

/// Generates evenly-spaced (x, f(x)) data points for plotting a function.
///
/// This is useful when you want full control over the [`LineSeriesOption`],
/// e.g. to configure sampling:
///
/// ```no_run
/// use liecharts::{ChartBuilder, LineSeriesOption, SeriesOption, SamplingOption};
/// use liecharts::builder::function_data;
///
/// ChartBuilder::new()
///     .with_series(SeriesOption::Line(
///         LineSeriesOption::new("y = x²", function_data(-1.0..=1.0, 100, |x| x * x))
///             .smooth(true)
///             .sampling(SamplingOption::lttb(200)),
///     ))
///     .build(800, 600)
///     .unwrap()
///     .render_to_svg("function.svg")
///     .unwrap();
/// ```
pub fn function_data(
    range: std::ops::RangeInclusive<f64>,
    steps: usize,
    f: impl Fn(f64) -> f64,
) -> Vec<DataPoint> {
    let start = *range.start();
    let end = *range.end();
    let step = if steps > 0 {
        (end - start) / steps as f64
    } else {
        end - start
    };

    (0..=steps)
        .map(|i| {
            let x = start + i as f64 * step;
            DataPoint::from((x, f(x)))
        })
        .collect()
}
