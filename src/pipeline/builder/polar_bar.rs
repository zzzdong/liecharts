//! PolarBar Builder: 将 PolarBarSeries 组装为 VisualElement

use vello_cpu::kurbo::{BezPath, Point};
use std::f64::consts::PI;

use crate::{
    error::Result,
    pipeline::builder::{fill_style, SeriesBuilder, Z_SERIES_FILL},
    pipeline::typed_series::{PolarBarSeries, RenderContext},
    visual::VisualElement,
};

pub struct PolarBarBuilder;

impl SeriesBuilder<PolarBarSeries> for PolarBarBuilder {
    fn build(series: &PolarBarSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.bars.len());

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 中心 X 在 50%，中心 Y 稍微向下偏移（55%）以平衡顶部空间
        let center_x = bounds.x0 + width * 0.5;
        let center_y = bounds.y0 + height * 0.55;

        // 半径缩放因子
        let min_dim = width.min(height);
        let radius_scale = min_dim * 0.5 * 0.75 / 100.0; // 假设最大半径为100

        for bar in &series.bars {
            // 将极坐标转换为笛卡尔坐标
            let angle_rad = bar.angle * PI / 180.0;
            let r = bar.radius * radius_scale;
            let end_x = center_x + r * angle_rad.cos();
            let end_y = center_y + r * angle_rad.sin();

            // 简化的极坐标柱状图：绘制扇形
            // 实际实现需要更复杂的路径计算
            let path = build_polar_bar_path(center_x, center_y, bar.angle, r, series.pad_angle);

            elements.push(VisualElement::Path {
                path,
                style: fill_style(series.color),
                z_index: Z_SERIES_FILL,
            });
        }

        Ok(elements)
    }
}

/// 构建极坐标柱状图路径（简化版）
fn build_polar_bar_path(center_x: f64, center_y: f64, angle: f64, radius: f64, _pad_angle: f64) -> BezPath {
    let mut path = BezPath::new();

    // 简化为从中心到边缘的扇形
    let angle_rad = angle * PI / 180.0;
    let start_angle = angle_rad - 0.1; // 简化宽度
    let end_angle = angle_rad + 0.1;

    // 起点（内圆）
    let inner_start = Point::new(
        center_x + 5.0 * start_angle.cos(),
        center_y + 5.0 * start_angle.sin()
    );
    path.move_to(inner_start);

    // 外圆弧
    let outer_end = Point::new(
        center_x + radius * end_angle.cos(),
        center_y + radius * end_angle.sin()
    );
    path.line_to(outer_end);

    // 内圆弧（反向）
    let inner_end = Point::new(
        center_x + 5.0 * end_angle.cos(),
        center_y + 5.0 * end_angle.sin()
    );
    path.line_to(inner_end);

    path.close_path();
    path
}
