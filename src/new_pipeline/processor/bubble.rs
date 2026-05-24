use vello_cpu::kurbo::Point;

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::{BubbleDataPoint, SeriesOption};
use crate::visual::{Color, FillStrokeStyle, Stroke, VisualElement, Z_LABEL, Z_SERIES_POINT};

pub struct BubbleProcessor {
    series_index: usize,
}

impl BubbleProcessor {
    pub fn new(series_index: usize) -> Self {
        Self { series_index }
    }
}

impl DataProcessor for BubbleProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let bubble = match series {
            SeriesOption::Bubble(b) => b,
            _ => return Err(crate::error::ChartError::DataError("Expected Bubble series".into())),
        };

        let bounds = spec.bounds;
        let x_axis_idx = spec.x_axis_indices.first().copied().unwrap_or(0);
        let y_axis_idx = bubble.y_axis_index.unwrap_or(0);

        let x_range = input.axis_ranges.get_x_range(x_axis_idx);
        let y_range = input.axis_ranges.get_y_range(y_axis_idx);

        let (x_min, x_max) = x_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 1.0));
        let (y_min, y_max) = y_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 100.0));

        let colors = &input.colors;
        let series_color = colors
            .series_colors
            .get(self.series_index)
            .copied()
            .unwrap_or(Color::new(100, 149, 237));

        let scale = bubble.symbol_size_scale.unwrap_or(1.0);

        let mut elements = Vec::new();

        for item in &bubble.data {
            let BubbleDataPoint { x, y, size, name } = item;

            let px = bounds.x0 + (x - x_min) / (x_max - x_min) * bounds.width();
            let py = bounds.y1 - (y - y_min) / (y_max - y_min) * bounds.height();
            let radius = size.unwrap_or(20.0).sqrt() * scale;

            elements.push(VisualElement::Circle {
                center: Point::new(px, py),
                radius,
                style: FillStrokeStyle {
                    fill: Some(Color::new(series_color.r, series_color.g, series_color.b).set_alpha(0.7)),
                    stroke: Some(Stroke {
                        color: Color::new(255, 255, 255),
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_POINT,
            });

            if let Some(n) = name {
                elements.push(VisualElement::TextRun {
                    text: n.clone(),
                    position: Point::new(px, py),
                    style: crate::visual::TextStyle {
                        font_size: 10.0,
                        color: Color::new(51, 51, 51),
                        align: crate::visual::TextAlign::Center,
                        vertical_align: crate::visual::TextBaseline::Middle,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                    z_index: Z_LABEL,
                });
            }
        }

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: Vec::new(),
            grid_lines: Vec::new(),
        })
    }
}
