use vello_cpu::kurbo::{Arc, BezPath, PathSeg, Point, Shape};

use crate::{
    error::Result,
    option::{DataPoint, PieSeriesOption, SeriesOption},
    pipeline::{
        accessors::StyleAccess,
        data_processor::{DataProcessor, DataProcessorInput},
        dataframe::{DataFrame, DataValue, PieDataTransformer, Series},
    },
    visual::{
        Color, FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement,
        Z_LABEL, Z_SERIES_FILL,
    },
};

pub struct PieProcessor;

impl Default for PieProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PieProcessor {
    pub fn new() -> Self {
        Self
    }

    fn resolve_center(&self, pie: &PieSeriesOption, bounds: &vello_cpu::kurbo::Rect) -> Point {
        let default_center = vec!["50%".to_string(), "50%".to_string()];
        let center = pie.center.as_ref().unwrap_or(&default_center);

        let cx = if !center.is_empty() {
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

        let inner = if !radius.is_empty() {
            Self::parse_percent_or_value(&radius[0], max_r)
        } else {
            0.0
        };
        let outer = if radius.len() > 1 {
            Self::parse_percent_or_value(&radius[1], max_r)
        } else {
            max_r
        };

        (inner, outer)
    }

    fn parse_percent_or_value(s: &str, reference: f64) -> f64 {
        if let Some(pct) = s.strip_suffix('%') {
            pct.parse::<f64>().unwrap_or(50.0) * reference / 100.0
        } else {
            s.parse::<f64>().unwrap_or(reference * 0.5)
        }
    }

    fn build_sector_path(
        center: Point,
        inner_radius: f64,
        outer_radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> BezPath {
        let sweep_angle = end_angle - start_angle;
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
    fn to_dataframe(
        &self,
        series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        let pie = match series {
            SeriesOption::Pie(p) => p,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Pie series".into(),
                ));
            }
        };

        let mut df = DataFrame::new();

        let categories: Vec<DataValue> = pie
            .data
            .iter()
            .map(|dp| match dp {
                DataPoint::Named(name, _) => DataValue::String(name.clone()),
                _ => DataValue::String("".into()),
            })
            .collect();

        let values: Vec<DataValue> = pie
            .data
            .iter()
            .map(|dp| match dp {
                DataPoint::Value(v) => DataValue::Float(*v),
                DataPoint::Named(_, v) => DataValue::Float(*v),
                DataPoint::XY(_, y) => DataValue::Float(*y),
            })
            .collect();

        df.add_column(Series::new("category", categories));
        df.add_column(Series::new("value", values));

        Ok(df)
    }

    fn transform(&self, df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame> {
        Ok(PieDataTransformer::transform(&df, &input.colors.palette))
    }

    fn to_visual_elements(
        &self,
        df: &DataFrame,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let series = &input.option.series[input.series_idx];
        let pie = match series {
            SeriesOption::Pie(p) => p,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Pie series".into(),
                ));
            }
        };

        let bounds = input.bounds;
        let center = self.resolve_center(pie, &bounds);
        let (inner_radius, outer_radius) = self.resolve_radius(pie, &bounds);
        let style = StyleAccess::from_df(df, input.colors.get_default_color());

        let mut elements = Vec::new();
        let mut label_elements = Vec::new();

        for i in 0..df.row_count() {
            let value = df
                .get_column("value")
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            if value <= 0.0 {
                continue;
            }

            let category = df
                .get_column("category")
                .and_then(|c| c.as_string(i))
                .unwrap_or_default();
            let color = style.color(i);
            let start_angle = df
                .get_column("start_angle")
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let end_angle = df
                .get_column("end_angle")
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);

            let path =
                Self::build_sector_path(center, inner_radius, outer_radius, start_angle, end_angle);

            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: Some(Stroke {
                        color: input.colors.border_color,
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_FILL,
            });

            if pie.label.as_ref().and_then(|l| l.show).unwrap_or(false) {
                let mid_angle = (start_angle + end_angle) / 2.0;
                let is_right_side = mid_angle.cos() >= 0.0;
                let is_outside = pie
                    .label
                    .as_ref()
                    .map(|l| l.position == Some(crate::option::LabelPosition::Outside))
                    .unwrap_or(true);

                if is_outside {
                    let arc_radius = outer_radius * 1.0;
                    let extend_radius = outer_radius * 1.15;
                    let label_offset = 20.0;

                    let arc_x = center.x + arc_radius * mid_angle.cos();
                    let arc_y = center.y + arc_radius * mid_angle.sin();
                    let elbow_x = center.x + extend_radius * mid_angle.cos();
                    let elbow_y = center.y + extend_radius * mid_angle.sin();

                    elements.push(VisualElement::Line {
                        start: Point::new(arc_x, arc_y),
                        end: Point::new(elbow_x, elbow_y),
                        style: StrokeStyle::new(input.colors.axis_line_color, 1.0),
                        z_index: Z_LABEL,
                    });

                    let label_line_end_x = elbow_x
                        + if is_right_side {
                            label_offset
                        } else {
                            -label_offset
                        };
                    elements.push(VisualElement::Line {
                        start: Point::new(elbow_x, elbow_y),
                        end: Point::new(label_line_end_x, elbow_y),
                        style: StrokeStyle::new(input.colors.axis_line_color, 1.0),
                        z_index: Z_LABEL,
                    });

                    let text_x = label_line_end_x + if is_right_side { 4.0 } else { -4.0 };
                    label_elements.push(VisualElement::TextRun {
                        text: format!("{}: {}", category, value as i64),
                        position: Point::new(text_x, elbow_y),
                        style: crate::visual::TextStyle {
                            font_size: pie.label.as_ref().and_then(|l| l.font_size).unwrap_or(12.0),
                            color: pie
                                .label
                                .as_ref()
                                .and_then(|l| l.color)
                                .map(|c| Color::new(c.r, c.g, c.b))
                                .unwrap_or(input.colors.text_color),
                            align: if is_right_side {
                                TextAlign::Left
                            } else {
                                TextAlign::Right
                            },
                            vertical_align: TextBaseline::Middle,
                            ..Default::default()
                        },
                        rotation: 0.0,
                        max_width: None,
                        layout: None,
                        z_index: Z_LABEL,
                    });
                } else {
                    let label_radius = outer_radius * 0.7;
                    let lx = center.x + label_radius * mid_angle.cos();
                    let ly = center.y + label_radius * mid_angle.sin();

                    label_elements.push(VisualElement::TextRun {
                        text: category,
                        position: Point::new(lx, ly),
                        style: crate::visual::TextStyle {
                            font_size: pie.label.as_ref().and_then(|l| l.font_size).unwrap_or(12.0),
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
            }
        }

        elements.extend(label_elements);
        Ok(elements)
    }
}
