//! Radar Materializer: 将 Radar SeriesSpec 转换为 RadarSeries

use vello_cpu::kurbo::Rect;

use crate::{
    Color,
    error::Result,
    pipeline::{
        materializer::SeriesMaterializer,
        typed_series::{RadarSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
};

pub struct RadarMaterializer;

impl SeriesMaterializer for RadarMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        _bounds: Rect,
        _axis_ranges: &ResolvedAxisRanges,
        color: Color,
        _colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let cfg = match &spec.config {
            SeriesConfig::Radar(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected RadarConfig".into(),
                ));
            }
        };

        // 从 DataFrame 获取数据
        let value_col = spec
            .data
            .get_column(&cfg.value_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.value_col.clone()))?;

        // 收集数值
        // 支持两种格式：
        // 1. 多行数值（每行一个指标值）
        // 2. 单行逗号分隔字符串（如 "95,80,75,90,85"）
        let mut values = Vec::new();

        for i in 0..spec.data.row_count() {
            if let Some(v) = value_col.as_f64(i) {
                // 格式1：直接数值。NaN/Inf 视为 0，保持与指标数对齐
                values.push(if v.is_finite() { v } else { 0.0 });
            } else if let Some(s) = value_col.as_string(i) {
                // 格式2：逗号分隔字符串
                for part in s.split(',') {
                    if let Ok(v) = part.trim().parse::<f64>() {
                        values.push(if v.is_finite() { v } else { 0.0 });
                    }
                }
            }
        }

        Ok(TypedSeries::Radar(RadarSeries {
            name: spec.name.clone(),
            color,
            indicators: cfg.indicators.clone(),
            values,
        }))
    }
}
