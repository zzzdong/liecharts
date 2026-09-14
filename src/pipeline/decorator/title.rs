//! 标题渲染
//!
//! 直接使用 lievisual 排版主标题和副标题，支持不同样式。

use lievisual::{
    Color,
    scene::{Element, SceneNode},
    text::{FontWeight, RichSpan, TextStyle, measure_text},
};
use vello_cpu::kurbo::Point;

use crate::{
    pipeline::{
        builder::{ColorExt, Z_TITLE},
        types::{ChartSpec, ColorContext},
    },
    theme::{DEFAULT_FONT_STACK, Theme},
};

/// 标题水平锚点（由 `left` / `right` 解析而来）
enum HAnchor {
    /// 距容器左侧 `d` 像素处的**锚点**（默认左对齐文本）
    Left(f64),
    /// 距容器右侧 `d` 像素处的锚点（默认右对齐文本）
    Right(f64),
    /// 水平居中
    Center,
}

/// 文本对齐（相对于锚点）
#[derive(Clone, Copy)]
enum HAlign {
    Left,
    Center,
    Right,
}

/// 解析 ECharts 的长度字面量：`"20%"` 相对 `total`，`"20"` / `20` 为像素。
fn parse_len(s: &str, total: f64) -> Option<f64> {
    let s = s.trim();
    if let Some(pct) = s.strip_suffix('%') {
        pct.trim().parse::<f64>().ok().map(|p| total * p / 100.0)
    } else {
        s.parse::<f64>().ok()
    }
}

/// 解析标题水平位置（ECharts `left` / `right`，缺省 `textAlign` 为 `left`）。
fn resolve_h_anchor(title: &crate::pipeline::types::TitleSpec, total: f64) -> HAnchor {
    if let Some(l) = title.left.as_deref() {
        let key = l.trim().to_ascii_lowercase();
        return match key.as_str() {
            "center" | "middle" => HAnchor::Center,
            "right" => HAnchor::Right(0.0),
            "left" | "auto" => HAnchor::Left(0.0),
            other => parse_len(other, total)
                .map(HAnchor::Left)
                .unwrap_or(HAnchor::Left(0.0)),
        };
    }
    if let Some(r) = title.right.as_deref() {
        let key = r.trim().to_ascii_lowercase();
        return match key.as_str() {
            "center" | "middle" => HAnchor::Center,
            "left" => HAnchor::Left(0.0),
            "right" | "auto" => HAnchor::Right(0.0),
            other => parse_len(other, total)
                .map(HAnchor::Right)
                .unwrap_or(HAnchor::Right(0.0)),
        };
    }
    // 未指定 left/right：ECharts 的 `left` 默认 'auto' + `textAlign` 默认 'left' → 贴左
    HAnchor::Left(0.0)
}

/// 由锚点、文本宽度与 `textAlign` 求文本块左边缘 x。
fn anchor_to_x(anchor: &HAnchor, text_align: Option<&str>, width: f64, text_w: f64) -> f64 {
    let (ax, default_align) = match *anchor {
        HAnchor::Left(d) => (d, HAlign::Left),
        HAnchor::Right(d) => (width - d, HAlign::Right),
        HAnchor::Center => (width / 2.0, HAlign::Center),
    };
    let align = match text_align.map(|a| a.trim().to_ascii_lowercase()) {
        Some(a) if a == "left" => HAlign::Left,
        Some(a) if a == "center" || a == "middle" => HAlign::Center,
        Some(a) if a == "right" => HAlign::Right,
        _ => default_align,
    };
    match align {
        HAlign::Left => ax,
        HAlign::Center => ax - text_w / 2.0,
        HAlign::Right => ax - text_w,
    }
}

/// 构建标题元素
///
/// 返回 (标题元素列表, 标题总高度)
pub fn render_title(
    spec: &ChartSpec,
    width: u32,
    height: u32,
    theme: &Theme,
    colors: &ColorContext,
) -> (Vec<SceneNode>, f64) {
    let mut elements = Vec::new();
    let mut title_height = 0.0;

    if let Some(title) = &spec.title {
        // `title.show: false` 不渲染（历史：被静默忽略）
        if !title.show {
            return (elements, 0.0);
        }
        let title_style = theme.get_title_text_style();
        let subtitle_style = theme.get_subtitle_text_style();

        // 从 ColorContext 获取颜色
        let title_color = title
            .color
            .unwrap_or(Color::from_hex(&title_style.color).unwrap_or(colors.text_color));
        let subtitle_color = title.subcolor.unwrap_or(
            Color::from_hex(&subtitle_style.color).unwrap_or(colors.text_secondary_color),
        );

        // 水平锚点（`left` / `right` / `textAlign`）
        let h_anchor = resolve_h_anchor(title, width as f64);

        // 垂直起点：显式 `top` 优先（像素或百分比），否则沿用既有 24px 留白
        let mut y_offset = match title.top.as_deref().map(str::trim) {
            Some(v) if !v.eq_ignore_ascii_case("auto") && !v.eq_ignore_ascii_case("top") => {
                parse_len(v, height as f64).unwrap_or(24.0)
            }
            _ => 24.0,
        };

        if let Some(text) = &title.text {
            // 构建文本样式
            let mut main_text_style = TextStyle::new(
                title_color,
                title.font_size.unwrap_or(title_style.font_size),
                title_style.font_family.clone(),
            );
            main_text_style.font_weight = FontWeight::Normal;

            let mut lv_style = main_text_style.clone();
            if lv_style.font_family.trim().is_empty()
                || lv_style
                    .font_family
                    .trim()
                    .eq_ignore_ascii_case("sans-serif")
            {
                lv_style.font_family = DEFAULT_FONT_STACK.to_string();
            }
            let layout =
                (*measure_text(&[RichSpan::new(text.clone(), lv_style)], None).layout).clone();
            let position_x = anchor_to_x(
                &h_anchor,
                title.text_align.as_deref(),
                width as f64,
                layout.width,
            );
            let position_y = y_offset;

            y_offset += layout.height;
            title_height += layout.height;

            elements.push(
                SceneNode::new(Element::Text {
                    spans: vec![RichSpan::new(text.clone(), main_text_style.clone())],
                    position: Point::new(position_x, position_y),
                    style: main_text_style,
                    layout: Some(std::sync::Arc::new(layout)),
                })
                .with_z(Z_TITLE),
            );
        }

        if let Some(subtext) = &title.subtext {
            let mut sub_text_style = TextStyle::new(
                subtitle_color,
                title.subfont_size.unwrap_or(subtitle_style.font_size),
                subtitle_style.font_family.clone(),
            );
            sub_text_style.font_weight = FontWeight::Normal;

            let mut lv_style = sub_text_style.clone();
            if lv_style.font_family.trim().is_empty()
                || lv_style
                    .font_family
                    .trim()
                    .eq_ignore_ascii_case("sans-serif")
            {
                lv_style.font_family = DEFAULT_FONT_STACK.to_string();
            }
            let layout =
                (*measure_text(&[RichSpan::new(subtext.clone(), lv_style)], None).layout).clone();
            let position_x = anchor_to_x(
                &h_anchor,
                title.text_align.as_deref(),
                width as f64,
                layout.width,
            );
            // `itemGap` 显式指定时按之，否则沿用既有 0.1 行高间距
            let gap = title.item_gap.unwrap_or(layout.height * 0.1);
            let position_y = y_offset + gap;
            title_height += gap + layout.height;
            elements.push(
                SceneNode::new(Element::Text {
                    spans: vec![RichSpan::new(subtext.clone(), sub_text_style.clone())],
                    position: Point::new(position_x, position_y),
                    style: sub_text_style,
                    layout: Some(std::sync::Arc::new(layout)),
                })
                .with_z(Z_TITLE),
            );
        }
    }

    (elements, title_height)
}
