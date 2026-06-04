//! Bar Builder: 将 BarSeries 组装为 VisualElement

use crate::{
    error::Result,
    pipeline::builder::{fill_style, SeriesBuilder, Z_SERIES_FILL},
    pipeline::typed_series::{BarSeries, RenderContext},
    visual::VisualElement,
};

pub struct BarBuilder;

impl SeriesBuilder<BarSeries> for BarBuilder {
    fn build(series: &BarSeries, _ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.bars.len());

        for bar in &series.bars {
            elements.push(VisualElement::Rect {
                rect: bar.rect,
                style: fill_style(series.color),
                z_index: Z_SERIES_FILL,
            });
        }

        Ok(elements)
    }
}
