//! PolarBar Materializer: 将 PolarBar SeriesSpec 转换为 PolarBarSeries

use vello_cpu::kurbo::Rect;

use crate::{
    Color,
    error::Result,
    pipeline::{
        materializer::SeriesMaterializer,
        typed_series::{PolarBarPoint, PolarBarSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
};

pub struct PolarBarMaterializer;

impl SeriesMaterializer for PolarBarMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,
        _axis_ranges: &ResolvedAxisRanges,
        color: Color,
        _colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let cfg = match &spec.config {
            SeriesConfig::PolarBar(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected PolarBarConfig".into(),
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

        // 类目名（数据项名称）列：显式配置优先，否则探测常见列名
        let category_vals = cfg
            .category_col
            .as_ref()
            .and_then(|c| spec.data.get_column(c))
            .or_else(|| {
                ["label", "category", "name"]
                    .iter()
                    .find_map(|c| spec.data.get_column(c))
            });

        // 计算极坐标中心点和最大半径
        let _center_x = bounds.x0 + bounds.width() / 2.0;
        let _center_y = bounds.y0 + bounds.height() / 2.0;
        let max_radius = bounds.width().min(bounds.height()) / 2.0 * 0.8;

        // 收集数据
        let mut bars = Vec::with_capacity(spec.data.row_count());
        let mut max_radius_val = 0.0_f64;

        // 先找到最大半径值用于归一化
        for i in 0..spec.data.row_count() {
            let radius_val = radius_col.as_f64(i).unwrap_or(0.0);
            if radius_val > max_radius_val {
                max_radius_val = radius_val;
            }
        }

        if max_radius_val <= 0.0 {
            max_radius_val = 1.0; // 避免除以0
        }

        for i in 0..spec.data.row_count() {
            let angle = angle_col.as_f64(i).unwrap_or(0.0);
            let radius_val = radius_col.as_f64(i).unwrap_or(0.0);

            // 基于实际数据最大值归一化半径
            let radius = (radius_val / max_radius_val).clamp(0.0, 1.0) * max_radius;

            // 从 palette 获取颜色（循环使用）
            let bar_color = _colors.get_series_color(i);

            bars.push(PolarBarPoint {
                angle: angle + cfg.start_angle,
                radius,
                value: radius_val,
                name: category_vals
                    .and_then(|s| s.as_string(i))
                    .unwrap_or_else(|| format!("Item {}", i)),
                color: bar_color,
            });
        }

        Ok(TypedSeries::PolarBar(PolarBarSeries {
            name: spec.name.clone(),
            color,
            pad_angle: cfg.pad_angle,
            start_angle: cfg.start_angle,
            bars,
        }))
    }
}
