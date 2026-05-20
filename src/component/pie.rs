use crate::{
    component::{ChartComponent, SeriesComponent, SeriesContext},
    layout::LayoutOutput,
    model::{ChartModel, PieSeries},
    pipeline::{
        builder::VisualBuilder,
        mapper::{CoordinateMapper, PolarPieMapper},
        transform::{DataTransformer, IdentityTransformer},
    },
    visual::{Color, VisualElement},
};

pub struct PieSeriesComponent {
    series: PieSeries,
    series_index: usize,
    grid_index: usize,
}

impl PieSeriesComponent {
    pub fn new(series: &PieSeries, series_index: usize) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index: series.grid_index,
        }
    }

    fn build_with_context(&self, ctx: &SeriesContext) -> Vec<VisualElement> {
        let plot_bounds = ctx.plot_bounds();

        let coord = crate::layout::DataCoordinateSystem::new(
            plot_bounds,
            (0.0, 100.0),
            vec![(0.0, 100.0)],
            false,
            false,
            0,
            0,
        );

        let resolved_series = crate::model::ResolvedSeries::Pie(self.series.clone());
        let transformer = IdentityTransformer;
        let transformed_list = transformer.transform(&[resolved_series]);
        let transformed = &transformed_list[0];

        let mapper = PolarPieMapper::new()
            .with_center(self.series.center.0, self.series.center.1)
            .with_radius(self.series.radius.0, self.series.radius.1);
        let mapped = mapper.map(transformed, &coord, 0);

        let label_config = if let Some(label) = &self.series.label {
            if label.show {
                crate::pipeline::LabelConfig {
                    show: true,
                    position: crate::pipeline::LabelPosition::Outside,
                    color: label.color,
                    font_size: label.font_size,
                    font_family: label.font_family.clone(),
                    formatter: None,
                }
            } else {
                crate::pipeline::LabelConfig::default()
            }
        } else {
            crate::pipeline::LabelConfig::default()
        };

        let colors: Vec<Color> = self
            .series
            .data
            .iter()
            .enumerate()
            .map(|(i, _)| {
                ctx.resolved
                    .colors
                    .get(i % ctx.resolved.colors.len())
                    .copied()
                    .unwrap_or(Color::new(0, 0, 0))
            })
            .collect();

        let builder = crate::pipeline::builder::PieVisualBuilder::new()
            .with_label_config(label_config)
            .with_border(Color::new(255, 255, 255), 1.0)
            .with_colors(colors.clone());

        builder.build(transformed, &mapped, &coord)
    }
}

impl SeriesComponent for PieSeriesComponent {
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

impl ChartComponent for PieSeriesComponent {
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
