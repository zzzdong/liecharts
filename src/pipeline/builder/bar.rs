//! Bar Builder: 将 BarSeries 组装为 VisualElement

use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, fill_style},
        typed_series::{BarSeries, RenderContext, SeriesLabelPosition},
    },
    visual::{TextAlign, TextBaseline, TextStyle, VisualElement},
};

pub struct BarBuilder;

impl SeriesBuilder<BarSeries> for BarBuilder {
    fn build(series: &BarSeries, _ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.bars.len());

        for bar in &series.bars {
            elements.push(VisualElement::Rect {
                rect: bar.rect,
                style: fill_style(series.color),
                z_index: Z_SERIES_FILL,
            });

            // 值标签
            if let Some(ref label_cfg) = series.label
                && label_cfg.show
            {
                let text = format_value(bar.value);
                let (x, y) = match label_cfg.position {
                    SeriesLabelPosition::Top => {
                        // 在柱子顶部上方
                        (bar.rect.x0 + bar.rect.width() / 2.0, bar.rect.y0 - 4.0)
                    }
                    SeriesLabelPosition::Inside => {
                        // 在柱子内部居中
                        (
                            bar.rect.x0 + bar.rect.width() / 2.0,
                            bar.rect.y0 + bar.rect.height() / 2.0,
                        )
                    }
                };

                elements.push(VisualElement::TextRun {
                    text,
                    position: Point::new(x, y),
                    style: TextStyle {
                        color: label_cfg.color,
                        font_size: label_cfg.font_size,
                        align: TextAlign::Center,
                        vertical_align: TextBaseline::Bottom,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                    z_index: Z_SERIES_LABEL,
                });
            }
        }

        Ok(elements)
    }
}

fn format_value(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{:.0}", v)
    } else {
        format!("{:.1}", v)
    }
}
