//! Boxplot Materializer: 将 Boxplot SeriesSpec 转换为 BoxplotSeries

use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        materializer::{SeriesMaterializer, map_y_to_pixel},
        typed_series::{BoxplotRect, BoxplotSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
    visual::Color,
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

        let cat_count = (x_range.max - x_range.min).max(1.0) as usize;
        let slot_width = bounds.width() / cat_count as f64;
        let box_width = (slot_width * 0.5).max(4.0); // 箱线图比 K 线略宽

        let mut boxes = Vec::with_capacity(spec.data.row_count());

        for i in 0..spec.data.row_count() {
            let min = min_col.as_f64(i).unwrap_or(0.0);
            let q1 = q1_col.as_f64(i).unwrap_or(0.0);
            let median = median_col.as_f64(i).unwrap_or(0.0);
            let q3 = q3_col.as_f64(i).unwrap_or(0.0);
            let max = max_col.as_f64(i).unwrap_or(0.0);
            let category = category_col.as_string(i).unwrap_or_default();

            let cat_idx = i % cat_count;
            let px = bounds.x0 + (cat_idx as f64 + 0.5) / cat_count as f64 * bounds.width();
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
