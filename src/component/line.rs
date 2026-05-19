use crate::{
    component::{ChartComponent, SeriesComponent, SeriesContext},
    layout::LayoutOutput,
    model::{ChartModel, LineSeries},
    pipeline::{builder::VisualBuilder, mapper::CoordinateMapper},
    visual::{Stroke, VisualElement},
};

pub struct LineSeriesComponent {
    series: LineSeries,
    series_index: usize,
    grid_index: usize,
}

impl LineSeriesComponent {
    pub fn new(series: &LineSeries, series_index: usize, grid_index: usize) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index,
        }
    }

    fn build_with_context(&self, ctx: &SeriesContext) -> Vec<VisualElement> {
        let coord = ctx.coord;

        // 1. Transform
        let transformer: Box<dyn crate::pipeline::transform::DataTransformer> =
            if self.series.stack.is_some() {
                Box::new(crate::pipeline::transform::StackedTransformer::new(
                    self.series.stack.clone(),
                ))
            } else {
                Box::new(crate::pipeline::transform::IdentityTransformer)
            };

        let all_series: Vec<crate::model::ResolvedSeries> = ctx.resolved.series.clone();
        let transformed_list = transformer.transform(&all_series);

        let transformed = match transformed_list
            .iter()
            .find(|t| t.series_index == self.series_index)
        {
            Some(t) => t,
            None => return Vec::new(),
        };

        // 2. Map
        let has_area = self.series.area_style.is_some();
        let mapper = crate::pipeline::mapper::CartesianLineMapper::new(self.series.smooth)
            .with_area(has_area);
        let mapped = mapper.map(transformed, coord, self.series.y_axis_index);

        // 3. Build
        let line_color = self.series.line_style.color;
        let area_color = if let Some(ref a) = self.series.area_style {
            a.color.or(Some(line_color))
        } else {
            None
        };
        let area_opacity = self
            .series
            .area_style
            .as_ref()
            .map(|a| a.opacity)
            .unwrap_or(0.7);

        let series_style = crate::pipeline::SeriesStyle {
            color: line_color,
            stroke: None,
            fill: None,
        };

        let builder = crate::pipeline::builder::LineVisualBuilder::new()
            .with_smooth(self.series.smooth)
            .with_show_symbol(self.series.symbol != crate::model::Symbol::None)
            .with_symbol_size(self.series.symbol_size)
            .with_line_style(Stroke {
                color: line_color,
                width: self.series.line_style.width,
            })
            .with_area(area_color, area_opacity)
            .with_series_style(series_style);

        builder.build(transformed, &mapped, coord)
    }
}

impl SeriesComponent for LineSeriesComponent {
    fn series_index(&self) -> usize {
        self.series_index
    }

    fn grid_index(&self) -> usize {
        self.grid_index
    }

    fn is_empty(&self) -> bool {
        self.series.data.is_empty()
    }
}

impl ChartComponent for LineSeriesComponent {
    fn build_visual_elements(
        &self,
        resolved: &ChartModel,
        layout: &LayoutOutput,
    ) -> Vec<VisualElement> {
        let ctx = match self.create_context(resolved, layout) {
            Some(ctx) => ctx,
            None => return Vec::new(),
        };
        self.build_with_context(&ctx)
    }
}
