//! Gauge Builder: 将 GaugeSeries 组装为 lievisual `SceneNode`
//!
//! 使用真圆弧 (Arc) 绘制平滑轨道，色带采用渐变色（绿→黄→红）。

use std::f64::consts::PI;

use lievisual::{
    Color,
    scene::{Element, FillStrokeStyle, GradientStop, LinearGradient, SceneNode},
    text::{RichSpan, TextAlign, TextBaseline, TextStyle},
};
use vello_cpu::kurbo::{Arc, BezPath, PathSeg, Point, Shape};

use crate::{
    error::Result,
    pipeline::{
        builder::{
            SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, Z_SERIES_LINE, circle, gradient_path,
            line, path, stroke_style,
        },
        typed_series::{GaugeSeries, RenderContext},
    },
};

pub struct GaugeBuilder;

impl SeriesBuilder<GaugeSeries> for GaugeBuilder {
    fn build(series: &GaugeSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
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
        let mut end_angle = series.end_angle * PI / 180.0;

        // 修正扫过跨 0°/360° 的角度：当结束角 <= 起始角时（如 225° -> -45°），
        // 实际是顺时针扫过 360°（结束角加一圈），使 sweep 为正、跨越 0° 的弧正确。
        if end_angle <= start_angle {
            end_angle += 2.0 * PI;
        }
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
            let bbox = filled_path.bounding_box();
            let gradient = LinearGradient {
                start: Point::new(bbox.x0, bbox.y0),
                end: Point::new(bbox.x1, bbox.y1),
                stops: vec![
                    GradientStop {
                        offset: 0.00,
                        color: Color::rgb(80, 180, 80), // 绿
                    },
                    GradientStop {
                        offset: 0.50,
                        color: Color::rgb(255, 200, 50), // 黄
                    },
                    GradientStop {
                        offset: 1.00,
                        color: Color::rgb(220, 80, 80), // 红
                    },
                ],
            };
            elements.push(gradient_path(filled_path, gradient, None, Z_SERIES_FILL));
        }

        // 绘制未填充的剩余段（淡灰色）
        if value_angle < end_angle - 1e-6 {
            let remaining_start = value_angle;
            let remaining_end = end_angle;
            let gray_path =
                build_arc_ribbon(center, inner_r, outer_r, remaining_start, remaining_end);

            elements.push(path(
                gray_path,
                FillStrokeStyle {
                    fill: Some(lievisual::scene::Fill::Solid(Color::rgb(220, 220, 220))),
                    stroke: None,
                },
                true,
                Z_SERIES_FILL,
            ));
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

            elements.push(line(
                Point::new(x1, y1),
                Point::new(x2, y2),
                stroke_style(Color::rgb(84, 85, 90), 1.5),
                Z_SERIES_LINE + 1,
            ));

            // 标签
            let label_val = series.min + (series.max - series.min) * i as f64 / split_number as f64;
            let lx = center.x + label_r * angle.cos();
            let ly = center.y + label_r * angle.sin();

            let label_text = if label_val.fract() == 0.0 {
                format!("{:.0}", label_val)
            } else {
                format!("{:.1}", label_val)
            };

            let mut style = TextStyle::new(Color::rgb(84, 85, 90), 11.0, "sans-serif");
            style.align = TextAlign::Center;
            style.baseline = TextBaseline::Middle;
            elements.push(
                SceneNode::new(Element::Text {
                    spans: vec![RichSpan::new(label_text, style.clone())],
                    position: Point::new(lx, ly),
                    style,
                    layout: None,
                })
                .with_z(Z_SERIES_LABEL),
            );
        }

        // 绘制指针
        let needle_length = radius - 20.0;
        let needle_end = Point::new(
            center.x + needle_length * value_angle.cos(),
            center.y + needle_length * value_angle.sin(),
        );

        elements.push(line(
            center,
            needle_end,
            stroke_style(series.color, 3.0),
            Z_SERIES_LINE + 1,
        ));

        // 中心圆点
        elements.push(circle(
            center,
            5.0,
            FillStrokeStyle {
                fill: Some(lievisual::scene::Fill::Solid(series.color)),
                stroke: None,
            },
            Z_SERIES_LINE + 2,
        ));

        // 中心数值显示
        let value_text = format!("{:.1}%", series.value);
        let mut style = TextStyle::new(Color::rgb(60, 60, 65), 28.0, "sans-serif");
        style.align = TextAlign::Center;
        style.baseline = TextBaseline::Middle;
        elements.push(
            SceneNode::new(Element::Text {
                spans: vec![RichSpan::new(value_text, style.clone())],
                position: Point::new(center.x, center.y + 4.0),
                style,
                layout: None,
            })
            .with_z(Z_SERIES_LABEL),
        );
        // 数值单位标签
        let mut style = TextStyle::new(Color::rgb(132, 132, 138), 12.0, "sans-serif");
        style.align = TextAlign::Center;
        style.baseline = TextBaseline::Middle;
        elements.push(
            SceneNode::new(Element::Text {
                spans: vec![RichSpan::new(series.name.clone(), style.clone())],
                position: Point::new(center.x, center.y - 28.0),
                style,
                layout: None,
            })
            .with_z(Z_SERIES_LABEL),
        );

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
