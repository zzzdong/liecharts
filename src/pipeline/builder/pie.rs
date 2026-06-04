//! Pie Builder: 将 PieSeries 组装为 VisualElement

use vello_cpu::kurbo::{BezPath, Point};
use std::f64::consts::PI;

use crate::{
    error::Result,
    option::FontWeight,
    pipeline::builder::{fill_style, SeriesBuilder, Z_SERIES_FILL},
    pipeline::typed_series::{LabelPosition, PieSeries, RenderContext},
    visual::{Color, FillStrokeStyle, FontStyle, Stroke, TextAlign, TextBaseline, TextStyle, VisualElement},
};

pub struct PieBuilder;

impl SeriesBuilder<PieSeries> for PieBuilder {
    fn build(series: &PieSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.slices.len() * 2);

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 饼图在 bounds 中心居中
        let center_x = bounds.x0 + width * 0.5;
        // 将圆心稍微向下偏移，以平衡标题和图例占用的顶部空间
        let center_y = bounds.y0 + height * 0.55;
        let center = Point::new(center_x, center_y);

        // 半径取宽高的较小值的一半，再乘以百分比
        let min_dim = width.min(height);
        let inner_radius = min_dim * 0.5 * series.radius_inner / 100.0;
        let outer_radius = min_dim * 0.5 * series.radius_outer / 100.0;

        let mut start_angle = 0.0; // 从 12 点钟方向开始

        for slice in &series.slices {
            let sweep_angle = slice.percent * 2.0 * PI;
            let end_angle = start_angle + sweep_angle;
            let mid_angle = start_angle + sweep_angle * 0.5;

            // 绘制扇形
            let path = build_arc_path(center, inner_radius, outer_radius, start_angle, end_angle);

            elements.push(VisualElement::Path {
                path,
                style: fill_style(slice.color),
                z_index: Z_SERIES_FILL,
            });

            // 绘制标签和引导线（如果启用）
            if series.label_show {
                let label_elements = build_label(
                    center,
                    outer_radius,
                    mid_angle,
                    slice,
                    series.label_position,
                    series.label_font_size,
                    ctx,
                );
                elements.extend(label_elements);
            }

            start_angle = end_angle;
        }

        Ok(elements)
    }
}

/// 构建标签和引导线
fn build_label(
    center: Point,
    outer_radius: f64,
    mid_angle: f64,
    slice: &crate::pipeline::typed_series::PieSlice,
    position: LabelPosition,
    font_size: f64,
    ctx: &RenderContext,
) -> Vec<VisualElement> {
    let mut elements = Vec::new();

    // 将角度转换为标准坐标系
    let angle = -PI / 2.0 + mid_angle;

    // 标签文本：名称 + 百分比
    let label_text = format!("{} {:.1}%", slice.name, slice.percent * 100.0);

    match position {
        LabelPosition::Outside => {
            // 2段式引导线：
            // 第1段：从圆弧中心出发，沿角度方向延伸
            // 第2段：水平指出，到标签的左或右侧边缘中心

            let is_right_side = angle.cos() >= 0.0;

            // 第1段起点：圆弧边缘（扇形中心）
            let line_start = Point::new(
                center.x + outer_radius * angle.cos(),
                center.y + outer_radius * angle.sin(),
            );

            // 第1段终点：沿角度方向延伸一段距离
            let first_segment_len = 20.0;
            let line_kink = Point::new(
                center.x + (outer_radius + first_segment_len) * angle.cos(),
                center.y + (outer_radius + first_segment_len) * angle.sin(),
            );

            // 第2段终点：水平指出，到标签位置
            let horizontal_len = 30.0;
            let line_end = Point::new(
                if is_right_side {
                    line_kink.x + horizontal_len
                } else {
                    line_kink.x - horizontal_len
                },
                line_kink.y, // 保持同一水平线
            );

            // 绘制引导线（两段折线）
            let mut guide_path = BezPath::new();
            guide_path.move_to(line_start);
            guide_path.line_to(line_kink);
            guide_path.line_to(line_end);

            elements.push(VisualElement::Path {
                path: guide_path,
                style: FillStrokeStyle {
                    fill: None,
                    stroke: Some(Stroke {
                        color: ctx.colors.text_secondary_color,
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_FILL + 1,
            });

            // 绘制标签文本
            // 标签位于第2段终点的左侧或右侧边缘中心
            let text_x = if is_right_side {
                line_end.x + 5.0 // 右侧：文本从终点右侧开始
            } else {
                line_end.x - 5.0 // 左侧：文本从终点左侧开始
            };
            let text_y = line_end.y;

            // 根据位置设置文本对齐方式
            let (align, baseline) = if is_right_side {
                (TextAlign::Left, TextBaseline::Middle)
            } else {
                (TextAlign::Right, TextBaseline::Middle)
            };

            elements.push(VisualElement::TextRun {
                text: label_text,
                position: Point::new(text_x, text_y),
                style: TextStyle {
                    color: ctx.colors.text_color,
                    font_size,
                    font_family: "sans-serif".to_string(),
                    font_weight: FontWeight::default(),
                    font_style: FontStyle::Normal,
                    align,
                    vertical_align: baseline,
                },
                rotation: 0.0,
                max_width: None,
                layout: None,
                z_index: Z_SERIES_FILL + 2,
            });
        }
        LabelPosition::Inside => {
            // 内部标签：放在扇形中心
            let label_radius = outer_radius * 0.7;
            let label_x = center.x + label_radius * angle.cos();
            let label_y = center.y + label_radius * angle.sin();

            elements.push(VisualElement::TextRun {
                text: label_text,
                position: Point::new(label_x, label_y),
                style: TextStyle {
                    color: Color::new(255, 255, 255), // 白色文字
                    font_size,
                    font_family: "sans-serif".to_string(),
                    font_weight: FontWeight::default(),
                    font_style: FontStyle::Normal,
                    align: TextAlign::Center,
                    vertical_align: TextBaseline::Middle,
                },
                rotation: 0.0,
                max_width: None,
                layout: None,
                z_index: Z_SERIES_FILL + 2,
            });
        }
    }

    elements
}

/// 构建扇形路径（使用真正的圆弧）
fn build_arc_path(
    center: Point,
    inner_radius: f64,
    outer_radius: f64,
    start_angle: f64,
    end_angle: f64,
) -> BezPath {
    let mut path = BezPath::new();

    // 将角度转换为标准坐标系（从 -PI/2 开始，顺时针）
    let start = -PI / 2.0 + start_angle;
    let end = -PI / 2.0 + end_angle;

    // 外圆弧起点
    let outer_start = Point::new(
        center.x + outer_radius * start.cos(),
        center.y + outer_radius * start.sin(),
    );

    path.move_to(outer_start);

    // 外圆弧终点（用于计算，但不直接使用）
    let _outer_end = Point::new(
        center.x + outer_radius * end.cos(),
        center.y + outer_radius * end.sin(),
    );

    // 使用椭圆弧命令绘制外圆弧
    let large_arc = (end - start).abs() > PI;
    add_arc_eliptical(&mut path, center, outer_radius, start, end, large_arc);

    if inner_radius > 0.0 {
        // 环形饼图
        // 内圆弧终点
        let inner_end = Point::new(
            center.x + inner_radius * end.cos(),
            center.y + inner_radius * end.sin(),
        );
        path.line_to(inner_end);

        // 使用椭圆弧命令绘制内圆弧（反向）
        add_arc_eliptical(&mut path, center, inner_radius, end, start, large_arc);

        // 内圆弧起点（连接回外圆弧起点）
        let _inner_start = Point::new(
            center.x + inner_radius * start.cos(),
            center.y + inner_radius * start.sin(),
        );
        path.line_to(outer_start);
    } else {
        // 实心饼图，连接回中心
        path.line_to(center);
        path.line_to(outer_start);
    }

    path.close_path();
    path
}

/// 添加椭圆弧到路径（使用 SVG 风格的圆弧）
/// 将圆弧分割为最多 4 段，每段使用三次贝塞尔曲线精确近似
fn add_arc_eliptical(
    path: &mut BezPath,
    center: Point,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    _large_arc: bool,
) {
    // 计算角度差
    let delta_angle = end_angle - start_angle;

    // 将圆弧分割为段，每段最多 PI/2（90度）
    let num_segments = ((delta_angle.abs() / (PI / 2.0)).ceil() as usize).max(1);
    let segment_angle = delta_angle / num_segments as f64;

    for i in 0..num_segments {
        let a1 = start_angle + segment_angle * i as f64;
        let a2 = start_angle + segment_angle * (i + 1) as f64;

        // 计算这段圆弧的贝塞尔曲线控制点
        // 使用常数 k = 4/3 * tan(θ/4) 来近似圆弧
        let theta = segment_angle;
        let k = (theta.abs() / 4.0).tan() * 4.0 / 3.0;

        // 点相对于圆心的坐标
        let _p1 = Point::new(radius * a1.cos(), radius * a1.sin());
        let p2 = Point::new(radius * a2.cos(), radius * a2.sin());

        // 控制点（相对于圆心）
        let cp1 = Point::new(
            radius * (a1.cos() - k * a1.sin()),
            radius * (a1.sin() + k * a1.cos()),
        );
        let cp2 = Point::new(
            radius * (a2.cos() + k * a2.sin()),
            radius * (a2.sin() - k * a2.cos()),
        );

        // 转换为绝对坐标
        let cp1_abs = Point::new(center.x + cp1.x, center.y + cp1.y);
        let cp2_abs = Point::new(center.x + cp2.x, center.y + cp2.y);
        let p2_abs = Point::new(center.x + p2.x, center.y + p2.y);

        path.curve_to(cp1_abs, cp2_abs, p2_abs);
    }
}
