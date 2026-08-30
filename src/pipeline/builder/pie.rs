//! Pie Builder: 将 PieSeries 组装为 lievisual `SceneNode`

use std::f64::consts::PI;

use lievisual::{
    Color,
    scene::{Element, FillStrokeStyle, SceneNode, Stroke},
    text::{RichSpan, TextAlign, TextBaseline, TextStyle},
};
use vello_cpu::kurbo::{BezPath, Point};

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, fill_style, path},
        typed_series::{LabelPosition, PieSeries, RenderContext},
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
    fn build(series: &PieSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
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
            let arc_path =
                build_arc_path(center, inner_radius, outer_radius, start_angle, end_angle);

            elements.push(path(arc_path, fill_style(slice.color), true, Z_SERIES_FILL));

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
                        if let Some(geo) =
                            compute_label_geometry(center, outer_radius, mid_angle, &text)
                        {
                            outside_labels.push(geo);
                        }
                    }
                }
            }

            start_angle = end_angle;
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
///
///   避免右侧大扇区被左侧小扇区推挤，也避免顶部小扇区引导线横穿饼图。
///
/// 参考 ECharts 的 `avoidOverlap` 思路：当同侧标签相互重叠时，不再用单向位移
/// 把标签一味向外推挤（会导致整体偏移、越界），而是围绕饼图圆心**对称均匀排布**，
/// 并把整组标签约束在画布范围内（见 [`distribute_vertical`]、[`distribute_horizontal`]）。
fn resolve_label_overlap(
    labels: Vec<PieLabelGeometry>,
    font_size: f64,
    center_x: f64,
    center_y: f64,
    canvas_width: f64,
    canvas_height: f64,
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

    // Left/Right 纵向避让，Top/Bottom 横向均匀分布
    rights = resolve_group(
        rights,
        font_size,
        false,
        center_x,
        center_y,
        canvas_width,
        canvas_height,
    );
    lefts = resolve_group(
        lefts,
        font_size,
        false,
        center_x,
        center_y,
        canvas_width,
        canvas_height,
    );
    tops = resolve_group(
        tops,
        font_size,
        true,
        center_x,
        center_y,
        canvas_width,
        canvas_height,
    );
    bottoms = resolve_group(
        bottoms,
        font_size,
        true,
        center_x,
        center_y,
        canvas_width,
        canvas_height,
    );

    let mut result = rights;
    result.extend(lefts);
    result.extend(tops);
    result.extend(bottoms);
    result
}

/// 对同一区域内的一组标签做避让。
///
/// - **Left/Right 区域**（`horizontal=false`）：文本沿饼图左/右外缘垂直排布，
///   组内做 **Y 轴避让**。
/// - **Top/Bottom 区域**（`horizontal=true`）：多个小扇区常聚集在正上/正下方，
///   其自然 x 坐标非常接近，若用单向位移避让会把标签逐一向外推挤，导致重叠或越界。
///   因此改用对称均匀分布（见 [`distribute_horizontal`]）。
fn resolve_group(
    group: Vec<PieLabelGeometry>,
    font_size: f64,
    horizontal: bool,
    center_x: f64,
    center_y: f64,
    canvas_width: f64,
    canvas_height: f64,
) -> Vec<PieLabelGeometry> {
    if group.len() < 2 {
        return group;
    }

    if horizontal {
        return distribute_horizontal(group, font_size, center_x, canvas_width);
    }
    distribute_vertical(group, font_size, center_y, canvas_height)
}

/// 左侧/右侧标签的纵向对称均匀分布。
///
/// 参考 ECharts 半椭圆贴合弧线的思路：同侧标签的理想位置沿扇区角度自然分布，
/// 只有当相邻标签相互重叠时，才围绕圆心 y（`center_y`）对称均匀排布，
/// 避免单向位移把标签整体推向画布边缘。
fn distribute_vertical(
    mut group: Vec<PieLabelGeometry>,
    font_size: f64,
    center_y: f64,
    canvas_height: f64,
) -> Vec<PieLabelGeometry> {
    // 按自然 y 排序，保持扇区顺序
    group.sort_by(|a, b| {
        a.text_y
            .partial_cmp(&b.text_y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 标签高度 + 纵向间距
    let label_h = font_size * 1.2;
    let natural_gap = label_h + 4.0;

    // 先判断是否存在重叠：相邻标签的 y 间距小于所需间距
    let has_overlap = group
        .windows(2)
        .any(|w| w[1].text_y - w[0].text_y < natural_gap);
    if !has_overlap {
        // 无重叠，保持贴合扇区的自然位置
        return group;
    }

    let n = group.len();

    // 画布纵向可用范围（两端预留安全边距）
    const SAFE_MARGIN: f64 = 8.0;
    let min_y = SAFE_MARGIN;
    let max_y = (canvas_height - SAFE_MARGIN - label_h).max(min_y);
    let available = (max_y - min_y).max(0.0);

    // 均匀间距：优先自然间距；若总高放不下则等比压缩，确保不越界
    let step = if n > 1 && (n - 1) as f64 * natural_gap > available {
        available / (n - 1) as f64
    } else {
        natural_gap
    };

    // 围绕圆心对称排布，并将起点夹到范围内
    let span = (n - 1) as f64 * step;
    let start_y = (center_y - span * 0.5).clamp(min_y, max_y - span);

    for (i, g) in group.iter_mut().enumerate() {
        let target = start_y + i as f64 * step;
        g.text_y = target;
        g.line_kink.y = target;
    }
    group
}

/// 顶部/底部标签的横向对称均匀分布。
///
/// 多个小扇区常聚集在正上/正下方，其自然 x 坐标都非常接近圆心 x，
/// 若用单向位移避让，会把标签逐一向外推挤（最外侧可能被推出画布），
/// 且中间标签可能仍挤在一起相互遮挡。
/// 这里改为围绕圆心对称均匀排布，并把标签整体约束在画布范围内。
fn distribute_horizontal(
    mut group: Vec<PieLabelGeometry>,
    font_size: f64,
    center_x: f64,
    canvas_width: f64,
) -> Vec<PieLabelGeometry> {
    // 按自然 x 排序，保持扇区顺序
    group.sort_by(|a, b| {
        a.text_x
            .partial_cmp(&b.text_x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let n = group.len();
    let max_w = group
        .iter()
        .map(|g| crate::pipeline::axis_label::estimate_text_size(&g.text, font_size).0)
        .fold(0.0_f64, f64::max);

    // 画布横向可用范围（两端预留安全边距，最右侧还需留出文本宽度）
    const SAFE_MARGIN: f64 = 8.0;
    let min_x = SAFE_MARGIN;
    let max_x = (canvas_width - SAFE_MARGIN - max_w).max(min_x);

    let natural_gap = max_w + 12.0;
    let available = (max_x - min_x).max(0.0);

    // 均匀间距：优先自然间距；若总宽放不下则等比压缩，确保不越界
    let step = if n > 1 && (n - 1) as f64 * natural_gap > available {
        available / (n - 1) as f64
    } else {
        natural_gap
    };

    // 围绕圆心对称排布，并将起点夹到范围内
    let span = (n - 1) as f64 * step;
    let start_x = (center_x - span * 0.5).clamp(min_x, max_x - span);

    for (i, g) in group.iter_mut().enumerate() {
        let target = start_x + i as f64 * step;
        g.text_x = target;
    }
    group
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
///
///   引导线统一「从扇形边缘沿角度方向径向引出」，向外不向内。
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
    let line_start = Point::new(center.x + outer_radius * c, center.y + outer_radius * s);

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
fn emit_label_elements(geo: &PieLabelGeometry, ctx: &RenderContext) -> Vec<SceneNode> {
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

    elements.push(path(
        guide_path,
        FillStrokeStyle {
            fill: None,
            stroke: Some(Stroke::new(ctx.colors.text_secondary_color, 1.0)),
        },
        false,
        Z_SERIES_FILL + 1,
    ));

    // 文本对齐：Left/Right 文本侧对齐；Top/Bottom 文本顶/底对齐且水平居中
    let (align, baseline) = match geo.region {
        PieRegion::Left => (TextAlign::Right, TextBaseline::Middle),
        PieRegion::Right => (TextAlign::Left, TextBaseline::Middle),
        PieRegion::Top => (TextAlign::Center, TextBaseline::Bottom),
        PieRegion::Bottom => (TextAlign::Center, TextBaseline::Top),
    };

    let mut style = TextStyle::new(ctx.colors.text_color, 12.0, "sans-serif");
    style.align = align;
    style.baseline = baseline;
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

#[cfg(test)]
mod tests {
    use vello_cpu::kurbo::Point;

    use super::*;

    fn mk_label(text_x: f64, text_y: f64) -> PieLabelGeometry {
        PieLabelGeometry {
            text: "样例标签".to_string(),
            text_x,
            text_y,
            line_start: Point::new(text_x, text_y),
            line_kink: Point::new(text_x, text_y),
            region: PieRegion::Right,
        }
    }

    #[test]
    fn test_vertical_no_overlap_keeps_natural_positions() {
        // 标签间距充足时不应被改动（贴合扇区）
        let group = vec![mk_label(10.0, 100.0), mk_label(10.0, 200.0)];
        let resolved = distribute_vertical(group, 12.0, 150.0, 300.0);
        assert_eq!(resolved[0].text_y, 100.0);
        assert_eq!(resolved[1].text_y, 200.0);
    }

    #[test]
    fn test_vertical_overlap_symmetrical_and_within_canvas() {
        // 三个重叠标签应围绕圆心对称分布，且不超出画布
        let group = vec![
            mk_label(10.0, 148.0),
            mk_label(10.0, 150.0),
            mk_label(10.0, 152.0),
        ];
        let resolved = distribute_vertical(group, 12.0, 150.0, 300.0);
        // 均匀间距
        let gap1 = resolved[1].text_y - resolved[0].text_y;
        let gap2 = resolved[2].text_y - resolved[1].text_y;
        assert!((gap1 - gap2).abs() < 1e-6);
        // 不越界
        assert!(resolved[0].text_y >= 8.0);
        assert!(resolved[2].text_y <= 300.0 - 8.0);
        // 围绕圆心对称
        assert!((resolved[0].text_y - 150.0).abs() - (resolved[2].text_y - 150.0).abs() < 1e-6);
    }

    #[test]
    fn test_vertical_crowded_compresses_to_stay_in_canvas() {
        // 标签过多放不下时压缩间距，保证全部落在画布内
        let group = (0..10).map(|_| mk_label(10.0, 150.0)).collect::<Vec<_>>();
        let resolved = distribute_vertical(group, 12.0, 150.0, 200.0);
        assert!(resolved[0].text_y >= 8.0);
        assert!(resolved[9].text_y <= 200.0 - 8.0 - 12.0 * 1.2);
    }

    #[test]
    fn test_horizontal_clustered_spreads_within_canvas() {
        // 顶部聚集的多个标签均匀分布且不越界
        let group = vec![
            mk_label(398.0, 10.0),
            mk_label(400.0, 10.0),
            mk_label(402.0, 10.0),
            mk_label(401.0, 10.0),
            mk_label(399.0, 10.0),
        ];
        let resolved = distribute_horizontal(group, 12.0, 400.0, 800.0);
        let xs: Vec<f64> = resolved.iter().map(|g| g.text_x).collect();
        // 均匀递增
        for w in xs.windows(2) {
            assert!(w[1] > w[0]);
        }
        let gap = xs[1] - xs[0];
        assert!((xs[4] - xs[3] - gap).abs() < 1e-6);
        // 不越界（最右文本宽度需留出空间）
        let max_w = resolved
            .iter()
            .map(|g| crate::pipeline::axis_label::estimate_text_size(&g.text, 12.0).0)
            .fold(0.0_f64, f64::max);
        assert!(xs[0] >= 8.0);
        assert!(xs[4] + max_w <= 800.0 - 8.0);
    }
}
