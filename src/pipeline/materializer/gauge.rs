//! Gauge Materializer: 将 Gauge SeriesSpec 转换为 GaugeSeries

use lievisual::kurbo::Rect;

use crate::{
    Color,
    error::Result,
    pipeline::{
        materializer::SeriesMaterializer,
        typed_series::{GaugeSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
};

pub struct GaugeMaterializer;

impl SeriesMaterializer for GaugeMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        _bounds: Rect,
        _axis_ranges: &ResolvedAxisRanges,
        color: Color,
        _colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let cfg = match &spec.config {
            SeriesConfig::Gauge(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected GaugeConfig".into(),
                ));
            }
        };

        // 从 DataFrame 获取数据
        let value_col = spec
            .data
            .get_column(&cfg.value_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.value_col.clone()))?;

        // 获取数值
        let value = value_col.as_f64(0).unwrap_or(0.0);

        Ok(TypedSeries::Gauge(GaugeSeries {
            name: spec.name.clone(),
            min: cfg.min,
            max: cfg.max,
            center: cfg.center,
            radius: cfg.radius,
            start_angle: cfg.start_angle,
            end_angle: cfg.end_angle,
            split_number: cfg.split_number,
            value,
            color,
            // Q2：显式字号原样透传，由 builder 按「显式 > 根 × 系数」解析
            axis_label_font_size: cfg.axis_label_font_size,
            title_font_size: cfg.title_font_size,
            detail_font_size: cfg.detail_font_size,
        }))
    }
}
