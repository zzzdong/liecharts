//! Heatmap Builder: 将 HeatmapSeries 组装为 VisualElement

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, fill_stroke_style, fill_style},
        typed_series::{HeatmapSeries, RenderContext},
    },
    visual::VisualElement,
};

pub struct HeatmapBuilder;

impl SeriesBuilder<HeatmapSeries> for HeatmapBuilder {
    fn build(series: &HeatmapSeries, _ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.cells.len());

        for cell in &series.cells {
            let style = if cell.border_width > 0.0 {
                if let Some(border) = cell.border_color {
                    fill_stroke_style(cell.color, border, cell.border_width)
                } else {
                    fill_style(cell.color)
                }
            } else {
                fill_style(cell.color)
            };

            elements.push(VisualElement::Rect {
                rect: cell.rect,
                style,
                z_index: Z_SERIES_FILL,
            });
        }

        Ok(elements)
    }
}
