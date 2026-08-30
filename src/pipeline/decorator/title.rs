//! 标题渲染
//!
//! 使用 layout_text 统一排版主标题和副标题，支持不同样式。

use lievisual::scene::{Element, SceneNode};
use lievisual::text::{FontWeight, RichSpan, TextStyle};
use vello_cpu::kurbo::Point;

use crate::{
    pipeline::types::{ChartSpec, ColorContext},
    text::create_text_layout,
    theme::Theme,
};
use lievisual::Color;
use crate::pipeline::builder::{ColorExt, Z_TITLE};

/// 构建标题元素
///
/// 返回 (标题元素列表, 标题总高度)
pub fn render_title(
    spec: &ChartSpec,
    width: u32,
    theme: &Theme,
    colors: &ColorContext,
) -> (Vec<SceneNode>, f64) {
    let mut elements = Vec::new();
    let mut title_height = 0.0;

    if let Some(title) = &spec.title {
        let title_style = theme.get_title_text_style();
        let subtitle_style = theme.get_subtitle_text_style();

        // 从 ColorContext 获取颜色
        let title_color = title
            .color
            .unwrap_or(Color::from_hex(&title_style.color).unwrap_or(colors.text_color));
        let subtitle_color = title.subcolor.unwrap_or(
            Color::from_hex(&subtitle_style.color).unwrap_or(colors.text_secondary_color),
        );

        let mut y_offset = 24.0;

        if let Some(text) = &title.text {
            // 构建文本样式
            let mut main_text_style = TextStyle::new(
                title_color,
                title.font_size.unwrap_or(title_style.font_size),
                title_style.font_family.clone(),
            );
            main_text_style.font_weight = FontWeight::Normal;

            let layout = create_text_layout(text, &main_text_style, None);
            let position_x = (width as f64 - layout.width) / 2.0;
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

            let layout = create_text_layout(subtext, &sub_text_style, None);
            let position_x = (width as f64 - layout.width) / 2.0;
            let position_y = y_offset + layout.height * 0.1;
            title_height += layout.height * 1.1;
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
