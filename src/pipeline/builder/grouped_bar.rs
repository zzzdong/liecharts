//! GroupedBar Builder: 将 GroupedBarSeries 组装为 lievisual `SceneNode`

use lievisual::scene::{Element, SceneNode};
use lievisual::text::{RichSpan, TextAlign, TextBaseline, TextStyle};
use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, fill_style, rect},
        typed_series::{GroupedBarSeries, RenderContext, SeriesLabelPosition},
    },
};

pub struct GroupedBarBuilder;

impl SeriesBuilder<GroupedBarSeries> for GroupedBarBuilder {
    fn build(series: &GroupedBarSeries, _ctx: &RenderContext) -> Result<Vec<SceneNode>> {
        let mut elements = Vec::with_capacity(series.rows.len());

        for row in &series.rows {
            elements.push(rect(
                row.bar_rect,
                fill_style(row.color),
                Z_SERIES_FILL,
            ));

            // 值标签
            if let Some(ref label_cfg) = series.label
                && label_cfg.show
            {
                let text = crate::pipeline::template::render_template(
                    label_cfg.formatter.as_deref(),
                    &crate::pipeline::template::TemplateContext {
                        series_name: Some(&series.sub_series[row.sub_series_idx].name),
                        name: Some(&row.category),
                        value: Some(row.value),
                        percent: None,
                    },
                    &format_value(row.value),
                );
                let bar_rect = row.bar_rect;
                let bar_dim = bar_rect.height().max(bar_rect.width());
                let is_horizontal = bar_rect.width() >= bar_rect.height();

                let (x, y, label_color, va) = if is_horizontal {
                    // 横向柱状图：文字在柱子末端右侧
                    (
                        bar_rect.x1 + 4.0,
                        bar_rect.y0 + bar_rect.height() / 2.0,
                        row.color,
                        TextBaseline::Middle,
                    )
                } else if bar_dim > 25.0 {
                    // 高柱子：内部顶部，白色文字
                    let y = if label_cfg.position == SeriesLabelPosition::Top {
                        bar_rect.y0 + 14.0
                    } else {
                        bar_rect.y0 + bar_rect.height() / 2.0
                    };
                    let va = if label_cfg.position == SeriesLabelPosition::Top {
                        TextBaseline::Top
                    } else {
                        TextBaseline::Middle
                    };
                    (
                        bar_rect.x0 + bar_rect.width() / 2.0,
                        y,
                        crate::visual::Color::rgb(255, 255, 255),
                        va,
                    )
                } else {
                    // 矮柱子：外部上方，柱子同色文字
                    (
                        bar_rect.x0 + bar_rect.width() / 2.0,
                        bar_rect.y0 - 4.0,
                        row.color,
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
