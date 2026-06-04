//! Bar Materializer: 将 Bar SeriesSpec 转换为 BarSeries

use vello_cpu::kurbo::Rect;

use crate::{
    error::Result,
    pipeline::{
        materializer::{map_x_to_pixel, map_y_to_pixel, SeriesMaterializer},
        types::{AxisType, ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
        typed_series::{BarRect, BarSeries, TypedSeries},
    },
    visual::Color,
};

pub struct BarMaterializer;

impl SeriesMaterializer for BarMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,
        axis_ranges: &ResolvedAxisRanges,
        color: Color,
        _colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let cfg = match &spec.config {
            SeriesConfig::Bar(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected BarConfig".into(),
                ))
            }
        };

        // 获取 X/Y 轴范围
        let x_range = axis_ranges
            .get_x_range(spec.x_axis_index)
            .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("X axis not found".into()))?;
        let y_range = axis_ranges
            .get_y_range(spec.y_axis_index)
            .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("Y axis not found".into()))?;

        // 判断是否为横向柱状图（Y 轴为分类轴）
        let is_horizontal = matches!(y_range.axis_type, AxisType::Category);

        // 类别列和数值列（用户配置：x_col是类别，y_col是数值）
        let cat_col = &cfg.x_col;
        let val_col = &cfg.y_col;

        // 从 DataFrame 获取数据
        let cat_vals = spec
            .data
            .get_column(cat_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cat_col.clone()))?;
        let val_vals = spec
            .data
            .get_column(val_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(val_col.clone()))?;

        // 将数据点映射到像素矩形
        let mut bars = Vec::with_capacity(spec.data.row_count());

        if is_horizontal {
            // 横向柱状图：Y轴是分类，X轴是数值
            let cat_count = (y_range.max - y_range.min).max(1.0) as usize;
            let bar_height = bounds.height() / cat_count as f64 * cfg.bar_width;
            // 基线：如果0在范围内，使用0；否则使用范围的最小值
            let baseline_x = if x_range.min <= 0.0 && x_range.max >= 0.0 {
                map_x_to_pixel(0.0, x_range, bounds)
            } else if x_range.min > 0.0 {
                // 所有值为正，基线在左边界
                bounds.x0
            } else {
                // 所有值为负，基线在右边界
                bounds.x1
            };

            for i in 0..spec.data.row_count() {
                let value = val_vals.as_f64(i).unwrap_or(0.0);
                let category = cat_vals.as_string(i).unwrap_or_default();

                // 计算 Y 位置（类别中心）
                let cat_idx = i % cat_count;
                let py = bounds.y1 - (cat_idx as f64 + 0.5) / cat_count as f64 * bounds.height();
                let px = map_x_to_pixel(value, x_range, bounds);

                // 创建矩形（从基线延伸到数据点）
                let rect = Rect::new(
                    px.min(baseline_x),
                    py - bar_height / 2.0,
                    px.max(baseline_x),
                    py + bar_height / 2.0,
                );

                bars.push(BarRect {
                    rect,
                    category,
                    value,
                });
            }
        } else {
            // 纵向柱状图：X轴是分类，Y轴是数值
            let cat_count = (x_range.max - x_range.min).max(1.0) as usize;
            let bar_width = bounds.width() / cat_count as f64 * cfg.bar_width;
            // 基线：如果0在范围内，使用0；否则使用范围的最小值（底部）
            let baseline_y = if y_range.min <= 0.0 && y_range.max >= 0.0 {
                map_y_to_pixel(0.0, y_range, bounds)
            } else if y_range.min > 0.0 {
                // 所有值为正，基线在底部
                bounds.y1
            } else {
                // 所有值为负，基线在顶部
                bounds.y0
            };

            for i in 0..spec.data.row_count() {
                let value = val_vals.as_f64(i).unwrap_or(0.0);
                let category = cat_vals.as_string(i).unwrap_or_default();

                // 计算 X 位置（类别中心）
                let cat_idx = i % cat_count;
                let px = bounds.x0 + (cat_idx as f64 + 0.5) / cat_count as f64 * bounds.width();
                let py = map_y_to_pixel(value, y_range, bounds);

                // 创建矩形（从基线延伸到数据点）
                let rect = Rect::new(
                    px - bar_width / 2.0,
                    py.min(baseline_y),
                    px + bar_width / 2.0,
                    py.max(baseline_y),
                );

                bars.push(BarRect {
                    rect,
                    category,
                    value,
                });
            }
        }

        Ok(TypedSeries::Bar(BarSeries {
            name: spec.name.clone(),
            color,
            bars,
        }))
    }
}
