//! 雷达图坐标轴渲染器
//!
//! 生成雷达图专用的同心多边形网格线和径向轴线，以及指示器标签。
//! 与笛卡尔坐标轴没有共享逻辑，完全独立的几何体系。

use std::f64::consts::PI;

use lievisual::scene::{FillStrokeStyle, SceneNode, Stroke};
use lievisual::text::{TextAlign, TextBaseline, TextStyle};
use vello_cpu::kurbo::{BezPath, Point};

use crate::{
    option::RadarIndicatorOption,
    pipeline::{
        builder::{line, path, text_el},
        types::{ColorContext, SubplotSpec},
    },
};
use lievisual::Color;
use crate::pipeline::builder::{Z_GRID, Z_LABEL};

/// 雷达图坐标轴渲染器
///
/// 渲染雷达图专用的：
/// - 同心多边形网格（多层）
/// - 从中心到各顶点的径向轴线
/// - 各顶点的指示器名称标签
pub struct RadarAxisRenderer;

impl RadarAxisRenderer {
    /// 渲染雷达图坐标轴（网格线和指示器标签）
    pub fn render(
        subplot: &SubplotSpec,
        indicators: &[RadarIndicatorOption],
        colors: &ColorContext,
    ) -> Vec<SceneNode> {
        let mut elements = Vec::new();

        let indicator_count = indicators.len().max(3);
        let bounds = subplot.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 中心点: X 50%, Y 55%（稍微向下偏移以平衡顶部空间）
        let center_x = bounds.x0 + width * 0.5;
        let center_y = bounds.y0 + height * 0.55;
        let center = Point::new(center_x, center_y);

        let min_dim = width.min(height);
        let radius = min_dim * 0.5 * 0.75;

        // 绘制同心多边形网格（5层）
        let grid_levels = 5;
        for level in 1..=grid_levels {
            let level_radius = radius * level as f64 / grid_levels as f64;
            let mut grid_path = BezPath::new();

            for i in 0..indicator_count {
                let angle = -PI / 2.0 + 2.0 * PI * i as f64 / indicator_count as f64;
                let x = center.x + level_radius * angle.cos();
                let y = center.y + level_radius * angle.sin();

                if i == 0 {
                    grid_path.move_to(Point::new(x, y));
                } else {
                    grid_path.line_to(Point::new(x, y));
                }
            }
            grid_path.close_path();

            elements.push(path(
                grid_path,
                FillStrokeStyle {
                    fill: None,
                    stroke: Some(Stroke::new(Color::rgb(200, 200, 200), 1.0)),
                },
                true,
                Z_GRID,
            ));
        }

        // 绘制从中心到各顶点的轴线
        for i in 0..indicator_count {
            let angle = -PI / 2.0 + 2.0 * PI * i as f64 / indicator_count as f64;
            let end_x = center.x + radius * angle.cos();
            let end_y = center.y + radius * angle.sin();

            elements.push(line(
                center,
                Point::new(end_x, end_y),
                Stroke::new(Color::rgb(200, 200, 200), 1.0),
                Z_GRID,
            ));
        }

        // 绘制指示器标签
        for (i, indicator) in indicators.iter().enumerate() {
            if let Some(ref name) = indicator.name {
                let angle = -PI / 2.0 + 2.0 * PI * i as f64 / indicator_count as f64;
                let label_radius = radius + 20.0;
                let label_x = center.x + label_radius * angle.cos();
                let label_y = center.y + label_radius * angle.sin();

                let mut style = TextStyle::new(colors.axis_label_color, 12.0, "sans-serif");
                style.align = TextAlign::Center;
                style.baseline = TextBaseline::Middle;
                elements.push(text_el(
                    name.clone(),
                    Point::new(label_x, label_y),
                    style,
                    Z_LABEL,
                ));
            }
        }

        elements
    }
}
