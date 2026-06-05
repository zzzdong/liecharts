//! Bubble Builder: 将 BubbleSeries 组装为 VisualElement

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_POINT, fill_stroke_style},
        typed_series::{BubbleSeries, RenderContext},
    },
    visual::VisualElement,
};

pub struct BubbleBuilder;

impl SeriesBuilder<BubbleSeries> for BubbleBuilder {
    fn build(series: &BubbleSeries, _ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::with_capacity(series.bubbles.len());

        for bubble in &series.bubbles {
            elements.push(VisualElement::Circle {
                center: bubble.center,
                radius: bubble.radius,
                style: fill_stroke_style(
                    series.color.with_alpha(128), // 半透明填充
                    series.color,
                    1.0,
                ),
                z_index: Z_SERIES_POINT,
            });
        }

        Ok(elements)
    }
}

/// 辅助 trait 扩展 Color
trait ColorExt {
    fn with_alpha(&self, alpha: u8) -> Self;
}

impl ColorExt for crate::visual::Color {
    fn with_alpha(&self, alpha: u8) -> Self {
        let mut color = *self;
        color.a = alpha;
        color
    }
}
