//! Scatter Builder: 将 ScatterSeries 组装为 VisualElement

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_POINT, fill_stroke_style},
        typed_series::{RenderContext, ScatterSeries},
    },
    visual::VisualElement,
};

pub struct ScatterBuilder;

impl SeriesBuilder<ScatterSeries> for ScatterBuilder {
    fn build(series: &ScatterSeries, _ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.points.len());

        for point in &series.points {
            elements.push(VisualElement::Circle {
                center: *point,
                radius: series.symbol_size / 2.0,
                style: fill_stroke_style(series.color, series.color, 1.0),
                z_index: Z_SERIES_POINT,
            });
        }

        Ok(elements)
    }
}
