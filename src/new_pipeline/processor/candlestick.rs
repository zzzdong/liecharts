use vello_cpu::kurbo::{Point, Rect};

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::{AxisType, SeriesOption};
use crate::visual::{Color, FillStrokeStyle, Stroke, VisualElement, Z_SERIES_FILL, Z_SERIES_LINE};

pub struct CandlestickProcessor {
    series_index: usize,
}

impl CandlestickProcessor {
    pub fn new(series_index: usize) -> Self {
        Self { series_index }
    }
}

impl DataProcessor for CandlestickProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let candle = match series {
            SeriesOption::Candlestick(c) => c,
            _ => return Err(crate::error::ChartError::DataError("Expected Candlestick series".into())),
        };

        let bounds = spec.bounds;
        let x_axis_idx = spec.x_axis_indices.first().copied().unwrap_or(0);
        let y_axis_idx = candle.y_axis_index.unwrap_or(0);

        let x_axis_config = input.option.x_axis.get(x_axis_idx);
        let x_range = input.axis_ranges.get_x_range(x_axis_idx);
        let y_range = input.axis_ranges.get_y_range(y_axis_idx);

        let (x_min, x_max) = x_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 1.0));
        let (y_min, y_max) = y_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 100.0));

        let is_cat_x = x_axis_config
            .and_then(|a| a.axis_type)
            .map(|t| t == AxisType::Category)
            .unwrap_or(false);

        let up_fill = Color::new(234, 85, 67);
        let down_fill = Color::new(80, 170, 94);
        let border_color = Color::new(51, 51, 51);

        let mut elements = Vec::new();

        for (i, dp) in candle.data.iter().enumerate() {
            let px = if is_cat_x {
                let cat_count = (x_max - x_min).max(1.0);
                let cat_width = bounds.width() / cat_count;
                bounds.x0 + (i as f64 + 0.5) * cat_width
            } else {
                bounds.x0 + (i as f64 + 0.5) / candle.data.len().max(1) as f64 * bounds.width()
            };

            let bar_width = if is_cat_x {
                let cat_count = (x_max - x_min).max(1.0);
                let cat_width = bounds.width() / cat_count;
                cat_width * 0.6
            } else {
                bounds.width() / candle.data.len().max(1) as f64 * 0.6
            };

            let open_y = bounds.y1 - (dp.open - y_min) / (y_max - y_min) * bounds.height();
            let close_y = bounds.y1 - (dp.close - y_min) / (y_max - y_min) * bounds.height();
            let low_y = bounds.y1 - (dp.low - y_min) / (y_max - y_min) * bounds.height();
            let high_y = bounds.y1 - (dp.high - y_min) / (y_max - y_min) * bounds.height();

            let is_up = dp.is_up();
            let fill_color = if is_up { up_fill } else { down_fill };

            let body_top = open_y.min(close_y);
            let body_height = (open_y - close_y).abs().max(1.0);

            let half_w = bar_width / 2.0;
            let line_x = px;

            elements.push(VisualElement::Line {
                start: Point::new(line_x, high_y),
                end: Point::new(line_x, low_y),
                style: crate::visual::StrokeStyle {
                    color: border_color,
                    width: 1.0,
                },
                z_index: Z_SERIES_LINE,
            });

            elements.push(VisualElement::Rect {
                rect: Rect::new(px - half_w, body_top, px + half_w, body_top + body_height),
                style: FillStrokeStyle {
                    fill: Some(fill_color),
                    stroke: Some(Stroke {
                        color: border_color,
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_FILL,
            });
        }

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: Vec::new(),
            grid_lines: Vec::new(),
        })
    }
}
