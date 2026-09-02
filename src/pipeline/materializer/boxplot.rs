//! Boxplot Materializer: 将 Boxplot SeriesSpec 转换为 BoxplotSeries

use vello_cpu::kurbo::{Point, Rect};

use crate::{
    Color,
    error::Result,
    pipeline::{
        materializer::{SeriesMaterializer, map_x_to_pixel, map_y_to_pixel},
        typed_series::{BoxplotRect, BoxplotSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
};

pub struct BoxplotMaterializer;

impl SeriesMaterializer for BoxplotMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,
        axis_ranges: &ResolvedAxisRanges,
        color: Color,
        _colors: &ColorContext,
    ) -> Result<TypedSeries> {
        let cfg = match &spec.config {
            SeriesConfig::Boxplot(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected BoxplotConfig".into(),
                ));
            }
        };

        let x_range = axis_ranges.get_x_range(spec.x_axis_index).ok_or_else(|| {
            crate::error::ChartError::InvalidAxisBinding("X axis not found".into())
        })?;
        let y_range = axis_ranges.get_y_range(spec.y_axis_index).ok_or_else(|| {
            crate::error::ChartError::InvalidAxisBinding("Y axis not found".into())
        })?;

        let category_col = spec
            .data
            .get_column(&cfg.category_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.category_col.clone()))?;
        let min_col = spec
            .data
            .get_column(&cfg.min_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.min_col.clone()))?;
        let q1_col = spec
            .data
            .get_column(&cfg.q1_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.q1_col.clone()))?;
        let median_col = spec
            .data
            .get_column(&cfg.median_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.median_col.clone()))?;
        let q3_col = spec
            .data
            .get_column(&cfg.q3_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.q3_col.clone()))?;
        let max_col = spec
            .data
            .get_column(&cfg.max_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.max_col.clone()))?;

        // 类目总数与留白风格直接取自解析结果，与坐标轴刻度口径严格一致
        let n_cat = x_range.category_count().max(1);
        let slot_width = bounds.width() / n_cat as f64;
        let box_width = (slot_width * 0.5).max(4.0); // 箱线图比 K 线略宽

        let mut boxes = Vec::with_capacity(spec.data.row_count());

        for i in 0..spec.data.row_count() {
            let mut five = [
                min_col.as_f64(i).unwrap_or(0.0),
                q1_col.as_f64(i).unwrap_or(0.0),
                median_col.as_f64(i).unwrap_or(0.0),
                q3_col.as_f64(i).unwrap_or(0.0),
                max_col.as_f64(i).unwrap_or(0.0),
            ];
            // 数值乱序（如 min>max、q1>q3）会产出负高矩形：统一排序归一，
            // 非有限值按 0 处理，避免 NaN 坐标。
            for v in five.iter_mut() {
                if !v.is_finite() {
                    *v = 0.0;
                }
            }
            five.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let (min, q1, median, q3, max) = (five[0], five[1], five[2], five[3], five[4]);
            let category = category_col.as_string(i).unwrap_or_default();

            let cat_idx = i.min(n_cat - 1);
            let px = map_x_to_pixel(x_range.category_value(cat_idx), x_range, bounds);
            let half_width = box_width / 2.0;

            let py_min = map_y_to_pixel(min, y_range, bounds);
            let py_q1 = map_y_to_pixel(q1, y_range, bounds);
            let py_median = map_y_to_pixel(median, y_range, bounds);
            let py_q3 = map_y_to_pixel(q3, y_range, bounds);
            let py_max = map_y_to_pixel(max, y_range, bounds);

            // whisker 垂直线：从 min 到 max
            let whisker_line = (Point::new(px, py_max), Point::new(px, py_min));
            // whisker 端点横线
            let top_whisker = (
                Point::new(px - half_width, py_max),
                Point::new(px + half_width, py_max),
            );
            let bottom_whisker = (
                Point::new(px - half_width, py_min),
                Point::new(px + half_width, py_min),
            );
            // 箱体：Q1 到 Q3
            let body_rect = Rect::new(px - half_width, py_q3, px + half_width, py_q1);
            // 中位数线
            let median_line = (
                Point::new(px - half_width, py_median),
                Point::new(px + half_width, py_median),
            );

            boxes.push(BoxplotRect {
                category,
                whisker_line,
                top_whisker,
                bottom_whisker,
                body_rect,
                median_line,
            });
        }

        let border_color = spec.item_style.border_color.unwrap_or(color);

        Ok(TypedSeries::Boxplot(BoxplotSeries {
            name: spec.name.clone(),
            color: spec.item_style.color.unwrap_or(color),
            border_color,
            border_width: spec.item_style.border_width.unwrap_or(1.5),
            boxes,
        }))
    }
}
