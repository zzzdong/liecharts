use std::f64::consts::PI;

use vello_cpu::kurbo::{Arc, BezPath, PathSeg, Point, Shape};

use crate::{
    error::Result,
    option::SeriesOption,
    pipeline::{
        data_processor::{DataProcessor, DataProcessorInput},
        dataframe::{DataFrame, DataValue, Series},
    },
    visual::{
        Color, FillStrokeStyle, StrokeStyle, TextAlign, TextBaseline, VisualElement, Z_AXIS,
        Z_LABEL, Z_SERIES_FILL,
    },
};

pub struct GaugeProcessor;

impl Default for GaugeProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl GaugeProcessor {
    pub fn new() -> Self {
        Self
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

    path.close_path();
    path
}

impl DataProcessor for GaugeProcessor {
    fn to_dataframe(
        &self,
        series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        let gauge = match series {
            SeriesOption::Gauge(g) => g,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Gauge series".into(),
                ));
            }
        };

        let mut df = DataFrame::new();

        // 提取数据到列
        let values: Vec<DataValue> = gauge
            .data
            .iter()
            .map(|d| DataValue::Float(d.value))
            .collect();
        let names: Vec<DataValue> = gauge
            .data
            .iter()
            .map(|d| DataValue::String(d.name.clone().unwrap_or_default()))
            .collect();

        df.add_column(Series::new("value", values));
        df.add_column(Series::new("name", names));

        Ok(df)
    }

    fn transform(&self, df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame> {
        let series = &input.option.series[input.series_idx];
        let gauge = match series {
            SeriesOption::Gauge(g) => g,
            _ => return Ok(df),
        };

        let bounds = input.bounds;
        let center = resolve_center(
            gauge
                .center
                .as_ref()
                .map(|v| v.first().map(|s| s.as_str()).unwrap_or("50%")),
            &bounds,
        );
        let radius = resolve_radius(gauge.radius.as_deref(), &bounds);

        let min_val = gauge.min.unwrap_or(0.0);
        let max_val = gauge.max.unwrap_or(100.0);
        let start_angle = gauge.start_angle.unwrap_or(-225.0) * PI / 180.0;
        let end_angle = gauge.end_angle.unwrap_or(45.0) * PI / 180.0;
        let total_sweep = end_angle - start_angle;
        let split_number = gauge.split_number.unwrap_or(10);

        let row_count = df.row_count();
        let mut df = df;

        // 添加计算列
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
            "radius",
            DataValue::Float(radius),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "min_val",
            DataValue::Float(min_val),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "max_val",
            DataValue::Float(max_val),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "start_angle",
            DataValue::Float(start_angle),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "end_angle",
            DataValue::Float(end_angle),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "total_sweep",
            DataValue::Float(total_sweep),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "split_number",
            DataValue::Integer(split_number as i64),
            row_count,
        ));

        Ok(df)
    }

    fn to_visual_elements(
        &self,
        df: &DataFrame,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let series = &input.option.series[input.series_idx];
        let gauge = match series {
            SeriesOption::Gauge(g) => g,
            _ => return Ok(Vec::new()),
        };

        let colors = &input.colors;
        let series_color = colors.get_default_color();
        let axis_color = colors.axis_label_color;

        let center_x = df
            .get_column("center_x")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(400.0);
        let center_y = df
            .get_column("center_y")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(300.0);
        let radius = df
            .get_column("radius")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(150.0);
        let min_val = df
            .get_column("min_val")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(0.0);
        let max_val = df
            .get_column("max_val")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(100.0);
        let start_angle = df
            .get_column("start_angle")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(-3.926);
        let total_sweep = df
            .get_column("total_sweep")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(4.712);
        let split_number = df
            .get_column("split_number")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(10.0) as i32;

        let center = Point::new(center_x, center_y);
        let mut elements = Vec::new();

        // 背景轨道（细色带）
        let track_width = 12.0;
        let inner_r = radius - track_width / 2.0;
        let outer_r = radius + track_width / 2.0;

        // 使用渐变色带：从绿色到黄色到红色
        let segments = 20;
        for i in 0..segments {
            let t0 = i as f64 / segments as f64;
            let t1 = (i + 1) as f64 / segments as f64;
            let seg_start = start_angle + total_sweep * t0;
            let seg_sweep = total_sweep * (t1 - t0) - 0.01;

            // 颜色渐变：绿(0.0) -> 黄(0.5) -> 红(1.0)
            let color = if t0 < 0.5 {
                // 绿到黄
                let local_t = t0 * 2.0;
                Color::new(
                    (80.0 + (255.0 - 80.0) * local_t) as u8,
                    (180.0 + (200.0 - 180.0) * local_t) as u8,
                    (80.0 + (50.0 - 80.0) * local_t) as u8,
                )
            } else {
                // 黄到红
                let local_t = (t0 - 0.5) * 2.0;
                Color::new(
                    (255.0 + (220.0 - 255.0) * local_t) as u8,
                    (200.0 + (80.0 - 200.0) * local_t) as u8,
                    (50.0 + (80.0 - 50.0) * local_t) as u8,
                )
            };

            let path = build_annular_sector(center, inner_r, outer_r, seg_start, seg_sweep);
            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: None,
                },
                z_index: Z_SERIES_FILL,
            });
        }

        // 刻度线和标签
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

        // 指针
        if let Some(dp) = gauge.data.first() {
            let ratio = ((dp.value - min_val) / (max_val - min_val)).clamp(0.0, 1.0);
            let pointer_angle = start_angle + total_sweep * ratio;
            let pointer_len = radius * 0.7;
            let tip_x = center.x + pointer_len * pointer_angle.cos();
            let tip_y = center.y + pointer_len * pointer_angle.sin();

            // 指针颜色使用系列颜色
            let pointer_color = series_color;

            elements.push(VisualElement::Line {
                start: center,
                end: Point::new(tip_x, tip_y),
                style: StrokeStyle {
                    color: pointer_color,
                    width: 2.5,
                },
                z_index: Z_AXIS,
            });

            elements.push(VisualElement::Circle {
                center,
                radius: 6.0,
                style: FillStrokeStyle {
                    fill: Some(pointer_color),
                    stroke: None,
                },
                z_index: Z_AXIS,
            });

            // 数值标签
            elements.push(VisualElement::TextRun {
                text: format!("{:.0}", dp.value),
                position: Point::new(center.x, center.y + radius * 0.4),
                style: crate::visual::TextStyle {
                    font_size: 24.0,
                    color: series_color,
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

        Ok(elements)
    }
}

fn resolve_center(center: Option<&str>, bounds: &vello_cpu::kurbo::Rect) -> Point {
    match center {
        Some(c) if c.ends_with('%') => {
            let pct = c.trim_end_matches('%').parse::<f64>().unwrap_or(50.0) / 100.0;
            Point::new(
                bounds.x0 + bounds.width() * pct,
                bounds.y0 + bounds.height() * pct,
            )
        }
        _ => Point::new(
            bounds.x0 + bounds.width() / 2.0,
            bounds.y0 + bounds.height() / 2.0,
        ),
    }
}

fn resolve_radius(radius: Option<&str>, bounds: &vello_cpu::kurbo::Rect) -> f64 {
    match radius {
        Some(r) if r.ends_with('%') => {
            let pct = r.trim_end_matches('%').parse::<f64>().unwrap_or(75.0) / 100.0;
            bounds.width().min(bounds.height()) / 2.0 * pct
        }
        _ => bounds.width().min(bounds.height()) / 2.0 * 0.75,
    }
}
