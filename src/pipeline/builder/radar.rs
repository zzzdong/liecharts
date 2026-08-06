//! Radar Builder: 将 RadarSeries 组装为 VisualElement

use std::f64::consts::PI;

use vello_cpu::kurbo::{BezPath, Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_LINE, Z_SERIES_POINT},
        typed_series::{RadarSeries, RenderContext},
    },
    visual::{
        Color, FillStrokeStyle, Stroke, TextAlign, TextBaseline, TextStyle, VisualElement, Z_LABEL,
    },
};

pub struct RadarBuilder;

impl SeriesBuilder<RadarSeries> for RadarBuilder {
    fn build(series: &RadarSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
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
        fill_color.a = 64; // 半透明

        elements.push(VisualElement::Path {
            path: path.clone(),
            style: FillStrokeStyle {
                fill: Some(fill_color),
                stroke: None,
            },
            z_index: Z_SERIES_LINE - 1,
        });

        // 描边
        elements.push(VisualElement::Path {
            path,
            style: FillStrokeStyle {
                fill: None,
                stroke: Some(Stroke {
                    color: series.color,
                    width: 2.0,
                }),
            },
            z_index: Z_SERIES_LINE,
        });

        // 数据点
        for point in &points {
            elements.push(VisualElement::Circle {
                center: *point,
                radius: 3.0,
                style: FillStrokeStyle {
                    fill: Some(series.color),
                    stroke: Some(Stroke {
                        color: series.color,
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_POINT,
            });
        }

        // Indicator 标签由 build_radar_indicators 在 subplot 级别调用一次，
        // 避免多系列雷达图重复绘制标签

        Ok(elements)
    }
}

/// 在 subplot 级别渲染雷达图的指示器标签（仅调用一次）
pub fn build_radar_indicators(series: &RadarSeries, bounds: Rect) -> Vec<VisualElement> {
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

        elements.push(VisualElement::TextRun {
            text: label,
            position: Point::new(lx, ly),
            style: TextStyle {
                color: Color::new(84, 85, 90),
                font_size: 12.0,
                align,
                vertical_align: va,
                ..Default::default()
            },
            rotation: 0.0,
            max_width: None,
            layout: None,
            z_index: Z_LABEL,
        });
    }

    elements
}
