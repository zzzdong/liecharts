use crate::component::{ChartComponent, SeriesComponent, SeriesContext};
use crate::layout::LayoutOutput;
use crate::model::{BubbleSeries, ChartModel};
use crate::visual::{FillStrokeStyle, VisualElement};
use vello_cpu::kurbo::Shape as KurboShape;
use vello_cpu::kurbo::{Circle, Point};

pub struct BubbleSeriesComponent {
    series: BubbleSeries,
    series_index: usize,
    grid_index: usize,
}

impl BubbleSeriesComponent {
    pub fn new(series: &BubbleSeries, series_index: usize, grid_index: usize) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index,
        }
    }

    fn build_with_context(&self, ctx: &SeriesContext) -> Vec<VisualElement> {
        let coord = ctx.coord;

        let mut elements = Vec::new();
        let color = self.series.item_style.color.unwrap_or(self.series.color);
        let opacity = self.series.item_style.opacity;
        let mut fill_color = color;
        fill_color.a = (fill_color.a as f64 * opacity).clamp(0.0, 255.0) as u8;

        for bubble in &self.series.data {
            let x = coord.x_to_pixel(bubble.x);
            let y = coord.y_to_pixel(bubble.y, self.series.y_axis_index);

            let radius = (bubble.size.sqrt() * self.series.symbol_size_scale).clamp(2.0, 50.0);

            let circle = Circle::new(Point::new(x, y), radius);
            let path = circle.into_path(0.1);

            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(fill_color),
                    stroke: Some(crate::visual::Stroke { color, width: 1.0 }),
                },
            });
        }

        elements
    }
}

impl SeriesComponent for BubbleSeriesComponent {
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

impl ChartComponent for BubbleSeriesComponent {
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
