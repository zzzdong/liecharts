//! Gauge Builder：ECharts 风格仪表盘（进度环 + 刻度 + 指针 + 中心锚点）
//!
//! 视觉规格参照 ECharts gauge（官方 gauge-progress 示例 + 默认项；
//! 尺寸单位 = radius/100，与 ECharts 的 width/length/distance 语义一致）：
//! - 轴环厚 17%（灰底 `#E6EBF8` 全环打底，进度色带叠加，外缘 = radius）
//! - 主刻度（splitLine）长 10%、宽 2、`#999`，起于环外缘外 1.5%
//! - 刻度标签**径向对齐**：`textAlign`/`textBaseline` 随角度取值
//!   （`cos→Left/Right/Center`、`sin→Top/Bottom/Middle`，同 ECharts GaugeView）
//! - 指针长 80%、宽 6%，风筝形多边形（替代细线），中心锚点圆环盖于其上
//! - 标题/数值 `offsetCenter [0,'20%'] / [0,'40%']`（中心下方），墨迹盒中心对齐
//!
//! 历史：旧实现环厚固定 18px（radius 的 8%，明显偏细）、指针为 3px 细线、
//! 刻度标签 Center/Middle 锚定（侧向位置视觉脱节）、`series.center` 字段
//! 被硬编码忽略。

use std::f64::consts::PI;

use lievisual::{
    Color,
    kurbo::{Arc, BezPath, PathSeg, Point, Rect, Shape},
    scene::{Element, FillStrokeStyle, GradientStop, LinearGradient, SceneNode, Stroke},
    text::{RichSpan, TextAlign, TextBaseline, TextStyle, measure_text},
};

use crate::{
    error::Result,
    pipeline::{
        builder::{
            SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, Z_SERIES_LINE, circle, gradient_path,
            line, path, resolve_radius,
        },
        typed_series::{GaugeSeries, RenderContext},
    },
};

/// ECharts 视觉常量（单位 = radius/100）
mod spec {
    /// 轴环厚度（ECharts gauge-progress 示例 axisLine.lineStyle.width=18，
    /// 略收至 17 给标签留头部空间）
    pub const RING_WIDTH: f64 = 17.0;
    /// 主刻度（splitLine）与环外缘的间距
    pub const SPLIT_LINE_GAP: f64 = 1.5;
    /// 主刻度长度（示例 splitLine.length=15，收至 10 避让顶部标题区）
    pub const SPLIT_LINE_LENGTH: f64 = 10.0;
    /// 标签与刻度末端的间距（合计 radius+14%，顶标签不与图题副标题重叠）
    pub const LABEL_GAP: f64 = 2.5;
    /// 指针长度（ECharts pointer.length '80%'）
    pub const POINTER_LENGTH: f64 = 80.0;
    /// 指针底宽（ECharts pointer.width 6）
    pub const POINTER_WIDTH: f64 = 6.0;
    /// 指针尾部越过中心的长度
    pub const POINTER_TAIL: f64 = 3.0;
    /// 中心锚点半径（anchor.size 的一半）
    pub const PIN_RADIUS: f64 = 5.5;
    /// 中心锚点描边宽
    pub const PIN_BORDER: f64 = 2.5;
    /// 标题 offsetCenter [0,'20%']
    pub const TITLE_OFFSET: f64 = 20.0;
    /// 数值 offsetCenter [0,'40%']
    pub const DETAIL_OFFSET: f64 = 40.0;
    /// 刻度标签字号系数（rem）：12px @ 根 12，与笛卡尔轴标签同口径
    pub const AXIS_LABEL_REM: f64 = 1.0;
    /// 标题（系列名）字号系数（rem）：16px @ 根 12（保留历史观感）
    pub const TITLE_REM: f64 = 16.0 / 12.0;
    /// 中心数值字号系数（rem）：30px @ 根 12（保留历史观感）
    pub const DETAIL_REM: f64 = 30.0 / 12.0;
}

pub struct GaugeBuilder;

impl SeriesBuilder<GaugeSeries> for GaugeBuilder {
    fn build(series: &GaugeSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
        let mut elements = Vec::new();

        let bounds = ctx.bounds;
        let width = bounds.width();
        let height = bounds.height();

        // 中心百分比真正取自 series.center（此前硬编码 50/55 忽略了字段；
        // compat 层默认 (50,50) 与 ECharts 默认一致）
        let center_x = bounds.x0 + width * series.center.0 / 100.0;
        let center_y = bounds.y0 + height * series.center.1 / 100.0;
        let center = Point::new(center_x, center_y);

        // P2a：radius 为**绝对像素**（api/compat 层按「画布 min/2」折算）；
        // P5 起由 `resolve_radius` 统一 clamp 到绘图区内接半径，
        // 未指定时按内接半径的 75% 自适应。
        let mut radius = resolve_radius(series.radius, width, height, 75.0);

        // 转换角度为弧度
        let start_angle = series.start_angle * PI / 180.0;
        let mut end_angle = series.end_angle * PI / 180.0;

        // 修正扫过跨 0°/360° 的角度：当结束角 <= 起始角时（如 225° -> -45°），
        // 实际是顺时针扫过 360°（结束角加一圈），使 sweep 为正、跨越 0° 的弧正确。
        if end_angle <= start_angle {
            end_angle += 2.0 * PI;
        }
        let total_sweep = end_angle - start_angle;

        // 计算数值角度。min == max（轴退化为单点）时避免除零：
        // 值不小于该点视为满格，否则视为 0。
        let span = series.max - series.min;
        let value_ratio = if span > 0.0 {
            ((series.value - series.min) / span).clamp(0.0, 1.0)
        } else if series.value >= series.min {
            1.0
        } else {
            0.0
        };
        let value_angle = start_angle + total_sweep * value_ratio;

        // Q2「用户显式根 + rem」：三处字号按「组件显式 > 全局根 × 系数」解析。
        // 系数取自历史默认观感（刻度 12 / 标题 16 / 数值 30 @ 根 12），
        // 用户设 `option.textStyle.fontSize` 时整体等比缩放；`series[].axisLabel
        // /title/detail.fontSize` 显式给出时绝对优先，不参与缩放。
        let rem_root = ctx.theme.tokens().text.body_size;
        let axis_label_size = series
            .axis_label_font_size
            .filter(|s| s.is_finite() && *s > 0.0)
            .unwrap_or(rem_root * spec::AXIS_LABEL_REM);
        let gauge_title_size = series
            .title_font_size
            .filter(|s| s.is_finite() && *s > 0.0)
            .unwrap_or(rem_root * spec::TITLE_REM);
        let detail_size = series
            .detail_font_size
            .filter(|s| s.is_finite() && *s > 0.0)
            .unwrap_or(rem_root * spec::DETAIL_REM);

        // 刻度标签文本（提前生成：收敛半径时需要最长标签的墨迹尺寸）
        let split_number = series.split_number.max(1);
        let tick_labels: Vec<String> = (0..=split_number)
            .map(|i| {
                let label_val =
                    series.min + (series.max - series.min) * i as f64 / split_number as f64;
                if label_val.fract() == 0.0 {
                    format!("{:.0}", label_val)
                } else {
                    format!("{:.1}", label_val)
                }
            })
            .collect();

        // 半径二次收敛：视觉外缘（环 + 刻度 + 标签 + 标签墨迹）不得越出绘图区。
        // `resolve_radius` 只按内接圆 clamp 环体，而刻度/标签在环外再伸 14%，
        // 顶部标签会侵入标题区与副标题相撞（详见 `clamp_radius_to_visual`）。
        radius = clamp_radius_to_visual(
            radius,
            center,
            bounds,
            start_angle,
            total_sweep,
            &tick_labels,
            axis_label_size,
        );
        let unit = radius / 100.0; // ECharts 尺寸单位

        // 轴环：外缘 = radius，厚 spec::RING_WIDTH%
        let ring_width = spec::RING_WIDTH * unit;
        let outer_r = radius;
        let inner_r = radius - ring_width;

        // 灰底全环（ECharts axisLine 打底：进度叠加其上，value=0/100 也完整）
        let track_path = build_arc_ribbon(center, inner_r, outer_r, start_angle, end_angle);
        elements.push(path(
            track_path,
            FillStrokeStyle {
                fill: Some(lievisual::scene::Fill::Solid(Color::rgb(230, 235, 248))),
                stroke: None,
            },
            true,
            Z_SERIES_FILL,
        ));

        // 绘制进度色带（平滑渐变：绿→黄→红）
        if value_angle > start_angle + 1e-6 {
            let filled_path = build_arc_ribbon(center, inner_r, outer_r, start_angle, value_angle);
            let bbox = filled_path.bounding_box();
            let gradient = LinearGradient::new(
                Point::new(bbox.x0, bbox.y0),
                Point::new(bbox.x1, bbox.y1),
                vec![
                    GradientStop {
                        offset: 0.00,
                        color: Color::rgb(80, 180, 80), // 绿
                    },
                    GradientStop {
                        offset: 0.50,
                        color: Color::rgb(255, 200, 50), // 黄
                    },
                    GradientStop {
                        offset: 1.00,
                        color: Color::rgb(220, 80, 80), // 红
                    },
                ],
            );
            elements.push(gradient_path(filled_path, gradient, None, Z_SERIES_FILL));
        }

        // 主刻度（splitLine）与标签
        let tick_start_r = outer_r + spec::SPLIT_LINE_GAP * unit;
        let tick_end_r = tick_start_r + spec::SPLIT_LINE_LENGTH * unit;
        let label_r = tick_end_r + spec::LABEL_GAP * unit;

        for (i, label_text) in tick_labels.iter().enumerate() {
            let angle = start_angle + total_sweep * i as f64 / split_number as f64;
            let (sin, cos) = angle.sin_cos();

            // 刻度线
            let x1 = center.x + tick_start_r * cos;
            let y1 = center.y + tick_start_r * sin;
            let x2 = center.x + tick_end_r * cos;
            let y2 = center.y + tick_end_r * sin;

            elements.push(line(
                Point::new(x1, y1),
                Point::new(x2, y2),
                crate::pipeline::builder::stroke_style(Color::rgb(153, 153, 153), 2.0),
                Z_SERIES_LINE + 1,
            ));

            // 标签：ECharts GaugeView 的径向对齐——
            // cos>0（右半）→ Left（文字向外/右伸展），cos<0 → Right；
            // sin>0（下半，屏幕坐标 y 向下）→ Top，sin<0 → Bottom。
            let mut style = TextStyle::new(Color::rgb(84, 85, 90), axis_label_size, "sans-serif");
            style.align = if cos > 1e-3 {
                TextAlign::Left
            } else if cos < -1e-3 {
                TextAlign::Right
            } else {
                TextAlign::Center
            };
            style.baseline = if sin > 1e-3 {
                TextBaseline::Top
            } else if sin < -1e-3 {
                TextBaseline::Bottom
            } else {
                TextBaseline::Middle
            };
            elements.push(
                SceneNode::new(Element::Text {
                    spans: vec![RichSpan::new(label_text.clone(), style.clone())],
                    position: Point::new(center.x + label_r * cos, center.y + label_r * sin),
                    style,
                    layout: None,
                })
                .with_z(Z_SERIES_LABEL),
            );
        }

        // 指针（ECharts 默认：长 80%、宽 6%，风筝形收尖，尾部略越中心）
        let needle_len = spec::POINTER_LENGTH * unit;
        let needle_half_w = spec::POINTER_WIDTH * unit / 2.0;
        let tail_len = spec::POINTER_TAIL * unit;
        let (ns, nc) = value_angle.sin_cos();
        let dir = Point::new(nc, ns);
        let perp = Point::new(-ns, nc);

        let needle = BezPath::from_vec(vec![
            lievisual::kurbo::PathEl::MoveTo(Point::new(
                center.x + dir.x * needle_len,
                center.y + dir.y * needle_len,
            )),
            lievisual::kurbo::PathEl::LineTo(Point::new(
                center.x + perp.x * needle_half_w,
                center.y + perp.y * needle_half_w,
            )),
            lievisual::kurbo::PathEl::LineTo(Point::new(
                center.x - dir.x * tail_len,
                center.y - dir.y * tail_len,
            )),
            lievisual::kurbo::PathEl::LineTo(Point::new(
                center.x - perp.x * needle_half_w,
                center.y - perp.y * needle_half_w,
            )),
            lievisual::kurbo::PathEl::ClosePath,
        ]);
        elements.push(path(
            needle,
            FillStrokeStyle {
                fill: Some(lievisual::scene::Fill::Solid(series.color)),
                stroke: None,
            },
            true,
            Z_SERIES_LINE + 2,
        ));

        // 中心锚点（ECharts anchor：白底 + 指针色圆环，盖在指针上方）
        elements.push(circle(
            center,
            spec::PIN_RADIUS * unit,
            FillStrokeStyle {
                fill: Some(lievisual::scene::Fill::Solid(Color::rgb(255, 255, 255))),
                stroke: Some(Stroke::new(series.color, spec::PIN_BORDER * unit)),
            },
            Z_SERIES_LINE + 3,
        ));

        // 中心数值显示（detail，offsetCenter [0,'40%']，墨迹盒中心对齐）
        let value_text = format!("{:.1}%", series.value);
        let value_style = TextStyle::new(Color::rgb(60, 60, 65), detail_size, "sans-serif");
        push_ink_centered_text(
            &mut elements,
            &value_text,
            Point::new(center.x, center.y + spec::DETAIL_OFFSET * unit),
            value_style,
            Z_SERIES_LABEL,
        );
        // 标题（title，offsetCenter [0,'20%']，位于数值上方）
        let title_style = TextStyle::new(Color::rgb(132, 132, 138), gauge_title_size, "sans-serif");
        push_ink_centered_text(
            &mut elements,
            &series.name,
            Point::new(center.x, center.y + spec::TITLE_OFFSET * unit),
            title_style,
            Z_SERIES_LABEL,
        );

        Ok(elements)
    }
}

/// 扫过范围 `[start, start+sweep]` 内 `f(θ)` 的最大值。
///
/// `f` 在 `best`（轴向极值角）处取 1；极值角被扫过时直接返回 1.0，
/// 否则退化为起止点两者中的较大者。
fn sweep_extreme(start: f64, sweep: f64, best: f64, f: impl Fn(f64) -> f64) -> f64 {
    let two_pi = 2.0 * PI;
    if (best - start).rem_euclid(two_pi) <= sweep {
        1.0
    } else {
        f(start).max(f(start + sweep))
    }
}

/// 把 gauge 半径收敛到「视觉外缘」不越出绘图区的值（只缩不放）。
///
/// 视觉外缘不止环体：刻度线伸出环外 `SPLIT_LINE_GAP + SPLIT_LINE_LENGTH`%（合计
/// 11.5%），标签再外扩 `LABEL_GAP`%（2.5%），且标签墨迹沿径向再伸一截——顶部
/// 标签向上（`baseline=Bottom`）、底部端点标签向下（`Top`）、左右标签向两侧
/// （`align=Right/Left`）。`resolve_radius` 只按内接圆 clamp 环体，于是
/// radius=75% 时顶部标签会探出绘图区上沿、侵入标题区与副标题相撞。
///
/// 按扫过范围内的四向极值计算可用半径：
/// - 顶（min y）极值角 270°、底（max y）90°、左（min x）180°、右（max x）0°；
/// - 极值角不在扫过范围内时，取起止点中更极的一端（如默认 270° 弧的底部极值
///   落在 45°/135° 端点，因子 = sin45° ≈ 0.707）；
/// - 各向再扣除标签墨迹（上下扣 `label_h`、左右扣 `label_w`）与安全边距。
///
/// 屏幕坐标系（y 向下）下角度 θ 的单位向量为 `(cos θ, sin θ)`：0°=右、90°=下、
/// 180°=左、270°=上，角度增大为顺时针——与 api 层默认 `start=-225°/end=45°`
/// 及 compat 层符号转换后的 ECharts 角度一致。
fn clamp_radius_to_visual(
    radius: f64,
    center: Point,
    bounds: Rect,
    start_angle: f64,
    total_sweep: f64,
    tick_labels: &[String],
    axis_label_size: f64,
) -> f64 {
    /// 与绘图区边缘（含上方标题区）的安全间距。
    const VISUAL_PAD: f64 = 10.0;
    /// 极端狭缝下的保底半径（避免退化成不可见的点）。
    const MIN_RADIUS: f64 = 20.0;

    // 扫过范围内 `f(θ)` 的最大值；`f` 在 `best` 处取 1（轴向极值）。
    // 极值角被扫过 → 1.0；否则退化为起止点较大者。
    let f_up = sweep_extreme(start_angle, total_sweep, 270_f64.to_radians(), |s| -s.sin());
    let f_down = sweep_extreme(start_angle, total_sweep, 90_f64.to_radians(), |s| s.sin());
    let f_left = sweep_extreme(start_angle, total_sweep, 180_f64.to_radians(), |s| -s.cos());
    let f_right = sweep_extreme(start_angle, total_sweep, 0.0, |s| s.cos());

    // 最长标签的墨迹盒（与绘制同口径：同字号、同字体族；align/baseline 不影响尺寸）
    let probe = TextStyle::new(Color::rgb(84, 85, 90), axis_label_size, "sans-serif");
    let (label_w, label_h) = tick_labels
        .iter()
        .map(|t| measure_text(&[RichSpan::new(t.clone(), probe.clone())], None).size)
        .fold((0.0_f64, 0.0_f64), |(w, h), s| {
            (w.max(s.width), h.max(s.height))
        });

    // 环外缘到标签锚点的距离 = radius × visual（刻度 + 标签间隙）
    let visual = 1.0 + (spec::SPLIT_LINE_GAP + spec::SPLIT_LINE_LENGTH + spec::LABEL_GAP) / 100.0;
    let candidates = [
        (center.y - (bounds.y0 + VISUAL_PAD) - label_h) / (f_up * visual),
        ((bounds.y1 - VISUAL_PAD) - center.y - label_h) / (f_down * visual),
        (center.x - (bounds.x0 + VISUAL_PAD) - label_w) / (f_left * visual),
        ((bounds.x1 - VISUAL_PAD) - center.x - label_w) / (f_right * visual),
    ];
    let max_r = candidates.into_iter().fold(f64::INFINITY, f64::min);
    radius.min(max_r.max(MIN_RADIUS))
}

/// 生成 cap 盒中心精确落在 `target` 的单行文本。
///
/// `align=Left`/`baseline=Top` 语义下两个后端的块偏移均为 0（锚点 = 块左上
/// 角），将 `cap_bounds()`（相对块原点）的中心反解进 position 即可，与图例
/// 文本对齐（`decorator/legend.rs`）同一机制。
fn push_ink_centered_text(
    elements: &mut Vec<SceneNode>,
    text: &str,
    target: Point,
    style: TextStyle,
    z: i32,
) {
    // 显式 Top/Left（锚点 = 块左上角，后端块偏移均为 0）——`TextStyle::new`
    // 默认 baseline 是 Alphabetic，直接用会把基线当块顶反解，文本整体偏高
    let mut style = style;
    style.align = TextAlign::Left;
    style.baseline = TextBaseline::Top;
    let layout = measure_text(
        &[RichSpan::new(text.to_string(), style.clone())],
        style.max_width,
    )
    .layout;
    let cb = layout.cap_bounds();
    let position = Point::new(
        target.x - (cb.min_x() + cb.max_x()) / 2.0,
        target.y - (cb.min_y() + cb.max_y()) / 2.0,
    );
    elements.push(
        SceneNode::new(Element::Text {
            spans: vec![RichSpan::new(text.to_string(), style.clone())],
            position,
            style,
            layout: Some(layout),
        })
        .with_z(z),
    );
}

/// 构建一段色带（环形扇区），使用真圆弧
fn build_arc_ribbon(center: Point, inner_r: f64, outer_r: f64, start: f64, end: f64) -> BezPath {
    let sweep = end - start;
    let mut path = BezPath::new();

    // 外圆弧起点
    let outer_start = Point::new(
        center.x + outer_r * start.cos(),
        center.y + outer_r * start.sin(),
    );
    path.move_to(outer_start);

    // 外圆弧
    add_arc(&mut path, center, outer_r, start, sweep);

    // 连接到内圆弧终点
    let inner_end = Point::new(
        center.x + inner_r * end.cos(),
        center.y + inner_r * end.sin(),
    );
    path.line_to(inner_end);

    // 内圆弧（反向）
    add_arc(&mut path, center, inner_r, end, -sweep);

    path.close_path();
    path
}

/// 使用 kurbo::Arc 添加真圆弧段到路径
fn add_arc(path: &mut BezPath, center: Point, radius: f64, start: f64, sweep: f64) {
    let arc = Arc {
        center,
        radii: (radius, radius).into(),
        start_angle: start,
        sweep_angle: sweep,
        x_rotation: 0.0,
    };

    // tolerance 越小越平滑
    arc.to_path(0.1).segments().for_each(|seg| match seg {
        PathSeg::Line(line) => path.line_to(line.p1),
        PathSeg::Quad(quad) => path.quad_to(quad.p1, quad.p2),
        PathSeg::Cubic(cubic) => path.curve_to(cubic.p1, cubic.p2, cubic.p3),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{pipeline::types::ColorContext, theme::Theme};

    /// 默认 270° 弧（屏幕坐标 135°→405°，即 api 默认 -225°/45°）。
    fn default_arc() -> (f64, f64) {
        ((-225_f64).to_radians(), 270_f64.to_radians())
    }

    fn tick_labels_0_100() -> Vec<String> {
        (0..=10).map(|i| format!("{}", i * 10)).collect()
    }

    /// 默认角度的 gauge 夹具（屏幕坐标系 -225°/45°，与 api 层默认一致）。
    fn gauge_series() -> GaugeSeries {
        GaugeSeries {
            name: "完成率".into(),
            min: 0.0,
            max: 100.0,
            center: (50.0, 55.0),
            radius: 225.0, // 绝对像素（api/compat 按画布 min/2 折算后的 75%）
            start_angle: -225.0,
            end_angle: 45.0,
            split_number: 10,
            value: 75.5,
            color: Color::rgb(80, 112, 221),
            axis_label_font_size: None,
            title_font_size: None,
            detail_font_size: None,
        }
    }

    /// 回归：四向极值因子——默认弧扫过顶/左/右（1.0），不扫过底（90°），
    /// 底部极值落在 45°/135° 端点，因子 = sin45° ≈ 0.707。
    #[test]
    fn sweep_extreme_factors_for_default_arc() {
        let (start, sweep) = default_arc();
        let half_sqrt2 = std::f64::consts::FRAC_1_SQRT_2;
        assert_eq!(
            sweep_extreme(start, sweep, 270_f64.to_radians(), |s| -s.sin()),
            1.0
        );
        assert!(
            (sweep_extreme(start, sweep, 90_f64.to_radians(), |s| s.sin()) - half_sqrt2).abs()
                < 1e-12
        );
        assert_eq!(
            sweep_extreme(start, sweep, 180_f64.to_radians(), |s| -s.cos()),
            1.0
        );
        assert_eq!(sweep_extreme(start, sweep, 0.0, |s| s.cos()), 1.0);
    }

    /// 回归：半径按视觉外缘收敛——顶部标签墨迹不得探出绘图区上沿。
    ///
    /// 历史 bug：`resolve_radius` 只按内接圆 clamp 环体，而刻度/标签在环外
    /// 再伸 14%，radius=75% 时顶部标签侵入标题区、与副标题相撞。
    /// 放得下时保持原值（只缩不放）。
    #[test]
    fn clamp_radius_to_visual_keeps_labels_inside_bounds() {
        // 800×600 画布的默认 gauge 布局：绘图区 (120,67)-(680,520)、
        // 中心 (420, 316.15)（center 50%/55%）。
        let bounds = Rect::new(120.0, 67.0, 680.0, 520.0);
        let center = Point::new(420.0, 316.15);
        let (start, sweep) = default_arc();
        let labels = tick_labels_0_100();

        let r = clamp_radius_to_visual(225.0, center, bounds, start, sweep, &labels, 12.0);
        assert!(r < 225.0, "顶部受限时半径应收缩，got {r}");

        let visual =
            1.0 + (spec::SPLIT_LINE_GAP + spec::SPLIT_LINE_LENGTH + spec::LABEL_GAP) / 100.0;
        let probe = TextStyle::new(Color::rgb(84, 85, 90), 12.0, "sans-serif");
        let label_h = measure_text(&[RichSpan::new("100", probe)], None)
            .size
            .height;
        let ink_top = center.y - r * visual - label_h;
        assert!(
            ink_top >= bounds.y0 + 10.0 - 1e-6,
            "顶部标签墨迹顶 {ink_top} 应距绘图区上沿 ≥10px"
        );

        let small = clamp_radius_to_visual(100.0, center, bounds, start, sweep, &labels, 12.0);
        assert_eq!(small, 100.0, "放得下时半径不应被动");
    }

    /// 回归：gauge 全部文本（刻度标签 + 中心标题/数值）的墨迹盒不越出绘图区。
    ///
    /// 同时守住两个历史问题：
    /// 1. 角度镜像——compat 曾把 ECharts 数学坐标系角度（225°/-45°）直接透传
    ///    给屏幕坐标系的 `GaugeConfig`，JSON 版仪表盘垂直镜像、起点标签落到
    ///    左上，墨迹探出绘图区上沿；
    /// 2. 半径未按视觉外缘收敛——顶部标签侵入标题区与副标题相撞。
    #[test]
    fn gauge_text_ink_stays_within_bounds() {
        let colors = ColorContext::default();
        let ctx = RenderContext {
            colors: &colors,
            theme: &Theme::echarts(),
            bounds: Rect::new(120.0, 67.0, 680.0, 520.0),
        };
        let elements = GaugeBuilder::build(&gauge_series(), &ctx).unwrap();

        let mut checked = 0usize;
        for node in &elements {
            let Some(Element::Text {
                spans,
                position,
                style,
                ..
            }) = node.element()
            else {
                continue;
            };
            let text = &spans[0].text;
            let size = measure_text(&[RichSpan::new(text.clone(), style.clone())], None).size;
            // 锚点语义：align/baseline 决定墨迹相对 position 的伸出方向
            let (ix0, ix1) = match style.align {
                TextAlign::Left => (position.x, position.x + size.width),
                TextAlign::Right => (position.x - size.width, position.x),
                _ => (position.x - size.width / 2.0, position.x + size.width / 2.0),
            };
            let (iy0, iy1) = match style.baseline {
                TextBaseline::Top => (position.y, position.y + size.height),
                TextBaseline::Bottom => (position.y - size.height, position.y),
                _ => (
                    position.y - size.height / 2.0,
                    position.y + size.height / 2.0,
                ),
            };
            let eps = 1e-6;
            assert!(
                ix0 >= ctx.bounds.x0 - eps && ix1 <= ctx.bounds.x1 + eps,
                "文本 {text:?} 墨迹 x [{ix0}, {ix1}] 越出绘图区 x [{}, {}]",
                ctx.bounds.x0,
                ctx.bounds.x1
            );
            assert!(
                iy0 >= ctx.bounds.y0 - eps && iy1 <= ctx.bounds.y1 + eps,
                "文本 {text:?} 墨迹 y [{iy0}, {iy1}] 越出绘图区 y [{}, {}]",
                ctx.bounds.y0,
                ctx.bounds.y1
            );
            checked += 1;
        }
        assert!(
            checked >= 13,
            "应检查 11 个刻度标签 + 中心标题/数值，got {checked}"
        );
    }

    /// 回归：起点标签（"0"）必须在中心**下方**（屏幕坐标 y 向下，默认弧
    /// 自左下 225° 起）。角度被镜像（数学坐标系直传）时它会落到左上。
    #[test]
    fn gauge_start_label_sits_below_center() {
        let colors = ColorContext::default();
        let ctx = RenderContext {
            colors: &colors,
            theme: &Theme::echarts(),
            bounds: Rect::new(120.0, 67.0, 680.0, 520.0),
        };
        let elements = GaugeBuilder::build(&gauge_series(), &ctx).unwrap();
        let series = gauge_series();
        let center = Point::new(
            ctx.bounds.x0 + ctx.bounds.width() * series.center.0 / 100.0,
            ctx.bounds.y0 + ctx.bounds.height() * series.center.1 / 100.0,
        );

        let start_label = elements
            .iter()
            .filter_map(|n| match n.element() {
                Some(Element::Text {
                    spans, position, ..
                }) if spans[0].text == "0" => Some(*position),
                _ => None,
            })
            .next()
            .expect("应渲染出起点刻度标签 \"0\"");
        assert!(
            start_label.y > center.y,
            "起点标签应在中心下方（y {} > {}），角度被镜像时会落到上方",
            start_label.y,
            center.y
        );
    }
}
