//! PolarScatter Builder: 将 PolarScatterSeries 组装为 VisualElement

use vello_cpu::kurbo::Point;
use std::f64::consts::PI;

use crate::{
    error::Result,
    pipeline::builder::{fill_stroke_style, SeriesBuilder, Z_SERIES_POINT},
    pipeline::typed_series::{PolarScatterSeries, RenderContext},
    visual::VisualElement,
};

pub struct PolarScatterBuilder;

impl SeriesBuilder<PolarScatterSeries> for PolarScatterBuilder {
    fn build(series: &PolarScatterSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.points.len());

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 中心 X 在 50%，中心 Y 稍微向下偏移（55%）以平衡顶部空间
        let center_x = bounds.x0 + width * 0.5;
        let center_y = bounds.y0 + height * 0.55;

        // 半径缩放因子
        let min_dim = width.min(height);
        let radius_scale = min_dim * 0.5 * 0.75 / 100.0; // 假设最大半径为100

        for point in &series.points {
            // 将极坐标转换为笛卡尔坐标
            let angle_rad = point.angle * PI / 180.0;
            let r = point.radius * radius_scale;
            let x = center_x + r * angle_rad.cos();
            let y = center_y + r * angle_rad.sin();

            let center = Point::new(x, y);

            elements.push(VisualElement::Circle {
                center,
                radius: series.symbol_size / 2.0,
                style: fill_stroke_style(series.color, series.color, 1.0),
                z_index: Z_SERIES_POINT,
            });
        }

        Ok(elements)
    }
}
