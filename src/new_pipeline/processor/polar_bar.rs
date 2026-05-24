use std::f64::consts::PI;

use vello_cpu::kurbo::{Arc, BezPath, PathSeg, Point, Shape};

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::SeriesOption;
use crate::visual::{Color, FillStrokeStyle, Stroke, VisualElement, Z_SERIES_FILL};

pub struct PolarBarProcessor {
    series_index: usize,
}

impl PolarBarProcessor {
    pub fn new(series_index: usize) -> Self {
        Self { series_index }
    }
}

impl DataProcessor for PolarBarProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let polar_bar = match series {
            SeriesOption::PolarBar(p) => p,
            _ => return Err(crate::error::ChartError::DataError("Expected PolarBar series".into())),
        };

        let bounds = spec.bounds;
        let cx = bounds.x0 + bounds.width() / 2.0;
        let cy = bounds.y0 + bounds.height() / 2.0;
        let center = Point::new(cx, cy);
        let max_radius = bounds.width().min(bounds.height()) / 2.0 * 0.85;

        let total: f64 = polar_bar.data.iter().map(|d| {
            match d {
                crate::option::DataPoint::Value(v) => *v,
                crate::option::DataPoint::Named(_, v) => *v,
                crate::option::DataPoint::XY(_, v) => *v,
            }
        }).sum();

        if total == 0.0 {
            return Ok(SubplotVisualData {
                series_elements: Vec::new(),
                axis_elements: Vec::new(),
                grid_lines: Vec::new(),
            });
        }

        let pad_angle_deg = polar_bar.pad_angle.unwrap_or(2.0);
        let pad_angle = pad_angle_deg * PI / 180.0;
        let start_angle_deg = polar_bar.start_angle.unwrap_or(0.0);
        let mut current_angle = (start_angle_deg - 90.0) * PI / 180.0;

        let colors = &input.colors;
        let mut elements = Vec::new();

        for (i, item) in polar_bar.data.iter().enumerate() {
            let value = match item {
                crate::option::DataPoint::Value(v) => *v,
                crate::option::DataPoint::Named(_, v) => *v,
                crate::option::DataPoint::XY(_, v) => *v,
            };
            if value <= 0.0 {
                continue;
            }

            let sweep = 2.0 * PI * (value / total) - pad_angle;
            if sweep <= 0.0 {
                current_angle += 2.0 * PI * (value / total);
                continue;
            }

            let color = colors
                .series_colors
                .get(i)
                .copied()
                .unwrap_or_else(|| Color::new(128, 128, 128));

            let path = build_annular_sector(center, 0.0, max_radius, current_angle, sweep);

            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: Some(Stroke {
                        color: Color::new(255, 255, 255),
                        width: 1.5,
                    }),
                },
                z_index: Z_SERIES_FILL,
            });

            current_angle += sweep + pad_angle;
        }

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: Vec::new(),
            grid_lines: Vec::new(),
        })
    }
}

fn build_annular_sector(center: Point, inner_r: f64, outer_r: f64, start: f64, sweep: f64) -> BezPath {
    let end = start + sweep;
    let mut path = BezPath::new();

    let x1 = center.x + outer_r * start.cos();
    let y1 = center.y + outer_r * start.sin();
    path.move_to(Point::new(x1, y1));

    let outer_arc = Arc {
        center,
        radii: (outer_r, outer_r).into(),
        start_angle: start,
        sweep_angle: sweep,
        x_rotation: 0.0,
    };
    outer_arc.to_path(0.1).segments().for_each(|seg| match seg {
        PathSeg::Line(line) => path.line_to(line.p1),
        PathSeg::Quad(quad) => path.quad_to(quad.p1, quad.p2),
        PathSeg::Cubic(cubic) => path.curve_to(cubic.p1, cubic.p2, cubic.p3),
    });

    if inner_r > 0.0 {
        let x3 = center.x + inner_r * end.cos();
        let y3 = center.y + inner_r * end.sin();
        path.line_to(Point::new(x3, y3));

        let inner_arc = Arc {
            center,
            radii: (inner_r, inner_r).into(),
            start_angle: end,
            sweep_angle: -sweep,
            x_rotation: 0.0,
        };
        inner_arc.to_path(0.1).segments().for_each(|seg| match seg {
            PathSeg::Line(line) => path.line_to(line.p1),
            PathSeg::Quad(quad) => path.quad_to(quad.p1, quad.p2),
            PathSeg::Cubic(cubic) => path.curve_to(cubic.p1, cubic.p2, cubic.p3),
        });
        path.line_to(Point::new(x1, y1));
    } else {
        path.line_to(center);
        let x_start = center.x + outer_r * start.cos();
        let y_start = center.y + outer_r * start.sin();
        path.line_to(Point::new(x_start, y_start));
    }

    path.close_path();
    path
}
