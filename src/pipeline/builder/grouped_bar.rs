//! GroupedBar Builder: 将 GroupedBarSeries 组装为 lievisual `SceneNode`

use lievisual::{
    Color,
    scene::{Element, SceneNode},
    text::{RichSpan, TextAlign, TextBaseline, TextStyle},
};
use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LABEL, fill_style, rect},
        typed_series::{GroupedBarSeries, RenderContext, SeriesLabelPosition},
    },
};

/// 柱外标签与柱体值端的间距（px）
const LABEL_OUTSIDE_GAP: f64 = 5.0;
/// 柱内标签与柱体值端的内边距（px）
const LABEL_INSIDE_PADDING: f64 = 5.0;

pub struct GroupedBarBuilder;

impl SeriesBuilder<GroupedBarSeries> for GroupedBarBuilder {
    fn build(series: &GroupedBarSeries, _ctx: &RenderContext) -> Result<Vec<SceneNode>> {
        let mut elements = Vec::with_capacity(series.rows.len());

        for row in &series.rows {
            elements.push(rect(row.bar_rect, fill_style(row.color), Z_SERIES_FILL));

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
                let is_horizontal = bar_rect.width() >= bar_rect.height();
                let negative = row.value < 0.0;

                // Inside 需柱体在“值方向”上放得下文字，否则回退到柱外避免溢出
                let value_extent = if is_horizontal {
                    bar_rect.width()
                } else {
                    bar_rect.height()
                };
                let inside = label_cfg.position == SeriesLabelPosition::Inside
                    && value_extent >= label_cfg.font_size + LABEL_INSIDE_PADDING * 2.0;

                let (x, y, ha, va) = if is_horizontal {
                    // 横向柱：值端在右（正值）/ 左（负值）
                    let cy = bar_rect.y0 + bar_rect.height() / 2.0;
                    match (inside, negative) {
                        (true, false) => (
                            bar_rect.x1 - LABEL_INSIDE_PADDING,
                            cy,
                            TextAlign::Right,
                            TextBaseline::Middle,
                        ),
                        (true, true) => (
                            bar_rect.x0 + LABEL_INSIDE_PADDING,
                            cy,
                            TextAlign::Left,
                            TextBaseline::Middle,
                        ),
                        (false, false) => (
                            bar_rect.x1 + LABEL_OUTSIDE_GAP,
                            cy,
                            TextAlign::Left,
                            TextBaseline::Middle,
                        ),
                        (false, true) => (
                            bar_rect.x0 - LABEL_OUTSIDE_GAP,
                            cy,
                            TextAlign::Right,
                            TextBaseline::Middle,
                        ),
                    }
                } else {
                    // 纵向柱：值端在上（正值）/ 下（负值）
                    let cx = bar_rect.x0 + bar_rect.width() / 2.0;
                    match (inside, negative) {
                        (true, false) => (
                            cx,
                            bar_rect.y0 + LABEL_INSIDE_PADDING,
                            TextAlign::Center,
                            TextBaseline::Top,
                        ),
                        (true, true) => (
                            cx,
                            bar_rect.y1 - LABEL_INSIDE_PADDING,
                            TextAlign::Center,
                            TextBaseline::Bottom,
                        ),
                        (false, false) => (
                            cx,
                            bar_rect.y0 - LABEL_OUTSIDE_GAP,
                            TextAlign::Center,
                            TextBaseline::Bottom,
                        ),
                        (false, true) => (
                            cx,
                            bar_rect.y1 + LABEL_OUTSIDE_GAP,
                            TextAlign::Center,
                            TextBaseline::Top,
                        ),
                    }
                };

                // 颜色优先取用户配置；否则柱内白字、柱外跟随该子系列色
                let label_color = label_cfg.color.unwrap_or(if inside {
                    Color::rgb(255, 255, 255)
                } else {
                    row.color
                });

                let mut style = TextStyle::new(label_color, label_cfg.font_size, "sans-serif");
                style.align = ha;
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
