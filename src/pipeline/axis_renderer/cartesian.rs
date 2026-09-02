//! 笛卡尔坐标轴渲染器
//!
//! 处理 X/Y 轴线和网格线的生成。内部根据轴类型（Value / Category）分支
//! 处理不同的刻度计算和标签生成策略。

use lievisual::{
    Color,
    scene::{Element, SceneNode, Stroke},
    text::{RichSpan, TextAlign, measure_text},
};
use vello_cpu::kurbo::{Point, Rect};

use super::axis_ticks;
use crate::pipeline::{
    axis_label::{
        auto_rotate, format_label, label_step, label_style, measure_labels, rotated_bounds,
    },
    builder::{Z_AXIS, Z_GRID, Z_LABEL, text_el},
    types::{
        AxisPosition, AxisSpec, AxisType, ColorContext, ResolvedAxisRanges, SubplotSpec,
        TextMeasurer,
    },
};

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

/// 生成单个 X 轴标签。
///
/// - 未旋转（0°）：文本块顶部居中对齐锚点（水平居中于数据点）。
/// - 旋转时：文本块开头（左上角）对准锚点（数据点），围绕该点旋转。
///   底部 X 轴顺时针旋转（文本向右下延伸）、顶部 X 轴逆时针旋转（文本向右上延伸）。
fn push_x_label(
    elements: &mut Vec<SceneNode>,
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
    let mut s = style;
    s.rotation = draw_rotation;
    elements.push(text_el(text.to_string(), Point::new(x, y), s, Z_LABEL));
}

/// 生成单个 Y 轴刻度标签（垂直墨迹盒中心对齐 + 贴轴侧后端锚定）。
///
/// - **垂直**：墨迹盒（`TextLayout::ink_bounds`，skrifa 按字形轮廓计算，相对
///   块左上角）的中心精确落在刻度 y。此前未旋转分支用 `TextBaseline::Middle`
///   （后端实现为行盒启发式 `alphabetic_baseline − em_height_ascent×0.5`），
///   旋转分支用行盒旋转 AABB 近似——对齐的都是**行盒**而非墨迹：行盒含
///   ascent/descent 设计边距，且墨迹随内容（数字、逗号/括号下探、负号、CJK）
///   变化，产生 1~2px 的内容相关偏差（表现为"标签看着稍偏下"）。与图例文本
///   对 symbol 的对齐方式（`decorator/legend.rs`）同源。
/// - **水平（贴轴侧）**：交给后端自身的 advance 盒锚定——左轴
///   `align=Right`（SVG `text-anchor=end`）、右轴 `align=Left`（start），
///   文字贴轴侧的边缘由渲染器钉在锚点 x 上，间隙 = 锚点预留（8px）− 刻度
///   长度（5px）− side bearing ≈ 3px，与历史行为一致。
///
///   历史坑：曾改为"墨迹盒贴轴侧边缘对齐锚点"（`align=Left` + parley 墨迹
///   宽反推 position.x），PNG 后端（用 parley 直接画 glyph）是精确的，但
///   SVG 后端的 `<text>` 由**浏览器排版**——浏览器解析 `sans-serif` 得到的
///   字体 advance 与 parley 嵌入字体不一致，文字实际画得比 parley 墨迹宽，
///   右缘向右越过预留间隙、与刻度重叠（"标签和 tick 连在一起"）。
///   advance 盒锚定对字体差异免疫（两端文字同时向画布外侧伸缩）。
///
/// 旋转（含 0°）统一处理：`baseline = Top`（锚点 = 块左上角）时，两个后端
/// 均以 `position` 为旋转中心、把 advance 盒按 align 平移后旋转，墨迹中心
/// 最终位置 = `position + R(θ)·(icx + dx_align, icy)`，反解 y：
/// `position.y = tick_y − [R(θ)·(icx + dx_align, icy)]_y`；x 直接用锚点
/// （贴轴侧由 align/advance 锚定，无需墨迹反推）。
fn push_y_tick_label(
    elements: &mut Vec<SceneNode>,
    text: &str,
    anchor: Point,
    rotation: f64,
    colors: &ColorContext,
    is_right: bool,
) {
    let style = label_style(colors); // Left/Top 语义：位置即预计算的块左上角
    let mut style = style;
    // 贴轴侧锚定：左轴文字向左伸展（advance 右缘 = 锚点），右轴相反
    style.align = if is_right {
        TextAlign::Left
    } else {
        TextAlign::Right
    };
    style.rotation = rotation; // 旋转仅作用于渲染变换，不影响 layout 测量
    let layout = measure_text(
        &[RichSpan::new(text.to_string(), style.clone())],
        style.max_width,
    )
    .layout;
    let ink = layout.ink_bounds();
    let icx = (ink.min_x() + ink.max_x()) / 2.0;
    let icy = (ink.min_y() + ink.max_y()) / 2.0;

    // 后端语义：块原点 = position，advance 盒按 align 平移 dx（Right →
    // −width，Left → 0），再绕 position 旋转 θ。
    let dx = if is_right { 0.0 } else { -layout.width };
    let (sn, cs) = rotation.sin_cos();
    // 墨迹中心相对锚点的最终 y 偏移：[R(θ)·(icx + dx, icy)]_y
    let py = anchor.y - ((icx + dx) * sn + icy * cs);

    elements.push(
        SceneNode::new(Element::Text {
            spans: vec![RichSpan::new(text.to_string(), style.clone())],
            position: Point::new(anchor.x, py),
            style,
            layout: Some(layout),
        })
        .with_z(Z_LABEL),
    );
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
    ) -> Vec<SceneNode> {
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

                // X 轴刻度短线（ECharts 默认 axisTick.show = true）
                Self::draw_x_ticks(&mut elements, bounds, axis_cfg, x_min, x_max, colors);

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

                // Y 轴刻度短线（ECharts 默认 axisTick.show = true）
                Self::draw_y_ticks(
                    &mut elements,
                    bounds,
                    axis_cfg,
                    y_min,
                    y_max,
                    colors,
                    is_right,
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

    fn draw_axis_line(elements: &mut Vec<SceneNode>, start: Point, end: Point, color: Color) {
        elements.push(crate::pipeline::builder::line(
            start,
            end,
            Stroke::new(color, 1.0),
            Z_AXIS,
        ));
    }

    /// 绘制 X 轴刻度短线：从轴线向外延伸 5px，与刻度标签位置对齐。
    fn draw_x_ticks(
        elements: &mut Vec<SceneNode>,
        bounds: Rect,
        axis_cfg: &AxisSpec,
        x_min: f64,
        x_max: f64,
        colors: &ColorContext,
    ) {
        const TICK_LEN: f64 = 5.0;
        let axis_y = if axis_cfg.position == AxisPosition::Top {
            bounds.y0
        } else {
            bounds.y1
        };
        // 朝轴线外侧延伸：底部轴向下、顶部轴向上
        let (y1, y2) = if axis_cfg.position == AxisPosition::Top {
            (axis_y - TICK_LEN, axis_y)
        } else {
            (axis_y, axis_y + TICK_LEN)
        };
        let xs: Vec<f64> = if axis_cfg.axis_type == AxisType::Category {
            let n = axis_cfg.categories.len();
            (0..n)
                .map(|i| bounds.x0 + (i as f64 + 0.5) / n as f64 * bounds.width())
                .collect()
        } else {
            let (positions, _) = axis_ticks(axis_cfg.axis_type, x_min, x_max);
            positions
                .iter()
                .map(|&t| bounds.x0 + t * bounds.width())
                .collect()
        };
        for x in xs {
            elements.push(crate::pipeline::builder::line(
                Point::new(x, y1),
                Point::new(x, y2),
                Stroke::new(colors.axis_line_color, 1.0),
                Z_AXIS,
            ));
        }
    }

    /// 绘制 Y 轴刻度短线：从轴线向外延伸 5px，与刻度标签位置对齐。
    fn draw_y_ticks(
        elements: &mut Vec<SceneNode>,
        bounds: Rect,
        axis_cfg: &AxisSpec,
        y_min: f64,
        y_max: f64,
        colors: &ColorContext,
        is_right: bool,
    ) {
        const TICK_LEN: f64 = 5.0;
        let axis_x = if is_right { bounds.x1 } else { bounds.x0 };
        // 朝轴线外侧延伸：左轴向左、右轴向右
        let (x1, x2) = if is_right {
            (axis_x, axis_x + TICK_LEN)
        } else {
            (axis_x - TICK_LEN, axis_x)
        };
        let ys: Vec<f64> = if axis_cfg.axis_type == AxisType::Category {
            let n = axis_cfg.categories.len();
            (0..n)
                .map(|i| bounds.y1 - (i as f64 + 0.5) / n as f64 * bounds.height())
                .collect()
        } else {
            let (positions, _) = axis_ticks(axis_cfg.axis_type, y_min, y_max);
            positions
                .iter()
                .map(|&t| bounds.y0 + (1.0 - t) * bounds.height())
                .collect()
        };
        for y in ys {
            elements.push(crate::pipeline::builder::line(
                Point::new(x1, y),
                Point::new(x2, y),
                Stroke::new(colors.axis_line_color, 1.0),
                Z_AXIS,
            ));
        }
    }

    fn draw_x_grid_lines(
        elements: &mut Vec<SceneNode>,
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
                    elements.push(crate::pipeline::builder::line(
                        Point::new(x, bounds.y0),
                        Point::new(x, bounds.y1),
                        Stroke::new(colors.grid_line_color, 0.5),
                        Z_GRID,
                    ));
                }
            }
        } else {
            // Value / Time / Log 轴：网格线位置与刻度位置一致
            let (positions, _labels) = axis_ticks(axis_cfg.axis_type, x_min, x_max);
            for t in positions {
                let x = bounds.x0 + t * bounds.width();
                elements.push(crate::pipeline::builder::line(
                    Point::new(x, bounds.y0),
                    Point::new(x, bounds.y1),
                    Stroke::new(colors.grid_line_color, 0.5),
                    Z_GRID,
                ));
            }
        }
    }

    fn draw_y_grid_lines(
        elements: &mut Vec<SceneNode>,
        bounds: Rect,
        axis_cfg: &AxisSpec,
        y_min: f64,
        y_max: f64,
        colors: &ColorContext,
    ) {
        let (positions, _labels) = axis_ticks(axis_cfg.axis_type, y_min, y_max);
        for t in positions {
            // Y 轴自下而上：t=(v-min)/(max-min)，像素 y = y0 + (1-t)*height
            let y = bounds.y0 + (1.0 - t) * bounds.height();
            elements.push(crate::pipeline::builder::line(
                Point::new(bounds.x0, y),
                Point::new(bounds.x1, y),
                Stroke::new(colors.grid_line_color, 0.5),
                Z_GRID,
            ));
        }
    }

    fn draw_x_tick_labels(
        elements: &mut Vec<SceneNode>,
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
            // Value / Time / Log 轴：统一通过 axis_ticks 生成刻度位置与标签
            let (norm_positions, tick_labels) = axis_ticks(axis_cfg.axis_type, x_min, x_max);
            if tick_labels.is_empty() {
                return;
            }
            let labels: Vec<String> = tick_labels
                .iter()
                .map(|label| format_label(label, &axis_cfg.label_formatter))
                .collect();
            // 刻度像素位置（X 轴自左向右，t=(v-min)/(max-min)）
            let positions: Vec<f64> = norm_positions
                .iter()
                .map(|&t| bounds.x0 + t * bounds.width())
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
            for i in (0..positions.len()).step_by(step) {
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
            let last_idx = positions.len() - 1;
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
        elements: &mut Vec<SceneNode>,
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
        let mut x = if is_right {
            bounds.x1 + 8.0
        } else {
            bounds.x0 - 8.0
        };

        // 先排版得到刻度标签的实际尺寸，再据此校准锚点，避免左轴标签超出画布左边界。
        // 对左轴（右对齐）锚点至少为 `max_label_w + 8`，确保标签左边缘距画布边界 ≥ 8px；
        // 若轴带名称，再预留名称带（旋转后厚度约 15px + 8px 间隙 + 名称锚点余量），
        // 使标签不与轴名称重叠（名称带由 GridPlanner 同步预留左侧空间）。
        let has_axis_name = axis_cfg.name.is_some();
        let mut adjust_left_anchor = |labels: &[String],
                                      rotation: f64,
                                      text_measurer: &mut TextMeasurer,
                                      colors: &ColorContext| {
            if is_right {
                return;
            }
            let style = label_style(colors);
            let mut max_proj_w: f64 = 0.0;
            for l in labels {
                let (w, h) = text_measurer.measure(l, &style);
                let (proj_w, _) = rotated_bounds(w, h, rotation);
                max_proj_w = max_proj_w.max(proj_w);
            }
            let name_band: f64 = if has_axis_name { 30.0 } else { 0.0 };
            let min_anchor = max_proj_w + 8.0 + name_band;
            if x < min_anchor {
                x = min_anchor;
            }
        };

        // Y 轴刻度标签：由模块级 `push_y_tick_label` 统一生成（墨迹盒对齐）

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
            adjust_left_anchor(&labels, rotation, text_measurer, colors);

            // 与柱状图渲染保持一致：category 0 在底部，category n-1 在顶部
            let mut last_rendered: Option<usize> = None;
            for i in (0..n).step_by(step) {
                last_rendered = Some(i);
                let t = (i as f64 + 0.5) / n as f64;
                let y = bounds.y1 - t * bounds.height();
                push_y_tick_label(
                    elements,
                    &labels[i],
                    Point::new(x, y),
                    rotation,
                    colors,
                    is_right,
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
                    push_y_tick_label(
                        elements,
                        &labels[last_idx],
                        Point::new(x, y),
                        rotation,
                        colors,
                        is_right,
                    );
                }
            }
            return;
        }

        // Value / Time / Log 轴：统一通过 axis_ticks 生成刻度位置与标签
        let (norm_positions, tick_labels) = axis_ticks(axis_cfg.axis_type, y_min, y_max);
        let labels: Vec<String> = tick_labels
            .iter()
            .map(|label| format_label(label, &axis_cfg.label_formatter))
            .collect();
        // Y 轴自下而上：t=(v-min)/(max-min)，像素 y = y0 + (1-t)*height
        let positions: Vec<f64> = norm_positions
            .iter()
            .map(|&t| bounds.y0 + (1.0 - t) * bounds.height())
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
        adjust_left_anchor(&labels, rotation, text_measurer, colors);

        for i in (0..positions.len()).step_by(step) {
            push_y_tick_label(
                elements,
                &labels[i],
                Point::new(x, positions[i]),
                rotation,
                colors,
                is_right,
            );
        }
        let last_idx = positions.len() - 1;
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
                push_y_tick_label(
                    elements,
                    &labels[last_idx],
                    Point::new(x, positions[last_idx]),
                    rotation,
                    colors,
                    is_right,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lievisual::scene::Element;
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

        let text_runs: Vec<&SceneNode> = elements
            .iter()
            .filter(|e| matches!(&e.element, Element::Text { .. }))
            .collect();
        let rotated = text_runs
            .iter()
            .filter(|e| {
                matches!(
                    &e.element,
                    Element::Text { style, .. } if style.rotation != 0.0
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
                &e.element,
                Element::Line { start, end, .. }
                    if start.y == 40.0 && end.y == 40.0 && start.x == 0.0 && end.x == 400.0
            )
        });
        assert!(has_top_line, "顶部 X 轴轴线应位于绘图区上边缘");

        // 标签绘制在绘图区上方
        let labels_above: Vec<&SceneNode> = elements
            .iter()
            .filter(|e| {
                matches!(
                    &e.element,
                    Element::Text { position, .. } if position.y < 40.0 && position.y >= 0.0
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

        let rotated_x_labels: Vec<&SceneNode> = elements
            .iter()
            .filter(|e| {
                matches!(
                    &e.element,
                    Element::Text { position, style, .. }
                        if style.rotation != 0.0 && position.x >= subplot.bounds.x0 && position.x <= subplot.bounds.x1
                )
            })
            .collect();
        assert!(!rotated_x_labels.is_empty(), "长标签应自动旋转");
        for label in &rotated_x_labels {
            if let Element::Text { position, .. } = &label.element {
                // 文本开头（旋转中心）应贴齐锚点（y1 + 14），不能上移到 y1 之上（侵入绘图区）
                let expected_top = subplot.bounds.y1 + 14.0;
                assert!(
                    position.y >= expected_top - 1.0,
                    "旋转标签文本开头应位于轴线下方（>={}), 实际 y={}",
                    expected_top,
                    position.y
                );
            }
        }
    }

    fn value_axis(position: AxisPosition, label_rotate: Option<f64>) -> AxisSpec {
        AxisSpec {
            axis_type: AxisType::Value,
            position,
            grid_index: 0,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories: vec![],
            boundary_gap: false,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }
    }

    /// 收集所有 Text 元素「墨迹盒中心」的最终像素 y。
    ///
    /// 与 `push_y_tick_label` 的定位通式一致：墨迹中心 = `position +
    /// R(θ)·(icx + dx_align, icy)`，其中 `dx_align = −width`（align=Right）/
    /// `0`（align=Left）——与两个后端「advance 盒按 align 平移后绕 position
    /// 旋转」的语义一致。
    fn rendered_ink_center_y(elements: &[SceneNode]) -> Vec<f64> {
        elements
            .iter()
            .filter_map(|e| {
                let Element::Text {
                    position,
                    style,
                    layout,
                    ..
                } = &e.element
                else {
                    return None;
                };
                let layout = layout.as_ref()?;
                let ink = layout.ink_bounds();
                let icx = (ink.min_x() + ink.max_x()) / 2.0;
                let icy = (ink.min_y() + ink.max_y()) / 2.0;
                let dx = match style.align {
                    TextAlign::Right => -layout.width,
                    _ => 0.0,
                };
                let (sn, cs) = style.rotation.sin_cos();
                Some(position.y + (icx + dx) * sn + icy * cs)
            })
            .collect()
    }

    /// 计算 Text 元素旋转后墨迹盒的最大像素 x（左轴墨迹右缘 = 贴轴一侧）。
    fn rendered_ink_max_x(e: &SceneNode) -> Option<f64> {
        let Element::Text {
            position,
            style,
            layout,
            ..
        } = &e.element
        else {
            return None;
        };
        let layout = layout.as_ref()?;
        let ink = layout.ink_bounds();
        let dx = match style.align {
            TextAlign::Right => -layout.width,
            _ => 0.0,
        };
        let (sn, cs) = style.rotation.sin_cos();
        let corners = [
            (ink.min_x(), ink.min_y()),
            (ink.max_x(), ink.min_y()),
            (ink.min_x(), ink.max_y()),
            (ink.max_x(), ink.max_y()),
        ];
        let max_px = corners
            .iter()
            .map(|&(x, y)| (x + dx) * cs - y * sn)
            .fold(f64::NEG_INFINITY, f64::max);
        Some(position.x + max_px)
    }

    /// Y 轴刻度标签的墨迹盒中心必须精确落在刻度像素 y 上（未旋转）。
    ///
    /// 回归：此前 `TextBaseline::Middle` 对齐的是行盒启发式
    /// （`alphabetic_baseline − em_height_ascent×0.5`），行盒含设计边距且墨迹
    /// 随内容变化（逗号/括号下探等），标签墨迹中心会比刻度低 1~2px
    /// （"标签看着稍偏下"）。
    #[test]
    fn y_tick_label_ink_center_aligns_with_tick() {
        let bounds = Rect::new(60.0, 40.0, 700.0, 500.0);
        let subplot = SubplotSpec {
            id: 0,
            bounds,
            series_indices: vec![],
            x_axis_indices: vec![],
            y_axis_indices: vec![0],
        };
        let y_axes = vec![value_axis(AxisPosition::Left, None)];
        let ranges = ResolvedAxisRanges {
            ranges: vec![ResolvedAxisRange {
                axis_index: 0,
                position: AxisPosition::Left,
                axis_type: AxisType::Value,
                min: 0.0,
                max: 100.0,
                is_user_defined: false,
                tick_count_hint: None,
                categories: vec![],
            }],
        };
        let colors = ColorContext::default();
        let mut measurer = TextMeasurer::new();
        let elements =
            CartesianAxisRenderer::render(&subplot, &[], &y_axes, &ranges, &colors, &mut measurer);

        // 期望刻度 y 集合（与渲染同一公式：y0 + (1-t)*height）
        let (norm_positions, _) = axis_ticks(AxisType::Value, 0.0, 100.0);
        let tick_ys: Vec<f64> = norm_positions
            .iter()
            .map(|&t| bounds.y0 + (1.0 - t) * bounds.height())
            .collect();
        assert!(tick_ys.len() >= 3, "value 轴应产生多个刻度");

        let centers = rendered_ink_center_y(&elements);
        assert!(
            centers.len() >= 3,
            "应渲染出多个 Y 轴刻度标签，实际 {}",
            centers.len()
        );
        for cy in &centers {
            let nearest = tick_ys
                .iter()
                .map(|&ty| (cy - ty).abs())
                .fold(f64::INFINITY, f64::min);
            assert!(
                nearest < 1e-6,
                "标签墨迹中心 y={cy} 应精确对齐某个刻度，最近偏差 {nearest}"
            );
        }
    }

    /// 旋转（90°）的 Y 轴刻度标签：墨迹中心仍须落在刻度 y 上，且墨迹盒整体
    /// 位于轴外侧（左轴墨迹右缘 ≤ 锚点 x），不侵入绘图区。
    #[test]
    fn y_rotated_tick_label_ink_center_aligns_with_tick() {
        let bounds = Rect::new(60.0, 40.0, 700.0, 500.0);
        let subplot = SubplotSpec {
            id: 0,
            bounds,
            series_indices: vec![],
            x_axis_indices: vec![],
            y_axis_indices: vec![0],
        };
        let y_axes = vec![value_axis(AxisPosition::Left, Some(90.0))];
        let ranges = ResolvedAxisRanges {
            ranges: vec![ResolvedAxisRange {
                axis_index: 0,
                position: AxisPosition::Left,
                axis_type: AxisType::Value,
                min: 0.0,
                max: 100.0,
                is_user_defined: false,
                tick_count_hint: None,
                categories: vec![],
            }],
        };
        let colors = ColorContext::default();
        let mut measurer = TextMeasurer::new();
        let elements =
            CartesianAxisRenderer::render(&subplot, &[], &y_axes, &ranges, &colors, &mut measurer);

        let (norm_positions, _) = axis_ticks(AxisType::Value, 0.0, 100.0);
        let tick_ys: Vec<f64> = norm_positions
            .iter()
            .map(|&t| bounds.y0 + (1.0 - t) * bounds.height())
            .collect();

        // 垂直：墨迹中心对齐刻度
        let centers = rendered_ink_center_y(&elements);
        assert!(!centers.is_empty(), "应渲染出旋转的 Y 轴刻度标签");
        for cy in &centers {
            let nearest = tick_ys
                .iter()
                .map(|&ty| (cy - ty).abs())
                .fold(f64::INFINITY, f64::min);
            assert!(
                nearest < 1e-6,
                "旋转标签墨迹中心 y={cy} 应精确对齐某个刻度，最近偏差 {nearest}"
            );
        }

        // 水平：墨迹右缘 ≤ 锚点 x（bounds.x0 − 8），不越过锚点侵入绘图区
        let anchor_x = bounds.x0 - 8.0;
        for e in &elements {
            if let Some(ink_max_x) = rendered_ink_max_x(e) {
                assert!(
                    ink_max_x <= anchor_x + 1e-6,
                    "旋转标签墨迹右缘 {ink_max_x} 应 ≤ 锚点 {anchor_x}（不侵入绘图区）"
                );
            }
        }
    }
}
