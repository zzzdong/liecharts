//! Bar Materializer: 将 Bar SeriesSpec 转换为 BarSeries

use vello_cpu::kurbo::Rect;

use crate::{
    Color,
    error::Result,
    pipeline::{
        materializer::{SeriesMaterializer, map_x_to_pixel, map_y_to_pixel},
        typed_series::{BarRect, BarSeries, TypedSeries},
        types::{AxisType, ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
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

        // 判断是否为横向柱状图（Y 轴为分类轴）
        let is_horizontal = matches!(y_range.axis_type, AxisType::Category);

        // 将数据点映射到像素矩形
        let mut bars = Vec::with_capacity(spec.data.row_count());

        if is_horizontal {
            // 横向柱状图：Y轴是分类，X轴是数值
            // 数据布局：x_col 是数值，y_col 是分类索引
            let x_vals = spec
                .data
                .get_column(&cfg.x_col)
                .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.x_col.clone()))?;
            let y_vals = spec
                .data
                .get_column(&cfg.y_col)
                .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.y_col.clone()))?;

            // 类目总数与留白风格直接取自解析结果，与坐标轴刻度口径严格一致
            let n_cat = y_range.category_count().max(1);
            let bar_height = bounds.height() / n_cat as f64 * cfg.bar_width;
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
                let value = x_vals.as_f64(i).unwrap_or(0.0);
                let cat_idx = y_vals.as_f64(i).unwrap_or(i as f64).min(n_cat as f64 - 1.0) as usize;

                // 类别标签从 Y 轴配置获取
                let category = if let Some(cat) = y_range.categories.get(cat_idx) {
                    cat.clone()
                } else {
                    format!("{}", cat_idx)
                };

                // 计算 Y 位置（类别中心）
                let py = map_y_to_pixel(y_range.category_value(cat_idx), y_range, bounds);
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
            // 数据布局：X列是索引，Y列是数值
            let x_vals = spec
                .data
                .get_column(&cfg.x_col)
                .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.x_col.clone()))?;
            let y_vals = spec
                .data
                .get_column(&cfg.y_col)
                .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.y_col.clone()))?;
            // 纵向柱状图：X轴是分类，Y轴是数值
            // 类目总数与留白风格直接取自解析结果，与坐标轴刻度口径严格一致
            let n_cat = x_range.category_count().max(1);
            let bar_width = bounds.width() / n_cat as f64 * cfg.bar_width;
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
                let value = y_vals.as_f64(i).unwrap_or(0.0);
                let cat_idx = x_vals.as_f64(i).unwrap_or(i as f64).min(n_cat as f64 - 1.0) as usize;

                // 类别标签从 X 轴配置获取
                let category = if let Some(cat) = x_range.categories.get(cat_idx) {
                    cat.clone()
                } else {
                    format!("{}", cat_idx)
                };

                // 计算 X 位置（类别中心）
                let px = map_x_to_pixel(x_range.category_value(cat_idx), x_range, bounds);
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

        let bar_values: Vec<f64> = bars.iter().map(|b| b.value).collect();
        let mark_lines = crate::pipeline::materializer::compute_mark_lines(
            &cfg.mark_line,
            &bar_values,
            y_range,
            bounds,
        );

        Ok(TypedSeries::Bar(BarSeries {
            name: spec.name.clone(),
            color,
            bars,
            label: crate::pipeline::materializer::bar_label_config(cfg),
            mark_lines,
        }))
    }
}
