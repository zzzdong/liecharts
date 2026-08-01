//! 笛卡尔坐标轴渲染器
//!
//! 处理 X/Y 轴线和网格线的生成。内部根据轴类型（Value / Category）分支
//! 处理不同的刻度计算和标签生成策略。

use vello_cpu::kurbo::{Point, Rect};

use super::compute_nice_ticks;
 use crate::{
    pipeline::types::{AxisSpec, AxisType, AxisPosition, ColorContext, ResolvedAxisRanges, SubplotSpec, TextMeasurer},
    visual::{
        Color, StrokeStyle, TextAlign, TextBaseline, TextStyle, VisualElement, Z_AXIS, Z_GRID,
        Z_LABEL,
    },
};

/// 应用 formatter 格式化标签文本
///
/// 支持 ECharts 风格的 formatter:
/// - "{value}" - 替换为数值
/// - "{value} 万人" - 带后缀的模板
fn format_label(value: &str, formatter: &Option<String>) -> String {
    let Some(fmt) = formatter else {
        return value.to_string();
    };
    fmt.replace("{value}", value)
}

/// 笛卡尔坐标轴渲染器
///
/// 用于折线、柱状、散点等标准 X/Y 坐标轴图表的轴线和网格线渲染。
pub struct CartesianAxisRenderer;

impl CartesianAxisRenderer {
    /// 为指定 subplot 生成 X/Y 轴线和网格线
    pub fn render(
        subplot: &SubplotSpec,
        x_axes: &[AxisSpec],
        y_axes: &[AxisSpec],
        axis_ranges: &ResolvedAxisRanges,
        colors: &ColorContext,
        text_measurer: &mut TextMeasurer,
    ) -> Vec<VisualElement> {
        let mut elements = Vec::new();
        let bounds = subplot.bounds;

        // ── X 轴线 ──
        for &x_axis_idx in &subplot.x_axis_indices {
            if let Some(axis_cfg) = x_axes.get(x_axis_idx) {
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
                Self::draw_x_grid_lines(&mut elements, bounds, axis_cfg, x_min, x_max, colors);

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
        for &y_axis_idx in &subplot.y_axis_indices {
            if let Some(axis_cfg) = y_axes.get(y_axis_idx) {
                let y_range = axis_ranges.get_y_range(y_axis_idx);
                let (y_min, y_max) = y_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 100.0));

                let is_right = y_range
                    .map(|r| r.position == AxisPosition::Right)
                    .unwrap_or(false);
                let axis_x = if is_right { bounds.x1 } else { bounds.x0 };

                Self::draw_axis_line(
                    &mut elements,
                    Point::new(axis_x, bounds.y0),
                    Point::new(axis_x, bounds.y1),
                    colors.axis_line_color,
                );

                if !is_right {
                    Self::draw_y_grid_lines(&mut elements, bounds, axis_cfg, y_min, y_max, colors);
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
        axis_cfg: &AxisSpec,
        x_min: f64,
        x_max: f64,
        colors: &ColorContext,
    ) {
        if axis_cfg.axis_type == AxisType::Category {
            let n = axis_cfg.categories.len();
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
        } else {
            // Value 轴：网格线位置与刻度位置一致
            // 避免网格线和标签错位导致标签超出画布
            let ticks = compute_nice_ticks(x_min, x_max, 5);
            let range = x_max - x_min;
            for &v in &ticks {
                let t = if range != 0.0 {
                    (v - x_min) / range
                } else {
                    0.5
                };
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
        _axis_cfg: &AxisSpec,
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
        axis_cfg: &AxisSpec,
        _x_min: f64,
        _x_max: f64,
        colors: &ColorContext,
        _text_measurer: &mut TextMeasurer,
    ) {
        let label_y = bounds.y1 + 14.0;
        if axis_cfg.axis_type == AxisType::Category {
            let n = axis_cfg.categories.len();
            if n == 0 {
                return;
            }
            for (i, label) in axis_cfg.categories.iter().enumerate() {
                let t = if n > 1 {
                    (i as f64 + 0.5) / n as f64
                } else {
                    0.5
                };
                let x = bounds.x0 + t * bounds.width();
                let formatted_label = format_label(label, &axis_cfg.label_formatter);
                elements.push(VisualElement::TextRun {
                    text: formatted_label,
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
        } else {
            let ticks = compute_nice_ticks(_x_min, _x_max, 5);
            let range = _x_max - _x_min;
            for &v in &ticks {
                let t = if range != 0.0 {
                    (v - _x_min) / range
                } else {
                    0.5
                };
                let x = bounds.x0 + t * bounds.width();
                let raw_label = if v.fract() == 0.0 {
                    format!("{:.0}", v)
                } else if (v * 100.0).fract() == 0.0 {
                    format!("{:.1}", v)
                } else {
                    format!("{:.2}", v)
                };
                let label = format_label(&raw_label, &axis_cfg.label_formatter);
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

    #[allow(clippy::too_many_arguments)]
    fn draw_y_tick_labels_side(
        elements: &mut Vec<VisualElement>,
        bounds: Rect,
        axis_cfg: &AxisSpec,
        y_min: f64,
        y_max: f64,
        colors: &ColorContext,
        _text_measurer: &mut TextMeasurer,
        is_right: bool,
    ) {
        let (x, align) = if is_right {
            (bounds.x1 + 8.0, TextAlign::Left)
        } else {
            (bounds.x0 - 8.0, TextAlign::Right)
        };

        if axis_cfg.axis_type == AxisType::Category {
            let n = axis_cfg.categories.len();
            if n == 0 {
                return;
            }
            // 与柱状图渲染保持一致：category 0 在底部，category n-1 在顶部
            for (i, label) in axis_cfg.categories.iter().enumerate() {
                let t = if n > 1 {
                    (i as f64 + 0.5) / n as f64
                } else {
                    0.5
                };
                // 反转 Y 位置：i=0 在底部（与柱状图 cat_idx 一致）
                let y = bounds.y1 - t * bounds.height();
                let formatted_label = format_label(label, &axis_cfg.label_formatter);
                elements.push(VisualElement::TextRun {
                    text: formatted_label,
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
            return;
        }

        let ticks = compute_nice_ticks(y_min, y_max, 5);
        let range = y_max - y_min;
        for &v in &ticks {
            let t = if range != 0.0 {
                (y_max - v) / range
            } else {
                0.5
            };
            let y = bounds.y0 + t * bounds.height();
            let raw_label = if v.fract() == 0.0 {
                format!("{:.0}", v)
            } else if (v * 100.0).fract() == 0.0 {
                format!("{:.1}", v)
            } else {
                format!("{:.2}", v)
            };
            let label = format_label(&raw_label, &axis_cfg.label_formatter);
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