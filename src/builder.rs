use crate::error::{ChartError, Result};
use crate::model::ChartModel;
use crate::option::{
    AxisOption, ColorOption, GridOption, LegendOption, LieChartOption, RadarOption, SeriesOption,
    TextStyleOption, TitleOption,
};
use crate::theme::{Theme, ThemeRegistry};

#[derive(Debug, Clone)]
pub struct ChartBuilder {
    theme_registry: ThemeRegistry,
    option: LieChartOption,
}

impl Default for ChartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartBuilder {
    pub fn new() -> Self {
        Self {
            theme_registry: ThemeRegistry::new(),
            option: LieChartOption::default(),
        }
    }

    pub fn from_option(option: LieChartOption) -> Self {
        Self {
            theme_registry: ThemeRegistry::new(),
            option,
        }
    }

    pub fn from_option_json(option: &str) -> Result<Self> {
        Ok(Self {
            theme_registry: ThemeRegistry::new(),
            option: serde_json::from_str(option)?,
        })
    }

    pub fn register_theme(mut self, theme: Theme) -> Self {
        self.theme_registry.register(theme);
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.option.theme = Some(theme.name.clone());
        self.theme_registry.register(theme);
        self
    }

    pub fn with_title(mut self, title: TitleOption) -> Self {
        self.option.title = Some(title);
        self
    }

    pub fn with_legend(mut self, legend: LegendOption) -> Self {
        self.option.legend = Some(legend);
        self
    }

    pub fn with_grid(mut self, grid: GridOption) -> Self {
        self.option.grid.push(grid);
        self
    }

    pub fn with_x_axis(mut self, axis: AxisOption) -> Self {
        self.option.x_axis.push(axis);
        self
    }

    pub fn with_y_axis(mut self, axis: AxisOption) -> Self {
        self.option.y_axis.push(axis);
        self
    }

    pub fn with_series(mut self, series: SeriesOption) -> Self {
        self.option.series.push(series);
        self
    }

    pub fn with_radar(mut self, radar: RadarOption) -> Self {
        self.option.radar = Some(radar);
        self
    }

    pub fn with_color(mut self, colors: Vec<ColorOption>) -> Self {
        self.option.color = Some(colors);
        self
    }

    pub fn with_background_color(mut self, color: ColorOption) -> Self {
        self.option.background_color = Some(color);
        self
    }

    pub fn with_text_style(mut self, style: TextStyleOption) -> Self {
        self.option.text_style = Some(style);
        self
    }

    pub fn build(self) -> Result<ChartModel> {
        let theme = match self.option.theme.as_deref() {
            Some(name) => self
                .theme_registry
                .get(name)
                .cloned()
                .ok_or_else(|| ChartError::InvalidTheme(format!("未找到主题: {}", name)))?,
            None => Theme::echarts(),
        };
        ChartModel::new(self.option, theme)
    }
}
