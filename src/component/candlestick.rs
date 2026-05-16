use crate::component::{ChartComponent, SeriesComponent, SeriesContext};
use crate::layout::LayoutOutput;
use crate::model::{CandlestickSeries, ResolvedOption};
use crate::visual::{Stroke, VisualElement, FillStrokeStyle};
use vello_cpu::kurbo::{BezPath, Point, Rect};

pub struct CandlestickSeriesComponent {
    series: CandlestickSeries,
    series_index: usize,
    grid_index: usize,
}

impl CandlestickSeriesComponent {
    pub fn new(series: &CandlestickSeries, series_index: usize, grid_index: usize) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index,
        }
    }

    fn build_with_context(&self, ctx: &SeriesContext) -> Vec<VisualElement> {
        let coord = ctx.coord;

        let cat_width = coord.category_width();
        let candle_width = cat_width * 0.6;
        let shadow_width = 1.5;

        let mut elements = Vec::new();

        for (idx, item) in self.series.data.iter().enumerate() {
            let x = coord.x_to_pixel(idx as f64 + 0.5);

            let high_y = coord.y_to_pixel(item.high, self.series.y_axis_index);
            let low_y = coord.y_to_pixel(item.low, self.series.y_axis_index);
            let open_y = coord.y_to_pixel(item.open, self.series.y_axis_index);
            let close_y = coord.y_to_pixel(item.close, self.series.y_axis_index);

            let is_up = item.close >= item.open;

            let (body_top, body_bottom) = if is_up {
                (close_y.min(open_y), close_y.max(open_y))
            } else {
                (open_y.min(close_y), open_y.max(close_y))
            };

            let body_color = if is_up { self.series.color_up } else { self.series.color_down };
            let border_color = if is_up {
                self.series.item_style.border_color.unwrap_or(body_color)
            } else {
                self.series.item_style.border_color0.unwrap_or(body_color)
            };

            let body_rect = Rect::new(
                x - candle_width / 2.0,
                body_top,
                x + candle_width / 2.0,
                body_bottom,
            );

            elements.push(VisualElement::Rect {
                rect: body_rect,
                style: FillStrokeStyle {
                    fill: Some(body_color),
                    stroke: Some(Stroke {
                        color: border_color,
                        width: 1.0,
                    }),
                },
            });

            let mut shadow = BezPath::new();
            shadow.move_to(Point::new(x, high_y));
            shadow.line_to(Point::new(x, body_top));
            shadow.move_to(Point::new(x, body_bottom));
            shadow.line_to(Point::new(x, low_y));

            elements.push(VisualElement::Path {
                path: shadow,
                style: FillStrokeStyle {
                    fill: None,
                    stroke: Some(Stroke {
                        color: border_color,
                        width: shadow_width,
                    }),
                },
            });
        }

        elements
    }
}

impl SeriesComponent for CandlestickSeriesComponent {
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

impl ChartComponent for CandlestickSeriesComponent {
    fn build_visual_elements(&self, resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<VisualElement> {
        let ctx = match self.create_context(resolved, layout) {
            Some(ctx) => ctx,
            None => return Vec::new(),
        };
        self.build_with_context(&ctx)
    }
}