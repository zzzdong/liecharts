//! Heatmap Materializer: 将 Heatmap SeriesSpec 转换为 HeatmapSeries

use vello_cpu::kurbo::Rect;

use crate::{
    error::{ChartError, Result},
    pipeline::{
        dataframe::DataValue,
        materializer::SeriesMaterializer,
        typed_series::{HeatmapCell, HeatmapSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRange, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
    visual::Color,
};

pub struct HeatmapMaterializer;

impl SeriesMaterializer for HeatmapMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,
        axis_ranges: &ResolvedAxisRanges,
        _color: Color,
        _colors: &ColorContext,
    ) -> Result<TypedSeries> {
        let cfg = match &spec.config {
            SeriesConfig::Heatmap(c) => c,
            _ => {
                return Err(ChartError::InvalidConfig("Expected HeatmapConfig".into()));
            }
        };

        let x_range = axis_ranges
            .get_x_range(spec.x_axis_index)
            .ok_or_else(|| ChartError::InvalidAxisBinding("X axis not found".into()))?;
        let y_range = axis_ranges
            .get_y_range(spec.y_axis_index)
            .ok_or_else(|| ChartError::InvalidAxisBinding("Y axis not found".into()))?;

        let x_vals = spec
            .data
            .get_column(&cfg.x_col)
            .ok_or_else(|| ChartError::MissingColumn(cfg.x_col.clone()))?;
        let y_vals = spec
            .data
            .get_column(&cfg.y_col)
            .ok_or_else(|| ChartError::MissingColumn(cfg.y_col.clone()))?;
        let value_vals = spec
            .data
            .get_column(&cfg.value_col)
            .ok_or_else(|| ChartError::MissingColumn(cfg.value_col.clone()))?;

        let n_x = axis_slot_count(x_range, &x_vals.data);
        let n_y = axis_slot_count(y_range, &y_vals.data);

        if n_x == 0 || n_y == 0 {
            return Ok(TypedSeries::Heatmap(HeatmapSeries {
                name: spec.name.clone(),
                cells: Vec::new(),
            }));
        }

        // 颜色映射：visualMap min/max，缺失时用数据范围
        let (vm_min, vm_max) = match (cfg.min, cfg.max) {
            (Some(min), Some(max)) => (min, max),
            _ => {
                let mut min = f64::INFINITY;
                let mut max = f64::NEG_INFINITY;
                for i in 0..value_vals.len() {
                    if let Some(v) = value_vals.as_f64(i) {
                        min = min.min(v);
                        max = max.max(v);
                    }
                }
                (
                    cfg.min.unwrap_or(if min.is_finite() { min } else { 0.0 }),
                    cfg.max.unwrap_or(if max.is_finite() { max } else { 1.0 }),
                )
            }
        };
        let (vm_min, vm_max) = if vm_max > vm_min {
            (vm_min, vm_max)
        } else {
            (vm_min, vm_min + 1.0)
        };

        let cell_w = bounds.width() / n_x as f64;
        let cell_h = bounds.height() / n_y as f64;

        let mut cells = Vec::with_capacity(spec.data.row_count());
        for i in 0..spec.data.row_count() {
            let (Some(x_idx), Some(y_idx), Some(value)) = (
                coord_to_slot(&x_vals.data, i, x_range, n_x),
                coord_to_slot(&y_vals.data, i, y_range, n_y),
                value_vals.as_f64(i),
            ) else {
                continue;
            };

            let rect = Rect::new(
                bounds.x0 + x_idx as f64 * cell_w,
                bounds.y1 - (y_idx + 1) as f64 * cell_h,
                bounds.x0 + (x_idx + 1) as f64 * cell_w,
                bounds.y1 - y_idx as f64 * cell_h,
            );

            cells.push(HeatmapCell {
                rect,
                value,
                color: value_to_color(value, vm_min, vm_max, &cfg.colors),
                border_color: cfg.border_color,
                border_width: cfg.border_width,
            });
        }

        Ok(TypedSeries::Heatmap(HeatmapSeries {
            name: spec.name.clone(),
            cells,
        }))
    }
}

/// 计算热力图在单个轴上的格子数。
///
/// 优先使用轴声明的 categories；否则统计 distinct 坐标数。
fn axis_slot_count(range: &ResolvedAxisRange, data: &[DataValue]) -> usize {
    if !range.categories.is_empty() {
        return range.categories.len();
    }
    let mut nums: Vec<u64> = Vec::new();
    let mut strs: Vec<&str> = Vec::new();
    for v in data {
        match v {
            DataValue::Float(f) => {
                if !nums.contains(&f.to_bits()) {
                    nums.push(f.to_bits());
                }
            }
            DataValue::Integer(i) => {
                let bits = (*i as f64).to_bits();
                if !nums.contains(&bits) {
                    nums.push(bits);
                }
            }
            DataValue::String(s) if !strs.contains(&s.as_str()) => {
                strs.push(s.as_str());
            }
            _ => {}
        }
    }
    nums.len() + strs.len()
}

/// 将一行数据的坐标值映射为格子索引。
///
/// 数值坐标：直接取整（要求与轴范围对齐，0 起）。
/// 字符串坐标：在轴 categories 中查找；找不到时尝试按 distinct 顺序推断。
fn coord_to_slot(
    data: &[DataValue],
    row: usize,
    range: &ResolvedAxisRange,
    n_slots: usize,
) -> Option<usize> {
    match data.get(row)? {
        DataValue::Float(f) => {
            if *f >= 0.0 && (*f as usize) < n_slots {
                Some(*f as usize)
            } else {
                None
            }
        }
        DataValue::Integer(i) => {
            if *i >= 0 && (*i as usize) < n_slots {
                Some(*i as usize)
            } else {
                None
            }
        }
        DataValue::String(s) => {
            if !range.categories.is_empty() {
                range.categories.iter().position(|c| c == s)
            } else {
                // 无 categories 时：按首次出现的顺序分配索引
                let mut seen: Vec<&str> = Vec::new();
                for v in data {
                    if let DataValue::String(c) = v
                        && !seen.contains(&c.as_str())
                    {
                        seen.push(c.as_str());
                    }
                }
                seen.iter().position(|c| *c == s)
            }
        }
        _ => None,
    }
}

/// 在渐变颜色表中线性插值出 value 对应的颜色。
fn value_to_color(value: f64, min: f64, max: f64, colors: &[Color]) -> Color {
    if colors.is_empty() {
        return Color::new(128, 128, 128);
    }
    if colors.len() == 1 {
        return colors[0];
    }
    let t = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let scaled = t * (colors.len() - 1) as f64;
    let idx = (scaled.floor() as usize).min(colors.len() - 2);
    let frac = scaled - idx as f64;
    lerp_color(colors[idx], colors[idx + 1], frac)
}

fn lerp_color(a: Color, b: Color, t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: (a.r as f64 + (b.r as f64 - a.r as f64) * t).round() as u8,
        g: (a.g as f64 + (b.g as f64 - a.g as f64) * t).round() as u8,
        b: (a.b as f64 + (b.b as f64 - a.b as f64) * t).round() as u8,
        a: (a.a as f64 + (b.a as f64 - a.a as f64) * t).round() as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_color_endpoints_and_midpoint() {
        let colors = vec![Color::new(0, 0, 0), Color::new(255, 255, 255)];
        assert_eq!(value_to_color(0.0, 0.0, 10.0, &colors), colors[0]);
        assert_eq!(value_to_color(10.0, 0.0, 10.0, &colors), colors[1]);
        let mid = value_to_color(5.0, 0.0, 10.0, &colors);
        assert_eq!((mid.r, mid.g, mid.b), (128, 128, 128));
    }

    #[test]
    fn test_value_to_color_out_of_range_clamps() {
        let colors = vec![Color::new(10, 20, 30), Color::new(200, 210, 220)];
        assert_eq!(value_to_color(-100.0, 0.0, 10.0, &colors), colors[0]);
        assert_eq!(value_to_color(100.0, 0.0, 10.0, &colors), colors[1]);
    }

    #[test]
    fn test_value_to_color_single_stop() {
        let colors = vec![Color::new(1, 2, 3)];
        assert_eq!(value_to_color(7.0, 0.0, 10.0, &colors), colors[0]);
    }
}
