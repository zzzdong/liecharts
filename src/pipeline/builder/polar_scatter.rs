//! PolarScatter Builder: 将 PolarScatterSeries 组装为 VisualElement

use std::f64::consts::PI;

use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_LABEL, Z_SERIES_POINT, fill_stroke_style},
        typed_series::{PolarScatterSeries, RenderContext},
    },
    visual::{Color, TextAlign, TextBaseline, TextStyle, VisualElement},
};

pub struct PolarScatterBuilder;

impl SeriesBuilder<PolarScatterSeries> for PolarScatterBuilder {
    fn build(series: &PolarScatterSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.points.len() * 2);

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 中心 X 在 50%，中心 Y 稍微向下偏移（55%）以平衡顶部空间
        let center_x = bounds.x0 + width * 0.5;
        let center_y = bounds.y0 + height * 0.55;

        // 最大半径（用于计算标签位置）
        let max_radius = width.min(height) / 2.0 * 0.8;
        let label_radius = max_radius * 1.12; // 风向标签放在外侧

        for point in &series.points {
            // radius 已经是像素空间值（来自 materializer），直接使用
            let angle_rad = point.angle * PI / 180.0;
            let x = center_x + point.radius * angle_rad.cos();
            let y = center_y + point.radius * angle_rad.sin();

            let center = Point::new(x, y);

            // 使用每个点自己的大小（基于风速）
            elements.push(VisualElement::Circle {
                center,
                radius: point.size,
                style: fill_stroke_style(series.color, series.color, 1.0),
                z_index: Z_SERIES_POINT,
            });

            // 计算风向标签位置（更外侧）
            let label_x = center_x + label_radius * angle_rad.cos();
            let label_y = center_y + label_radius * angle_rad.sin();

            // 获取风向名称
            let wind_direction = angle_to_wind_direction(point.angle);

            elements.push(VisualElement::TextRun {
                text: wind_direction,
                position: Point::new(label_x, label_y),
                style: TextStyle {
                    color: Color::new(84, 85, 90),
                    font_size: 10.0,
                    font_family: "sans-serif".to_string(),
                    font_weight: Default::default(),
                    font_style: Default::default(),
                    align: TextAlign::Center,
                    vertical_align: TextBaseline::Middle,
                },
                rotation: 0.0,
                max_width: None,
                layout: None,
                z_index: Z_SERIES_LABEL,
            });
        }

        Ok(elements)
    }
}

/// 将角度转换为风向名称
fn angle_to_wind_direction(angle: f64) -> String {
    // 标准化角度到 0-360
    let normalized = ((angle % 360.0) + 360.0) % 360.0;

    // 16个风向（每22.5度一个）
    let directions = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];

    let index = ((normalized + 11.25) / 22.5) as usize % 16;
    directions[index].to_string()
}
