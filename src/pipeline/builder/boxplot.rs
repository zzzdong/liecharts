//! Boxplot Builder: 将 BoxplotSeries 组装为 VisualElement

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LINE, stroke_style},
        typed_series::{BoxplotSeries, RenderContext},
    },
    visual::{FillStrokeStyle, Stroke, VisualElement},
};

pub struct BoxplotBuilder;

impl SeriesBuilder<BoxplotSeries> for BoxplotBuilder {
    fn build(series: &BoxplotSeries, _ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.boxes.len() * 6); // 每个箱线图 6 个视觉元素
        let median_color = crate::visual::Color::new(255, 255, 255);

        for b in &series.boxes {
            let color = series.color;
            let border_color = series.border_color;
            let line_width = series.border_width;

            // whisker 垂直线：min 到 max
            elements.push(VisualElement::Line {
                start: b.whisker_line.0,
                end: b.whisker_line.1,
                style: stroke_style(border_color, line_width),
                z_index: Z_SERIES_LINE,
            });
            // whisker 顶端横线（max）
            elements.push(VisualElement::Line {
                start: b.top_whisker.0,
                end: b.top_whisker.1,
                style: stroke_style(border_color, line_width),
                z_index: Z_SERIES_LINE,
            });
            // whisker 底端横线（min）
            elements.push(VisualElement::Line {
                start: b.bottom_whisker.0,
                end: b.bottom_whisker.1,
                style: stroke_style(border_color, line_width),
                z_index: Z_SERIES_LINE,
            });

            // 箱体：Q1 到 Q3，填充 + 边框
            elements.push(VisualElement::Rect {
                rect: b.body_rect,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: Some(Stroke {
                        color: border_color,
                        width: line_width,
                    }),
                },
                z_index: Z_SERIES_FILL,
            });

            // 中位数线
            elements.push(VisualElement::Line {
                start: b.median_line.0,
                end: b.median_line.1,
                style: stroke_style(median_color, line_width.max(1.5)),
                z_index: Z_SERIES_LINE,
            });
        }

        Ok(elements)
    }
}
