//! Bar Builder: 将 BarSeries 组装为 lievisual `SceneNode`

use lievisual::{
    Color,
    scene::{Element, SceneNode},
    text::{RichSpan, TextAlign, TextBaseline, TextStyle},
};
use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, fill_style, render_mark_lines},
        typed_series::{BarSeries, RenderContext, SeriesLabelPosition},
    },
};

pub struct BarBuilder;

impl SeriesBuilder<BarSeries> for BarBuilder {
    fn build(series: &BarSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
        let mut elements = Vec::with_capacity(series.bars.len());

        for bar in &series.bars {
            elements.push(crate::pipeline::builder::rect(
                bar.rect,
                fill_style(series.color),
                Z_SERIES_FILL,
            ));

            // 值标签
            if let Some(ref label_cfg) = series.label
                && label_cfg.show
            {
                let text = crate::pipeline::template::render_template(
                    label_cfg.formatter.as_deref(),
                    &crate::pipeline::template::TemplateContext {
                        series_name: Some(&series.name),
                        name: Some(&bar.category),
                        value: Some(bar.value),
                        percent: None,
                    },
                    &format_value(bar.value),
                );
                let bar_height = bar.rect.height();
                // 高柱子（>25px）：内部顶部，白色文字
                // 矮柱子（≤25px）：外部上方，柱子同色文字
                let (x, y, label_color, va) = if bar_height > 25.0 {
                    let y = if label_cfg.position == SeriesLabelPosition::Top
                        && (bar.value >= 0.0 || (bar.value < 0.0 && bar_height > 25.0))
                    {
                        // Inside top: 4px from top of bar
                        bar.rect.y0 + 14.0
                    } else {
                        // Inside middle
                        bar.rect.y0 + bar.rect.height() / 2.0
                    };
                    let va = if label_cfg.position == SeriesLabelPosition::Top {
                        TextBaseline::Top
                    } else {
                        TextBaseline::Middle
                    };
                    (
                        bar.rect.x0 + bar.rect.width() / 2.0,
                        y,
                        Color::rgb(255, 255, 255),
                        va,
                    )
                } else {
                    // 外部上方，柱子同色文字
                    (
                        bar.rect.x0 + bar.rect.width() / 2.0,
                        bar.rect.y0 - 4.0,
                        series.color,
                        TextBaseline::Bottom,
                    )
                };

                let mut style = TextStyle::new(label_color, label_cfg.font_size, "sans-serif");
                style.align = TextAlign::Center;
                style.baseline = va;
                elements.push(
                    SceneNode::new(Element::Text {
                        spans: vec![RichSpan::new(text, style.clone())],
                        position: Point::new(x, y),
                        style,
                        layout: None,
                    })
                    .with_z(Z_SERIES_LABEL),
                );
            }
        }

        // 标注线（markLine）
        render_mark_lines(&mut elements, &series.mark_lines, ctx.bounds);

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
