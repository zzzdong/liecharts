//! Bubble Materializer: 将 Bubble SeriesSpec 转换为 BubbleSeries

use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        materializer::{SeriesMaterializer, map_x_to_pixel, map_y_to_pixel},
        typed_series::{Bubble, BubbleSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
    visual::Color,
};

pub struct BubbleMaterializer;

impl SeriesMaterializer for BubbleMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,
        axis_ranges: &ResolvedAxisRanges,
        color: Color,
        _colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let cfg = match &spec.config {
            SeriesConfig::Bubble(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected BubbleConfig".into(),
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

        // 可选的大小列
        let size_vals = cfg
            .size_col
            .as_ref()
            .and_then(|col| spec.data.get_column(col));

        // 可选的名称列
        let name_vals = cfg
            .name_col
            .as_ref()
            .and_then(|col| spec.data.get_column(col));

        // 将数据点映射到像素空间
        let mut bubbles = Vec::with_capacity(spec.data.row_count());

        for i in 0..spec.data.row_count() {
            let x = x_vals.as_f64(i);
            let y = y_vals.as_f64(i);

            if let (Some(x), Some(y)) = (x, y) {
                let px = map_x_to_pixel(x, x_range, bounds);
                let py = map_y_to_pixel(y, y_range, bounds);

                // 计算气泡大小：使用 sqrt 使面积代表数值，而非半径
                let radius = if let Some(size_series) = size_vals {
                    size_series.as_f64(i).unwrap_or(10.0).sqrt() * cfg.symbol_size_scale
                } else {
                    10.0 * cfg.symbol_size_scale
                };

                // 获取名称
                let name = name_vals
                    .as_ref()
                    .and_then(|s| s.as_string(i))
                    .unwrap_or_default();

                bubbles.push(Bubble {
                    center: Point::new(px, py),
                    radius,
                    name,
                });
            }
        }

        Ok(TypedSeries::Bubble(BubbleSeries {
            name: spec.name.clone(),
            color,
            bubbles,
        }))
    }
}
