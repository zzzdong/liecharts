//! Candlestick Builder: 将 CandlestickSeries 组装为 VisualElement

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LINE, fill_stroke_style, stroke_style},
        typed_series::{CandlestickSeries, RenderContext},
    },
    visual::VisualElement,
};

pub struct CandlestickBuilder;

impl SeriesBuilder<CandlestickSeries> for CandlestickBuilder {
    fn build(series: &CandlestickSeries, _ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.candles.len() * 3); // 每个蜡烛有上影线、下影线和实体

        for candle in &series.candles {
            let color = if candle.is_up {
                series.up_color
            } else {
                series.down_color
            };

            // 上影线
            elements.push(VisualElement::Line {
                start: candle.high_line.0,
                end: candle.high_line.1,
                style: stroke_style(color, 1.0),
                z_index: Z_SERIES_LINE,
            });

            // 下影线
            elements.push(VisualElement::Line {
                start: candle.low_line.0,
                end: candle.low_line.1,
                style: stroke_style(color, 1.0),
                z_index: Z_SERIES_LINE,
            });

            // 实体
            elements.push(VisualElement::Rect {
                rect: candle.body_rect,
                style: fill_stroke_style(color, color, 1.0),
                z_index: Z_SERIES_FILL,
            });
        }

        Ok(elements)
    }
}
