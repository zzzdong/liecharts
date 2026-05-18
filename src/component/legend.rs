use crate::component::ChartComponent;
use crate::layout::LayoutOutput;
use crate::model::{Legend, ResolvedOption};
use crate::visual::{Color, FillStrokeStyle, TextAlign, TextBaseline, VisualElement};
use crate::text::{compute_text_offset, create_text_layout};
use vello_cpu::kurbo::{Point, Rect};

pub struct LegendComponent {
    legend: Legend,
}

impl LegendComponent {
    pub fn new(legend: &Legend) -> Self {
        Self { legend: legend.clone() }
    }
}

impl ChartComponent for LegendComponent {
    fn build_visual_elements(&self, resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        if !self.legend.show {
            return elements;
        }

        if let Some(bbox) = layout.legend_bbox {
            let item_height = self.legend.item_height;
            let symbol_size = self.legend.symbol_size;
            let font_config = &self.legend.text_style;

            let item_widths: Vec<f64> = self.legend.data.iter()
                .map(|name| {
                    let text_w = crate::layout::measure_text_size(name, font_config).width;
                    symbol_size + 5.0 + text_w + 10.0
                })
                .collect();

            match self.legend.orient {
                crate::model::Orient::Horizontal => {
                    let mut x = bbox.x0 + 10.0;
                    let mut y = bbox.y0 + 5.0;
                    let mut col_count = 0_usize;

                    for (i, name) in self.legend.data.iter().enumerate() {
                        let color = resolved.series_color_by_name(name)
                            .or_else(|| resolved.colors.get(i % resolved.colors.len()).copied())
                            .unwrap_or(Color::new(0, 0, 0));
                        let iw = item_widths[i];

                        if col_count > 0 && x + iw > bbox.x1 - 5.0 {
                            col_count = 0;
                            x = bbox.x0 + 10.0;
                            y += item_height;
                        }

                        let center_y = y + item_height / 2.0;
                        let symbol_y = center_y - symbol_size / 2.0;

                        elements.push(VisualElement::Rect {
                            rect: Rect::new(x, symbol_y, x + symbol_size, symbol_y + symbol_size),
                            style: FillStrokeStyle {
                                fill: Some(color),
                                stroke: None,
                            },
                        });

                        let layout_obj = create_text_layout(name, font_config, None);
                        let (_, y_offset) = compute_text_offset(&layout_obj, TextAlign::Left, TextBaseline::Middle);

                        elements.push(VisualElement::TextRun {
                            text: name.clone(),
                            position: Point::new(x + symbol_size + 5.0, center_y + y_offset),
                            style: crate::model::TextStyle {
                                color: font_config.color,
                                font_size: font_config.font_size,
                                font_family: font_config.font_family.clone(),
                                font_weight: font_config.font_weight,
                                font_style: font_config.font_style,
                                align: TextAlign::Left,
                                vertical_align: TextBaseline::Top,
                            },
                            rotation: 0.0,
                            max_width: None,
                            layout: Some(layout_obj),
                        });

                        x += iw;
                        col_count += 1;
                    }
                }
                crate::model::Orient::Vertical => {
                    for (i, name) in self.legend.data.iter().enumerate() {
                        let color = resolved.series_color_by_name(name)
                            .or_else(|| resolved.colors.get(i % resolved.colors.len()).copied())
                            .unwrap_or(Color::new(0, 0, 0));

                        let x = bbox.x0 + 10.0;
                        let y = bbox.y0 + i as f64 * item_height + 5.0;
                        let center_y = y + item_height / 2.0;
                        let symbol_y = center_y - symbol_size / 2.0;

                        elements.push(VisualElement::Rect {
                            rect: Rect::new(x, symbol_y, x + symbol_size, symbol_y + symbol_size),
                            style: FillStrokeStyle {
                                fill: Some(color),
                                stroke: None,
                            },
                        });

                        let layout_obj = create_text_layout(name, font_config, None);
                        let (_, y_offset) = compute_text_offset(&layout_obj, TextAlign::Left, TextBaseline::Middle);

                        elements.push(VisualElement::TextRun {
                            text: name.clone(),
                            position: Point::new(x + symbol_size + 5.0, center_y + y_offset),
                            style: crate::model::TextStyle {
                                color: font_config.color,
                                font_size: font_config.font_size,
                                font_family: font_config.font_family.clone(),
                                font_weight: font_config.font_weight,
                                font_style: font_config.font_style,
                                align: TextAlign::Left,
                                vertical_align: TextBaseline::Top,
                            },
                            rotation: 0.0,
                            max_width: None,
                            layout: Some(layout_obj),
                        });
                    }
                }
            }
        }

        elements
    }
}
