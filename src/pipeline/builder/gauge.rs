//! Gauge Builder: 将 GaugeSeries 组装为 VisualElement
//!
//! 使用真圆弧 (Arc) 绘制平滑轨道，色带采用渐变色（绿→黄→红）。

use std::f64::consts::PI;

use vello_cpu::kurbo::{Arc, BezPath, PathSeg, Point, Shape};

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LINE, stroke_style},
        typed_series::{GaugeSeries, RenderContext},
    },
    visual::{
        Color, FillStrokeStyle, GradientDef, TextAlign, TextBaseline, TextStyle, VisualElement,
        Z_LABEL,
    },
};

pub struct GaugeBuilder;

impl SeriesBuilder<GaugeSeries> for GaugeBuilder {
    fn build(series: &GaugeSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::new();

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 中心 X 在 50%，中心 Y 稍微向下偏移（55%）以平衡顶部空间
        let center_x = bounds.x0 + width * 0.5;
        let center_y = bounds.y0 + height * 0.55;
        let center = Point::new(center_x, center_y);

        // 半径取宽高的较小值的一半，再乘以百分比
        let min_dim = width.min(height);
        let radius = min_dim * 0.5 * (series.radius / 100.0);

        // 转换角度为弧度
        let start_angle = series.start_angle * PI / 180.0;
        let end_angle = series.end_angle * PI / 180.0;
        let total_sweep = end_angle - start_angle;

        // 计算数值角度
        let value_ratio = ((series.value - series.min) / (series.max - series.min)).clamp(0.0, 1.0);
        let value_angle = start_angle + total_sweep * value_ratio;

        // 色带参数
        let track_width = 18.0;
        let band_radius = radius - 10.0 + track_width / 2.0;
        let inner_r = band_radius - track_width / 2.0;
        let outer_r = band_radius + track_width / 2.0;

        // 绘制填充色带（使用平滑渐变）
        if value_angle > start_angle + 1e-6 {
            let filled_path = build_arc_ribbon(center, inner_r, outer_r, start_angle, value_angle);
            elements.push(VisualElement::GradientPath {
                path: filled_path,
                gradient: GradientDef::new(vec![
                    (0.00, Color::new(80, 180, 80)),  // 绿
                    (0.50, Color::new(255, 200, 50)), // 黄
                    (1.00, Color::new(220, 80, 80)),  // 红
                ]),
                stroke: None,
                z_index: Z_SERIES_FILL,
            });
        }

        // 绘制未填充的剩余段（淡灰色）
        if value_angle < end_angle - 1e-6 {
            let remaining_start = value_angle;
            let remaining_end = end_angle;
            let gray_path =
                build_arc_ribbon(center, inner_r, outer_r, remaining_start, remaining_end);

            elements.push(VisualElement::Path {
                path: gray_path,
                style: FillStrokeStyle {
                    fill: Some(Color::new(220, 220, 220)),
                    stroke: None,
                },
                z_index: Z_SERIES_FILL,
            });
        }

        // 绘制刻度线和标签
        let split_number = series.split_number.max(1);
        let tick_inner = outer_r + 4.0; // 刻度线起点（紧贴色带外侧）
        let tick_outer = tick_inner + 8.0; // 刻度线终点
        let label_r = tick_outer + 14.0; // 标签位置

        for i in 0..=split_number {
            let angle = start_angle + total_sweep * i as f64 / split_number as f64;

            // 刻度线
            let x1 = center.x + tick_inner * angle.cos();
            let y1 = center.y + tick_inner * angle.sin();
            let x2 = center.x + tick_outer * angle.cos();
            let y2 = center.y + tick_outer * angle.sin();

            elements.push(VisualElement::Line {
                start: Point::new(x1, y1),
                end: Point::new(x2, y2),
                style: stroke_style(Color::new(84, 85, 90), 1.5),
                z_index: Z_SERIES_LINE + 1,
            });

            // 标签
            let label_val = series.min + (series.max - series.min) * i as f64 / split_number as f64;
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
                style: TextStyle {
                    color: Color::new(84, 85, 90),
                    font_size: 11.0,
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

        // 绘制指针
        let needle_length = radius - 20.0;
        let needle_end = Point::new(
            center.x + needle_length * value_angle.cos(),
            center.y + needle_length * value_angle.sin(),
        );

        elements.push(VisualElement::Line {
            start: center,
            end: needle_end,
            style: stroke_style(series.color, 3.0),
            z_index: Z_SERIES_LINE + 1,
        });

        // 中心圆点
        elements.push(VisualElement::Circle {
            center,
            radius: 5.0,
            style: FillStrokeStyle {
                fill: Some(series.color),
                stroke: None,
            },
            z_index: Z_SERIES_LINE + 2,
        });

        // 中心数值显示
        let value_text = format!("{:.1}%", series.value);
        elements.push(VisualElement::TextRun {
            text: value_text,
            position: Point::new(center.x, center.y + 4.0),
            style: TextStyle {
                color: Color::new(60, 60, 65),
                font_size: 28.0,
                align: TextAlign::Center,
                vertical_align: TextBaseline::Middle,
                ..Default::default()
            },
            rotation: 0.0,
            max_width: None,
            layout: None,
            z_index: Z_LABEL,
        });
        // 数值单位标签
        elements.push(VisualElement::TextRun {
            text: series.name.clone(),
            position: Point::new(center.x, center.y - 28.0),
            style: TextStyle {
                color: Color::new(132, 132, 138),
                font_size: 12.0,
                align: TextAlign::Center,
                vertical_align: TextBaseline::Middle,
                ..Default::default()
            },
            rotation: 0.0,
            max_width: None,
            layout: None,
            z_index: Z_LABEL,
        });

        Ok(elements)
    }
}

/// 构建一段色带（环形扇区），使用真圆弧
fn build_arc_ribbon(center: Point, inner_r: f64, outer_r: f64, start: f64, end: f64) -> BezPath {
    let sweep = end - start;
    let mut path = BezPath::new();

    // 外圆弧起点
    let outer_start = Point::new(
        center.x + outer_r * start.cos(),
        center.y + outer_r * start.sin(),
    );
    path.move_to(outer_start);

    // 外圆弧
    add_arc(&mut path, center, outer_r, start, sweep);

    // 连接到内圆弧终点
    let inner_end = Point::new(
        center.x + inner_r * end.cos(),
        center.y + inner_r * end.sin(),
    );
    path.line_to(inner_end);

    // 内圆弧（反向）
    add_arc(&mut path, center, inner_r, end, -sweep);

    path.close_path();
    path
}

/// 使用 kurbo::Arc 添加真圆弧段到路径
fn add_arc(path: &mut BezPath, center: Point, radius: f64, start: f64, sweep: f64) {
    let arc = Arc {
        center,
        radii: (radius, radius).into(),
        start_angle: start,
        sweep_angle: sweep,
        x_rotation: 0.0,
    };

    // tolerance 越小越平滑
    arc.to_path(0.1).segments().for_each(|seg| match seg {
        PathSeg::Line(line) => path.line_to(line.p1),
        PathSeg::Quad(quad) => path.quad_to(quad.p1, quad.p2),
        PathSeg::Cubic(cubic) => path.curve_to(cubic.p1, cubic.p2, cubic.p3),
    });
}
