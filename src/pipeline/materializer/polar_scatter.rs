//! PolarScatter Materializer: 将 PolarScatter SeriesSpec 转换为 PolarScatterSeries

use vello_cpu::kurbo::Rect;

use crate::{
    Color,
    error::Result,
    pipeline::{
        materializer::SeriesMaterializer,
        typed_series::{PolarPoint, PolarScatterSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
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
                ));
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
        let _center_x = bounds.x0 + bounds.width() / 2.0;
        let _center_y = bounds.y0 + bounds.height() / 2.0;
        let max_radius = bounds.width().min(bounds.height()) / 2.0 * 0.8;

        // 收集数据
        let mut points = Vec::with_capacity(spec.data.row_count());
        let mut max_radius_val = 0.0_f64;

        // 先找到最大半径值用于归一化
        for i in 0..spec.data.row_count() {
            let radius_val = radius_col.as_f64(i).unwrap_or(0.0);
            if radius_val > max_radius_val {
                max_radius_val = radius_val;
            }
        }

        if max_radius_val <= 0.0 {
            max_radius_val = 1.0;
        }

        // 计算点大小的范围（基于风速值）
        let mut min_val = f64::MAX;
        let mut max_val = f64::MIN;
        for i in 0..spec.data.row_count() {
            let val = radius_col.as_f64(i).unwrap_or(0.0);
            min_val = min_val.min(val);
            max_val = max_val.max(val);
        }
        if max_val <= min_val {
            min_val = 0.0;
            max_val = 1.0;
        }

        // 点大小范围：最小 3px，最大 15px
        let min_size = 3.0;
        let max_size = 15.0;

        for i in 0..spec.data.row_count() {
            let angle = angle_col.as_f64(i).unwrap_or(0.0);
            let radius_val = radius_col.as_f64(i).unwrap_or(0.0);

            // 基于实际数据最大值归一化半径
            let radius = (radius_val / max_radius_val).clamp(0.0, 1.0) * max_radius;

            // 根据风速值计算点大小
            let size = if max_val > min_val {
                min_size + (radius_val - min_val) / (max_val - min_val) * (max_size - min_size)
            } else {
                min_size
            };

            points.push(PolarPoint {
                angle,
                radius,
                value: radius_val,
                name: format!("Point {}", i),
                size,
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
