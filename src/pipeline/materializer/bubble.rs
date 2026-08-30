//! Bubble Materializer: 将 Bubble SeriesSpec 转换为 BubbleSeries

use vello_cpu::kurbo::{Point, Rect};

use crate::{
    Color,
    error::Result,
    pipeline::{
        materializer::{SeriesMaterializer, map_x_to_pixel, map_y_to_pixel},
        typed_series::{Bubble, BubbleSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
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
        let row_count = spec.data.row_count();
        let mut bubbles = Vec::with_capacity(row_count);

        // 第一遍：收集 sqrt(size)，用于归一化到合理的半径区间。
        // 直接把 size 映射为半径会让最大值与最小值的视觉差异过于悬殊
        // （size=400 → r=20，size=10 → r=3，再乘 scale 后小气泡几乎不可见）。
        let sqrt_sizes: Vec<Option<f64>> = (0..row_count)
            .map(|i| size_vals.and_then(|s| s.as_f64(i)).map(|v| v.sqrt()))
            .collect();
        let s_min = sqrt_sizes
            .iter()
            .filter_map(|s| *s)
            .fold(f64::INFINITY, f64::min);
        let s_max = sqrt_sizes
            .iter()
            .filter_map(|s| *s)
            .fold(f64::NEG_INFINITY, f64::max);
        let has_size_range = s_max > s_min && s_min.is_finite();
        // 半径区间：随绘图区尺寸缩放，并受 symbol_size_scale 整体调节
        const MIN_RADIUS: f64 = 4.0;
        const MAX_RADIUS: f64 = 18.0;
        let r_min = MIN_RADIUS * cfg.symbol_size_scale;
        let r_max = MAX_RADIUS * cfg.symbol_size_scale;

        for (i, sqrt_size) in sqrt_sizes.iter().enumerate() {
            let x = x_vals.as_f64(i);
            let y = y_vals.as_f64(i);

            if let (Some(x), Some(y)) = (x, y) {
                let px = map_x_to_pixel(x, x_range, bounds);
                let py = map_y_to_pixel(y, y_range, bounds);

                // 计算气泡大小：sqrt(size) 线性归一化到 [r_min, r_max]，
                // 面积随数值单调变化且小气泡仍可见
                let radius = match *sqrt_size {
                    Some(v) if has_size_range => {
                        r_min + (v - s_min) / (s_max - s_min) * (r_max - r_min)
                    }
                    Some(_) => (r_min + r_max) / 2.0,
                    None => 10.0 * cfg.symbol_size_scale,
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
