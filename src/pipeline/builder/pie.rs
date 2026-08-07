//! Pie Builder: 将 PieSeries 组装为 VisualElement

use std::f64::consts::PI;

use vello_cpu::kurbo::{BezPath, Point};

use crate::{
    error::Result,
    option::FontWeight,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, fill_style},
        typed_series::{LabelPosition, PieSeries, RenderContext},
    },
    visual::{
        Color, FillStrokeStyle, FontStyle, Stroke, TextAlign, TextBaseline, TextStyle,
        VisualElement,
    },
};

/// 饼图外部标签的几何布局（用于碰撞避让）
struct PieLabelGeometry {
    /// 标签展示文本
    text: String,
    /// 文本锚点 X（引导线第 2 段终点外侧）
    text_x: f64,
    /// 文本锚点 Y（引导线第 2 段终点）
    text_y: f64,
    /// 引导线第 1 段起点（扇形边缘，沿角度方向径向）
    line_start: Point,
    /// 引导线折点
    line_kink: Point,
    /// 标签所在区域
    region: PieRegion,
}

pub struct PieBuilder;

impl SeriesBuilder<PieSeries> for PieBuilder {
    fn build(series: &PieSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.slices.len() * 2);

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 饼图在 bounds 中心居中
        let center_x = bounds.x0 + width * 0.5;
        // 将圆心稍微向下偏移，以平衡标题和图例占用的顶部空间
        let center_y = bounds.y0 + height * 0.55;
        let center = Point::new(center_x, center_y);

        // 半径取宽高的较小值的一半，再乘以百分比
        let min_dim = width.min(height);
        let inner_radius = min_dim * 0.5 * series.radius_inner / 100.0;
        let outer_radius = min_dim * 0.5 * series.radius_outer / 100.0;

        let mut start_angle = 0.0; // 从 12 点钟方向开始

        // 第一遍：绘制扇形，并收集外部标签几何（供碰撞避让）
        let mut outside_labels: Vec<PieLabelGeometry> = Vec::new();

        for slice in &series.slices {
            let sweep_angle = slice.percent * 2.0 * PI;
            let end_angle = start_angle + sweep_angle;
            let mid_angle = start_angle + sweep_angle * 0.5;

            // 绘制扇形
            let path = build_arc_path(center, inner_radius, outer_radius, start_angle, end_angle);

            elements.push(VisualElement::Path {
                path,
                style: fill_style(slice.color),
                z_index: Z_SERIES_FILL,
            });

            // 标签
            if series.label_show {
                match series.label_position {
                    LabelPosition::Inside => {
                        let label_elements = build_inside_label(
                            center,
                            outer_radius,
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
                            outer_radius,
                            mid_angle,
                            &text,
                        ) {
                            outside_labels.push(geo);
                        }
                    }
                }
            }

            start_angle = end_angle;
        }

        // 碰撞避让：对相邻标签做一维 Y 轴避让，避免重叠
        let resolved = resolve_label_overlap(outside_labels, series.label_font_size);

        // 第二遍：用避让后的几何生成引导线 + 文本
        for geo in resolved {
            elements.extend(emit_label_elements(&geo, ctx));
        }

        Ok(elements)
    }
}

/// 饼图外部标签的碰撞避让。
///
/// 结合「引导线两段式」的布局要求（第 1 段「圆心 -> 扇形边缘」隐含不显示）：
/// - **第 2 段**：从扇形边缘沿扇区角度方向径向引出（`line_start -> line_kink`）；
/// - **第 3 段**：指向文本侧边（`line_kink -> line_end`），保持水平/垂直，指向文本端点的垂直/水平中心。
///
/// 碰撞避让原则：**按区域分组、独立避让**——
/// - Left/Right 区域：文本在饼图左/右外缘垂直排布，组内做 **Y 轴避让**；
/// - Top/Bottom 区域：文本在饼图上方/下方，组内做 **X 轴避让**。
/// 避免右侧大扇区被左侧小扇区推挤，也避免顶部小扇区引导线横穿饼图。
fn resolve_label_overlap(
    labels: Vec<PieLabelGeometry>,
    font_size: f64,
) -> Vec<PieLabelGeometry> {
    if labels.len() < 2 {
        return labels;
    }

    let (mut tops, mut bottoms, mut lefts, mut rights): (
        Vec<PieLabelGeometry>,
        Vec<PieLabelGeometry>,
        Vec<PieLabelGeometry>,
        Vec<PieLabelGeometry>,
    ) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for g in labels {
        match g.region {
            PieRegion::Top => tops.push(g),
            PieRegion::Bottom => bottoms.push(g),
            PieRegion::Left => lefts.push(g),
            PieRegion::Right => rights.push(g),
        }
    }

    // Left/Right 纵向避让，Top/Bottom 横向避让
    rights = resolve_group(rights, font_size, false);
    lefts = resolve_group(lefts, font_size, false);
    tops = resolve_group(tops, font_size, true);
    bottoms = resolve_group(bottoms, font_size, true);

    let mut result = rights;
    result.extend(lefts);
    result.extend(tops);
    result.extend(bottoms);
    result
}

/// 对同一侧的一组标签做 Y 轴避让。
///
/// 对同一区域内的一组标签做避让。
///
/// 用统一的位移避让策略算出组内各标签的目标坐标后，**直接移动折点与文本**，
/// 引导线第 1 段（`line_start -> line_kink`）随折点移动，整体在扇形外侧。
/// - `horizontal=false`（Left/Right）：沿 Y 轴错开；
/// - `horizontal=true`（Top/Bottom）：沿 X 轴错开。
fn resolve_group(
    group: Vec<PieLabelGeometry>,
    font_size: f64,
    horizontal: bool,
) -> Vec<PieLabelGeometry> {
    use crate::pipeline::collision::{CollisionResolver, DisplacementResolver, LabelBox};

    if group.len() < 2 {
        return group;
    }

    // 避让间隙：
    // - 纵向（Left/Right）：用标签高度（上下不重叠）
    // - 横向（Top/Bottom）：用最大文本宽度（左右不重叠）
    let gap = if horizontal {
        group
            .iter()
            .map(|g| crate::pipeline::axis_label::estimate_text_size(&g.text, font_size).0)
            .fold(0.0_f64, f64::max)
            + 8.0
    } else {
        font_size * 1.2
    };

    // 用统一的位移避让策略算出本组内各标签的目标坐标
    let axis = if horizontal { 1 } else { 0 };
    let boxes: Vec<LabelBox> = group
        .iter()
        .map(|g| {
            if horizontal {
                LabelBox::new(g.text_x, 0.0, 1.0, font_size * 1.2)
            } else {
                LabelBox::new(0.0, g.text_y, 1.0, font_size * 1.2)
            }
        })
        .collect();
    let resolver = DisplacementResolver::new(gap, axis);
    let resolved = resolver.resolve(boxes);

    let mut result = group;
    for (i, box_) in resolved.iter().enumerate() {
        if horizontal {
            // 横向避让：避让坐标在 box_.x，移动文本 x 与折点 x
            let target = box_.x;
            if (target - result[i].text_x).abs() < 1e-6 {
                continue;
            }
            result[i].text_x = target;
            result[i].line_kink.x = target;
        } else {
            // 纵向避让：避让坐标在 box_.y，移动文本 y 与折点 y
            let target = box_.y;
            if (target - result[i].text_y).abs() < 1e-6 {
                continue;
            }
            result[i].text_y = target;
            result[i].line_kink.y = target;
        }
    }
    result
}

/// 饼图外部标签所在区域（决定文本放置与避让方向）
#[derive(Debug, Clone, Copy, PartialEq)]
enum PieRegion {
    Top,
    Bottom,
    Left,
    Right,
}

/// 计算饼图外部标签的几何布局（引导线两段 + 文本锚点）。
///
/// 按扇区角度将标签归入四区，避免引导线横穿饼图：
/// - **Left/Right**：文本在饼图左/右外缘垂直排布（x 固定），后续做 Y 轴避让；
/// - **Top/Bottom**：文本在饼图上方/下方沿角度方向外置，后续做 X 轴避让。
/// 引导线统一「从扇形边缘沿角度方向径向引出」，向外不向内。
fn compute_label_geometry(
    center: Point,
    outer_radius: f64,
    mid_angle: f64,
    text: &str,
) -> Option<PieLabelGeometry> {
    // 将角度转换为标准坐标系（0°=右侧，π/2=下方）
    let angle = -PI / 2.0 + mid_angle;
    let dir = (angle.cos(), angle.sin());
    let (c, s) = dir;

    // 扇形边缘（引导线第 2 段起点）
    let line_start = Point::new(
        center.x + outer_radius * c,
        center.y + outer_radius * s,
    );

    // 判断区域：仅角度非常接近正上/正下（|cos| 很小）才归顶部/底部，
    // 其余按左右分侧（避免多个标签横向堆积在顶部导致超界/重叠）。
    let region = if s < -0.30 && c.abs() < 0.5 {
        PieRegion::Top
    } else if s > 0.30 && c.abs() < 0.5 {
        PieRegion::Bottom
    } else if c >= 0.0 {
        PieRegion::Right
    } else {
        PieRegion::Left
    };

    // 文本锚点：按区域外置在饼图轮廓外侧
    let text_margin = 12.0;
    let (text_x, text_y) = match region {
        PieRegion::Left => (center.x - outer_radius - text_margin, line_start.y),
        PieRegion::Right => (center.x + outer_radius + text_margin, line_start.y),
        // 顶部/底部：文本在饼图上方/下方，x 沿扇形方向
        PieRegion::Top => (line_start.x, center.y - outer_radius - text_margin),
        PieRegion::Bottom => (line_start.x, center.y + outer_radius + text_margin),
    };

    // 引导线第 2 段径向长度（从扇形边缘沿角度方向延伸）
    let radial_len = 12.0;
    let line_kink = Point::new(
        center.x + (outer_radius + radial_len) * c,
        center.y + (outer_radius + radial_len) * s,
    );

    Some(PieLabelGeometry {
        text: text.to_string(),
        text_x,
        text_y,
        line_start,
        line_kink,
        region,
    })
}

/// 用最终几何生成引导线 + 文本元素。
fn emit_label_elements(geo: &PieLabelGeometry, ctx: &RenderContext) -> Vec<VisualElement> {
    let mut elements = Vec::new();

    // 引导线第 3 段终点：指向文本侧边/顶底端点中心
    let line_end = match geo.region {
        PieRegion::Left => Point::new(geo.text_x + 5.0, geo.text_y),
        PieRegion::Right => Point::new(geo.text_x - 5.0, geo.text_y),
        PieRegion::Top => Point::new(geo.text_x, geo.text_y + 5.0),
        PieRegion::Bottom => Point::new(geo.text_x, geo.text_y - 5.0),
    };

    // 绘制引导线（两段折线，第 1 段「圆心->扇形边缘」隐含不显示）：
    // 第 2 段（径向）：扇形边缘(line_start) -> 折点(line_kink)
    // 第 3 段（水平）：折点 -> 文本侧边中点(line_end)
    let mut guide_path = BezPath::new();
    guide_path.move_to(geo.line_start);
    guide_path.line_to(geo.line_kink);
    guide_path.line_to(line_end);

    elements.push(VisualElement::Path {
        path: guide_path,
        style: FillStrokeStyle {
            fill: None,
            stroke: Some(Stroke {
                color: ctx.colors.text_secondary_color,
                width: 1.0,
            }),
        },
        z_index: Z_SERIES_FILL + 1,
    });

    // 文本对齐：Left/Right 文本侧对齐；Top/Bottom 文本顶/底对齐且水平居中
    let (align, baseline) = match geo.region {
        PieRegion::Left => (TextAlign::Right, TextBaseline::Middle),
        PieRegion::Right => (TextAlign::Left, TextBaseline::Middle),
        PieRegion::Top => (TextAlign::Center, TextBaseline::Bottom),
        PieRegion::Bottom => (TextAlign::Center, TextBaseline::Top),
    };

    elements.push(VisualElement::TextRun {
        text: geo.text.clone(),
        position: Point::new(geo.text_x, geo.text_y),
        style: TextStyle {
            color: ctx.colors.text_color,
            font_size: 12.0,
            font_family: "sans-serif".to_string(),
            font_weight: FontWeight::default(),
            font_style: FontStyle::Normal,
            align,
            vertical_align: baseline,
        },
        rotation: 0.0,
        max_width: None,
        layout: None,
        z_index: Z_SERIES_FILL + 2,
    });

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
) -> Vec<VisualElement> {
    let mut elements = Vec::new();

    // 将角度转换为标准坐标系
    let angle = -PI / 2.0 + mid_angle;

    // 标签文本
    let label_text = format_label_text(slice, formatter);

    // 内部标签：放在扇形中心
    let label_radius = outer_radius * 0.7;
    let label_x = center.x + label_radius * angle.cos();
    let label_y = center.y + label_radius * angle.sin();

    elements.push(VisualElement::TextRun {
        text: label_text,
        position: Point::new(label_x, label_y),
        style: TextStyle {
            color: Color::new(255, 255, 255), // 白色文字
            font_size: 12.0,
            font_family: "sans-serif".to_string(),
            font_weight: FontWeight::default(),
            font_style: FontStyle::Normal,
            align: TextAlign::Center,
            vertical_align: TextBaseline::Middle,
        },
        rotation: 0.0,
        max_width: None,
        layout: None,
        z_index: Z_SERIES_FILL + 2,
    });

    elements
}

/// 格式化饼图标签文本。
///
/// 通过统一的模板引擎替换 ECharts 占位符：
/// - `{b}`：数据项名称（`slice.name`）
/// - `{c}`：数据项数值（`slice.value`）
/// - `{d}`：百分比（`slice.percent`，保留 1 位小数）
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
        &format!("{} {:.1}%", slice.name, slice.percent * 100.0),
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
        // 使用常数 k = 4/3 * tan(θ/4) 来近似圆弧
        let theta = segment_angle;
        let k = (theta.abs() / 4.0).tan() * 4.0 / 3.0;

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
