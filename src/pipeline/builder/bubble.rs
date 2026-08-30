//! Bubble Builder: 将 BubbleSeries 组装为 lievisual `SceneNode`

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_POINT, circle, fill_stroke_style},
        typed_series::{BubbleSeries, RenderContext},
    },
};

pub struct BubbleBuilder;

impl SeriesBuilder<BubbleSeries> for BubbleBuilder {
    fn build(
        series: &BubbleSeries,
        _ctx: &RenderContext,
    ) -> Result<Vec<lievisual::scene::SceneNode>> {
        let mut elements = Vec::with_capacity(series.bubbles.len());

        for bubble in &series.bubbles {
            let mut fill = series.color;
            fill.a = 128; // 半透明填充
            elements.push(circle(
                bubble.center,
                bubble.radius,
                fill_stroke_style(fill, series.color, 1.0),
                Z_SERIES_POINT,
            ));
        }

        Ok(elements)
    }
}
