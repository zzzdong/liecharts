//! Gauge Builder: 将 GaugeSeries 组装为 VisualElement

use vello_cpu::kurbo::{BezPath, Point};
use std::f64::consts::PI;

use crate::{
    error::Result,
    pipeline::builder::{stroke_style, SeriesBuilder, Z_SERIES_LINE, Z_SERIES_FILL},
    pipeline::typed_series::{GaugeSeries, RenderContext},
    visual::{VisualElement, FillStrokeStyle, Stroke},
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

        // 1. 绘制背景弧
        let bg_path = build_arc_path(center, radius - 10.0, start_angle, end_angle);
        elements.push(VisualElement::Path {
            path: bg_path,
            style: FillStrokeStyle {
                fill: None,
                stroke: Some(Stroke {
                    color: series.color.with_alpha(50),
                    width: 10.0,
                }),
            },
            z_index: Z_SERIES_LINE - 1,
        });

        // 2. 计算数值角度
        let value_ratio = ((series.value - series.min) / (series.max - series.min)).clamp(0.0, 1.0);
        let value_angle = start_angle + (end_angle - start_angle) * value_ratio;

        // 3. 绘制数值弧
        let value_path = build_arc_path(center, radius - 10.0, start_angle, value_angle);
        elements.push(VisualElement::Path {
            path: value_path,
            style: FillStrokeStyle {
                fill: None,
                stroke: Some(Stroke {
                    color: series.color,
                    width: 10.0,
                }),
            },
            z_index: Z_SERIES_LINE,
        });

        // 4. 绘制指针
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

        // 5. 中心圆点
        elements.push(VisualElement::Circle {
            center,
            radius: 5.0,
            style: FillStrokeStyle {
                fill: Some(series.color),
                stroke: None,
            },
            z_index: Z_SERIES_LINE + 2,
        });

        Ok(elements)
    }
}

/// 构建圆弧路径
fn build_arc_path(center: Point, radius: f64, start_angle: f64, end_angle: f64) -> BezPath {
    let mut path = BezPath::new();

    let steps = 20;
    let step = (end_angle - start_angle) / steps as f64;

    let start = Point::new(
        center.x + radius * start_angle.cos(),
        center.y + radius * start_angle.sin(),
    );
    path.move_to(start);

    for i in 1..=steps {
        let angle = start_angle + step * i as f64;
        let point = Point::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        );
        path.line_to(point);
    }

    path
}

/// 辅助 trait 扩展 Color
trait ColorExt {
    fn with_alpha(&self, alpha: u8) -> Self;
}

impl ColorExt for crate::visual::Color {
    fn with_alpha(&self, alpha: u8) -> Self {
        let mut color = *self;
        color.a = alpha;
        color
    }
}
