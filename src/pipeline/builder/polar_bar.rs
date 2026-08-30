//! PolarBar Builder: 将 PolarBarSeries 组装为 lievisual `SceneNode`

use std::f64::consts::PI;

use lievisual::{
    Color,
    scene::{Element, SceneNode},
    text::{RichSpan, TextAlign, TextBaseline, TextStyle},
};
use vello_cpu::kurbo::{BezPath, Point};

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, fill_style, line, path},
        typed_series::{PolarBarSeries, RenderContext},
    },
};

pub struct PolarBarBuilder;

impl SeriesBuilder<PolarBarSeries> for PolarBarBuilder {
    fn build(series: &PolarBarSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
        let mut elements = Vec::with_capacity(series.bars.len() * 3); // 柱子 + 引导线 + 标签

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 中心 X 在 50%，中心 Y 稍微向下偏移（55%）以平衡顶部空间
        let center_x = bounds.x0 + width * 0.5;
        let center_y = bounds.y0 + height * 0.55;

        let num_bars = series.bars.len();
        // 每个柱的扫掠角度（度），考虑 pad_angle 作为间隔
        let sweep_deg = if num_bars > 0 {
            (360.0 / num_bars as f64) - series.pad_angle
        } else {
            30.0
        };

        // 最大半径（用于计算标签位置）
        let max_radius = width.min(height) / 2.0 * 0.8;
        let label_radius = max_radius * 1.15; // 标签放在柱子外侧

        for bar in &series.bars {
            // radius 已经是像素空间值（来自 materializer）。
            // angle 为罗盘角（0°=正上方，顺时针增加），转换为屏幕角：
            // 屏幕 0°=正右，顺时针增加 → screen = compass - 90°。
            let r = bar.radius;
            let angle_rad = (bar.angle - 90.0) * PI / 180.0;

            // 生成从中心到边缘的扇形路径（内部使用屏幕角）
            let p = build_polar_bar_path(center_x, center_y, bar.angle - 90.0, r, sweep_deg);

            // 使用柱子自己的颜色
            elements.push(path(p, fill_style(bar.color), true, Z_SERIES_FILL));

            // 计算柱子外边缘中点位置
            let outer_x = center_x + r * angle_rad.cos();
            let outer_y = center_y + r * angle_rad.sin();

            // 计算标签位置（更外侧）
            let label_x = center_x + label_radius * angle_rad.cos();
            let label_y = center_y + label_radius * angle_rad.sin();

            // 添加引导线（从柱子外边缘到标签）
            elements.push(line(
                Point::new(outer_x, outer_y),
                Point::new(label_x, label_y),
                lievisual::scene::Stroke::new(Color::rgb(200, 200, 200), 1.0),
                Z_SERIES_LABEL,
            ));

            // 添加类目名标签（无类目名时回退数值）
            let label_text = if bar.name.is_empty() {
                format!("{:.0}", bar.value)
            } else {
                bar.name.clone()
            };
            let mut style = TextStyle::new(Color::rgb(60, 60, 65), 11.0, "sans-serif");
            style.align = TextAlign::Center;
            style.baseline = TextBaseline::Middle;
            elements.push(
                SceneNode::new(Element::Text {
                    spans: vec![RichSpan::new(label_text, style.clone())],
                    position: Point::new(label_x, label_y),
                    style,
                    layout: None,
                })
                .with_z(Z_SERIES_LABEL),
            );
        }

        Ok(elements)
    }
}

/// 构建极坐标柱状图路径 — 从中心延伸到外缘的扇形
fn build_polar_bar_path(
    center_x: f64,
    center_y: f64,
    angle: f64,     // 柱中心角度（度）
    radius: f64,    // 外半径（像素）
    sweep_deg: f64, // 扫掠角度（度）
) -> BezPath {
    let mut path = BezPath::new();

    let half_sweep = sweep_deg / 2.0;
    let start_angle_deg = angle - half_sweep;
    let end_angle_deg = angle + half_sweep;

    let start_rad = start_angle_deg * PI / 180.0;
    let end_rad = end_angle_deg * PI / 180.0;

    // 移动到中心点
    path.move_to(Point::new(center_x, center_y));

    // 画到外弧的起点
    let outer_start = Point::new(
        center_x + radius * start_rad.cos(),
        center_y + radius * start_rad.sin(),
    );
    path.line_to(outer_start);

    // 画外弧（用小线段近似，保证与其他图表风格一致）
    let segments = (sweep_deg.abs() * 0.5).ceil().max(2.0) as usize;
    for i in 1..=segments {
        let t = i as f64 / segments as f64;
        let rad = start_rad + (end_rad - start_rad) * t;
        let p = Point::new(center_x + radius * rad.cos(), center_y + radius * rad.sin());
        path.line_to(p);
    }

    // 回到中心点
    path.line_to(Point::new(center_x, center_y));
    path.close_path();

    path
}
