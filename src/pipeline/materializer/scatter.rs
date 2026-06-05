//! Scatter Materializer: 将 Scatter SeriesSpec 转换为 ScatterSeries

use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        materializer::{SeriesMaterializer, map_x_to_pixel, map_y_to_pixel},
        typed_series::{ScatterSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
    visual::Color,
};

pub struct ScatterMaterializer;

impl SeriesMaterializer for ScatterMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,
        axis_ranges: &ResolvedAxisRanges,
        color: Color,
        _colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let cfg = match &spec.config {
            SeriesConfig::Scatter(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected ScatterConfig".into(),
                ));
            }
        };

        // 获取 X/Y 轴范围
        let x_range = axis_ranges.get_x_range(spec.x_axis_index).ok_or_else(|| {
            crate::error::ChartError::InvalidAxisBinding("X axis not found".into())
        })?;
        let y_range = axis_ranges.get_y_range(spec.y_axis_index).ok_or_else(|| {
            crate::error::ChartError::InvalidAxisBinding("Y axis not found".into())
        })?;

        // 从 DataFrame 获取数据
        let x_vals = spec
            .data
            .get_column(&cfg.x_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.x_col.clone()))?;
        let y_vals = spec
            .data
            .get_column(&cfg.y_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.y_col.clone()))?;

        // 将数据点映射到像素空间
        let mut points = Vec::with_capacity(spec.data.row_count());

        for i in 0..spec.data.row_count() {
            let x = x_vals.as_f64(i);
            let y = y_vals.as_f64(i);

            if let (Some(x), Some(y)) = (x, y) {
                let px = map_x_to_pixel(x, x_range, bounds);
                let py = map_y_to_pixel(y, y_range, bounds);
                points.push(Point::new(px, py));
            }
        }

        Ok(TypedSeries::Scatter(ScatterSeries {
            name: spec.name.clone(),
            color,
            symbol_size: cfg.symbol_size,
            points,
        }))
    }
}
