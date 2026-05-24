use std::f64::consts::PI;

use vello_cpu::kurbo::{Arc, BezPath, PathSeg, Point, Shape};

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::SeriesOption;
use crate::visual::{
    Color, FillStrokeStyle, GradientDef, StrokeStyle, TextAlign, TextBaseline, VisualElement,
    Z_AXIS, Z_LABEL, Z_SERIES_FILL,
};

pub struct GaugeProcessor {
    series_index: usize,
}

impl GaugeProcessor {
    pub fn new(series_index: usize) -> Self {
        Self { series_index }
    }
}

impl DataProcessor for GaugeProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let gauge = match series {
            SeriesOption::Gauge(g) => g,
            _ => return Err(crate::error::ChartError::DataError("Expected Gauge series".into())),
        };

        let bounds = spec.bounds;
        let center = resolve_center(gauge.center.as_deref(), &bounds);
        let radius = resolve_radius(gauge.radius.as_deref(), &bounds);

        let min_val = gauge.min.unwrap_or(0.0);
        let max_val = gauge.max.unwrap_or(100.0);
        let start_angle = gauge.start_angle.unwrap_or(-225.0) * PI / 180.0;
        let end_angle = gauge.end_angle.unwrap_or(45.0) * PI / 180.0;
        let total_sweep = end_angle - start_angle;
        let split_number = gauge.split_number.unwrap_or(10);

        let axis_color = Color::new(84, 85, 90);
        let colors = &input.colors;
        let series_color = colors
            .series_colors
            .get(self.series_index)
            .copied()
            .unwrap_or(Color::new(80, 112, 221));

        let mut elements = Vec::new();

        let bg_arc = Arc {
            center,
            radii: (radius, radius).into(),
            start_angle,
            sweep_angle: total_sweep,
            x_rotation: 0.0,
        };
        let mut bg_path = BezPath::new();
        let first_seg = bg_arc.to_path(0.1);
        if let Some(seg) = first_seg.segments().next() {
            match seg {
                PathSeg::Line(line) => bg_path.move_to(line.p0),
                PathSeg::Quad(quad) => bg_path.move_to(quad.p0),
                PathSeg::Cubic(cubic) => bg_path.move_to(cubic.p0),
            }
        }
        bg_arc.to_path(0.1).segments().for_each(|seg| match seg {
            PathSeg::Line(line) => bg_path.line_to(line.p1),
            PathSeg::Quad(quad) => bg_path.quad_to(quad.p1, quad.p2),
            PathSeg::Cubic(cubic) => bg_path.curve_to(cubic.p1, cubic.p2, cubic.p3),
        });
        elements.push(VisualElement::GradientPath {
            path: bg_path,
            gradient: GradientDef::new(vec![
                (0.0, Color::new(200, 220, 240)),
                (1.0, series_color),
            ]),
            stroke: None,
            z_index: Z_SERIES_FILL,
        });

        let tick_inner = radius - 8.0;
        let tick_outer = radius;
        for i in 0..=split_number {
            let angle = start_angle + total_sweep * i as f64 / split_number as f64;
            let x1 = center.x + tick_inner * angle.cos();
            let y1 = center.y + tick_inner * angle.sin();
            let x2 = center.x + tick_outer * angle.cos();
            let y2 = center.y + tick_outer * angle.sin();
            elements.push(VisualElement::Line {
                start: Point::new(x1, y1),
                end: Point::new(x2, y2),
                style: StrokeStyle {
                    color: axis_color,
                    width: 1.5,
                },
                z_index: Z_AXIS,
            });

            let label_val = min_val + (max_val - min_val) * i as f64 / split_number as f64;
            let label_r = radius - 22.0;
            let lx = center.x + label_r * angle.cos();
            let ly = center.y + label_r * angle.sin();
            let label_text = if label_val.fract() == 0.0 {
                format!("{:.0}", label_val)
            } else {
                format!("{:.1}", label_val)
            };
            elements.push(VisualElement::TextRun {
                text: label_text,
                position: Point::new(lx, ly),
                style: crate::visual::TextStyle {
                    font_size: 10.0,
                    color: axis_color,
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

        if let Some(dp) = gauge.data.first() {
            let ratio = ((dp.value - min_val) / (max_val - min_val)).clamp(0.0, 1.0);
            let pointer_angle = start_angle + total_sweep * ratio;
            let pointer_len = radius * 0.7;
            let tip_x = center.x + pointer_len * pointer_angle.cos();
            let tip_y = center.y + pointer_len * pointer_angle.sin();

            elements.push(VisualElement::Line {
                start: center,
                end: Point::new(tip_x, tip_y),
                style: StrokeStyle {
                    color: Color::new(200, 50, 50),
                    width: 2.5,
                },
                z_index: Z_AXIS,
            });

            elements.push(VisualElement::Circle {
                center,
                radius: 6.0,
                style: FillStrokeStyle {
                    fill: Some(Color::new(200, 50, 50)),
                    stroke: None,
                },
                z_index: Z_AXIS,
            });

            let value_text = if dp.value.fract() == 0.0 {
                format!("{:.0}", dp.value)
            } else {
                format!("{:.1}", dp.value)
            };
            let name_text = dp.name.as_deref().unwrap_or("");
            let title_y = center.y + radius * 0.4;

            if !name_text.is_empty() {
                elements.push(VisualElement::TextRun {
                    text: name_text.to_string(),
                    position: Point::new(center.x, title_y),
                    style: crate::visual::TextStyle {
                        font_size: 12.0,
                        color: Color::new(100, 100, 100),
                        align: TextAlign::Center,
                        vertical_align: TextBaseline::Top,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                    z_index: Z_LABEL,
                });
            }

            elements.push(VisualElement::TextRun {
                text: value_text,
                position: Point::new(center.x, title_y + 16.0),
                style: crate::visual::TextStyle {
                    font_size: 20.0,
                    color: series_color,
                    align: TextAlign::Center,
                    vertical_align: TextBaseline::Top,
                    ..Default::default()
                },
                rotation: 0.0,
                max_width: None,
                layout: None,
                z_index: Z_LABEL,
            });
        }

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: Vec::new(),
            grid_lines: Vec::new(),
        })
    }
}

fn resolve_center(center: Option<&[String]>, bounds: &vello_cpu::kurbo::Rect) -> Point {
    let default = ["50%".to_string(), "55%".to_string()];
    let c = center.unwrap_or(&default);
    let cx = parse_pct(c.first().unwrap_or(&"50%".to_string()), bounds.width());
    let cy = parse_pct(c.get(1).unwrap_or(&"55%".to_string()), bounds.height());
    Point::new(bounds.x0 + cx, bounds.y0 + cy)
}

fn resolve_radius(radius: Option<&str>, bounds: &vello_cpu::kurbo::Rect) -> f64 {
    let max_r = bounds.width().min(bounds.height()) * 0.5;
    parse_pct(radius.unwrap_or("75%"), max_r * 2.0)
}

fn parse_pct(s: &str, reference: f64) -> f64 {
    if let Some(pct) = s.strip_suffix('%') {
        pct.parse::<f64>().unwrap_or(50.0) * reference / 100.0
    } else {
        s.parse::<f64>().unwrap_or(reference * 0.5)
    }
}
