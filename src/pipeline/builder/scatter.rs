//! Scatter Builder: 将 ScatterSeries 组装为 lievisual `SceneNode`

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_POINT, circle, fill_stroke_style},
        typed_series::{RenderContext, ScatterSeries},
    },
};

pub struct ScatterBuilder;

impl SeriesBuilder<ScatterSeries> for ScatterBuilder {
    fn build(series: &ScatterSeries, _ctx: &RenderContext) -> Result<Vec<lievisual::scene::SceneNode>> {
        let mut elements = Vec::with_capacity(series.points.len());

        for point in &series.points {
            elements.push(circle(
                *point,
                series.symbol_size / 2.0,
                fill_stroke_style(series.color, series.color, 1.0),
                Z_SERIES_POINT,
            ));
        }

        Ok(elements)
    }
}
