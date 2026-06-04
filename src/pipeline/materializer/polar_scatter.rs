//! PolarScatter Materializer: 将 PolarScatter SeriesSpec 转换为 PolarScatterSeries

use vello_cpu::kurbo::Rect;

use crate::{
    error::Result,
    pipeline::{
        materializer::SeriesMaterializer,
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
        typed_series::{PolarPoint, PolarScatterSeries, TypedSeries},
    },
    visual::Color,
};

pub struct PolarScatterMaterializer;

impl SeriesMaterializer for PolarScatterMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,
        _axis_ranges: &ResolvedAxisRanges,
        color: Color,
        _colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let cfg = match &spec.config {
            SeriesConfig::PolarScatter(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected PolarScatterConfig".into(),
                ))
            }
        };

        // 从 DataFrame 获取数据
        let angle_col = spec
            .data
            .get_column(&cfg.angle_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.angle_col.clone()))?;
        let radius_col = spec
            .data
            .get_column(&cfg.radius_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.radius_col.clone()))?;

        // 计算极坐标中心点和最大半径
        let center_x = bounds.x0 + bounds.width() / 2.0;
        let center_y = bounds.y0 + bounds.height() / 2.0;
        let max_radius = bounds.width().min(bounds.height()) / 2.0 * 0.8;

        // 收集数据
        let mut points = Vec::with_capacity(spec.data.row_count());

        for i in 0..spec.data.row_count() {
            let angle = angle_col.as_f64(i).unwrap_or(0.0);
            let radius_val = radius_col.as_f64(i).unwrap_or(0.0);

            // 归一化半径
            let radius = (radius_val / 100.0).clamp(0.0, 1.0) * max_radius;

            points.push(PolarPoint {
                angle,
                radius,
                value: radius_val,
                name: format!("Point {}", i),
            });
        }

        Ok(TypedSeries::PolarScatter(PolarScatterSeries {
            name: spec.name.clone(),
            color,
            symbol_size: cfg.symbol_size,
            points,
        }))
    }
}
