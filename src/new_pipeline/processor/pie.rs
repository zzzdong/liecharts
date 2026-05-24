use vello_cpu::kurbo::{Arc, BezPath, PathSeg, Point, Shape};

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::{DataPoint, PieSeriesOption, SeriesOption};
use crate::visual::{Color, FillStrokeStyle, Stroke, TextAlign, TextBaseline, VisualElement, Z_LABEL, Z_SERIES_FILL};

pub struct PieProcessor {
    series_index: usize,
}

impl PieProcessor {
    pub fn new(series_index: usize) -> Self {
        Self { series_index }
    }

    fn parse_percent_or_value(s: &str, reference: f64) -> f64 {
        if let Some(pct) = s.strip_suffix('%') {
            pct.parse::<f64>().unwrap_or(50.0) * reference / 100.0
        } else {
            s.parse::<f64>().unwrap_or(reference * 0.5)
        }
    }

    fn resolve_center(&self, pie: &PieSeriesOption, bounds: &vello_cpu::kurbo::Rect) -> Point {
        let default_center = vec!["50%".to_string(), "50%".to_string()];
        let center = pie.center.as_ref().unwrap_or(&default_center);
        let cx = if center.len() > 0 {
            Self::parse_percent_or_value(&center[0], bounds.width())
        } else {
            bounds.width() * 0.5
        };
        let cy = if center.len() > 1 {
            Self::parse_percent_or_value(&center[1], bounds.height())
        } else {
            bounds.height() * 0.5
        };
        Point::new(bounds.x0 + cx, bounds.y0 + cy)
    }

    fn resolve_radius(&self, pie: &PieSeriesOption, bounds: &vello_cpu::kurbo::Rect) -> (f64, f64) {
        let default_radius = vec!["0%".to_string(), "75%".to_string()];
        let radius = pie.radius.as_ref().unwrap_or(&default_radius);
        let max_r = bounds.width().min(bounds.height()) * 0.5;
        let inner = if radius.len() > 0 {
            Self::parse_percent_or_value(&radius[0], max_r * 2.0)
        } else {
            0.0
        };
        let outer = if radius.len() > 1 {
            Self::parse_percent_or_value(&radius[1], max_r * 2.0)
        } else {
            max_r
        };
        (inner, outer)
    }

    fn extract_value(dp: &DataPoint) -> f64 {
        match dp {
            DataPoint::Value(v) => *v,
            DataPoint::Named(_, v) => *v,
            DataPoint::XY(_, y) => *y,
        }
    }

    fn extract_name(dp: &DataPoint) -> Option<String> {
        match dp {
            DataPoint::Named(name, _) => Some(name.clone()),
            _ => None,
        }
    }

    fn build_sector_path(
        center: Point,
        inner_radius: f64,
        outer_radius: f64,
        start_angle: f64,
        sweep_angle: f64,
    ) -> BezPath {
        let end_angle = start_angle + sweep_angle;
        let mut path = BezPath::new();

        let x1 = center.x + outer_radius * start_angle.cos();
        let y1 = center.y + outer_radius * start_angle.sin();
        path.move_to(Point::new(x1, y1));

        let arc = Arc {
            center,
            radii: (outer_radius, outer_radius).into(),
            start_angle,
            sweep_angle,
            x_rotation: 0.0,
        };
        arc.to_path(0.1).segments().for_each(|seg| match seg {
            PathSeg::Line(line) => path.line_to(line.p1),
            PathSeg::Quad(quad) => path.quad_to(quad.p1, quad.p2),
            PathSeg::Cubic(cubic) => path.curve_to(cubic.p1, cubic.p2, cubic.p3),
        });

        if inner_radius > 0.0 {
            let x3 = center.x + inner_radius * end_angle.cos();
            let y3 = center.y + inner_radius * end_angle.sin();
            path.line_to(Point::new(x3, y3));

            let inner_arc = Arc {
                center,
                radii: (inner_radius, inner_radius).into(),
                start_angle: end_angle,
                sweep_angle: -sweep_angle,
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
            path.line_to(Point::new(x1, y1));
        }
        path.close_path();
        path
    }
}

impl DataProcessor for PieProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let pie = match series {
            SeriesOption::Pie(p) => p,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Pie series".into(),
                ))
            }
        };

        let bounds = spec.bounds;
        let center = self.resolve_center(pie, &bounds);
        let (inner_radius, outer_radius) = self.resolve_radius(pie, &bounds);

        // 计算总和
        let total: f64 = pie.data.iter().map(|d| Self::extract_value(d)).sum();
        if total == 0.0 {
            return Ok(SubplotVisualData {
                series_elements: Vec::new(),
                axis_elements: Vec::new(),
                grid_lines: Vec::new(),
            });
        }

        // 分配颜色
        let colors = &input.colors;

        let mut elements = Vec::new();
        let mut label_elements = Vec::new();
        let mut current_angle = -std::f64::consts::FRAC_PI_2; // 从 12 点方向开始

        for (i, item) in pie.data.iter().enumerate() {
            let value = Self::extract_value(item);
            if value <= 0.0 {
                continue;
            }

            let sweep_angle = 2.0 * std::f64::consts::PI * (value / total);
            let mid_angle = current_angle + sweep_angle * 0.5;

            let color = colors
                .series_colors
                .get(i)
                .copied()
                .unwrap_or(Color::new(128, 128, 128));

            let path = Self::build_sector_path(center, inner_radius, outer_radius, current_angle, sweep_angle);

            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: Some(Stroke {
                        color: Color::new(255, 255, 255),
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_FILL,
            });

            // 标签（只在外半径 > 0 时添加）
            if outer_radius > 0.0 {
                let name = Self::extract_name(item)
                    .unwrap_or_else(|| format!("{}", i));
                let label_text = format!("{}", name);
                let label_radius = outer_radius * 0.7;
                let lx = center.x + label_radius * mid_angle.cos();
                let ly = center.y + label_radius * mid_angle.sin();

                label_elements.push(VisualElement::TextRun {
                    text: label_text,
                    position: Point::new(lx, ly),
                    style: crate::visual::TextStyle {
                        font_size: 12.0,
                        color,
                        align: TextAlign::Center,
                        vertical_align: TextBaseline::Middle,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                    z_index: Z_LABEL,
                });
            }

            current_angle += sweep_angle;
        }

        elements.extend(label_elements);

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: Vec::new(),
            grid_lines: Vec::new(),
        })
    }
}