//! PolarScatter Builder: 将 PolarScatterSeries 组装为 lievisual `SceneNode`

use std::collections::HashSet;
use std::f64::consts::PI;

use lievisual::scene::{Element, SceneNode};
use lievisual::text::{RichSpan, TextAlign, TextBaseline, TextStyle};
use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    pipeline::{
        builder::{
            SeriesBuilder, Z_SERIES_LABEL, Z_SERIES_POINT, circle, fill_stroke_style,
        },
        typed_series::{PolarScatterSeries, RenderContext},
    },
};

pub struct PolarScatterBuilder;

impl SeriesBuilder<PolarScatterSeries> for PolarScatterBuilder {
    fn build(series: &PolarScatterSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
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

        // 收集每个数据点的风向，去重后每个风向只渲染一个标签
        let mut shown_directions = HashSet::new();

        for point in &series.points {
            // radius 已经是像素空间值（来自 materializer），直接使用
            let angle_rad = point.angle * PI / 180.0;
            let x = center_x + point.radius * angle_rad.cos();
            let y = center_y + point.radius * angle_rad.sin();

            let center = Point::new(x, y);

            // 使用每个点自己的大小（基于风速）
            elements.push(circle(
                center,
                point.size,
                fill_stroke_style(series.color, series.color, 1.0),
                Z_SERIES_POINT,
            ));

            // 获取风向名称，同一风向只渲染一次标签（放在该扇区中心，避免重叠）
            let wind_direction = angle_to_wind_direction(point.angle);
            if shown_directions.insert(wind_direction.clone()) {
                // 用该风向扇区的中心角度定位标签，保证相邻风向标签均匀分布
                let sector_center = direction_center_angle(point.angle);
                let label_angle = sector_center * PI / 180.0;
                let label_x = center_x + label_radius * label_angle.cos();
                let label_y = center_y + label_radius * label_angle.sin();

                let mut style =
                    TextStyle::new(crate::visual::Color::rgb(84, 85, 90), 10.0, "sans-serif");
                style.align = TextAlign::Center;
                style.baseline = TextBaseline::Middle;
                elements.push(
                    SceneNode::new(Element::Text {
                        spans: vec![RichSpan::new(wind_direction, style.clone())],
                        position: Point::new(label_x, label_y),
                        style,
                        layout: None,
                    })
                    .with_z(Z_SERIES_LABEL),
                );
            }
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

/// 返回给定角度所在风向扇区的中心角度（度）。
///
/// 风向按 22.5° 分 16 个扇区，扇区 `i` 的中心角度为 `i * 22.5 + 11.25`。
/// 用于让每个风向标签落在其扇区正中间，保证相邻标签均匀分布、不重叠。
fn direction_center_angle(angle: f64) -> f64 {
    let normalized = ((angle % 360.0) + 360.0) % 360.0;
    let index = ((normalized + 11.25) / 22.5) as usize % 16;
    index as f64 * 22.5 + 11.25
}
