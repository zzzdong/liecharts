use std::f64::consts::PI;

use vello_cpu::kurbo::{Arc, BezPath, PathSeg, Point, Shape};

use crate::{
    error::Result,
    option::SeriesOption,
    pipeline::{
        data_processor::{DataProcessor, DataProcessorInput},
        dataframe::{DataFrame, DataValue, Series},
        mapper::{CoordinateMapper, PolarMapper},
    },
    text::create_text_layout,
    visual::{
        FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement, Z_AXIS,
        Z_LABEL, Z_SERIES_FILL,
    },
};

pub struct PolarBarProcessor;

impl PolarBarProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl DataProcessor for PolarBarProcessor {
    fn to_dataframe(
        &self,
        series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        let polar_bar = match series {
            SeriesOption::PolarBar(p) => p,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected PolarBar series".into(),
                ));
            }
        };

        let mut df = DataFrame::new();

        let values: Vec<DataValue> = polar_bar
            .data
            .iter()
            .map(|d| {
                let v = match d {
                    crate::option::DataPoint::Value(v) => *v,
                    crate::option::DataPoint::Named(_, v) => *v,
                    crate::option::DataPoint::XY(_, v) => *v,
                };
                DataValue::Float(v)
            })
            .collect();

        df.add_column(Series::new("value", values));

        Ok(df)
    }

    fn transform(&self, df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame> {
        let series = &input.option.series[input.series_idx];
        let polar_bar = match series {
            SeriesOption::PolarBar(p) => p,
            _ => return Ok(df),
        };

        let max_value: f64 = polar_bar
            .data
            .iter()
            .map(|d| match d {
                crate::option::DataPoint::Value(v) => *v,
                crate::option::DataPoint::Named(_, v) => *v,
                crate::option::DataPoint::XY(_, v) => *v,
            })
            .fold(0.0, |max, v| v.max(max));

        let data_count = polar_bar.data.len().max(1);
        let pad_angle_deg = polar_bar.pad_angle.unwrap_or(2.0);
        let start_angle_deg = polar_bar.start_angle.unwrap_or(0.0);

        let row_count = df.row_count();
        let mut df = df;

        df.add_column(Series::new_constant(
            "max_value",
            DataValue::Float(max_value),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "data_count",
            DataValue::Float(data_count as f64),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "pad_angle_deg",
            DataValue::Float(pad_angle_deg),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "start_angle_deg",
            DataValue::Float(start_angle_deg),
            row_count,
        ));

        Ok(df)
    }

    fn mapper(&self) -> Box<dyn CoordinateMapper> {
        Box::new(PolarMapper::new(0.85))
    }

    fn to_visual_elements(
        &self,
        df: &DataFrame,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let series = &input.option.series[input.series_idx];
        let polar_bar = match series {
            SeriesOption::PolarBar(p) => p,
            _ => return Ok(Vec::new()),
        };

        let colors = &input.colors;

        let cx = df
            .get_column("center_x")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(400.0);
        let cy = df
            .get_column("center_y")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(300.0);
        let max_radius = df
            .get_column("max_radius")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(150.0);
        let max_value = df
            .get_column("max_value")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(0.0);
        let data_count = df
            .get_column("data_count")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(1.0) as usize;
        let pad_angle_deg = df
            .get_column("pad_angle_deg")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(2.0);
        let start_angle_deg = df
            .get_column("start_angle_deg")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(0.0);

        if max_value <= 0.0 || data_count == 0 {
            return Ok(Vec::new());
        }

        let center = Point::new(cx, cy);
        let pad_angle = pad_angle_deg * PI / 180.0;
        let sweep_per_bar = (2.0 * PI) / data_count as f64 - pad_angle;
        let mut current_angle = (start_angle_deg - 90.0) * PI / 180.0;

        let mut elements = Vec::new();
        let mut bar_info: Vec<(f64, f64, usize)> = Vec::new();

        for (i, item) in polar_bar.data.iter().enumerate() {
            let value = match item {
                crate::option::DataPoint::Value(v) => *v,
                crate::option::DataPoint::Named(_, v) => *v,
                crate::option::DataPoint::XY(_, v) => *v,
            };
            if value <= 0.0 {
                current_angle += sweep_per_bar + pad_angle;
                continue;
            }

            bar_info.push((current_angle + sweep_per_bar / 2.0, value, i));

            let radius = max_radius * (value / max_value);
            let color = colors.get_data_color(i);

            let path =
                build_annular_sector(center, 0.0, radius, current_angle, sweep_per_bar.max(0.01));

            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: Some(Stroke {
                        color: colors.border_color,
                        width: 1.5,
                    }),
                },
                z_index: Z_SERIES_FILL,
            });

            current_angle += sweep_per_bar + pad_angle;
        }

        let first_polar_idx = (0..input.option.series.len())
            .find(|&idx| matches!(&input.option.series[idx], SeriesOption::PolarBar(_)));
        let is_first_polar = first_polar_idx == Some(input.series_idx);

        if is_first_polar {
            let grid_levels = 5;
            for level in 1..=grid_levels {
                let r = max_radius * level as f64 / grid_levels as f64;
                elements.push(VisualElement::Circle {
                    center,
                    radius: r,
                    style: FillStrokeStyle {
                        fill: None,
                        stroke: Some(Stroke {
                            color: colors.grid_line_color,
                            width: 1.0,
                        }),
                    },
                    z_index: Z_AXIS,
                });

                let label_value = max_value * level as f64 / grid_levels as f64;
                let label_text = if label_value >= 1000.0 {
                    format!("{:.0}", label_value)
                } else if label_value >= 10.0 {
                    format!("{:.1}", label_value)
                } else {
                    format!("{:.2}", label_value)
                };
                elements.push(VisualElement::TextRun {
                    text: label_text,
                    position: Point::new(center.x + r + 4.0, center.y),
                    style: crate::visual::TextStyle {
                        font_size: 10.0,
                        color: colors.axis_label_color,
                        align: TextAlign::Left,
                        vertical_align: TextBaseline::Middle,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                    z_index: Z_LABEL,
                });
            }

            for (bar_angle, _bar_value, bar_idx) in &bar_info {
                let angle = *bar_angle;
                let end_x = center.x + max_radius * angle.cos();
                let end_y = center.y + max_radius * angle.sin();

                elements.push(VisualElement::Line {
                    start: center,
                    end: Point::new(end_x, end_y),
                    style: StrokeStyle::new(colors.grid_line_color, 1.0),
                    z_index: Z_AXIS,
                });

                let label_text = input
                    .option
                    .legend
                    .as_ref()
                    .and_then(|legend| legend.data.as_ref())
                    .and_then(|legend_data| legend_data.get(*bar_idx))
                    .cloned()
                    .unwrap_or_else(|| (bar_idx + 1).to_string());

                let label_r = max_radius + 18.0;
                let label_x = center.x + label_r * angle.cos();
                let label_y = center.y + label_r * angle.sin();

                let text_layout = create_text_layout(
                    &label_text,
                    &crate::visual::TextStyle {
                        font_size: 11.0,
                        color: colors.axis_label_color,
                        align: TextAlign::Center,
                        ..Default::default()
                    },
                    None,
                );

                elements.push(VisualElement::TextRun {
                    text: label_text,
                    position: Point::new(label_x - text_layout.width() as f64 / 2.0, label_y),
                    style: crate::visual::TextStyle {
                        font_size: 11.0,
                        color: colors.axis_label_color,
                        align: TextAlign::Left,
                        vertical_align: TextBaseline::Middle,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: Some(text_layout),
                    z_index: Z_LABEL,
                });
            }
        }

        Ok(elements)
    }
}

fn build_annular_sector(
    center: Point,
    inner_r: f64,
    outer_r: f64,
    start: f64,
    sweep: f64,
) -> BezPath {
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
        path.line_to(Point::new(x1, y1));
    }

    path.close_path();
    path
}
