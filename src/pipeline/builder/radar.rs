//! Radar Builder: 将 RadarSeries 组装为 lievisual `SceneNode`

use std::f64::consts::PI;

use lievisual::scene::{Element, FillStrokeStyle, SceneNode, Stroke};
use lievisual::text::{RichSpan, TextAlign, TextBaseline, TextStyle};
use vello_cpu::kurbo::{BezPath, Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_LINE, Z_SERIES_POINT, Z_SERIES_FILL},
        typed_series::{RadarSeries, RenderContext},
    },
};

pub struct RadarBuilder;

impl SeriesBuilder<RadarSeries> for RadarBuilder {
    fn build(series: &RadarSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
        let mut elements = Vec::new();

        let indicator_count = series.indicators.len().max(3);
        let value_count = series.values.len();

        if value_count == 0 {
            return Ok(elements);
        }

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 将百分比坐标转换为像素坐标
        // 中心 X 在 50%，中心 Y 稍微向下偏移（55%）以平衡顶部空间（标题和图例）
        let center_x = bounds.x0 + width * 0.5;
        let center_y = bounds.y0 + height * 0.55;
        let center = Point::new(center_x, center_y);

        // 半径取宽高的较小值的一半，再乘以百分比（75%，留出边距给标签）
        let min_dim = width.min(height);
        let radius = min_dim * 0.5 * 0.75;

        // 计算多边形顶点
        let mut points = Vec::with_capacity(indicator_count);
        let max_value = series.values.iter().cloned().fold(0.0, f64::max).max(1.0);

        for i in 0..indicator_count {
            let angle = -PI / 2.0 + 2.0 * PI * i as f64 / indicator_count as f64;
            let value = series.values.get(i).copied().unwrap_or(0.0);
            let r = radius * (value / max_value);

            let x = center.x + r * angle.cos();
            let y = center.y + r * angle.sin();
            points.push(Point::new(x, y));
        }

        // 构建多边形路径
        let mut path = BezPath::new();
        if !points.is_empty() {
            path.move_to(points[0]);
            for point in &points[1..] {
                path.line_to(*point);
            }
            path.close_path();
        }

        // 填充区域（半透明）
        let mut fill_color = series.color;
        fill_color.a = 64.0 / 255.0; // 半透明

        elements.push(crate::pipeline::builder::path(
            path.clone(),
            FillStrokeStyle {
                fill: Some(lievisual::scene::Fill::Solid(fill_color)),
                stroke: None,
            },
            true,
            Z_SERIES_FILL,
        ));

        // 描边
        elements.push(crate::pipeline::builder::path(
            path,
            FillStrokeStyle {
                fill: None,
                stroke: Some(Stroke::new(series.color, 2.0)),
            },
            true,
            Z_SERIES_LINE,
        ));

        // 数据点
        for point in &points {
            elements.push(crate::pipeline::builder::circle(
                *point,
                3.0,
                FillStrokeStyle {
                    fill: Some(lievisual::scene::Fill::Solid(series.color)),
                    stroke: Some(Stroke::new(series.color, 1.0)),
                },
                Z_SERIES_POINT,
            ));
        }

        // Indicator 标签由 build_radar_indicators 在 subplot 级别调用一次，
        // 避免多系列雷达图重复绘制标签

        Ok(elements)
    }
}

/// 在 subplot 级别渲染雷达图的指示器标签（仅调用一次）
pub fn build_radar_indicators(series: &RadarSeries, bounds: Rect) -> Vec<SceneNode> {
    let mut elements = Vec::new();

    let indicator_count = series.indicators.len().max(3);

    let width = bounds.width();
    let height = bounds.height();

    // 中心 X 在 50%，中心 Y 稍微向下偏移（55%）以平衡顶部空间
    let center_x = bounds.x0 + width * 0.5;
    let center_y = bounds.y0 + height * 0.55;

    // 半径取宽高的较小值的一半，再乘以 75%（留出边距给标签）
    let min_dim = width.min(height);
    let radius = min_dim * 0.5 * 0.75;

    // Indicator 标签（绘制在每个轴的末端外侧）
    let label_r = radius + 16.0;
    for i in 0..indicator_count {
        let angle = -PI / 2.0 + 2.0 * PI * i as f64 / indicator_count as f64;
        let lx = center_x + label_r * angle.cos();
        let ly = center_y + label_r * angle.sin();

        let label = series
            .indicators
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("{}", i + 1));

        // 根据角度选择对齐方式
        let (align, va) = if angle.cos() > 0.3 {
            (TextAlign::Left, TextBaseline::Middle)
        } else if angle.cos() < -0.3 {
            (TextAlign::Right, TextBaseline::Middle)
        } else {
            (TextAlign::Center, TextBaseline::Middle)
        };

        let mut style = TextStyle::new(crate::visual::Color::rgb(84, 85, 90), 12.0, "sans-serif");
        style.align = align;
        style.baseline = va;
        elements.push(
            SceneNode::new(Element::Text {
                spans: vec![RichSpan::new(label, style.clone())],
                position: Point::new(lx, ly),
                style,
                layout: None,
            })
            .with_z(crate::pipeline::builder::Z_AXIS_LABEL),
        );
    }

    elements
}
