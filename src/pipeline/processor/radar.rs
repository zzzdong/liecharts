use std::f64::consts::PI;

use vello_cpu::kurbo::{BezPath, Point};

use crate::{
    error::Result,
    option::{RadarOption, SeriesOption},
    pipeline::{
        data_processor::{DataProcessor, DataProcessorInput},
        dataframe::{DataFrame, DataValue, Series},
    },
    visual::{
        FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement, Z_AXIS,
        Z_LABEL, Z_SERIES_FILL, Z_SERIES_POINT,
    },
};

pub struct RadarProcessor;

impl RadarProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl DataProcessor for RadarProcessor {
    fn to_dataframe(
        &self,
        series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        let radar = match series {
            SeriesOption::Radar(r) => r,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Radar series".into(),
                ));
            }
        };

        let mut df = DataFrame::new();

        let names: Vec<DataValue> = radar
            .data
            .iter()
            .map(|d| DataValue::String(d.name.clone().unwrap_or_default()))
            .collect();
        df.add_column(Series::new("name", names));

        let values: Vec<DataValue> = radar
            .data
            .iter()
            .map(|d| {
                let val_str = d
                    .value
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                DataValue::String(val_str)
            })
            .collect();
        df.add_column(Series::new("value", values));

        Ok(df)
    }

    fn transform(&self, df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame> {
        let series_name = df
            .get_column("name")
            .and_then(|c| c.as_string(0))
            .unwrap_or_default();

        let series_idx = input
            .option
            .series
            .iter()
            .position(|s| {
                if let SeriesOption::Radar(r) = s {
                    r.data.first().and_then(|d| d.name.as_ref()) == Some(&series_name)
                } else {
                    false
                }
            })
            .unwrap_or(0);

        let radar_cfg = input.option.radar.as_ref();
        let bounds = input.bounds;

        let center = resolve_center(radar_cfg, &bounds);
        let (_, outer_radius) = resolve_radius(radar_cfg, &bounds);

        let num_axes = radar_cfg
            .and_then(|r| r.indicator.as_ref())
            .map(|v| v.len())
            .unwrap_or(0);
        let split_number = radar_cfg.and_then(|r| r.split_number).unwrap_or(5);

        let row_count = df.row_count();
        let mut df = df;

        df.add_column(Series::new_constant(
            "center_x",
            DataValue::Float(center.x),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "center_y",
            DataValue::Float(center.y),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "outer_radius",
            DataValue::Float(outer_radius),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "num_axes",
            DataValue::Integer(num_axes as i64),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "split_number",
            DataValue::Integer(split_number as i64),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "series_idx",
            DataValue::Integer(series_idx as i64),
            row_count,
        ));

        Ok(df)
    }

    fn to_visual_elements(
        &self,
        df: &DataFrame,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let series_idx = df
            .get_column("series_idx")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(0.0) as usize;

        let series = &input.option.series[series_idx];
        let radar = match series {
            SeriesOption::Radar(r) => r,
            _ => return Ok(Vec::new()),
        };

        let radar_cfg = input.option.radar.as_ref();
        let colors = &input.colors;

        let center_x = df
            .get_column("center_x")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(400.0);
        let center_y = df
            .get_column("center_y")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(300.0);
        let outer_radius = df
            .get_column("outer_radius")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(150.0);
        let num_axes = df
            .get_column("num_axes")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(0.0) as usize;
        let split_number = df
            .get_column("split_number")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(5.0) as i32;

        if num_axes < 3 {
            return Ok(Vec::new());
        }

        let indicators = radar_cfg
            .and_then(|r| r.indicator.as_ref())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let center = Point::new(center_x, center_y);
        let mut elements = Vec::new();

        let first_radar_idx = (0..input.option.series.len())
            .find(|&idx| matches!(&input.option.series[idx], SeriesOption::Radar(_)));
        let is_first_radar = first_radar_idx == Some(series_idx);

        if is_first_radar {
            let axis_color = colors.axis_line_color;
            let grid_color = colors.grid_line_color;

            for level in 1..=split_number {
                let r = outer_radius * level as f64 / split_number as f64;
                let path = polygon_path(center, r, num_axes);
                elements.push(VisualElement::Path {
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
                elements.push(VisualElement::Line {
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
                    elements.push(VisualElement::TextRun {
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

        let color_idx = (0..input.option.series.len())
            .filter(|&idx| matches!(&input.option.series[idx], SeriesOption::Radar(_)))
            .position(|idx| idx == series_idx)
            .unwrap_or(0);

        let series_color = colors.get_series_color(color_idx);

        for data in &radar.data {
            if data.value.len() != num_axes {
                continue;
            }

            let mut points = Vec::new();
            for (i, &val) in data.value.iter().enumerate() {
                let max_val = indicators.get(i).and_then(|ind| ind.max).unwrap_or(100.0);
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
                        fill: Some(input.colors.border_color),
                        stroke: Some(Stroke {
                            color: series_color,
                            width: 2.0,
                        }),
                    },
                    z_index: Z_SERIES_POINT,
                });
            }
        }

        Ok(elements)
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
    let inner = radius.first().map(|s| parse_pct(s, max_r)).unwrap_or(0.0);
    let outer = radius
        .get(1)
        .map(|s| parse_pct(s, max_r))
        .unwrap_or(max_r * 0.65);
    (inner, outer)
}

fn parse_pct(s: &str, reference: f64) -> f64 {
    if let Some(pct) = s.strip_suffix('%') {
        pct.parse::<f64>().unwrap_or(50.0) * reference / 100.0
    } else {
        s.parse::<f64>().unwrap_or(reference * 0.5)
    }
}
