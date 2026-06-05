//! Line Builder: 将 LineSeries 组装为 VisualElement

use vello_cpu::kurbo::{BezPath, Point};

use crate::{
    error::Result,
    pipeline::{
        builder::{
            SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, Z_SERIES_LINE, Z_SERIES_POINT,
            fill_stroke_style,
        },
        typed_series::{LineSeries, RenderContext, SeriesLabelPosition, SymbolType},
    },
    visual::{FillStrokeStyle, TextAlign, TextBaseline, TextStyle, VisualElement},
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
            let area_path = if let Some(ref baseline_points) = series.baseline_points {
                // 堆叠面积：使用上一系列的轮廓作为底部边界
                build_stacked_area_path(&series.points, baseline_points, series.smooth)
            } else {
                // 普通面积：使用平坦基线
                build_area_path(&series.points, series.baseline_y, series.smooth)
            };
            elements.push(VisualElement::Path {
                path: area_path,
                style: FillStrokeStyle {
                    fill: Some(fill),
                    stroke: None,
                },
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
                let symbol_elements =
                    build_symbol(point, series.symbol_type, series.symbol_size, series.color);
                elements.extend(symbol_elements);
            }
        }

        // 4. 值标签
        if let Some(ref label_cfg) = series.label
            && label_cfg.show
        {
            for (point, value) in series.points.iter().zip(series.values.iter()) {
                let text = format_value(*value);
                let (x, y) = match label_cfg.position {
                    SeriesLabelPosition::Top => (point.x, point.y - series.symbol_size - 4.0),
                    SeriesLabelPosition::Inside => {
                        (point.x, point.y - series.symbol_size - 4.0) // 折线图不支持内部，回退到上方
                    }
                };

                elements.push(VisualElement::TextRun {
                    text,
                    position: Point::new(x, y),
                    style: TextStyle {
                        color: label_cfg.color,
                        font_size: label_cfg.font_size,
                        align: TextAlign::Center,
                        vertical_align: TextBaseline::Bottom,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                    z_index: Z_SERIES_LABEL,
                });
            }
        }

        Ok(elements)
    }
}

fn format_value(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{:.0}", v)
    } else {
        format!("{:.1}", v)
    }
}

/// 构建堆叠面积填充路径（顶部和底部都是轮廓线）
fn build_stacked_area_path(top_points: &[Point], bottom_points: &[Point], smooth: bool) -> BezPath {
    let mut path = BezPath::new();

    if top_points.is_empty() {
        return path;
    }

    // 顶部轮廓：从左到右
    path.move_to(top_points[0]);
    if smooth {
        append_smooth_segments(&mut path, top_points);
    } else {
        for point in &top_points[1..] {
            path.line_to(*point);
        }
    }

    // 底部轮廓：从右到左（反向）
    // 底部轮廓来自上一系列的顶部轮廓，使用直线闭合（视觉上不明显）
    for point in bottom_points.iter().rev() {
        path.line_to(*point);
    }

    path.close_path();

    path
}

/// 构建面积填充路径
fn build_area_path(points: &[Point], baseline_y: f64, smooth: bool) -> BezPath {
    let mut path = BezPath::new();

    if points.is_empty() {
        return path;
    }

    // 移动到第一个点
    path.move_to(points[0]);

    // 绘制线条（顶部轮廓）
    if smooth {
        append_smooth_segments(&mut path, points);
    } else {
        for point in &points[1..] {
            path.line_to(*point);
        }
    }

    // 闭合到基线
    let last = points.last().unwrap();
    path.line_to(Point::new(last.x, baseline_y));
    path.line_to(Point::new(points[0].x, baseline_y));
    path.close_path();

    path
}

/// 构建平滑曲线路径（Catmull-Rom 样条 → 三次贝塞尔）
fn build_smooth_path(points: &[Point]) -> BezPath {
    let mut path = BezPath::new();

    if points.is_empty() {
        return path;
    }

    path.move_to(points[0]);
    append_smooth_segments(&mut path, points);
    path
}

/// 追加平滑曲线段（Catmull-Rom → 三次贝塞尔）到已有路径
fn append_smooth_segments(path: &mut BezPath, points: &[Point]) {
    let n = points.len();
    if n < 2 {
        return;
    }

    let tension = 0.5;

    for i in 0..n - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < n {
            points[i + 2]
        } else {
            points[n - 1]
        };

        let cp1_x = p1.x + (p2.x - p0.x) * tension / 3.0;
        let cp1_y = p1.y + (p2.y - p0.y) * tension / 3.0;
        let cp2_x = p2.x - (p3.x - p1.x) * tension / 3.0;
        let cp2_y = p2.y - (p3.y - p1.y) * tension / 3.0;

        path.curve_to(Point::new(cp1_x, cp1_y), Point::new(cp2_x, cp2_y), p2);
    }
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
fn build_symbol(
    center: &Point,
    symbol_type: SymbolType,
    size: f64,
    color: crate::visual::Color,
) -> Vec<VisualElement> {
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
