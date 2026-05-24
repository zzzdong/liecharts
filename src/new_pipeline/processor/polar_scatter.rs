use std::f64::consts::PI;

use vello_cpu::kurbo::Point;

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::SeriesOption;
use crate::visual::{Color, FillStrokeStyle, Stroke, VisualElement, Z_SERIES_POINT};

pub struct PolarScatterProcessor {
    series_index: usize,
}

impl PolarScatterProcessor {
    pub fn new(series_index: usize) -> Self {
        Self { series_index }
    }
}

impl DataProcessor for PolarScatterProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let polar_scatter = match series {
            SeriesOption::PolarScatter(p) => p,
            _ => return Err(crate::error::ChartError::DataError("Expected PolarScatter series".into())),
        };

        let bounds = spec.bounds;
        let cx = bounds.x0 + bounds.width() / 2.0;
        let cy = bounds.y0 + bounds.height() / 2.0;
        let center = Point::new(cx, cy);
        let max_radius = bounds.width().min(bounds.height()) / 2.0 * 0.8;

        let max_data_radius = polar_scatter
            .data
            .iter()
            .map(|d| d.radius)
            .fold(0.0_f64, f64::max)
            .max(1.0);

        let colors = &input.colors;
        let series_color = colors
            .series_colors
            .get(self.series_index)
            .copied()
            .unwrap_or(Color::new(100, 149, 237));

        let default_symbol_size = polar_scatter.symbol_size.unwrap_or(10.0);

        let mut elements = Vec::new();

        for dp in &polar_scatter.data {
            let angle_rad = dp.angle * PI / 180.0 - PI / 2.0;
            let r_ratio = dp.radius / max_data_radius;
            let r = max_radius * r_ratio;

            let px = center.x + r * angle_rad.cos();
            let py = center.y + r * angle_rad.sin();

            let symbol_size = dp.symbol_size.unwrap_or(default_symbol_size);
            let radius = symbol_size / 2.0;

            elements.push(VisualElement::Circle {
                center: Point::new(px, py),
                radius,
                style: FillStrokeStyle {
                    fill: Some(series_color.set_alpha(0.7)),
                    stroke: Some(Stroke {
                        color: Color::new(255, 255, 255),
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_POINT,
            });
        }

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: Vec::new(),
            grid_lines: Vec::new(),
        })
    }
}
