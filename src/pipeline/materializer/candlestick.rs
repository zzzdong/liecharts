//! Candlestick Materializer: 将 Candlestick SeriesSpec 转换为 CandlestickSeries

use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        materializer::{SeriesMaterializer, map_y_to_pixel},
        typed_series::{CandleRect, CandlestickSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
    visual::Color,
};

pub struct CandlestickMaterializer;

impl SeriesMaterializer for CandlestickMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,
        axis_ranges: &ResolvedAxisRanges,
        _color: Color,
        colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let cfg = match &spec.config {
            SeriesConfig::Candlestick(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected CandlestickConfig".into(),
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
        let category_col = spec
            .data
            .get_column(&cfg.category_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.category_col.clone()))?;
        let open_col = spec
            .data
            .get_column(&cfg.open_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.open_col.clone()))?;
        let close_col = spec
            .data
            .get_column(&cfg.close_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.close_col.clone()))?;
        let low_col = spec
            .data
            .get_column(&cfg.low_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.low_col.clone()))?;
        let high_col = spec
            .data
            .get_column(&cfg.high_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.high_col.clone()))?;

        // 计算蜡烛宽度 — K线图通常比较紧凑窄小
        let cat_count = (x_range.max - x_range.min).max(1.0) as usize;
        let slot_width = bounds.width() / cat_count as f64;
        let candle_width = (slot_width * 0.35).max(2.0); // 窄小紧凑，最小 2px

        // 将数据点映射到像素空间
        let mut candles = Vec::with_capacity(spec.data.row_count());

        for i in 0..spec.data.row_count() {
            let open = open_col.as_f64(i).unwrap_or(0.0);
            let close = close_col.as_f64(i).unwrap_or(0.0);
            let low = low_col.as_f64(i).unwrap_or(0.0);
            let high = high_col.as_f64(i).unwrap_or(0.0);
            let category = category_col.as_string(i).unwrap_or_default();

            // 计算 X 位置（类别中心）
            let cat_idx = i % cat_count;
            let px = bounds.x0 + (cat_idx as f64 + 0.5) / cat_count as f64 * bounds.width();

            // 映射 Y 坐标
            let py_open = map_y_to_pixel(open, y_range, bounds);
            let py_close = map_y_to_pixel(close, y_range, bounds);
            let py_low = map_y_to_pixel(low, y_range, bounds);
            let py_high = map_y_to_pixel(high, y_range, bounds);

            // 创建影线
            let high_line = (
                Point::new(px, py_high),
                Point::new(px, py_open.max(py_close)),
            );
            let low_line = (
                Point::new(px, py_open.min(py_close)),
                Point::new(px, py_low),
            );

            // 创建实体矩形
            let body_rect = Rect::new(
                px - candle_width / 2.0,
                py_open.min(py_close),
                px + candle_width / 2.0,
                py_open.max(py_close),
            );

            let is_up = close >= open;

            candles.push(CandleRect {
                category,
                high_line,
                low_line,
                body_rect,
                is_up,
            });
        }

        Ok(TypedSeries::Candlestick(CandlestickSeries {
            name: spec.name.clone(),
            up_color: colors.up_color,
            down_color: colors.down_color,
            candles,
        }))
    }
}
