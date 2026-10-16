//! Pie Builder: 将 PieSeries 组装为 lievisual `SceneNode`

use std::f64::consts::PI;

use lievisual::{
    Color,
    kurbo::{BezPath, Point},
    scene::{Element, FillStrokeStyle, SceneNode, Stroke},
    text::{RichSpan, TextAlign, TextBaseline, TextStyle, measure_text},
};

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, fill_style, path, resolve_pie_radii},
        typed_series::{LabelPosition, PieSeries, RenderContext},
    },
};

/// 引导线第 2 段（径向）长度：从扇形边缘沿扇区中线外推的距离。
///
/// ECharts v6 `labelLine.length` 实测 = 15px（参考 line-simple.svg：折点半径
/// ≈ r + 15）。
const LABEL_LINE_LEN: f64 = 15.0;
/// 引导线第 3 段（水平）长度：折点到文本边的距离（ECharts `labelLine.length2`
/// 实测 = 30px）。
const LABEL_LINE_END: f64 = 30.0;
/// 引导线终点与文本盒之间的间隙（实测 5px）。
const LABEL_TEXT_GAP: f64 = 5.0;
/// 文本白色描边的 8 向偏移（单位 px）。
///
/// ECharts 用 `stroke=#fff stroke-width=2 paint-order=stroke` 给外置标签加白边；
/// lievisual 的 `TextStyle` 没有文本描边能力，用 8 向 1px 白色副本近似（先铺白字、
/// 再叠正常色文本），视觉上得到一圈约 1px 的白色描边。
const LABEL_HALO_OFFSETS: [(f64, f64); 8] = [
    (-1.0, 0.0),
    (1.0, 0.0),
    (0.0, -1.0),
    (0.0, 1.0),
    (-0.707, -0.707),
    (0.707, -0.707),
    (-0.707, 0.707),
    (0.707, 0.707),
];

/// 饼图外部标签的几何布局（ECharts label layout 思路）
struct PieLabelGeometry {
    /// 标签展示文本
    text: String,
    /// 所属扇区颜色（引导线与文字描边同色，ECharts 语义）
    color: Color,
    /// 标签所在侧（自然位置在圆心的左/右）
    side: PieSide,
    /// 扇区中线角度（弧度，0 = 12 点钟方向，顺时针为正）
    mid_angle: f64,
    /// 引导线第 1 段起点（扇形边缘）
    line_start: Point,
    /// 引导线折点（外圆 `r + LABEL_LINE_LEN` 上）
    line_kink: Point,
    /// 自然锚点 y（折点 y，未做同侧避让）
    natural_y: f64,
    /// 文本锚点 y（同侧避让后）
    text_y: f64,
    /// 文本锚点 x（由最终折点推出）
    text_x: f64,
}

/// 标签所在侧（自然位置在圆心的左/右）
///
/// ECharts 同口径：12 点附近的密集小扇区全部落到同一侧（实测 line-simple.svg
/// 中 6 个 <1% 的扇区标签全在左侧），不做上/下分组。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PieSide {
    Left,
    Right,
}

pub struct PieBuilder;

impl SeriesBuilder<PieSeries> for PieBuilder {
    fn build(series: &PieSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
        let mut elements = Vec::with_capacity(series.slices.len() * 2);

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 饼图在 bounds 中心居中（ECharts 语义 `center: ['50%','50%']`）。
        // 历史实现把圆心下移 5%（0.55）为标题/图例让位，但官方 SVG 实测圆心
        // 就在画布正中（标题/图例不参与饼图的定位），故回退到 0.5。
        let center_x = bounds.x0 + width * 0.5;
        let center_y = bounds.y0 + height * 0.5;
        let center = Point::new(center_x, center_y);

        // P2a：radius 为**绝对像素**（api/compat 层按「画布 min/2」折算）；
        // P5 起由 `resolve_pie_radii` 统一 clamp 到绘图区内接半径，避免多
        // subplot / 紧边距时越界；外径被 clamp 时内径同比例缩放（零宽圆环防护）。
        let (inner_radius, outer_radius) = resolve_pie_radii(
            series.radius_inner,
            series.radius_outer,
            width,
            height,
            75.0,
        );

        let mut start_angle = 0.0; // 从 12 点钟方向开始

        // 第一遍：绘制扇形，并收集外部标签几何（供碰撞避让）
        let mut outside_labels: Vec<PieLabelGeometry> = Vec::new();

        // 玫瑰图：外径按数值缩放（`radius` 线性、`area` 按面积即 √）
        let rose_max = series
            .slices
            .iter()
            .map(|s| s.value.abs())
            .fold(0.0_f64, f64::max)
            .max(1e-9);
        let span = outer_radius - inner_radius;

        for slice in &series.slices {
            let sweep_angle = slice.percent * 2.0 * PI;
            // `padAngle`：扇区两端各内缩 pad/2（不越过扇区中线）
            let pad = (series.pad_angle * 0.5).min(sweep_angle * 0.5).max(0.0);
            // `clockwise: false` 时整圈镜像（角度取负），标签中线同步取负
            let dir = if series.clockwise { 1.0 } else { -1.0 };
            let a0 = dir * (start_angle + pad);
            let a1 = dir * (start_angle + sweep_angle - pad);
            let mid_angle = dir * (start_angle + sweep_angle * 0.5);

            // 玫瑰图扇区外径
            let slice_outer = match series.rose_type {
                Some(crate::pipeline::types::PieRoseType::Radius) => {
                    inner_radius + span * (slice.value.abs() / rose_max).clamp(0.0, 1.0)
                }
                Some(crate::pipeline::types::PieRoseType::Area) => {
                    inner_radius + span * (slice.value.abs() / rose_max).clamp(0.0, 1.0).sqrt()
                }
                None => outer_radius,
            };

            // 绘制扇形
            let arc_path = build_arc_path(center, inner_radius, slice_outer, a0, a1);

            elements.push(path(arc_path, fill_style(slice.color), true, Z_SERIES_FILL));

            // 标签
            if series.label_show {
                match series.label_position {
                    LabelPosition::Inside => {
                        let label_elements = build_inside_label(
                            center,
                            slice_outer,
                            mid_angle,
                            slice,
                            series.label_formatter.as_deref(),
                            ctx,
                        );
                        elements.extend(label_elements);
                    }
                    LabelPosition::Outside => {
                        let text = format_label_text(slice, series.label_formatter.as_deref());
                        if let Some(geo) = compute_label_geometry(
                            center,
                            slice_outer,
                            mid_angle,
                            &text,
                            slice.color,
                        ) {
                            outside_labels.push(geo);
                        }
                    }
                }
            }

            start_angle += sweep_angle;
        }

        // 碰撞避让：对相邻标签做避让，避免重叠
        let resolved = resolve_label_overlap(
            outside_labels,
            series.label_font_size,
            center.x,
            center.y,
            width,
            height,
        );

        // 第二遍：用避让后的几何生成引导线 + 文本
        for geo in resolved {
            elements.extend(emit_label_elements(
                &geo,
                ctx,
                series.label_font_size,
                series.label_line_show,
            ));
        }

        Ok(elements)
    }
}

/// 饼图外部标签的布局（ECharts label layout 思路，算法替换而非常量对齐）。
///
/// 与 ECharts `PieView` + `avoidLabelOverlap` 同源，分两阶段：
///
/// 1. **沿半径外推**（[`compute_label_geometry`] + [`finalize_label_geometry`]）：
///    每个标签的引导线折点始终落在「扇形边缘沿扇区中线外推 `LABEL_LINE_LEN`」
///    的外圆上；文本锚点由折点沿水平方向引出 `LABEL_LINE_END + LABEL_TEXT_GAP`。
///    因此引导线水平段长度恒定（30px + 5px 间隙），与扇区大小无关——实测
///    line-simple.svg 中 8 个标签的水平段全部是 30px。
/// 2. **同侧纵向避让**（[`resolve_side_group`]）：标签只按自然位置分左右两侧；
///    同侧内按「沿该侧自上而下」排序，从最上方标签起逐个
///    `y = max(自然 y, 上一位 y + pitch)` 下推（`pitch = 字号 × 1.2`），
///    自然位置不冲突的标签保持不动（如 82% 大扇区）；整组越界时整体上移，
///    仍放不下才按可用高度压缩步长（保序、不出画布）。
fn resolve_label_overlap(
    labels: Vec<PieLabelGeometry>,
    font_size: f64,
    center_x: f64,
    center_y: f64,
    canvas_width: f64,
    canvas_height: f64,
) -> Vec<PieLabelGeometry> {
    if labels.len() < 2 {
        let mut out = labels;
        for g in out.iter_mut() {
            finalize_label_geometry(g, font_size, center_x, center_y, canvas_width);
        }
        return out;
    }

    let (lefts, rights): (Vec<_>, Vec<_>) =
        labels.into_iter().partition(|g| g.side == PieSide::Left);

    let mut result = Vec::with_capacity(lefts.len() + rights.len());
    result.extend(resolve_side_group(
        lefts,
        PieSide::Left,
        font_size,
        center_x,
        center_y,
        canvas_width,
        canvas_height,
    ));
    result.extend(resolve_side_group(
        rights,
        PieSide::Right,
        font_size,
        center_x,
        center_y,
        canvas_width,
        canvas_height,
    ));
    result
}

/// 单侧标签的纵向避让 + 折点/锚点反推。
fn resolve_side_group(
    mut group: Vec<PieLabelGeometry>,
    side: PieSide,
    font_size: f64,
    center_x: f64,
    center_y: f64,
    canvas_width: f64,
    canvas_height: f64,
) -> Vec<PieLabelGeometry> {
    // 沿该侧「自上而下」排序：左侧角度降序（360°→180°）、右侧角度升序
    // （0°→180°）。半圆内 y 随角度单调，故等价于 natural_y 升序；y 相同的
    // 聚集扇区按角度定序——与官方参考一致（离 12 点最近的标签排最上）。
    let angle_desc = side == PieSide::Left;
    group.sort_by(|a, b| {
        let ord = a
            .mid_angle
            .partial_cmp(&b.mid_angle)
            .unwrap_or(std::cmp::Ordering::Equal);
        if angle_desc { ord.reverse() } else { ord }
    });

    // 自顶向下逐项下推：自然位置不冲突的标签保持原位（ECharts 实测 pitch
    // ≈ 字号 + 2；这里取字号×1.2，为 CJK 下探字形留余量）
    let pitch = font_size * 1.2;
    let mut prev_y = f64::NEG_INFINITY;
    for g in group.iter_mut() {
        g.text_y = if prev_y.is_finite() {
            g.natural_y.max(prev_y + pitch)
        } else {
            g.natural_y
        };
        prev_y = g.text_y;
    }

    // 画布纵向约束：整体上移；仍放不下则按可用高度压缩步长（保序）
    const SAFE_MARGIN: f64 = 8.0;
    let min_y = SAFE_MARGIN;
    let max_y = (canvas_height - SAFE_MARGIN - font_size).max(min_y);
    let n = group.len();
    if n > 0 && group[n - 1].text_y > max_y {
        let overflow = group[n - 1].text_y - max_y;
        if group[0].text_y - overflow >= min_y {
            for g in group.iter_mut() {
                g.text_y -= overflow;
            }
        } else if n > 1 {
            let step = (max_y - min_y) / (n - 1) as f64;
            for (i, g) in group.iter_mut().enumerate() {
                g.text_y = min_y + i as f64 * step;
            }
        }
    }

    for g in group.iter_mut() {
        finalize_label_geometry(g, font_size, center_x, center_y, canvas_width);
    }
    group
}

/// 由避让后的 `text_y` 反推引导线折点与文本锚点 x。
///
/// 折点取「外圆 `r + LABEL_LINE_LEN` 与水平线 `y = text_y` 的交点（该侧解）」，
/// 文本锚点 = 折点 ± (`LABEL_LINE_END` + `LABEL_TEXT_GAP`)；最后把文本盒夹回
/// 画布内（ECharts 同样把标签约束在视口内）。
fn finalize_label_geometry(
    g: &mut PieLabelGeometry,
    font_size: f64,
    center_x: f64,
    center_y: f64,
    canvas_width: f64,
) {
    // 折点半径 = 扇形边缘半径 + 径向长度（玫瑰图扇区外径不一，按各自半径算）
    let kink_r = (g.line_start - Point::new(center_x, center_y)).hypot() + LABEL_LINE_LEN;
    let dy = (g.text_y - center_y).clamp(-kink_r, kink_r);
    let dx = (kink_r * kink_r - dy * dy).max(0.0).sqrt();
    let (kink_x, outward) = match g.side {
        PieSide::Left => (center_x - dx, -1.0),
        PieSide::Right => (center_x + dx, 1.0),
    };
    g.line_kink = Point::new(kink_x, g.text_y);
    g.text_x = kink_x + outward * (LABEL_LINE_END + LABEL_TEXT_GAP);

    // 文本盒不得越出画布（左缘 ≥ 安全边距 / 右缘 ≤ 画布宽 − 安全边距）
    const SAFE_MARGIN: f64 = 8.0;
    let text_w = measure_label_width(&g.text, font_size);
    match g.side {
        PieSide::Left => {
            g.text_x = g
                .text_x
                .max(SAFE_MARGIN + text_w)
                .min(canvas_width - SAFE_MARGIN)
        }
        PieSide::Right => {
            g.text_x = g
                .text_x
                .max(SAFE_MARGIN)
                .min(canvas_width - SAFE_MARGIN - text_w)
        }
    }
}

/// 标签文本的自然宽度（parley 实测，与渲染口径一致）。
fn measure_label_width(text: &str, font_size: f64) -> f64 {
    let style = TextStyle::new(Color::rgb(0, 0, 0), font_size, "sans-serif");
    measure_text(&[RichSpan::new(text.to_string(), style)], None)
        .layout
        .width
}

/// 计算饼图外部标签的自然几何（引导线两段 + 文本锚点）。
///
/// 只算「沿半径外推」的自然位置：折点 = 扇形边缘沿扇区中线外推
/// `LABEL_LINE_LEN`，文本锚点 y = 折点 y；x 留待同侧避让后由
/// [`finalize_label_geometry`] 反推（水平段长度恒定，与扇区大小无关）。
fn compute_label_geometry(
    center: Point,
    slice_outer: f64,
    mid_angle: f64,
    text: &str,
    color: Color,
) -> Option<PieLabelGeometry> {
    // 将角度转换为标准坐标系（0°=右侧，π/2=下方）
    let angle = -PI / 2.0 + mid_angle;
    let (c, s) = (angle.cos(), angle.sin());

    // 引导线第 1 段起点：扇形边缘（沿扇区中线）
    let line_start = Point::new(center.x + slice_outer * c, center.y + slice_outer * s);
    // 自然折点：沿扇区中线外推 `LABEL_LINE_LEN`
    let kink_r = slice_outer + LABEL_LINE_LEN;
    let line_kink = Point::new(center.x + kink_r * c, center.y + kink_r * s);

    Some(PieLabelGeometry {
        text: text.to_string(),
        color,
        // 自然位置在圆心哪一侧，标签就归哪侧（12 点附近的密集小扇区会全部
        // 落到同侧，与官方 SVG 一致）
        side: if c < 0.0 {
            PieSide::Left
        } else {
            PieSide::Right
        },
        mid_angle,
        line_start,
        line_kink,
        natural_y: line_kink.y,
        text_y: line_kink.y,
        text_x: line_kink.x,
    })
}

/// 用最终几何生成引导线 + 文本元素。
fn emit_label_elements(
    geo: &PieLabelGeometry,
    ctx: &RenderContext,
    font_size: f64,
    show_guide_line: bool,
) -> Vec<SceneNode> {
    let mut elements = Vec::new();

    // 引导线第 3 段终点：指向文本侧边中点
    let line_end = match geo.side {
        PieSide::Left => Point::new(geo.text_x + LABEL_TEXT_GAP, geo.text_y),
        PieSide::Right => Point::new(geo.text_x - LABEL_TEXT_GAP, geo.text_y),
    };

    // 绘制引导线（两段折线）：
    // 第 2 段（径向）：扇形边缘(line_start) -> 折点(line_kink，外圆上)
    // 第 3 段（水平）：折点 -> 文本侧边(line_end)，长度恒定
    // 颜色取**扇区色**（ECharts 语义：labelLine 继承系列色，实测 line-simple.svg
    // 中引导线 stroke 与扇区 fill 一致）；`labelLine.show: false` 时不画。
    if show_guide_line {
        let mut guide_path = BezPath::new();
        guide_path.move_to(geo.line_start);
        guide_path.line_to(geo.line_kink);
        guide_path.line_to(line_end);

        elements.push(path(
            guide_path,
            FillStrokeStyle {
                fill: None,
                stroke: Some(Stroke::new(geo.color, 1.0)),
            },
            false,
            Z_SERIES_FILL + 1,
        ));
    }

    // 文本对齐：按所在侧做侧对齐，垂直居中（与引导线终点同高）
    let (align, baseline) = match geo.side {
        PieSide::Left => (TextAlign::Right, TextBaseline::Middle),
        PieSide::Right => (TextAlign::Left, TextBaseline::Middle),
    };

    let mut style = TextStyle::new(ctx.colors.text_color, font_size, "sans-serif");
    style.align = align;
    style.baseline = baseline;

    // 白色描边底衬：ECharts 用 `stroke=#fff stroke-width=2 paint-order=stroke`
    // 让标签压在扇区/引导线上仍可读。lievisual 的 TextStyle 没有文本描边能力，
    // 这里用「8 向 1px 白色副本」近似：先铺白字，再叠正常色文本。
    for (dx, dy) in LABEL_HALO_OFFSETS {
        let mut halo_style = style.clone();
        halo_style.color = Color::WHITE;
        elements.push(
            SceneNode::new(Element::Text {
                spans: vec![RichSpan::new(geo.text.clone(), halo_style.clone())],
                position: Point::new(geo.text_x + dx, geo.text_y + dy),
                style: halo_style,
                layout: None,
            })
            .with_z(Z_SERIES_FILL + 2),
        );
    }

    elements.push(
        SceneNode::new(Element::Text {
            spans: vec![RichSpan::new(geo.text.clone(), style.clone())],
            position: Point::new(geo.text_x, geo.text_y),
            style,
            layout: None,
        })
        .with_z(Z_SERIES_FILL + 2),
    );

    elements
}

/// 构建饼图内部标签（放于扇形内部中心，无需碰撞避让）。
fn build_inside_label(
    center: Point,
    outer_radius: f64,
    mid_angle: f64,
    slice: &crate::pipeline::typed_series::PieSlice,
    formatter: Option<&str>,
    _ctx: &RenderContext,
) -> Vec<SceneNode> {
    let mut elements = Vec::new();

    // 将角度转换为标准坐标系
    let angle = -PI / 2.0 + mid_angle;

    // 标签文本
    let label_text = format_label_text(slice, formatter);

    // 内部标签：放在扇形中心
    let label_radius = outer_radius * 0.7;
    let label_x = center.x + label_radius * angle.cos();
    let label_y = center.y + label_radius * angle.sin();

    let mut style = TextStyle::new(Color::rgb(255, 255, 255), 12.0, "sans-serif");
    style.align = TextAlign::Center;
    style.baseline = TextBaseline::Middle;
    elements.push(
        SceneNode::new(Element::Text {
            spans: vec![RichSpan::new(label_text, style.clone())],
            position: Point::new(label_x, label_y),
            style,
            layout: None,
        })
        .with_z(Z_SERIES_FILL + 2),
    );

    elements
}

/// 格式化饼图标签文本。
///
/// 通过统一的模板引擎替换 ECharts 占位符：
/// - `{b}`：数据项名称（`slice.name`）
/// - `{c}`：数据项数值（`slice.value`）
/// - `{d}`：百分比（`slice.percent`，保留 2 位小数，与 ECharts 一致）
/// - `{a}`/`{name}`：系列名
///
/// 未提供模板时，回退到默认的 `"名称 百分比%"` 格式。
fn format_label_text(
    slice: &crate::pipeline::typed_series::PieSlice,
    formatter: Option<&str>,
) -> String {
    crate::pipeline::template::render_template(
        formatter,
        &crate::pipeline::template::TemplateContext {
            series_name: Some(&slice.name),
            name: Some(&slice.name),
            value: Some(slice.value),
            percent: Some(slice.percent * 100.0),
        },
        &format!("{} {:.2}%", slice.name, slice.percent * 100.0),
    )
}

/// 构建扇形路径（使用真正的圆弧）
fn build_arc_path(
    center: Point,
    inner_radius: f64,
    outer_radius: f64,
    start_angle: f64,
    end_angle: f64,
) -> BezPath {
    let mut path = BezPath::new();

    // 将角度转换为标准坐标系（从 -PI/2 开始，顺时针）
    let start = -PI / 2.0 + start_angle;
    let end = -PI / 2.0 + end_angle;

    // 外圆弧起点
    let outer_start = Point::new(
        center.x + outer_radius * start.cos(),
        center.y + outer_radius * start.sin(),
    );

    path.move_to(outer_start);

    // 外圆弧终点（用于计算，但不直接使用）
    let _outer_end = Point::new(
        center.x + outer_radius * end.cos(),
        center.y + outer_radius * end.sin(),
    );

    // 使用椭圆弧命令绘制外圆弧
    let large_arc = (end - start).abs() > PI;
    add_arc_eliptical(&mut path, center, outer_radius, start, end, large_arc);

    if inner_radius > 0.0 {
        // 环形饼图
        // 内圆弧终点
        let inner_end = Point::new(
            center.x + inner_radius * end.cos(),
            center.y + inner_radius * end.sin(),
        );
        path.line_to(inner_end);

        // 使用椭圆弧命令绘制内圆弧（反向）
        add_arc_eliptical(&mut path, center, inner_radius, end, start, large_arc);

        // 内圆弧起点（连接回外圆弧起点）
        let _inner_start = Point::new(
            center.x + inner_radius * start.cos(),
            center.y + inner_radius * start.sin(),
        );
        path.line_to(outer_start);
    } else {
        // 实心饼图，连接回中心
        path.line_to(center);
        path.line_to(outer_start);
    }

    path.close_path();
    path
}

/// 添加椭圆弧到路径（使用 SVG 风格的圆弧）
/// 将圆弧分割为最多 4 段，每段使用三次贝塞尔曲线精确近似
fn add_arc_eliptical(
    path: &mut BezPath,
    center: Point,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    _large_arc: bool,
) {
    // 计算角度差
    let delta_angle = end_angle - start_angle;

    // 将圆弧分割为段，每段最多 PI/2（90度）
    let num_segments = ((delta_angle.abs() / (PI / 2.0)).ceil() as usize).max(1);
    let segment_angle = delta_angle / num_segments as f64;

    for i in 0..num_segments {
        let a1 = start_angle + segment_angle * i as f64;
        let a2 = start_angle + segment_angle * (i + 1) as f64;

        // 计算这段圆弧的贝塞尔曲线控制点
        // 使用常数 k = 4/3 * tan(θ/4) 来近似圆弧。
        // 注意：k 必须**带符号**（即取决于 segment_angle 的符号）。环形图的内弧
        // 是反向绘制的（end -> start，segment_angle < 0），若对 θ 取绝对值，
        // 控制点会落在切线反方向：内弧大幅变形、路径自交，圆环渲染错乱。
        // 详见回归测试 `annular_arc_inner_edge_stays_on_inner_radius`。
        let theta = segment_angle;
        let k = (theta / 4.0).tan() * 4.0 / 3.0;

        // 点相对于圆心的坐标
        let _p1 = Point::new(radius * a1.cos(), radius * a1.sin());
        let p2 = Point::new(radius * a2.cos(), radius * a2.sin());

        // 控制点（相对于圆心）
        let cp1 = Point::new(
            radius * (a1.cos() - k * a1.sin()),
            radius * (a1.sin() + k * a1.cos()),
        );
        let cp2 = Point::new(
            radius * (a2.cos() + k * a2.sin()),
            radius * (a2.sin() - k * a2.cos()),
        );

        // 转换为绝对坐标
        let cp1_abs = Point::new(center.x + cp1.x, center.y + cp1.y);
        let cp2_abs = Point::new(center.x + cp2.x, center.y + cp2.y);
        let p2_abs = Point::new(center.x + p2.x, center.y + p2.y);

        path.curve_to(cp1_abs, cp2_abs, p2_abs);
    }
}

#[cfg(test)]
mod tests {
    use lievisual::kurbo::{PathEl, Point};

    use super::*;

    /// 构造左侧标签的自然几何（`mid_angle` 以弧度计，0 = 12 点钟方向）。
    fn mk_label(mid_angle: f64, text: &str) -> PieLabelGeometry {
        let center = Point::new(400.0, 300.0);
        compute_label_geometry(center, 200.0, mid_angle, text, Color::rgb(80, 112, 221)).unwrap()
    }

    /// 环形扇区的内弧必须贴合内半径。
    ///
    /// 回归：内弧是反向绘制的（end -> start），贝塞尔近似常数 k=4/3·tan(θ/4)
    /// 必须带符号；曾因对 θ 取 `abs` 使内弧控制点落到切线反方向，内弧朝错误
    /// 方向鼓出、路径自交（甜甜圈图形状扭曲）。
    #[test]
    fn annular_arc_inner_edge_stays_on_inner_radius() {
        let center = Point::new(0.0, 0.0);
        let inner = 50.0;
        let outer = 100.0;
        // 90° 扇区（单段贝塞尔）
        let path = build_arc_path(center, inner, outer, 0.0, PI / 2.0);

        let mut prev = center;
        let mut inner_curves = 0usize;
        for el in path.elements() {
            match *el {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => prev = p,
                PathEl::CurveTo(c1, c2, p) => {
                    // 起点在内半径上 => 这段是内弧（外弧起点在外半径上）
                    let d0 = (prev - center).hypot();
                    if (d0 - inner).abs() < 1e-6 {
                        inner_curves += 1;
                        // 三次贝塞尔在 t=0.5 处：B = (P0 + 3·C1 + 3·C2 + P3) / 8
                        // （全部以圆心为原点用 Vec2 计算，Point 不支持这些运算）
                        let mid = ((prev - center)
                            + (c1 - center) * 3.0
                            + (c2 - center) * 3.0
                            + (p - center))
                            / 8.0;
                        let dm = mid.hypot();
                        assert!(
                            (dm - inner).abs() < inner * 0.02,
                            "内弧中点半径应≈{inner}，实际 {dm}"
                        );
                    }
                    prev = p;
                }
                _ => {}
            }
        }
        assert!(inner_curves > 0, "未在路径中找到内弧段");
    }

    /// 自然位置不冲突的同侧标签：保持贴合扇区，不被改动。
    #[test]
    fn same_side_far_apart_keeps_natural_positions() {
        let center = (400.0, 300.0);
        let group = vec![
            mk_label(200_f64.to_radians(), "A"),
            mk_label(300_f64.to_radians(), "B"),
        ];
        let resolved = resolve_label_overlap(group, 12.0, center.0, center.1, 800.0, 600.0);
        let mut ys: Vec<f64> = resolved.iter().map(|g| g.text_y).collect();
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // 200° 在左下、300° 在正下：自然 y 相差很远（≈ 93.8 + 200）
        let expected_gap = {
            let kink_r = 200.0 + LABEL_LINE_LEN;
            let y200 = 300.0 - kink_r * 200_f64.to_radians().cos();
            let y300 = 300.0 - kink_r * 300_f64.to_radians().cos();
            (y300 - y200).abs()
        };
        assert!(
            (ys[1] - ys[0] - expected_gap).abs() < 1e-6,
            "间距充足时 y 应保持自然位置，实际差 {} vs {}",
            ys[1] - ys[0],
            expected_gap
        );
    }

    /// 同侧标签重叠时：自上而下逐个下推，间距 = 字号×1.2，顺序不变。
    #[test]
    fn same_side_overlap_pushes_down_with_pitch() {
        let center = (400.0, 300.0);
        // 6 个标签聚集在 12 点附近（自然 y 几乎相同）
        let angles = [352.0_f64, 354.0, 356.0, 358.0, 359.0, 359.5];
        let group: Vec<_> = angles
            .iter()
            .map(|a| mk_label(a.to_radians(), "标签"))
            .collect();
        let resolved = resolve_label_overlap(group, 12.0, center.0, center.1, 800.0, 600.0);
        assert_eq!(resolved.len(), 6);

        // 按 y 升序（= 自上而下）检查步长
        let mut ys: Vec<f64> = resolved.iter().map(|g| g.text_y).collect();
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for w in ys.windows(2) {
            let gap = w[1] - w[0];
            assert!(
                (gap - 12.0 * 1.2).abs() < 1e-6,
                "同侧标签间距应为 pitch=14.4，实际 {gap}"
            );
        }
        // 最上方标签保持自然 y（不被上移）
        let top_natural = {
            let kink_r = 200.0 + LABEL_LINE_LEN;
            300.0 - kink_r * (359.5_f64.to_radians()).cos()
        };
        let top = ys[0];
        assert!(
            (top - top_natural).abs() < 1e-6,
            "最上方标签应保持自然 y，实际 {top} vs {top_natural}"
        );
    }

    /// 折点必须落在外圆 `r + LABEL_LINE_LEN` 上，文本锚点 = 折点 ± 35。
    #[test]
    fn kink_stays_on_outer_circle_and_anchor_offset() {
        let center = (400.0, 300.0);
        let group = vec![
            mk_label(200_f64.to_radians(), "左下"),
            mk_label(340_f64.to_radians(), "左上"),
        ];
        let resolved = resolve_label_overlap(group, 12.0, center.0, center.1, 800.0, 600.0);
        for g in &resolved {
            let r = (g.line_kink - Point::new(center.0, center.1)).hypot();
            let expect_r = 200.0 + LABEL_LINE_LEN;
            assert!(
                (r - expect_r).abs() < 1e-6,
                "折点应在外圆 {expect_r} 上，实际 {r}"
            );
            let dx = (g.text_x - g.line_kink.x).abs();
            assert!(
                (dx - (LABEL_LINE_END + LABEL_TEXT_GAP)).abs() < 1e-6,
                "文本锚点与折点的水平距离应恒为 35，实际 {dx}"
            );
            assert_eq!(
                g.text_y, g.line_kink.y,
                "引导线第 3 段应水平（折点与文本同高）"
            );
        }
    }

    /// 12 点附近的聚集小扇区全部落到同一侧（不再有上/下分组导致的横向堆叠）。
    #[test]
    fn clustered_near_top_share_one_side() {
        let center = (400.0, 300.0);
        let group: Vec<_> = [358.0_f64, 359.0, 359.6, 0.4, 1.0]
            .iter()
            .map(|a| mk_label(a.to_radians(), "小扇区"))
            .collect();
        let resolved = resolve_label_overlap(group, 12.0, center.0, center.1, 800.0, 600.0);
        let sides: Vec<PieSide> = resolved.iter().map(|g| g.side).collect();
        // 358°~359.6° 在左、0.4°~1.0° 在右（按自然位置 x 分侧）
        assert_eq!(sides.iter().filter(|s| **s == PieSide::Left).count(), 3);
        assert_eq!(sides.iter().filter(|s| **s == PieSide::Right).count(), 2);
    }

    /// 标签过多放不下时：整体上移/压缩，全部留在画布内。
    #[test]
    fn crowded_group_stays_inside_canvas() {
        let center = (400.0, 300.0);
        let group: Vec<_> = (0..12)
            .map(|i| mk_label((200.0 + i as f64 * 12.0).to_radians(), "拥挤标签"))
            .collect();
        let resolved = resolve_label_overlap(group, 12.0, center.0, center.1, 800.0, 600.0);
        for g in &resolved {
            assert!(g.text_y >= 8.0, "标签 y 不得越出画布上缘: {}", g.text_y);
            assert!(
                g.text_y <= 600.0 - 8.0 - 12.0,
                "标签 y 不得越出画布下缘: {}",
                g.text_y
            );
            assert!(
                g.text_x >= 8.0 && g.text_x <= 800.0 - 8.0,
                "标签 x 不得越出画布: {}",
                g.text_x
            );
        }
    }
}
