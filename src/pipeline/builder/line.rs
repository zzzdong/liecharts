//! Line Builder: 将 LineSeries 组装为 VisualElement

use vello_cpu::kurbo::{BezPath, Point};

use crate::{
    error::Result,
    pipeline::builder::{fill_stroke_style, stroke_style, SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LINE, Z_SERIES_POINT},
    pipeline::typed_series::{LineSeries, RenderContext, SymbolType},
    visual::{FillStrokeStyle, VisualElement},
};

pub struct LineBuilder;

impl SeriesBuilder<LineSeries> for LineBuilder {
    fn build(series: &LineSeries, _ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::new();

        if series.points.len() < 2 {
            return Ok(elements);
        }

        // 1. 面积填充（使用已解析的像素基线）
        if let Some(area_color) = &series.area_color {
            let alpha = (255.0 * series.area_opacity).clamp(0.0, 255.0) as u8;
            let mut fill = *area_color;
            fill.a = alpha;
            let area_path = build_area_path(&series.points, series.baseline_y);
            elements.push(VisualElement::Path {
                path: area_path,
                style: FillStrokeStyle { fill: Some(fill), stroke: None },
                z_index: Z_SERIES_FILL,
            });
        }

        // 2. 线条
        let path = if series.smooth {
            build_smooth_path(&series.points)
        } else {
            build_polyline_path(&series.points)
        };
        elements.push(VisualElement::Path {
            path,
            style: FillStrokeStyle {
                fill: None,
                stroke: Some(crate::visual::Stroke {
                    color: series.color,
                    width: series.line_width,
                }),
            },
            z_index: Z_SERIES_LINE,
        });

        // 3. 数据点符号
        if series.symbol_type != SymbolType::None {
            for point in &series.points {
                let symbol_elements = build_symbol(point, series.symbol_type, series.symbol_size, series.color);
                elements.extend(symbol_elements);
            }
        }

        Ok(elements)
    }
}

/// 构建面积填充路径
fn build_area_path(points: &[Point], baseline_y: f64) -> BezPath {
    let mut path = BezPath::new();

    if points.is_empty() {
        return path;
    }

    // 移动到第一个点
    path.move_to(points[0]);

    // 绘制线条
    for point in &points[1..] {
        path.line_to(*point);
    }

    // 闭合到基线
    let last = points.last().unwrap();
    path.line_to(Point::new(last.x, baseline_y));
    path.line_to(Point::new(points[0].x, baseline_y));
    path.close_path();

    path
}

/// 构建平滑曲线路径（使用 Catmull-Rom 样条简化版）
fn build_smooth_path(points: &[Point]) -> BezPath {
    let mut path = BezPath::new();

    if points.is_empty() {
        return path;
    }

    if points.len() == 1 {
        path.move_to(points[0]);
        return path;
    }

    path.move_to(points[0]);

    // 简化的平滑曲线：使用二次贝塞尔曲线
    for i in 1..points.len() {
        let prev = points[i - 1];
        let curr = points[i];

        if i == 1 {
            path.line_to(curr);
        } else {
            // 使用控制点创建平滑曲线
            let mid = Point::new((prev.x + curr.x) / 2.0, (prev.y + curr.y) / 2.0);
            path.quad_to(prev, mid);
            path.line_to(curr);
        }
    }

    path
}

/// 构建折线路径
fn build_polyline_path(points: &[Point]) -> BezPath {
    let mut path = BezPath::new();

    if points.is_empty() {
        return path;
    }

    path.move_to(points[0]);

    for point in &points[1..] {
        path.line_to(*point);
    }

    path
}

/// 构建符号元素
fn build_symbol(center: &Point, symbol_type: SymbolType, size: f64, color: crate::visual::Color) -> Vec<VisualElement> {
    let mut elements = Vec::new();

    match symbol_type {
        SymbolType::Circle => {
            elements.push(VisualElement::Circle {
                center: *center,
                radius: size,
                style: fill_stroke_style(color, color, 1.0),
                z_index: Z_SERIES_POINT,
            });
        }
        SymbolType::Rect => {
            let rect = vello_cpu::kurbo::Rect::new(
                center.x - size,
                center.y - size,
                center.x + size,
                center.y + size,
            );
            elements.push(VisualElement::Rect {
                rect,
                style: fill_stroke_style(color, color, 1.0),
                z_index: Z_SERIES_POINT,
            });
        }
        _ => {
            // 默认使用圆形
            elements.push(VisualElement::Circle {
                center: *center,
                radius: size,
                style: fill_stroke_style(color, color, 1.0),
                z_index: Z_SERIES_POINT,
            });
        }
    }

    elements
}
