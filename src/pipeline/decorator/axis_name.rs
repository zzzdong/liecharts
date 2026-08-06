//! 轴名称渲染
//!
//! 渲染 X/Y 轴的名称标签，支持旋转和位置调整。

use vello_cpu::kurbo::Point;

use crate::{
    pipeline::types::{AxisPosition, ChartSpec, ColorContext, SubplotSpec},
    text::create_text_layout,
    theme::Theme,
    visual::{Color, TextAlign, TextBaseline, TextStyle, VisualElement, Z_LABEL},
};

/// 构建轴名称元素
pub fn render_axis_name(
    spec: &ChartSpec,
    width: u32,
    height: u32,
    specs: &[SubplotSpec],
    _colors: &ColorContext,
    theme: &Theme,
) -> Vec<VisualElement> {
    let mut elements = Vec::new();

    let axis_label_style = theme.get_axis_label_style();
    let label_color = Color::from_hex(&axis_label_style.color).unwrap_or(Color::new(84, 85, 90));

    for spec_item in specs {
        let bounds = spec_item.bounds;

        // 处理 Y 轴名称
        for (i, &y_axis_idx) in spec_item.y_axis_indices.iter().enumerate() {
            if let Some(y_axis) = spec.y_axes.get(y_axis_idx)
                && let Some(name) = &y_axis.name
            {
                let is_right = y_axis.position == AxisPosition::Right
                    || (y_axis.position != AxisPosition::Right && i > 0);

                let (initial_align, initial_baseline) = (TextAlign::Left, TextBaseline::Top);
                let text_style = TextStyle {
                    font_size: axis_label_style.font_size,
                    color: label_color,
                    font_family: axis_label_style.font_family.clone(),
                    align: initial_align,
                    vertical_align: initial_baseline,
                    ..Default::default()
                };
                let text_layout = create_text_layout(name, &text_style, None);
                let _text_width = text_layout.width() as f64;
                let text_height = text_layout.height() as f64;

                let margin = 15.0;
                let label_margin = 8.0;
                let max_label_width = 35.0;
                let (x, rotation, align, baseline) = if is_right {
                    let min_anchor_x = bounds.x1 + 8.0 + max_label_width + label_margin;
                    let anchor_x = min_anchor_x
                        .max(bounds.x1 + label_margin)
                        .min(width as f64 - margin - text_height);
                    (
                        anchor_x,
                        std::f64::consts::FRAC_PI_2,
                        TextAlign::Left,
                        TextBaseline::Top,
                    )
                } else {
                    let label_left_edge = bounds.x0 - 8.0 - max_label_width;
                    let max_anchor_x = label_left_edge - label_margin - text_height;
                    let anchor_x = max_anchor_x.max(margin);
                    (
                        anchor_x,
                        -std::f64::consts::FRAC_PI_2,
                        TextAlign::Left,
                        TextBaseline::Top,
                    )
                };
                let y = bounds.y0 + bounds.height() / 2.0;

                elements.push(VisualElement::TextRun {
                    text: name.clone(),
                    position: Point::new(x, y),
                    style: TextStyle {
                        font_size: axis_label_style.font_size,
                        color: label_color,
                        font_family: axis_label_style.font_family.clone(),
                        align,
                        vertical_align: baseline,
                        ..Default::default()
                    },
                    rotation,
                    max_width: None,
                    layout: Some(Box::new(text_layout)),
                    z_index: Z_LABEL,
                });
            }
        }

        // 处理 X 轴名称
        for (i, &x_axis_idx) in spec_item.x_axis_indices.iter().enumerate() {
            if let Some(x_axis) = spec.x_axes.get(x_axis_idx)
                && let Some(name) = &x_axis.name
            {
                let is_top = x_axis.position == AxisPosition::Top
                    || (x_axis.position != AxisPosition::Top && i > 0);

                let x = bounds.x0 + bounds.width() / 2.0;
                let font_size = axis_label_style.font_size;
                let y = if is_top {
                    (bounds.y0 - 25.0).max(font_size)
                } else {
                    (bounds.y1 + 35.0).min(height as f64 - font_size - 10.0)
                };

                elements.push(VisualElement::TextRun {
                    text: name.clone(),
                    position: Point::new(x, y),
                    style: TextStyle {
                        font_size: axis_label_style.font_size,
                        color: label_color,
                        font_family: axis_label_style.font_family.clone(),
                        align: TextAlign::Center,
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

    elements
}
