//! 笛卡尔坐标轴渲染器
//!
//! 处理 X/Y 轴线和网格线的生成。内部根据轴类型（Value / Category）分支
//! 处理不同的刻度计算和标签生成策略。

use vello_cpu::kurbo::{Point, Rect};

use super::compute_nice_ticks;
use crate::{
    pipeline::{
        axis_label::{auto_rotate, label_step, rotated_bounds},
        types::{
            AxisPosition, AxisSpec, AxisType, ColorContext, ResolvedAxisRanges, SubplotSpec,
            TextMeasurer,
        },
    },
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

/// 格式化数值刻度标签
fn format_tick(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{:.0}", v)
    } else if (v * 100.0).fract() == 0.0 {
        format!("{:.1}", v)
    } else {
        format!("{:.2}", v)
    }
}

/// 轴标签基准样式（渲染时位置已预计算为文本块左上角，故统一使用 Left/Top 对齐）
fn label_style(colors: &ColorContext) -> TextStyle {
    TextStyle {
        font_size: 11.0,
        color: colors.axis_label_color,
        align: TextAlign::Left,
        vertical_align: TextBaseline::Top,
        ..Default::default()
    }
}

/// 计算 X/Y 轴标签的旋转角度（弧度）与抽稀步长。
///
/// 优先使用用户显式配置的 `axisLabel.rotate`；未配置时根据实测文本宽度自动选择
/// 0° → 45° → 90°，直到旋转后的投影宽度能放进一个刻度槽。
fn choose_label_layout(
    labels: &[String],
    slot_w: f64,
    user_rotate_deg: Option<f64>,
    measurer: &mut TextMeasurer,
    colors: &ColorContext,
) -> (f64, usize) {
    let (max_w, max_h) = measure_labels(labels, measurer, colors);
    let rotation = match user_rotate_deg {
        Some(deg) => deg.to_radians(),
        None => auto_rotate(max_w, max_h, slot_w),
    };
    let (projected_w, _) = rotated_bounds(max_w, max_h, rotation);
    (rotation, label_step(projected_w, slot_w))
}

/// Y 轴标签布局：不自动旋转（避免长标签纵向挤压），仅尊重用户旋转，
/// 按旋转后的投影高度计算抽稀步长，防止纵向密集时互相遮挡。
fn choose_y_label_layout(
    labels: &[String],
    slot_h: f64,
    user_rotate_deg: Option<f64>,
    measurer: &mut TextMeasurer,
    colors: &ColorContext,
) -> (f64, usize) {
    let (max_w, max_h) = measure_labels(labels, measurer, colors);
    let rotation = user_rotate_deg.map(|deg| deg.to_radians()).unwrap_or(0.0);
    let (_, projected_h) = rotated_bounds(max_w, max_h, rotation);
    (rotation, label_step(projected_h, slot_h))
}

/// 实测所有标签的最大宽/高
fn measure_labels(
    labels: &[String],
    measurer: &mut TextMeasurer,
    colors: &ColorContext,
) -> (f64, f64) {
    let style = label_style(colors);
    let mut max_w: f64 = 0.0;
    let mut max_h: f64 = 0.0;
    for label in labels {
        let (w, h): (f64, f64) = measurer.measure(label, &style);
        max_w = max_w.max(w);
        max_h = max_h.max(h);
    }
    (max_w, max_h)
}

/// 生成单个 X 轴标签。
///
/// - 未旋转（0°）：文本块顶部居中对齐锚点（水平居中于数据点）。
/// - 旋转时：文本块开头（左上角）对准锚点（数据点），围绕该点旋转。
///   底部 X 轴顺时针旋转（文本向右下延伸）、顶部 X 轴逆时针旋转（文本向右上延伸）。
///
/// 这样锚点（= 旋转中心 = 文本左上角）就是旋转后 y 坐标最小的点，
/// 因此旋转长标签时文本整体保持在锚点一侧、不跨过坐标轴侵入绘图区，
/// 也避免"绕顶边中点旋转"导致文本条上端甩进绘图区。
fn push_x_label(
    elements: &mut Vec<VisualElement>,
    text: &str,
    anchor: Point,
    rotation: f64,
    downward: bool,
    colors: &ColorContext,
    measurer: &mut TextMeasurer,
) {
    let style = label_style(colors);
    let (w, _h) = measurer.measure(text, &style);
    let (x, y, draw_rotation) = if rotation == 0.0 {
        (anchor.x - w / 2.0, anchor.y, 0.0)
    } else {
        // 文本开头对准数据点；顶部轴用负角使文本向右上延伸
        let draw_rotation = if downward { rotation } else { -rotation };
        (anchor.x, anchor.y, draw_rotation)
    };
    elements.push(VisualElement::TextRun {
        text: text.to_string(),
        position: Point::new(x, y),
        style,
        rotation: draw_rotation,
        max_width: None,
        layout: None,
        z_index: Z_LABEL,
    });
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

                // 轴线：顶部 X 轴画在绘图区上边缘，其余画在下边缘
                let axis_y = if axis_cfg.position == AxisPosition::Top {
                    bounds.y0
                } else {
                    bounds.y1
                };
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
        x_min: f64,
        x_max: f64,
        colors: &ColorContext,
        text_measurer: &mut TextMeasurer,
    ) {
        if !axis_cfg.label_show {
            return;
        }
        // 顶部 X 轴的标签绘制在绘图区上方，其余绘制在下方
        let label_y = if axis_cfg.position == AxisPosition::Top {
            bounds.y0 - 14.0
        } else {
            bounds.y1 + 14.0
        };
        // 底部 X 轴标签向下延伸（顺时针旋转），顶部 X 轴标签向上延伸（逆时针）
        let downward = axis_cfg.position != AxisPosition::Top;
        if axis_cfg.axis_type == AxisType::Category {
            let n = axis_cfg.categories.len();
            if n == 0 {
                return;
            }
            let labels: Vec<String> = axis_cfg
                .categories
                .iter()
                .map(|l| format_label(l, &axis_cfg.label_formatter))
                .collect();
            // 每个类别占据一个槽位
            let slot_w = bounds.width() / n as f64;
            let (rotation, step) = choose_label_layout(
                &labels,
                slot_w,
                axis_cfg.label_rotate,
                text_measurer,
                colors,
            );

            let mut last_rendered: Option<usize> = None;
            for i in (0..n).step_by(step) {
                last_rendered = Some(i);
                let cx = bounds.x0 + (i as f64 + 0.5) / n as f64 * bounds.width();
                push_x_label(
                    elements,
                    &labels[i],
                    Point::new(cx, label_y),
                    rotation,
                    downward,
                    colors,
                    text_measurer,
                );
            }
            // 最后一个标签：若与上一个渲染的标签间距足够则补上（首尾可见）
            let last_idx = n - 1;
            if step > 1
                && last_rendered != Some(last_idx)
                && let Some(prev) = last_rendered
            {
                let gap = (last_idx - prev) as f64 * slot_w;
                let style = label_style(colors);
                let (w, h) = text_measurer.measure(&labels[last_idx], &style);
                let (pw, _) = rotated_bounds(w, h, rotation);
                if gap >= pw {
                    let cx = bounds.x1 - slot_w / 2.0;
                    push_x_label(
                        elements,
                        &labels[last_idx],
                        Point::new(cx, label_y),
                        rotation,
                        downward,
                        colors,
                        text_measurer,
                    );
                }
            }
        } else {
            let ticks = compute_nice_ticks(x_min, x_max, 5);
            if ticks.is_empty() {
                return;
            }
            let range = x_max - x_min;
            let labels: Vec<String> = ticks
                .iter()
                .map(|&v| format_label(&format_tick(v), &axis_cfg.label_formatter))
                .collect();

            // 刻度间距（像素）
            let positions: Vec<f64> = ticks
                .iter()
                .map(|&v| {
                    let t = if range != 0.0 {
                        (v - x_min) / range
                    } else {
                        0.5
                    };
                    bounds.x0 + t * bounds.width()
                })
                .collect();
            let slot_w = if positions.len() > 1 {
                positions
                    .windows(2)
                    .map(|w| w[1] - w[0])
                    .fold(f64::INFINITY, f64::min)
            } else {
                bounds.width()
            };
            let (rotation, step) = choose_label_layout(
                &labels,
                slot_w,
                axis_cfg.label_rotate,
                text_measurer,
                colors,
            );

            let mut last_rendered: Option<usize> = None;
            for i in (0..ticks.len()).step_by(step) {
                last_rendered = Some(i);
                push_x_label(
                    elements,
                    &labels[i],
                    Point::new(positions[i], label_y),
                    rotation,
                    downward,
                    colors,
                    text_measurer,
                );
            }
            let last_idx = ticks.len() - 1;
            if step > 1
                && last_rendered != Some(last_idx)
                && let Some(prev) = last_rendered
            {
                let gap = positions[last_idx] - positions[prev];
                let style = label_style(colors);
                let (w, h) = text_measurer.measure(&labels[last_idx], &style);
                let (pw, _) = rotated_bounds(w, h, rotation);
                if gap >= pw {
                    push_x_label(
                        elements,
                        &labels[last_idx],
                        Point::new(positions[last_idx], label_y),
                        rotation,
                        downward,
                        colors,
                        text_measurer,
                    );
                }
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
        text_measurer: &mut TextMeasurer,
        is_right: bool,
    ) {
        if !axis_cfg.label_show {
            return;
        }
        let (x, align) = if is_right {
            (bounds.x1 + 8.0, TextAlign::Left)
        } else {
            (bounds.x0 - 8.0, TextAlign::Right)
        };

        // 生成单个 Y 轴标签：未旋转时保持现有对齐方式，旋转时锚点对齐旋转后包围盒
        // 的外侧边缘（左侧轴贴右边缘、右侧轴贴左边缘），并保持垂直居中于刻度，
        // 避免旋转标签被拉到轴线另一侧而侵入绘图区。
        let push_y_label = |elements: &mut Vec<VisualElement>,
                            text: &str,
                            anchor: Point,
                            rotation: f64,
                            colors: &ColorContext,
                            text_measurer: &mut TextMeasurer| {
            if rotation == 0.0 {
                elements.push(VisualElement::TextRun {
                    text: text.to_string(),
                    position: anchor,
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
            } else {
                let style = label_style(colors);
                let (w, h) = text_measurer.measure(text, &style);
                let (s, c) = rotation.sin_cos();
                let rotated_w = w * c.abs() + h * s.abs();
                let rotated_h = w * s.abs() + h * c.abs();
                let x = if is_right {
                    anchor.x
                } else {
                    anchor.x - rotated_w
                };
                let y = anchor.y - rotated_h / 2.0;
                elements.push(VisualElement::TextRun {
                    text: text.to_string(),
                    position: Point::new(x, y),
                    style,
                    rotation,
                    max_width: None,
                    layout: None,
                    z_index: Z_LABEL,
                });
            }
        };

        if axis_cfg.axis_type == AxisType::Category {
            let n = axis_cfg.categories.len();
            if n == 0 {
                return;
            }
            let labels: Vec<String> = axis_cfg
                .categories
                .iter()
                .map(|l| format_label(l, &axis_cfg.label_formatter))
                .collect();
            let slot_h = bounds.height() / n as f64;
            let (rotation, step) = choose_y_label_layout(
                &labels,
                slot_h,
                axis_cfg.label_rotate,
                text_measurer,
                colors,
            );

            // 与柱状图渲染保持一致：category 0 在底部，category n-1 在顶部
            let mut last_rendered: Option<usize> = None;
            for i in (0..n).step_by(step) {
                last_rendered = Some(i);
                let t = (i as f64 + 0.5) / n as f64;
                let y = bounds.y1 - t * bounds.height();
                push_y_label(
                    elements,
                    &labels[i],
                    Point::new(x, y),
                    rotation,
                    colors,
                    text_measurer,
                );
            }
            let last_idx = n - 1;
            if step > 1
                && last_rendered != Some(last_idx)
                && let Some(prev) = last_rendered
            {
                let gap = (last_idx - prev) as f64 * slot_h;
                let style = label_style(colors);
                let (w, h) = text_measurer.measure(&labels[last_idx], &style);
                let (_, ph) = rotated_bounds(w, h, rotation);
                if gap >= ph {
                    let y = bounds.y1 - (n as f64 - 0.5) / n as f64 * bounds.height();
                    push_y_label(
                        elements,
                        &labels[last_idx],
                        Point::new(x, y),
                        rotation,
                        colors,
                        text_measurer,
                    );
                }
            }
            return;
        }

        let ticks = compute_nice_ticks(y_min, y_max, 5);
        let range = y_max - y_min;
        let labels: Vec<String> = ticks
            .iter()
            .map(|&v| format_label(&format_tick(v), &axis_cfg.label_formatter))
            .collect();
        let positions: Vec<f64> = ticks
            .iter()
            .map(|&v| {
                let t = if range != 0.0 {
                    (y_max - v) / range
                } else {
                    0.5
                };
                bounds.y0 + t * bounds.height()
            })
            .collect();
        let slot_h = if positions.len() > 1 {
            positions
                .windows(2)
                .map(|w| w[0] - w[1])
                .fold(f64::INFINITY, f64::min)
        } else {
            bounds.height()
        };
        let (rotation, step) = choose_y_label_layout(
            &labels,
            slot_h,
            axis_cfg.label_rotate,
            text_measurer,
            colors,
        );

        for i in (0..ticks.len()).step_by(step) {
            push_y_label(
                elements,
                &labels[i],
                Point::new(x, positions[i]),
                rotation,
                colors,
                text_measurer,
            );
        }
        let last_idx = ticks.len() - 1;
        if step > 1
            && last_idx > 0
            && !(positions.len() - 1).is_multiple_of(step)
            && let Some(prev) = (0..positions.len()).rev().find(|&i| i % step == 0)
        {
            let gap = positions[prev] - positions[last_idx];
            let style = label_style(colors);
            let (w, h) = text_measurer.measure(&labels[last_idx], &style);
            let (_, ph) = rotated_bounds(w, h, rotation);
            if gap >= ph {
                push_y_label(
                    elements,
                    &labels[last_idx],
                    Point::new(x, positions[last_idx]),
                    rotation,
                    colors,
                    text_measurer,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use vello_cpu::kurbo::Rect;

    use super::*;
    use crate::pipeline::types::{ResolvedAxisRange, ResolvedAxisRanges};

    fn category_axis(position: AxisPosition, categories: Vec<String>) -> AxisSpec {
        AxisSpec {
            axis_type: AxisType::Category,
            position,
            grid_index: 0,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories,
            boundary_gap: true,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate: None,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }
    }

    #[test]
    fn test_dense_long_labels_rotate_and_thin() {
        let subplot = SubplotSpec {
            id: 0,
            bounds: Rect::new(0.0, 0.0, 400.0, 300.0),
            series_indices: vec![],
            x_axis_indices: vec![0],
            y_axis_indices: vec![0],
        };
        // 100 个长日期标签，槽宽仅 4px → 自动旋转 90° 并按步长抽稀
        let categories: Vec<String> = (0..100)
            .map(|i| format!("2024-01-{:02}", i % 30 + 1))
            .collect();
        let x_axes = vec![category_axis(AxisPosition::Bottom, categories)];
        let y_axes = vec![category_axis(AxisPosition::Left, vec!["A".into()])];
        let ranges = ResolvedAxisRanges {
            ranges: vec![
                ResolvedAxisRange {
                    axis_index: 0,
                    position: AxisPosition::Bottom,
                    axis_type: AxisType::Category,
                    min: 0.0,
                    max: 100.0,
                    is_user_defined: false,
                    tick_count_hint: None,
                    categories: vec![],
                },
                ResolvedAxisRange {
                    axis_index: 0,
                    position: AxisPosition::Left,
                    axis_type: AxisType::Category,
                    min: 0.0,
                    max: 1.0,
                    is_user_defined: false,
                    tick_count_hint: None,
                    categories: vec![],
                },
            ],
        };
        let colors = ColorContext::default();
        let mut measurer = TextMeasurer::new();

        let elements = CartesianAxisRenderer::render(
            &subplot,
            &x_axes,
            &y_axes,
            &ranges,
            &colors,
            &mut measurer,
        );

        let text_runs: Vec<&VisualElement> = elements
            .iter()
            .filter(|e| matches!(e, VisualElement::TextRun { .. }))
            .collect();
        let rotated = text_runs
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    VisualElement::TextRun { rotation, .. } if *rotation != 0.0
                )
            })
            .count();
        assert!(rotated > 0, "密集长标签应自动旋转");
        assert!(
            text_runs.len() < 100,
            "旋转后仍放不下时按步长抽稀，实际渲染 {} 个标签",
            text_runs.len()
        );
    }

    #[test]
    fn test_top_x_axis_line_and_labels_above() {
        let subplot = SubplotSpec {
            id: 0,
            bounds: Rect::new(0.0, 40.0, 400.0, 300.0),
            series_indices: vec![],
            x_axis_indices: vec![0],
            y_axis_indices: vec![0],
        };
        let x_axes = vec![category_axis(
            AxisPosition::Top,
            vec!["周一".into(), "周二".into(), "周三".into()],
        )];
        let y_axes = vec![category_axis(AxisPosition::Left, vec!["A".into()])];
        let ranges = ResolvedAxisRanges {
            ranges: vec![
                ResolvedAxisRange {
                    axis_index: 0,
                    position: AxisPosition::Top,
                    axis_type: AxisType::Category,
                    min: 0.0,
                    max: 3.0,
                    is_user_defined: false,
                    tick_count_hint: None,
                    categories: vec![],
                },
                ResolvedAxisRange {
                    axis_index: 0,
                    position: AxisPosition::Left,
                    axis_type: AxisType::Category,
                    min: 0.0,
                    max: 1.0,
                    is_user_defined: false,
                    tick_count_hint: None,
                    categories: vec![],
                },
            ],
        };
        let colors = ColorContext::default();
        let mut measurer = TextMeasurer::new();

        let elements = CartesianAxisRenderer::render(
            &subplot,
            &x_axes,
            &y_axes,
            &ranges,
            &colors,
            &mut measurer,
        );

        // 轴线画在绘图区上边缘
        let has_top_line = elements.iter().any(|e| {
            matches!(
                e,
                VisualElement::Line { start, end, .. }
                    if start.y == 40.0 && end.y == 40.0 && start.x == 0.0 && end.x == 400.0
            )
        });
        assert!(has_top_line, "顶部 X 轴轴线应位于绘图区上边缘");

        // 标签绘制在绘图区上方
        let labels_above: Vec<&VisualElement> = elements
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    VisualElement::TextRun { position, .. } if position.y < 40.0 && position.y >= 0.0
                )
            })
            .collect();
        assert!(
            labels_above.len() >= 3,
            "顶部 X 轴标签应绘制在绘图区上方，实际 {} 个",
            labels_above.len()
        );
    }

    #[test]
    fn test_rotated_bottom_x_label_stays_below_axis() {
        // 底部 X 轴 + 长日期标签 → 自动旋转 90°。
        // 回归：旋转标签的文本开头（左上角，= 旋转中心）应对准数据点正下方
        // （轴线 y1 + 14px），文本整体向下延伸、不跨过坐标轴侵入绘图区。
        let subplot = SubplotSpec {
            id: 0,
            bounds: Rect::new(50.0, 30.0, 750.0, 500.0), // x1=750, y1=500
            series_indices: vec![],
            x_axis_indices: vec![0],
            y_axis_indices: vec![0],
        };
        let x_axes = vec![category_axis(
            AxisPosition::Bottom,
            vec![
                "2026-07-27 00:00:00 +0800 CST".into(),
                "2026-07-28 00:00:00 +0800 CST".into(),
                "2026-07-29 00:00:00 +0800 CST".into(),
                "2026-07-30 00:00:00 +0800 CST".into(),
                "2026-07-31 00:00:00 +0800 CST".into(),
                "2026-08-01 00:00:00 +0800 CST".into(),
                "2026-08-02 00:00:00 +0800 CST".into(),
            ],
        )];
        let y_axes = vec![category_axis(AxisPosition::Left, vec!["A".into()])];
        let ranges = ResolvedAxisRanges {
            ranges: vec![
                ResolvedAxisRange {
                    axis_index: 0,
                    position: AxisPosition::Bottom,
                    axis_type: AxisType::Category,
                    min: 0.0,
                    max: 7.0,
                    is_user_defined: false,
                    tick_count_hint: None,
                    categories: vec![],
                },
                ResolvedAxisRange {
                    axis_index: 0,
                    position: AxisPosition::Left,
                    axis_type: AxisType::Category,
                    min: 0.0,
                    max: 1.0,
                    is_user_defined: false,
                    tick_count_hint: None,
                    categories: vec![],
                },
            ],
        };
        let colors = ColorContext::default();
        let mut measurer = TextMeasurer::new();

        let elements = CartesianAxisRenderer::render(
            &subplot,
            &x_axes,
            &y_axes,
            &ranges,
            &colors,
            &mut measurer,
        );

        let rotated_x_labels: Vec<&VisualElement> = elements
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    VisualElement::TextRun { position, rotation, .. }
                        if *rotation != 0.0 && position.x >= subplot.bounds.x0 && position.x <= subplot.bounds.x1
                )
            })
            .collect();
        assert!(!rotated_x_labels.is_empty(), "长标签应自动旋转");
        for label in &rotated_x_labels {
            if let VisualElement::TextRun { position, .. } = label {
                // 文本开头（旋转中心）应贴齐锚点（y1 + 14），不能上移到 y1 之上（侵入绘图区）
                let expected_top = subplot.bounds.y1 + 14.0;
                assert!(
                    (*position).y >= expected_top - 1.0,
                    "旋转标签文本开头应位于轴线下方（>={}), 实际 y={}",
                    expected_top,
                    position.y
                );
            }
        }
    }
}
