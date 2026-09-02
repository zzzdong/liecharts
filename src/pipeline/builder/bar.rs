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

/// 柱外标签与柱体值端的间距（px）
const LABEL_OUTSIDE_GAP: f64 = 5.0;
/// 柱内标签与柱体值端的内边距（px）
const LABEL_INSIDE_PADDING: f64 = 5.0;

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
                let x = bar.rect.x0 + bar.rect.width() / 2.0;
                // 负值柱的“值端”在下方（rect.y1），正值柱在上方（rect.y0）
                let negative = bar.value < 0.0;

                // Top/Bottom = 值端外侧（柱外），Inside = 值端内侧（柱内）
                // 柱内放不下时（高度不足）自动回退到外侧，避免文字溢出柱体
                let inside = label_cfg.position == SeriesLabelPosition::Inside
                    && bar.rect.height() >= label_cfg.font_size + LABEL_INSIDE_PADDING * 2.0;

                let (y, va) = match (inside, negative) {
                    // 柱内：贴值端内侧
                    (true, false) => (bar.rect.y0 + LABEL_INSIDE_PADDING, TextBaseline::Top),
                    (true, true) => (bar.rect.y1 - LABEL_INSIDE_PADDING, TextBaseline::Bottom),
                    // 柱外：贴值端外侧
                    (false, false) => (bar.rect.y0 - LABEL_OUTSIDE_GAP, TextBaseline::Bottom),
                    (false, true) => (bar.rect.y1 + LABEL_OUTSIDE_GAP, TextBaseline::Top),
                };

                // 颜色优先取用户配置；否则柱内用白字、柱外跟随系列色（对齐 ECharts 默认观感）
                let label_color = label_cfg.color.unwrap_or(if inside {
                    Color::rgb(255, 255, 255)
                } else {
                    series.color
                });

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
