use crate::component::{ChartComponent, SeriesComponent, SeriesContext};
use crate::layout::LayoutOutput;
use crate::model::{ResolvedOption, ScatterSeries};
use crate::pipeline::mapper::{CartesianScatterMapper, CoordinateMapper};
use crate::pipeline::transform::{DataTransformer, IdentityTransformer};
use crate::pipeline::builder::VisualBuilder;
use crate::visual::{Stroke, VisualElement};

pub struct ScatterSeriesComponent {
    series: ScatterSeries,
    series_index: usize,
    grid_index: usize,
}

impl ScatterSeriesComponent {
    pub fn new(series: &ScatterSeries, series_index: usize, grid_index: usize) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index,
        }
    }

    fn build_with_context(&self, ctx: &SeriesContext) -> Vec<VisualElement> {
        let coord = ctx.coord;

        // 1. Transform
        let resolved_series = crate::model::ResolvedSeries::Scatter(self.series.clone());
        let transformer = IdentityTransformer;
        let transformed_list = transformer.transform(&[resolved_series]);
        let transformed = &transformed_list[0];

        // 2. Map
        let mapper = CartesianScatterMapper::new();
        let mapped = mapper.map(transformed, coord, self.series.y_axis_index);

        // 3. Build
        let color = ctx.get_series_color(self.series.item_style.color);

        let border_stroke = self.series.item_style.border_color.map(|c| Stroke {
            color: c,
            width: self.series.item_style.border_width,
        });

        let series_style = crate::pipeline::SeriesStyle {
            color,
            stroke: border_stroke,
            fill: Some(color),
        };

        let builder = crate::pipeline::builder::ScatterVisualBuilder::new()
            .with_symbol_size(self.series.symbol_size)
            .with_series_style(series_style);

        builder.build(transformed, &mapped, coord)
    }
}

impl SeriesComponent for ScatterSeriesComponent {
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

impl ChartComponent for ScatterSeriesComponent {
    fn build_visual_elements(&self, resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<VisualElement> {
        let ctx = match self.create_context(resolved, layout) {
            Some(ctx) => ctx,
            None => return Vec::new(),
        };
        self.build_with_context(&ctx)
    }
}