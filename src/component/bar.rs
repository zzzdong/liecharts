use crate::component::{ChartComponent, SeriesComponent, SeriesContext};
use crate::layout::LayoutOutput;
use crate::model::{BarSeries, ChartModel};
use crate::pipeline::builder::{BarVisualBuilder, VisualBuilder};
use crate::pipeline::mapper::{CartesianBarMapper, CoordinateMapper};
use crate::pipeline::transform::{DataTransformer, IdentityTransformer};
use crate::visual::{Stroke, VisualElement};

pub struct BarSeriesComponent {
    series: BarSeries,
    series_index: usize,
    grid_index: usize,
}

impl BarSeriesComponent {
    pub fn new(series: &BarSeries, series_index: usize, grid_index: usize) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index,
        }
    }

    fn build_with_context(&self, ctx: &SeriesContext) -> Vec<VisualElement> {
        let coord = ctx.coord;

        // 1. Transform
        let resolved_series = crate::model::ResolvedSeries::Bar(self.series.clone());
        let transformer = IdentityTransformer;
        let transformed_list = transformer.transform(&[resolved_series]);
        let transformed = &transformed_list[0];

        // 2. Map
        let cat_width = coord.category_width();
        let bar_width = self.series.bar_width.unwrap_or(cat_width * 0.6);
        let bar_width_ratio = bar_width / cat_width;

        let mapper = CartesianBarMapper::new().with_bar_width_ratio(bar_width_ratio);
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

        let label_config = if let Some(label) = &self.series.label {
            if label.show {
                use crate::pipeline::LabelPosition as PL;
                let pos = match label.position {
                    crate::model::LabelPosition::Top => PL::Top,
                    crate::model::LabelPosition::Inside => PL::Inside,
                    _ => PL::Top,
                };
                crate::pipeline::LabelConfig {
                    show: true,
                    position: pos,
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

        let builder = BarVisualBuilder::new()
            .with_series_style(series_style)
            .with_label_config(label_config);

        builder.build(transformed, &mapped, coord)
    }
}

impl SeriesComponent for BarSeriesComponent {
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

impl ChartComponent for BarSeriesComponent {
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
