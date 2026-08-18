//! Boxplot Builder: 将 BoxplotSeries 组装为 lievisual `SceneNode`

use lievisual::scene::{FillStrokeStyle, SceneNode, Stroke};

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LINE, line, rect, stroke_style},
        typed_series::{BoxplotSeries, RenderContext},
    },
};

pub struct BoxplotBuilder;

impl SeriesBuilder<BoxplotSeries> for BoxplotBuilder {
    fn build(series: &BoxplotSeries, _ctx: &RenderContext) -> Result<Vec<SceneNode>> {
        let mut elements = Vec::with_capacity(series.boxes.len() * 6); // 每个箱线图 6 个视觉元素
        let median_color = crate::visual::Color::rgb(255, 255, 255);

        for b in &series.boxes {
            let color = series.color;
            let border_color = series.border_color;
            let line_width = series.border_width;

            // whisker 垂直线：min 到 max
            elements.push(line(
                b.whisker_line.0,
                b.whisker_line.1,
                stroke_style(border_color, line_width),
                Z_SERIES_LINE,
            ));
            // whisker 顶端横线（max）
            elements.push(line(
                b.top_whisker.0,
                b.top_whisker.1,
                stroke_style(border_color, line_width),
                Z_SERIES_LINE,
            ));
            // whisker 底端横线（min）
            elements.push(line(
                b.bottom_whisker.0,
                b.bottom_whisker.1,
                stroke_style(border_color, line_width),
                Z_SERIES_LINE,
            ));

            // 箱体：Q1 到 Q3，填充 + 边框
            elements.push(rect(
                b.body_rect,
                FillStrokeStyle {
                    fill: Some(lievisual::scene::Fill::Solid(color)),
                    stroke: Some(Stroke::new(border_color, line_width)),
                },
                Z_SERIES_FILL,
            ));

            // 中位数线
            elements.push(line(
                b.median_line.0,
                b.median_line.1,
                stroke_style(median_color, line_width.max(1.5)),
                Z_SERIES_LINE,
            ));
        }

        Ok(elements)
    }
}
