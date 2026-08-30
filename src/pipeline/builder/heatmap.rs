//! Heatmap Builder: 将 HeatmapSeries 组装为 lievisual `SceneNode`

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, fill_stroke_style, fill_style, rect},
        typed_series::{HeatmapSeries, RenderContext},
    },
};

pub struct HeatmapBuilder;

impl SeriesBuilder<HeatmapSeries> for HeatmapBuilder {
    fn build(
        series: &HeatmapSeries,
        _ctx: &RenderContext,
    ) -> Result<Vec<lievisual::scene::SceneNode>> {
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

            elements.push(rect(cell.rect, style, Z_SERIES_FILL));
        }

        Ok(elements)
    }
}
