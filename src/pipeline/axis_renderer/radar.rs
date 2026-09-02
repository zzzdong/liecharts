//! 雷达图坐标轴渲染器
//!
//! 生成雷达图专用的同心多边形网格线和径向轴线。
//! 指示器名称标签由 `builder::radar::build_radar_indicators` 在 subplot
//! 级别绘制（按角度选择对齐方向），本模块只负责网格几何，不重复绘制。

use std::f64::consts::PI;

use lievisual::{
    Color,
    scene::{FillStrokeStyle, SceneNode, Stroke},
};
use vello_cpu::kurbo::{BezPath, Point};

use crate::pipeline::{
    builder::{Z_GRID, line, path},
    types::SubplotSpec,
};

/// 雷达图坐标轴渲染器
///
/// 渲染雷达图专用的：
/// - 同心多边形网格（多层，顶点数 = 指示器数量）
/// - 从中心到各顶点的径向轴线
pub struct RadarAxisRenderer;

impl RadarAxisRenderer {
    /// 渲染雷达图坐标轴（同心多边形网格 + 径向轴线）
    ///
    /// `indicator_count`：维度数，来自第一个雷达系列的指示器配置，
    /// 与数据多边形（`RadarBuilder`）和指示器标签（`build_radar_indicators`）
    /// 同源同口径；`< 3` 时兜底为 3。
    ///
    /// 历史坑：本函数曾接收 `&[RadarIndicatorOption]`，调用方传空数组导致
    /// 网格退化为三角形而数据多边形是 N 边形——已改为显式传维度数，
    /// 使"空指示器"不再可能静默改变网格形状。
    pub fn render(subplot: &SubplotSpec, indicator_count: usize) -> Vec<SceneNode> {
        let mut elements = Vec::new();

        let indicator_count = indicator_count.max(3);
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

        elements
    }
}
