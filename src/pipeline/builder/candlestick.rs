//! Candlestick Builder: 将 CandlestickSeries 组装为 VisualElement

use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, Z_SERIES_LINE, fill_stroke_style, stroke_style},
        typed_series::{CandlestickSeries, RenderContext},
    },
    visual::{FillStrokeStyle, Stroke, VisualElement},
};

pub struct CandlestickBuilder;

impl SeriesBuilder<CandlestickSeries> for CandlestickBuilder {
    fn build(series: &CandlestickSeries, _ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.candles.len() * 5); // 每个蜡烛：上影线+端线、下影线+端线、实体

        for candle in &series.candles {
            let color = if candle.is_up {
                series.up_color
            } else {
                series.down_color
            };
            let body_width = candle.body_rect.x1 - candle.body_rect.x0;
            let half_width = body_width / 2.0;
            let cx = candle.body_rect.x0 + half_width; // 蜡烛中心 x

            // 上影线 + 端点横线
            elements.push(VisualElement::Line {
                start: candle.high_line.0,
                end: candle.high_line.1,
                style: stroke_style(color, 1.0),
                z_index: Z_SERIES_LINE,
            });
            // 上影线顶端的横线（与蜡烛体同宽）
            elements.push(VisualElement::Line {
                start: Point::new(cx - half_width, candle.high_line.0.y),
                end: Point::new(cx + half_width, candle.high_line.0.y),
                style: stroke_style(color, 1.0),
                z_index: Z_SERIES_LINE,
            });

            // 下影线 + 端点横线
            elements.push(VisualElement::Line {
                start: candle.low_line.0,
                end: candle.low_line.1,
                style: stroke_style(color, 1.0),
                z_index: Z_SERIES_LINE,
            });
            // 下影线底端的横线（与蜡烛体同宽）
            elements.push(VisualElement::Line {
                start: Point::new(cx - half_width, candle.low_line.1.y),
                end: Point::new(cx + half_width, candle.low_line.1.y),
                style: stroke_style(color, 1.0),
                z_index: Z_SERIES_LINE,
            });

            // 实体 — 阳线空心（仅描边），阴线实心（填充）
            if candle.is_up {
                // 阳线：仅描边，空心
                elements.push(VisualElement::Rect {
                    rect: candle.body_rect,
                    style: FillStrokeStyle {
                        fill: None,
                        stroke: Some(Stroke { color, width: 1.5 }),
                    },
                    z_index: Z_SERIES_FILL,
                });
            } else {
                // 阴线：实心填充
                elements.push(VisualElement::Rect {
                    rect: candle.body_rect,
                    style: fill_stroke_style(color, color, 1.0),
                    z_index: Z_SERIES_FILL,
                });
            }
        }

        Ok(elements)
    }
}
