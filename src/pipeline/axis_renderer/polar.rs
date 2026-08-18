//! 极坐标轴渲染器
//!
//! 生成极坐标图表专用的同心圆网格线和径向射线，以及半径标签。
//! 与笛卡尔坐标轴没有共享逻辑，完全独立的几何体系。

use std::f64::consts::PI;

use lievisual::scene::{FillStrokeStyle, SceneNode, Stroke};
use lievisual::text::{TextAlign, TextBaseline, TextStyle};
use vello_cpu::kurbo::{Circle, Point, Shape as KurboShape};

use crate::{
    pipeline::{
        builder::{path, text_el},
        types::{ColorContext, SubplotSpec, TextMeasurer},
    },
};
use lievisual::Color;
use crate::pipeline::builder::{Z_GRID, Z_LABEL};

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
    ) -> Vec<SceneNode> {
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

            elements.push(path(
                circle_path,
                FillStrokeStyle {
                    fill: None,
                    stroke: Some(Stroke::new(Color::rgb(200, 200, 200), 1.0)),
                },
                false,
                Z_GRID,
            ));

            // 添加半径标签
            let label_value = level * 100 / grid_levels;
            let mut style = TextStyle::new(colors.axis_label_color, 10.0, "sans-serif");
            style.align = TextAlign::Left;
            style.baseline = TextBaseline::Middle;
            elements.push(text_el(
                label_value.to_string(),
                Point::new(center.x + level_radius + 5.0, center.y),
                style,
                Z_LABEL,
            ));
        }

        // 绘制角度射线（8个方向）
        let angle_count = 8;
        for i in 0..angle_count {
            let angle = -PI / 2.0 + 2.0 * PI * i as f64 / angle_count as f64;
            let end_x = center.x + radius * angle.cos();
            let end_y = center.y + radius * angle.sin();

            elements.push(crate::pipeline::builder::line(
                center,
                Point::new(end_x, end_y),
                Stroke::new(Color::rgb(200, 200, 200), 1.0),
                Z_GRID,
            ));
        }

        elements
    }
}
