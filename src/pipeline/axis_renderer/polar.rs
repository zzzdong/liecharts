//! 极坐标轴渲染器
//!
//! 生成极坐标图表专用的同心圆网格线和径向射线，以及半径标签。
//! 与笛卡尔坐标轴没有共享逻辑，完全独立的几何体系。

use std::f64::consts::PI;

use vello_cpu::kurbo::{Circle, Point, Shape as KurboShape};

use crate::{
    pipeline::types::{ColorContext, SubplotSpec, TextMeasurer},
    visual::{
        Color, FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, TextStyle,
        VisualElement, Z_GRID, Z_LABEL,
    },
};

/// 极坐标轴渲染器
///
/// 渲染极坐标图表专用的：
/// - 同心圆网格（多层）
/// - 从中心向外辐射的角度射线
/// - 半径刻度标签
pub struct PolarAxisRenderer;

impl PolarAxisRenderer {
    /// 渲染极坐标网格（同心圆和射线）
    pub fn render(
        subplot: &SubplotSpec,
        colors: &ColorContext,
        _text_measurer: &mut TextMeasurer,
    ) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        let bounds = subplot.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 中心点: X 50%, Y 55%
        let center_x = bounds.x0 + width * 0.5;
        let center_y = bounds.y0 + height * 0.55;
        let center = Point::new(center_x, center_y);

        let min_dim = width.min(height);
        let radius = min_dim * 0.5 * 0.75;

        // 绘制同心圆网格（5层）
        let grid_levels = 5;
        for level in 1..=grid_levels {
            let level_radius = radius * level as f64 / grid_levels as f64;

            let circle = Circle::new(center, level_radius);
            let circle_path = circle.to_path(0.1);

            elements.push(VisualElement::Path {
                path: circle_path,
                style: FillStrokeStyle {
                    fill: None,
                    stroke: Some(Stroke {
                        color: Color::new(200, 200, 200),
                        width: 1.0,
                    }),
                },
                z_index: Z_GRID,
            });

            // 添加半径标签
            let label_value = (level * 100 / grid_levels) as i32;
            elements.push(VisualElement::TextRun {
                text: label_value.to_string(),
                position: Point::new(center.x + level_radius + 5.0, center.y),
                style: TextStyle {
                    color: colors.axis_label_color,
                    font_size: 10.0,
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

        // 绘制角度射线（8个方向）
        let angle_count = 8;
        for i in 0..angle_count {
            let angle = -PI / 2.0 + 2.0 * PI * i as f64 / angle_count as f64;
            let end_x = center.x + radius * angle.cos();
            let end_y = center.y + radius * angle.sin();

            elements.push(VisualElement::Line {
                start: center,
                end: Point::new(end_x, end_y),
                style: StrokeStyle {
                    color: Color::new(200, 200, 200),
                    width: 1.0,
                },
                z_index: Z_GRID,
            });
        }

        elements
    }
}
