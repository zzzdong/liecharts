//! Heatmap Builder: 将 HeatmapSeries 组装为 lievisual `SceneNode`

use lievisual::{
    scene::{Element, SceneNode},
    text::{RichSpan, TextAlign, TextBaseline, TextStyle},
};
use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    pipeline::{
        builder::{
            SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, fill_stroke_style, fill_style, rect,
        },
        typed_series::{HeatmapSeries, RenderContext},
    },
};

pub struct HeatmapBuilder;

impl SeriesBuilder<HeatmapSeries> for HeatmapBuilder {
    fn build(
        series: &HeatmapSeries,
        _ctx: &RenderContext,
    ) -> Result<Vec<lievisual::scene::SceneNode>> {
        let mut elements = Vec::with_capacity(series.cells.len());

        for cell in &series.cells {
            let style = if cell.border_width > 0.0 {
                if let Some(border) = cell.border_color {
                    fill_stroke_style(cell.color, border, cell.border_width)
                } else {
                    fill_style(cell.color)
                }
            } else {
                fill_style(cell.color)
            };

            elements.push(rect(cell.rect, style, Z_SERIES_FILL));
        }

        // 单元格数值标签（`series[].label.show`）：居中显示在色块内
        if let Some(label) = &series.label
            && label.show
        {
            for cell in &series.cells {
                let text = crate::pipeline::template::render_template(
                    label.formatter.as_deref(),
                    &crate::pipeline::template::TemplateContext {
                        series_name: Some(&series.name),
                        name: None,
                        value: Some(cell.value),
                        percent: None,
                    },
                    &format_value(cell.value),
                );
                // 深/浅底色自动反色，保证可读性
                let bg = cell.color;
                let luminance = 0.299 * bg.r as f64 + 0.587 * bg.g as f64 + 0.114 * bg.b as f64;
                let color = label.color.unwrap_or(if luminance > 140.0 {
                    crate::Color::rgb(40, 40, 40)
                } else {
                    crate::Color::rgb(255, 255, 255)
                });
                let mut style = TextStyle::new(color, label.font_size, "sans-serif");
                style.align = TextAlign::Center;
                style.baseline = TextBaseline::Middle;
                elements.push(
                    SceneNode::new(Element::Text {
                        spans: vec![RichSpan::new(text, style.clone())],
                        position: Point::new(
                            (cell.rect.x0 + cell.rect.x1) / 2.0,
                            (cell.rect.y0 + cell.rect.y1) / 2.0,
                        ),
                        style,
                        layout: None,
                    })
                    .with_z(Z_SERIES_LABEL),
                );
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
