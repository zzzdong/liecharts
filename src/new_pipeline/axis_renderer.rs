use vello_cpu::kurbo::{Point, Rect};

use crate::new_pipeline::types::{ColorContext, ResolvedAxisRanges, SubplotSpec, TextMeasurer};
use crate::option::{AxisOption, AxisType, ChartOption};
use crate::visual::{
    Color, StrokeStyle, TextAlign, TextBaseline, VisualElement, Z_AXIS, Z_GRID, Z_LABEL,
};
use crate::visual::TextStyle;

/// 在新管线中为单个 subplot 生成坐标轴和网格线视觉元素
pub struct AxisRenderer;

impl AxisRenderer {
    /// 为指定 subplot 生成 X/Y 轴线和网格线
    pub fn render(
        spec: &SubplotSpec,
        option: &ChartOption,
        axis_ranges: &ResolvedAxisRanges,
        colors: &ColorContext,
        text_measurer: &mut TextMeasurer,
    ) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        let bounds = spec.bounds;
        if bounds.width() <= 0.0 || bounds.height() <= 0.0 {
            return elements;
        }

        // ── X 轴线 ──
        for &x_axis_idx in &spec.x_axis_indices {
            let axis_config = option.x_axis.get(x_axis_idx);
            if let Some(axis_cfg) = axis_config {
                let x_range = axis_ranges.get_x_range(x_axis_idx);
                let (x_min, x_max) = x_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 1.0));

                // 轴线（底部）
                let axis_y = bounds.y1;
                Self::draw_axis_line(
                    &mut elements,
                    Point::new(bounds.x0, axis_y),
                    Point::new(bounds.x1, axis_y),
                    colors.axis_line_color,
                );

                // X 轴网格线 (垂直方向)
                Self::draw_x_grid_lines(
                    &mut elements,
                    bounds,
                    axis_cfg,
                    x_min,
                    x_max,
                    colors,
                );

                // X 轴刻度标签
                Self::draw_x_tick_labels(
                    &mut elements,
                    bounds,
                    axis_cfg,
                    x_min,
                    x_max,
                    colors,
                    text_measurer,
                );
            }
        }

        // ── Y 轴线 ──
        for &y_axis_idx in &spec.y_axis_indices {
            let axis_config = option.y_axis.get(y_axis_idx);
            if let Some(axis_cfg) = axis_config {
                let y_range = axis_ranges.get_y_range(y_axis_idx);
                let (y_min, y_max) = y_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 100.0));

                let is_right = y_range
                    .map(|r| r.position == crate::option::AxisPosition::Right)
                    .unwrap_or(false);
                let axis_x = if is_right { bounds.x1 } else { bounds.x0 };

                Self::draw_axis_line(
                    &mut elements,
                    Point::new(axis_x, bounds.y0),
                    Point::new(axis_x, bounds.y1),
                    colors.axis_line_color,
                );

                if !is_right {
                    Self::draw_y_grid_lines(
                        &mut elements,
                        bounds,
                        axis_cfg,
                        y_min,
                        y_max,
                        colors,
                    );
                }

                Self::draw_y_tick_labels_side(
                    &mut elements,
                    bounds,
                    axis_cfg,
                    y_min,
                    y_max,
                    colors,
                    text_measurer,
                    is_right,
                );
            }
        }

        elements
    }

    fn draw_axis_line(elements: &mut Vec<VisualElement>, start: Point, end: Point, color: Color) {
        elements.push(VisualElement::Line {
            start,
            end,
            style: StrokeStyle { color, width: 1.0 },
            z_index: Z_AXIS,
        });
    }

    fn draw_x_grid_lines(
        elements: &mut Vec<VisualElement>,
        bounds: Rect,
        axis_cfg: &AxisOption,
        _x_min: f64,
        _x_max: f64,
        colors: &ColorContext,
    ) {
        // 为 Category 轴生成网格线
        if axis_cfg.axis_type == Some(AxisType::Category) {
            if let Some(data) = &axis_cfg.data {
                let n = data.len();
                if n > 1 {
                    for i in 0..=n {
                        let t = i as f64 / n as f64;
                        let x = bounds.x0 + t * bounds.width();
                        elements.push(VisualElement::Line {
                            start: Point::new(x, bounds.y0),
                            end: Point::new(x, bounds.y1),
                            style: StrokeStyle {
                                color: colors.grid_line_color,
                                width: 0.5,
                            },
                            z_index: Z_GRID,
                        });
                    }
                }
            }
        } else {
            // Value 轴：生成 5 根网格线
            for i in 0..6 {
                let t = i as f64 / 5.0;
                let x = bounds.x0 + t * bounds.width();
                elements.push(VisualElement::Line {
                    start: Point::new(x, bounds.y0),
                    end: Point::new(x, bounds.y1),
                    style: StrokeStyle {
                        color: colors.grid_line_color,
                        width: 0.5,
                    },
                    z_index: Z_GRID,
                });
            }
        }
    }

    fn draw_y_grid_lines(
        elements: &mut Vec<VisualElement>,
        bounds: Rect,
        _axis_cfg: &AxisOption,
        y_min: f64,
        y_max: f64,
        colors: &ColorContext,
    ) {
        let ticks = compute_nice_ticks(y_min, y_max, 5);
        for &v in &ticks {
            let t = if y_max != y_min {
                (y_max - v) / (y_max - y_min)
            } else {
                0.5
            };
            let y = bounds.y0 + t * bounds.height();
            elements.push(VisualElement::Line {
                start: Point::new(bounds.x0, y),
                end: Point::new(bounds.x1, y),
                style: StrokeStyle {
                    color: colors.grid_line_color,
                    width: 0.5,
                },
                z_index: Z_GRID,
            });
        }
    }

    fn draw_x_tick_labels(
        elements: &mut Vec<VisualElement>,
        bounds: Rect,
        axis_cfg: &AxisOption,
        _x_min: f64,
        _x_max: f64,
        colors: &ColorContext,
        _text_measurer: &mut TextMeasurer,
    ) {
        let label_y = bounds.y1 + 14.0;
        if axis_cfg.axis_type == Some(AxisType::Category) {
            if let Some(data) = &axis_cfg.data {
                let n = data.len();
                if n == 0 {
                    return;
                }
                for (i, label) in data.iter().enumerate() {
                    let t = if n > 1 {
                        (i as f64 + 0.5) / n as f64
                    } else {
                        0.5
                    };
                    let x = bounds.x0 + t * bounds.width();
                    elements.push(VisualElement::TextRun {
                        text: label.clone(),
                        position: Point::new(x, label_y),
                        style: TextStyle {
                            font_size: 11.0,
                            color: colors.axis_label_color,
                            align: TextAlign::Center,
                            vertical_align: TextBaseline::Top,
                            ..Default::default()
                        },
                        rotation: 0.0,
                        max_width: None,
                        layout: None,
                        z_index: Z_LABEL,
                    });
                }
            }
        } else {
            // Value X axis
            let ticks = compute_nice_ticks(_x_min, _x_max, 5);
            let range = _x_max - _x_min;
            for &v in &ticks {
                let t = if range != 0.0 { (v - _x_min) / range } else { 0.5 };
                let x = bounds.x0 + t * bounds.width();
                let label = if v.fract() == 0.0 {
                    format!("{:.0}", v)
                } else if (v * 100.0).fract() == 0.0 {
                    format!("{:.1}", v)
                } else {
                    format!("{:.2}", v)
                };
                elements.push(VisualElement::TextRun {
                    text: label,
                    position: Point::new(x, label_y),
                    style: TextStyle {
                        font_size: 11.0,
                        color: colors.axis_label_color,
                        align: TextAlign::Center,
                        vertical_align: TextBaseline::Top,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                    z_index: Z_LABEL,
                });
            }
        }
    }

    fn draw_y_tick_labels_side(
        elements: &mut Vec<VisualElement>,
        bounds: Rect,
        axis_cfg: &AxisOption,
        y_min: f64,
        y_max: f64,
        colors: &ColorContext,
        _text_measurer: &mut TextMeasurer,
        is_right: bool,
    ) {
        let (x, align) = if is_right {
            (bounds.x1 + 6.0, TextAlign::Left)
        } else {
            (bounds.x0 - 6.0, TextAlign::Right)
        };

        if axis_cfg.axis_type == Some(AxisType::Category) {
            if let Some(data) = &axis_cfg.data {
                let n = data.len();
                if n == 0 {
                    return;
                }
                for (i, label) in data.iter().enumerate() {
                    let t = if n > 1 {
                        (i as f64 + 0.5) / n as f64
                    } else {
                        0.5
                    };
                    let y = bounds.y0 + t * bounds.height();
                    elements.push(VisualElement::TextRun {
                        text: label.clone(),
                        position: Point::new(x, y),
                        style: TextStyle {
                            font_size: 11.0,
                            color: colors.axis_label_color,
                            align,
                            vertical_align: TextBaseline::Middle,
                            ..Default::default()
                        },
                        rotation: 0.0,
                        max_width: None,
                        layout: None,
                        z_index: Z_LABEL,
                    });
                }
            }
            return;
        }

        let ticks = compute_nice_ticks(y_min, y_max, 5);
        let range = y_max - y_min;
        for &v in &ticks {
            let t = if range != 0.0 { (y_max - v) / range } else { 0.5 };
            let y = bounds.y0 + t * bounds.height();
            let label = if v.fract() == 0.0 {
                format!("{:.0}", v)
            } else if (v * 100.0).fract() == 0.0 {
                format!("{:.1}", v)
            } else {
                format!("{:.2}", v)
            };
            elements.push(VisualElement::TextRun {
                text: label,
                position: Point::new(x, y),
                style: TextStyle {
                    font_size: 11.0,
                    color: colors.axis_label_color,
                    align,
                    vertical_align: TextBaseline::Middle,
                    ..Default::default()
                },
                rotation: 0.0,
                max_width: None,
                layout: None,
                z_index: Z_LABEL,
            });
        }
    }
}

/// 计算"美观"的刻度值序列
fn compute_nice_ticks(min: f64, max: f64, max_ticks: usize) -> Vec<f64> {
    if max <= min || max_ticks == 0 {
        return vec![min];
    }

    let range = max - min;
    let rough_step = range / max_ticks as f64;

    // 取整到"美观"的步长
    let magnitude = 10_f64.powf(rough_step.log10().floor());
    let residual = rough_step / magnitude;
    let nice_step = if residual <= 1.5 {
        1.0
    } else if residual <= 3.5 {
        2.0
    } else if residual <= 7.5 {
        5.0
    } else {
        10.0
    } * magnitude;

    let nice_min = (min / nice_step).floor() * nice_step;
    let nice_max = (max / nice_step).ceil() * nice_step;

    let mut ticks = Vec::new();
    let mut v = nice_min;
    while v <= nice_max + nice_step * 1e-10 {
        if v >= min - nice_step * 1e-10 && v <= max + nice_step * 1e-10 {
            ticks.push(v);
        }
        v += nice_step;
    }

    if ticks.is_empty() {
        vec![min, max]
    } else {
        ticks
    }
}