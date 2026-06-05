//! PolarBar Builder: 将 PolarBarSeries 组装为 VisualElement

use std::f64::consts::PI;

use vello_cpu::kurbo::{BezPath, Point};

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, fill_style},
        typed_series::{PolarBarSeries, RenderContext},
    },
    visual::{Color, StrokeStyle, TextAlign, TextBaseline, TextStyle, VisualElement},
};

pub struct PolarBarBuilder;

impl SeriesBuilder<PolarBarSeries> for PolarBarBuilder {
    fn build(series: &PolarBarSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
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
            // radius 已经是像素空间值（来自 materializer）
            let r = bar.radius;
            let angle_rad = bar.angle * PI / 180.0;

            // 生成从中心到边缘的扇形路径
            let path = build_polar_bar_path(center_x, center_y, bar.angle, r, sweep_deg);

            // 使用柱子自己的颜色
            elements.push(VisualElement::Path {
                path,
                style: fill_style(bar.color),
                z_index: Z_SERIES_FILL,
            });

            // 计算柱子外边缘中点位置
            let outer_x = center_x + r * angle_rad.cos();
            let outer_y = center_y + r * angle_rad.sin();

            // 计算标签位置（更外侧）
            let label_x = center_x + label_radius * angle_rad.cos();
            let label_y = center_y + label_radius * angle_rad.sin();

            // 添加引导线（从柱子外边缘到标签）
            elements.push(VisualElement::Line {
                start: Point::new(outer_x, outer_y),
                end: Point::new(label_x, label_y),
                style: StrokeStyle::new(Color::new(200, 200, 200), 1.0),
                z_index: Z_SERIES_LABEL,
            });

            // 添加数值标签
            let label_text = format!("{:.0}", bar.value);
            elements.push(VisualElement::TextRun {
                text: label_text,
                position: Point::new(label_x, label_y),
                style: TextStyle {
                    color: Color::new(60, 60, 65),
                    font_size: 11.0,
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
