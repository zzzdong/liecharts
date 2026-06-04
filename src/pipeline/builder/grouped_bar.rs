//! GroupedBar Builder: 将 GroupedBarSeries 组装为 VisualElement

use crate::{
    error::Result,
    pipeline::builder::{fill_style, SeriesBuilder, Z_SERIES_FILL},
    pipeline::typed_series::{GroupedBarSeries, RenderContext},
    visual::VisualElement,
};

pub struct GroupedBarBuilder;

impl SeriesBuilder<GroupedBarSeries> for GroupedBarBuilder {
    fn build(series: &GroupedBarSeries, _ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.rows.len());

        for row in &series.rows {
            elements.push(VisualElement::Rect {
                rect: row.bar_rect,
                style: fill_style(row.color),
                z_index: Z_SERIES_FILL,
            });
        }

        Ok(elements)
    }
}
