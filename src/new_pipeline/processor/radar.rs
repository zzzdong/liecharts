use std::f64::consts::PI;

use vello_cpu::kurbo::{BezPath, Point};

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::{SeriesOption, RadarOption};
use crate::visual::{
    Color, FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement,
    Z_AXIS, Z_LABEL, Z_SERIES_FILL, Z_SERIES_POINT,
};

pub struct RadarProcessor {
    series_index: usize,
}

impl RadarProcessor {
    pub fn new(series_index: usize) -> Self {
        Self { series_index }
    }
}

impl DataProcessor for RadarProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let radar = match series {
            SeriesOption::Radar(r) => r,
            _ => return Err(crate::error::ChartError::DataError("Expected Radar series".into())),
        };

        let radar_cfg = input.option.radar.as_ref();
        let bounds = spec.bounds;
        let center = resolve_center(radar_cfg, &bounds);
        let (_, outer_radius) = resolve_radius(radar_cfg, &bounds);
        let indicators = radar_cfg
            .and_then(|r| r.indicator.as_ref())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let split_number = radar_cfg
            .and_then(|r| r.split_number)
            .unwrap_or(5);

        let num_axes = indicators.len();
        if num_axes < 3 {
            return Ok(SubplotVisualData {
                series_elements: Vec::new(),
                axis_elements: Vec::new(),
                grid_lines: Vec::new(),
            });
        }

        let colors = &input.colors;
        let mut elements = Vec::new();
        let mut grid_elements = Vec::new();

        let is_first_radar = spec.series_indices.iter().copied().find(|&idx| {
            matches!(&input.option.series[idx], SeriesOption::Radar(_))
        }) == Some(self.series_index);

        if is_first_radar {
            let axis_color = colors.axis_line_color;
            let grid_color = colors.grid_line_color;

            for level in 1..=split_number {
                let r = outer_radius * level as f64 / split_number as f64;
                let path = polygon_path(center, r, num_axes);
                grid_elements.push(VisualElement::Path {
                    path,
                    style: FillStrokeStyle {
                        fill: None,
                        stroke: Some(Stroke {
                            color: grid_color,
                            width: 0.5,
                        }),
                    },
                    z_index: Z_SERIES_FILL,
                });
            }

            for i in 0..num_axes {
                let angle = axis_angle(i, num_axes);
                let ex = center.x + outer_radius * angle.cos();
                let ey = center.y + outer_radius * angle.sin();
                grid_elements.push(VisualElement::Line {
                    start: center,
                    end: Point::new(ex, ey),
                    style: StrokeStyle {
                        color: axis_color,
                        width: 0.5,
                    },
                    z_index: Z_AXIS,
                });
            }

            for (i, ind) in indicators.iter().enumerate() {
                if let Some(name) = &ind.name {
                    let angle = axis_angle(i, num_axes);
                    let label_r = outer_radius + 14.0;
                    let lx = center.x + label_r * angle.cos();
                    let ly = center.y + label_r * angle.sin();
                    grid_elements.push(VisualElement::TextRun {
                        text: name.clone(),
                        position: Point::new(lx, ly),
                        style: crate::visual::TextStyle {
                            font_size: 11.0,
                            color: colors.axis_label_color,
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
            }
        }

        let series_color = colors
            .series_colors
            .get(self.series_index)
            .copied()
            .unwrap_or(Color::new(100, 149, 237));

        for data in &radar.data {
            if data.value.len() != num_axes {
                continue;
            }

            let mut points = Vec::new();
            for (i, &val) in data.value.iter().enumerate() {
                let max_val = indicators[i].max.unwrap_or(100.0);
                let ratio = if max_val > 0.0 { val / max_val } else { 0.0 };
                let r = outer_radius * ratio.clamp(0.0, 1.0);
                let angle = axis_angle(i, num_axes);
                points.push(Point::new(
                    center.x + r * angle.cos(),
                    center.y + r * angle.sin(),
                ));
            }

            let mut path = BezPath::new();
            if let Some(first) = points.first() {
                path.move_to(*first);
                for p in &points[1..] {
                    path.line_to(*p);
                }
                path.close_path();
            }

            let alpha_fill = series_color.set_alpha(0.3);
            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(alpha_fill),
                    stroke: Some(Stroke {
                        color: series_color,
                        width: 2.0,
                    }),
                },
                z_index: Z_SERIES_FILL,
            });

            for pt in &points {
                elements.push(VisualElement::Circle {
                    center: *pt,
                    radius: 4.0,
                    style: FillStrokeStyle {
                        fill: Some(Color::new(255, 255, 255)),
                        stroke: Some(Stroke {
                            color: series_color,
                            width: 2.0,
                        }),
                    },
                    z_index: Z_SERIES_POINT,
                });
            }
        }

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: grid_elements,
            grid_lines: Vec::new(),
        })
    }
}

fn axis_angle(i: usize, n: usize) -> f64 {
    -PI / 2.0 + 2.0 * PI * i as f64 / n as f64
}

fn polygon_path(center: Point, radius: f64, n: usize) -> BezPath {
    let mut path = BezPath::new();
    for i in 0..n {
        let angle = axis_angle(i, n);
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        if i == 0 {
            path.move_to(Point::new(x, y));
        } else {
            path.line_to(Point::new(x, y));
        }
    }
    path.close_path();
    path
}

fn resolve_center(radar: Option<&RadarOption>, bounds: &vello_cpu::kurbo::Rect) -> Point {
    let default = vec!["50%".to_string(), "55%".to_string()];
    let center = radar.and_then(|r| r.center.as_ref()).unwrap_or(&default);
    let cx = parse_pct(center.first().unwrap_or(&"50%".to_string()), bounds.width());
    let cy = parse_pct(center.get(1).unwrap_or(&"55%".to_string()), bounds.height());
    Point::new(bounds.x0 + cx, bounds.y0 + cy)
}

fn resolve_radius(radar: Option<&RadarOption>, bounds: &vello_cpu::kurbo::Rect) -> (f64, f64) {
    let default = vec!["0%".to_string(), "65%".to_string()];
    let radius = radar.and_then(|r| r.radius.as_ref()).unwrap_or(&default);
    let max_r = bounds.width().min(bounds.height()) * 0.5;
    let inner = radius.first().map(|s| parse_pct(s, max_r * 2.0)).unwrap_or(0.0);
    let outer = radius.get(1).map(|s| parse_pct(s, max_r * 2.0)).unwrap_or(max_r);
    (inner, outer)
}

fn parse_pct(s: &str, reference: f64) -> f64 {
    if let Some(pct) = s.strip_suffix('%') {
        pct.parse::<f64>().unwrap_or(50.0) * reference / 100.0
    } else {
        s.parse::<f64>().unwrap_or(reference * 0.5)
    }
}
