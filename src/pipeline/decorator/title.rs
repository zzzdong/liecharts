//! 标题渲染
//!
//! 使用 layout_text 统一排版主标题和副标题，支持不同样式。

use vello_cpu::kurbo::Point;

use crate::{
    pipeline::types::{ChartSpec, ColorContext},
    text::create_text_layout,
    theme::Theme,
    visual::{Color, TextStyle, VisualElement, Z_TITLE},
};

/// 构建标题元素
///
/// 返回 (标题元素列表, 标题总高度)
pub fn render_title(
    spec: &ChartSpec,
    width: u32,
    theme: &Theme,
    colors: &ColorContext,
) -> (Vec<VisualElement>, f64) {
    let mut elements = Vec::new();
    let mut title_height = 0.0;

    if let Some(title) = &spec.title {
        let title_style = theme.get_title_text_style();
        let subtitle_style = theme.get_subtitle_text_style();

        // 从 ColorContext 获取颜色
        let title_color = title.color.unwrap_or(
            Color::from_hex(&title_style.color).unwrap_or(colors.text_color)
        );
        let subtitle_color = title.subcolor.unwrap_or(
            Color::from_hex(&subtitle_style.color).unwrap_or(colors.text_secondary_color)
        );

        let mut y_offset = 24.0;

        if let Some(text) = &title.text {
            // 构建文本样式
            let main_text_style = TextStyle {
                font_size: title.font_size.unwrap_or(title_style.font_size),
                color: title_color,
                font_family: title_style.font_family.clone(),
                font_weight: crate::option::FontWeight::Named(
                    crate::option::FontWeightNamed::Normal,
                ),
                ..Default::default()
            };

            let layout = create_text_layout(text, &main_text_style, None);
            let position_x = (width as f64 - layout.width() as f64) / 2.0;
            let position_y = y_offset;

            y_offset += layout.height() as f64;
            title_height += layout.height() as f64;

            elements.push(VisualElement::TextRun {
                text: text.clone(),
                position: Point::new(position_x, position_y),
                style: main_text_style,
                rotation: 0.0,
                max_width: None,
                layout: Some(Box::new(layout)),
                z_index: Z_TITLE,
            });
        }

        if let Some(subtext) = &title.subtext {
            let sub_text_style = TextStyle {
                font_size: title.subfont_size.unwrap_or(subtitle_style.font_size),
                color: subtitle_color,
                font_family: subtitle_style.font_family.clone(),
                font_weight: crate::option::FontWeight::Named(
                    crate::option::FontWeightNamed::Normal,
                ),
                ..Default::default()
            };

            let layout = create_text_layout(subtext, &sub_text_style, None);
            let position_x = (width as f64 - layout.width() as f64) / 2.0;
            let position_y = y_offset + layout.height() as f64 * 0.1;
            title_height += layout.height() as f64 * 1.1;
            elements.push(VisualElement::TextRun {
                text: subtext.clone(),
                position: Point::new(position_x, position_y),
                style: sub_text_style,
                rotation: 0.0,
                max_width: None,
                layout: Some(Box::new(layout)),
                z_index: Z_TITLE,
            });
        }
    }

    (elements, title_height)
}