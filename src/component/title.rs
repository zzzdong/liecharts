use crate::component::ChartComponent;
use crate::layout::LayoutOutput;
use crate::model::{ChartModel, Title};
use crate::text::create_text_layout;
use crate::visual::{TextAlign, TextBaseline, VisualElement};
use vello_cpu::kurbo::Point;

pub struct TitleComponent {
    title: Title,
}

impl TitleComponent {
    pub fn new(title: &Title) -> Self {
        Self {
            title: title.clone(),
        }
    }
}

impl ChartComponent for TitleComponent {
    fn build_visual_elements(
        &self,
        _resolved: &ChartModel,
        layout: &LayoutOutput,
    ) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        if let Some(bbox) = layout.title_bbox {
            let title_font = &self.title.text_style;
            let title_layout = create_text_layout(&self.title.text, title_font, None);
            let text_width = title_layout.width() as f64;
            let text_height = title_layout.height() as f64;

            let pos_x = bbox.center().x - text_width / 2.0;
            let pos_y = bbox.y0 + 2.0;

            elements.push(VisualElement::TextRun {
                text: self.title.text.clone(),
                position: Point::new(pos_x, pos_y),
                style: crate::model::TextStyle {
                    color: title_font.color,
                    font_size: title_font.font_size,
                    font_family: title_font.font_family.clone(),
                    font_weight: title_font.font_weight,
                    font_style: title_font.font_style,
                    align: TextAlign::Left,
                    vertical_align: TextBaseline::Top,
                },
                rotation: 0.0,
                max_width: None,
                layout: Some(title_layout),
            });

            if let Some(subtext) = &self.title.subtext {
                let sub_font = self
                    .title
                    .subtext_style
                    .as_ref()
                    .unwrap_or(&self.title.text_style);
                let sub_layout = create_text_layout(subtext, sub_font, None);
                let sub_width = sub_layout.width() as f64;

                let sub_pos_x = bbox.center().x - sub_width / 2.0;
                let sub_pos_y = pos_y + text_height + 2.0;

                elements.push(VisualElement::TextRun {
                    text: subtext.clone(),
                    position: Point::new(sub_pos_x, sub_pos_y),
                    style: crate::model::TextStyle {
                        color: sub_font.color,
                        font_size: sub_font.font_size,
                        font_family: sub_font.font_family.clone(),
                        font_weight: sub_font.font_weight,
                        font_style: sub_font.font_style,
                        align: TextAlign::Left,
                        vertical_align: TextBaseline::Top,
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: Some(sub_layout),
                });
            }
        }

        elements
    }
}
