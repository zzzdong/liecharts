use vello_cpu::kurbo::Point;

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::{DataPoint, SeriesOption};
use crate::visual::{Color, FillStrokeStyle, Stroke, VisualElement};

pub struct ScatterProcessor {
    series_index: usize,
}

impl ScatterProcessor {
    pub fn new(series_index: usize) -> Self {
        Self { series_index }
    }

    fn extract_xy(dp: &DataPoint) -> Option<(f64, f64)> {
        match dp {
            DataPoint::XY(x, y) => Some((*x, *y)),
            DataPoint::Named(_, v) => Some((0.0, *v)),
            DataPoint::Value(v) => Some((0.0, *v)),
        }
    }

    fn extract_name(dp: &DataPoint) -> Option<String> {
        match dp {
            DataPoint::Named(name, _) => Some(name.clone()),
            _ => None,
        }
    }
}

impl DataProcessor for ScatterProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let scatter = match series {
            SeriesOption::Scatter(s) => s,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Scatter series".into(),
                ))
            }
        };

        let bounds = spec.bounds;

        let x_axis_idx = spec.x_axis_indices.first().copied().unwrap_or(0);
        let y_axis_idx = scatter.y_axis_index.unwrap_or(0);

        let x_range = input
            .axis_ranges
            .ranges
            .iter()
            .find(|r| r.axis_index == x_axis_idx);
        let y_range = input
            .axis_ranges
            .ranges
            .iter()
            .find(|r| r.axis_index == y_axis_idx);

        let (x_min, x_max) = x_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 1.0));
        let (y_min, y_max) = y_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 100.0));

        let colors = &input.colors;
        let series_color = colors
            .series_colors
            .get(self.series_index)
            .copied()
            .unwrap_or(Color::new(100, 149, 237));

        let symbol_size = scatter.symbol_size.unwrap_or(10.0);

        let mut elements = Vec::new();

        for (_i, item) in scatter.data.iter().enumerate() {
            let (xv, yv) = match Self::extract_xy(item) {
                Some(xy) => xy,
                None => continue,
            };

            let px = bounds.x0 + (xv - x_min) / (x_max - x_min) * bounds.width();
            let py = bounds.y1 - (yv - y_min) / (y_max - y_min) * bounds.height();

            elements.push(VisualElement::Circle {
                center: Point::new(px, py),
                radius: symbol_size / 2.0,
                style: FillStrokeStyle {
                    fill: Some(series_color),
                    stroke: Some(Stroke {
                        color: Color::new(255, 255, 255),
                        width: 1.0,
                    }),
                },
            });

            // 标签
            if let Some(name) = Self::extract_name(item) {
                elements.push(VisualElement::TextRun {
                    text: name,
                    position: Point::new(px + symbol_size / 2.0 + 3.0, py),
                    style: crate::model::TextStyle {
                        font_size: 10.0,
                        color: series_color,
                        align: crate::visual::TextAlign::Left,
                        vertical_align: crate::visual::TextBaseline::Middle,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
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