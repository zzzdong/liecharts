//! Line Builder: 将 LineSeries 组装为 lievisual `SceneNode`

use lievisual::scene::{Element, SceneNode};
use lievisual::text::{RichSpan, TextAlign, TextBaseline, TextStyle};
use vello_cpu::kurbo::{BezPath, Point};

use crate::{
    error::Result,
    pipeline::{
        builder::{
            SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, Z_SERIES_LINE, Z_SERIES_POINT,
            circle, fill_stroke_style, path, rect, render_mark_lines,
        },
        typed_series::{LineSeries, RenderContext, SeriesLabelPosition, StepType, SymbolType},
    },
};

pub struct LineBuilder;

impl SeriesBuilder<LineSeries> for LineBuilder {
    fn build(series: &LineSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
        let mut elements = Vec::new();

        // 1. 面积填充（使用已解析的像素基线；单个数据点画不出面积）
        if series.points.len() >= 2
            && let Some(area_color) = &series.area_color
        {
            let alpha = (255.0 * series.area_opacity).clamp(0.0, 255.0) as u8;
            let mut fill = *area_color;
            fill.a = f64::from(alpha) / 255.0;
            let area_path = if let Some(ref baseline_points) = series.baseline_points {
                // 堆叠面积：使用上一系列的轮廓作为底部边界
                build_stacked_area_path(&series.points, baseline_points, series.smooth, series.step)
            } else {
                // 普通面积：使用平坦基线
                build_area_path(
                    &series.points,
                    series.baseline_y,
                    series.smooth,
                    series.step,
                )
            };
            elements.push(path(
                area_path,
                crate::pipeline::builder::fill_style(fill),
                false,
                Z_SERIES_FILL,
            ));
        }

        // 2. 线条（单个数据点画不出线，跳过）
        if series.points.len() >= 2 {
            let p = if let Some(step) = series.step {
                build_step_path(&series.points, step)
            } else if series.smooth {
                build_smooth_path(&series.points)
            } else {
                build_polyline_path(&series.points)
            };
            elements.push(path(
                p,
                lievisual::scene::FillStrokeStyle {
                    fill: None,
                    stroke: Some(lievisual::scene::Stroke::new(series.color, series.line_width)),
                },
                false,
                Z_SERIES_LINE,
            ));
        }

        // 3. 数据点符号（单个数据点也要渲染，否则完全看不到）
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
                let text = crate::pipeline::template::render_template(
                    label_cfg.formatter.as_deref(),
                    &crate::pipeline::template::TemplateContext {
                        series_name: Some(&series.name),
                        name: Some(&series.name),
                        value: Some(*value),
                        percent: None,
                    },
                    &format_value(*value),
                );
                let (x, y) = match label_cfg.position {
                    SeriesLabelPosition::Top => (point.x, point.y - series.symbol_size - 4.0),
                    SeriesLabelPosition::Inside => {
                        (point.x, point.y - series.symbol_size - 4.0) // 折线图不支持内部，回退到上方
                    }
                };

                let mut style = TextStyle::new(label_cfg.color, label_cfg.font_size, "sans-serif");
                style.align = TextAlign::Center;
                style.baseline = TextBaseline::Bottom;
                elements.push(
                    SceneNode::new(Element::Text {
                        spans: vec![RichSpan::new(text, style.clone())],
                        position: Point::new(x, y),
                        style,
                        layout: None,
                    })
                    .with_z(Z_SERIES_LABEL),
                );
            }
        }

        // 5. 标注线（markLine）
        render_mark_lines(&mut elements, &series.mark_lines, ctx.bounds);

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
fn build_stacked_area_path(
    top_points: &[Point],
    bottom_points: &[Point],
    smooth: bool,
    step: Option<StepType>,
) -> BezPath {
    let mut path = BezPath::new();

    if top_points.is_empty() {
        return path;
    }

    // 顶部轮廓：从左到右
    path.move_to(top_points[0]);
    if let Some(st) = step {
        append_step_segments(&mut path, top_points, st);
    } else if smooth {
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
fn build_area_path(
    points: &[Point],
    baseline_y: f64,
    smooth: bool,
    step: Option<StepType>,
) -> BezPath {
    let mut path = BezPath::new();

    if points.is_empty() {
        return path;
    }

    // 移动到第一个点
    path.move_to(points[0]);

    // 绘制线条（顶部轮廓）
    if let Some(st) = step {
        append_step_segments(&mut path, points, st);
    } else if smooth {
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

/// 构建步进折线路径
fn build_step_path(points: &[Point], step: StepType) -> BezPath {
    let mut path = BezPath::new();

    if points.is_empty() {
        return path;
    }

    path.move_to(points[0]);
    append_step_segments(&mut path, points, step);
    path
}

/// 追加步进线段到已有路径
fn append_step_segments(path: &mut BezPath, points: &[Point], step: StepType) {
    for i in 0..points.len() - 1 {
        let p1 = points[i];
        let p2 = points[i + 1];
        match step {
            StepType::Start => {
                // 先垂直再水平
                path.line_to(Point::new(p1.x, p2.y));
                path.line_to(p2);
            }
            StepType::Middle => {
                // 水平 → 垂直 → 水平
                let mid_x = (p1.x + p2.x) / 2.0;
                path.line_to(Point::new(mid_x, p1.y));
                path.line_to(Point::new(mid_x, p2.y));
                path.line_to(p2);
            }
            StepType::End => {
                // 先水平再垂直
                path.line_to(Point::new(p2.x, p1.y));
                path.line_to(p2);
            }
        }
    }
}

/// 构建符号元素
fn build_symbol(
    center: &Point,
    symbol_type: SymbolType,
    size: f64,
    color: crate::visual::Color,
) -> Vec<SceneNode> {
    let mut elements = Vec::new();

    match symbol_type {
        SymbolType::Circle => {
            elements.push(circle(
                *center,
                size,
                fill_stroke_style(color, color, 1.0),
                Z_SERIES_POINT,
            ));
        }
        SymbolType::EmptyCircle => {
            // 空心圆：只描边、不填充（ECharts line 默认符号）
            elements.push(circle(
                *center,
                size,
                lievisual::scene::FillStrokeStyle {
                    fill: None,
                    stroke: Some(lievisual::scene::Stroke::new(color, 1.0)),
                },
                Z_SERIES_POINT,
            ));
        }
        SymbolType::Rect => {
            let r = vello_cpu::kurbo::Rect::new(
                center.x - size,
                center.y - size,
                center.x + size,
                center.y + size,
            );
            elements.push(rect(r, fill_stroke_style(color, color, 1.0), Z_SERIES_POINT));
        }
        _ => {
            // 默认使用圆形
            elements.push(circle(
                *center,
                size,
                fill_stroke_style(color, color, 1.0),
                Z_SERIES_POINT,
            ));
        }
    }

    elements
}

#[cfg(test)]
mod tests {
    use vello_cpu::kurbo::Rect;

    use super::*;
    use crate::{pipeline::types::ColorContext, theme::Theme};

    fn single_point_series() -> LineSeries {
        LineSeries {
            name: "test".into(),
            color: crate::visual::Color::rgb(80, 112, 221),
            line_width: 2.0,
            smooth: true,
            step: None,
            area_color: None,
            area_opacity: 0.5,
            symbol_type: SymbolType::EmptyCircle,
            symbol_size: 4.0,
            points: vec![Point::new(405.0, 60.0)],
            baseline_y: 300.0,
            baseline_points: None,
            values: vec![70840845.0],
            label: None,
            mark_lines: Vec::new(),
        }
    }

    #[test]
    fn test_single_point_renders_hollow_symbol() {
        let colors = ColorContext::default();
        let ctx = RenderContext {
            colors: &colors,
            theme: &Theme::echarts(),
            bounds: Rect::new(60.0, 60.0, 740.0, 540.0),
        };
        let elements = LineBuilder::build(&single_point_series(), &ctx).unwrap();

        // 单个数据点画不出线，但必须渲染出数据点符号
        assert_eq!(elements.len(), 1, "单点折线图应渲染出 1 个符号");
        match &elements[0].element {
            Element::Circle { style, .. } => {
                assert!(style.fill.is_none(), "ECharts 默认折线符号是空心圆");
                assert!(style.stroke.is_some());
            }
            other => panic!("期望 Circle，实际 {:?}", other),
        }
    }

    #[test]
    fn test_multi_point_renders_line_and_symbols() {
        let mut series = single_point_series();
        series.points = vec![
            Point::new(100.0, 300.0),
            Point::new(300.0, 200.0),
            Point::new(500.0, 100.0),
        ];
        series.values = vec![1.0, 2.0, 3.0];
        let colors = ColorContext::default();
        let ctx = RenderContext {
            colors: &colors,
            theme: &Theme::echarts(),
            bounds: Rect::new(60.0, 60.0, 740.0, 540.0),
        };
        let elements = LineBuilder::build(&series, &ctx).unwrap();

        assert!(
            elements
                .iter()
                .any(|e| matches!(&e.element, Element::Path { .. }))
        );
        assert_eq!(
            elements
                .iter()
                .filter(|e| matches!(&e.element, Element::Circle { .. }))
                .count(),
            3
        );
    }
}
