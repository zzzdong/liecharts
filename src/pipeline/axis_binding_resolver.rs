use crate::pipeline::types::{
    AxisSpec, AxisType, ChartType, ResolvedAxisRange, ResolvedAxisRanges, SeriesSpec, SubplotSpec,
};

/// 轴绑定解析器
///
/// 职责：
/// - 解析每个轴关联的数据范围
/// - 结合用户指定的 min/max，计算出最终轴范围
pub struct AxisBindingResolver<'a> {
    x_axes: &'a [AxisSpec],
    y_axes: &'a [AxisSpec],
    series: &'a [SeriesSpec],
}

impl<'a> AxisBindingResolver<'a> {
    pub fn new(x_axes: &'a [AxisSpec], y_axes: &'a [AxisSpec], series: &'a [SeriesSpec]) -> Self {
        Self {
            x_axes,
            y_axes,
            series,
        }
    }

    /// 解析所有 subplot 的轴绑定关系，输出协调后的轴范围
    pub fn resolve(&self, specs: &[SubplotSpec]) -> ResolvedAxisRanges {
        let mut ranges = Vec::new();

        // 处理 x 轴
        for axis_idx in 0..self.x_axes.len() {
            let axis = &self.x_axes[axis_idx];

            // 判断该轴关联的 subplot 中是否有非 Candlestick 的系列
            // 对 value 轴强制包含 0，确保 nice_ticks 不会延伸到数据范围之外
            let force_include_zero = specs.iter().any(|s| {
                s.x_axis_indices.contains(&axis_idx)
                    && s.series_indices.iter().any(|&si| {
                        self.series
                            .get(si)
                            .is_some_and(|ser| !matches!(ser.chart_type(), ChartType::Candlestick))
                    })
            });

            let (data_min, data_max) = self.collect_x_axis_range(axis_idx, specs);

            // Category 轴：collect_x_axis_range 已经返回正确范围
            let (resolved_min, resolved_max) = if matches!(axis.axis_type, AxisType::Category) {
                (data_min, data_max)
            } else {
                self.compute_final_range(
                    axis.min,
                    axis.max,
                    data_min,
                    data_max,
                    axis.axis_type,
                    axis.categories.len(),
                    axis.boundary_gap,
                    force_include_zero,
                )
            };

            ranges.push(ResolvedAxisRange {
                axis_index: axis_idx,
                position: axis.position,
                axis_type: axis.axis_type,
                min: resolved_min,
                max: resolved_max,
                is_user_defined: axis.min.is_some() || axis.max.is_some(),
                tick_count_hint: None,
                categories: axis.categories.clone(),
            });
        }

        // 处理 y 轴
        for axis_idx in 0..self.y_axes.len() {
            let axis = &self.y_axes[axis_idx];
            let (data_min, data_max) = self.collect_y_axis_range(axis_idx, specs);

            // 判断该轴关联的 subplot 中是否有非 Candlestick 的系列
            let force_include_zero = specs.iter().any(|s| {
                s.y_axis_indices.contains(&axis_idx)
                    && s.series_indices.iter().any(|&si| {
                        self.series
                            .get(si)
                            .is_some_and(|ser| !matches!(ser.chart_type(), ChartType::Candlestick))
                    })
            });

            // 热力图的 category Y 轴：范围已由 collect_y_axis_range 按 distinct 坐标数算好，
            // 跳过 compute_final_range，避免在未声明 categories 时退化为 (0, 1)。
            let is_heatmap_category = matches!(axis.axis_type, AxisType::Category)
                && specs.iter().any(|s| {
                    s.y_axis_indices.contains(&axis_idx)
                        && s.series_indices.iter().any(|&si| {
                            self.series
                                .get(si)
                                .is_some_and(|ser| matches!(ser.chart_type(), ChartType::Heatmap))
                        })
                });

            let (resolved_min, resolved_max) = if is_heatmap_category {
                (data_min, data_max)
            } else {
                self.compute_final_range(
                    axis.min,
                    axis.max,
                    data_min,
                    data_max,
                    axis.axis_type,
                    axis.categories.len(),
                    axis.boundary_gap,
                    force_include_zero,
                )
            };

            ranges.push(ResolvedAxisRange {
                axis_index: axis_idx,
                position: axis.position,
                axis_type: axis.axis_type,
                min: resolved_min,
                max: resolved_max,
                is_user_defined: axis.min.is_some() || axis.max.is_some(),
                tick_count_hint: None,
                categories: axis.categories.clone(),
            });
        }

        ResolvedAxisRanges { ranges }
    }

    /// 收集 X 轴关联的所有 series 数据值
    fn collect_x_axis_range(&self, axis_idx: usize, specs: &[SubplotSpec]) -> (f64, f64) {
        let grids_with_axis: Vec<usize> = specs
            .iter()
            .filter(|s| s.x_axis_indices.contains(&axis_idx))
            .map(|s| s.id)
            .collect();

        if grids_with_axis.is_empty() {
            return (0.0, 0.0);
        }

        let axis = &self.x_axes[axis_idx];

        // Category 轴：根据数据点数量计算范围
        if matches!(axis.axis_type, AxisType::Category) {
            let mut max_count = 0;
            for series in self.series {
                if !self.x_series_bound_to_axis(axis_idx, series, specs) {
                    continue;
                }
                let count = if matches!(series.chart_type(), ChartType::Heatmap) {
                    // 热力图数据是一行一个格子（x*y 行），不能按行数算轴范围；
                    // 优先使用轴声明的 categories，否则统计 distinct 坐标数。
                    if !axis.categories.is_empty() {
                        axis.categories.len()
                    } else {
                        series
                            .data
                            .get_column(series.config.x_col_name())
                            .map(count_distinct_values)
                            .unwrap_or(0)
                    }
                } else {
                    series.data.row_count()
                };
                if count > max_count {
                    max_count = count;
                }
            }
            if max_count == 0 {
                return (0.0, 1.0);
            }
            // 返回索引范围：0 到 n-1（或 n，取决于 boundary_gap）
            if axis.boundary_gap {
                return (0.0, max_count as f64);
            } else {
                return (0.0, (max_count - 1) as f64);
            }
        }

        let mut all_y: Vec<f64> = Vec::new();
        let mut all_x: Vec<f64> = Vec::new();
        let mut bound_series: Vec<&SeriesSpec> = Vec::new();

        for series in self.series {
            if !self.x_series_bound_to_axis(axis_idx, series, specs) {
                continue;
            }
            all_y.extend(series.y_values());
            all_x.extend(series.x_values());
            bound_series.push(series);
        }

        // 对数值 X 轴使用 x_values 确定范围
        // 水平柱状图通过 API 层的列交换已确保 x_col 是数值列
        let values = all_x;

        if values.is_empty() {
            return (0.0, 100.0);
        }

        let data_min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let mut data_max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // 检查是否有堆叠（用于横向柱状图，其值在 X 轴上）
        if matches!(axis.axis_type, AxisType::Value) {
            let has_stacked_bars = bound_series.iter().any(|s| s.stack.is_some());
            if has_stacked_bars {
                use std::collections::HashMap;
                let mut stack_groups: HashMap<Option<String>, Vec<&SeriesSpec>> = HashMap::new();
                for s in &bound_series {
                    stack_groups.entry(s.stack.clone()).or_default().push(s);
                }

                for group in stack_groups.values() {
                    if group.len() <= 1 {
                        continue;
                    }

                    let max_rows = group.iter().map(|s| s.data.row_count()).max().unwrap_or(0);
                    if max_rows == 0 {
                        continue;
                    }

                    // 对于数值 X 轴上的堆叠（横向柱状图），使用 x_col 作为值列
                    for row in 0..max_rows {
                        let mut row_total = 0.0;
                        for s in group {
                            if let Some(col) = s.data.get_column(s.config.x_col_name())
                                && let Some(v) = col.as_f64(row)
                            {
                                row_total += v;
                            }
                        }
                        if row_total > data_max {
                            data_max = row_total;
                        }
                    }
                }
            }
        }

        (data_min, data_max)
    }

    /// X 轴系列绑定判断：
    ///
    /// 1. 系列声明的 `x_axis_index` 与轴一致 → 绑定
    /// 2. 否则仅当系列所在 subplot 包含该轴、且系列声明的 x 轴不在该 subplot 的
    ///    轴列表中（如只声明了一个轴、系列未显式指定）时，回退绑定到第一个轴。
    ///
    /// 避免多轴场景下（如混合图的左右轴）所有系列都同时计入每个轴的范围。
    fn x_series_bound_to_axis(
        &self,
        axis_idx: usize,
        series: &SeriesSpec,
        specs: &[SubplotSpec],
    ) -> bool {
        let Some(subplot) = specs.iter().find(|s| s.id == series.grid_index) else {
            return false;
        };
        if !subplot.x_axis_indices.contains(&axis_idx) {
            return false;
        }
        if series.x_axis_index == axis_idx {
            return true;
        }
        !subplot.x_axis_indices.contains(&series.x_axis_index)
            && subplot.x_axis_indices.first() == Some(&axis_idx)
    }

    /// Y 轴系列绑定判断，规则同 [`Self::x_series_bound_to_axis`]。
    fn y_series_bound_to_axis(
        &self,
        axis_idx: usize,
        series: &SeriesSpec,
        specs: &[SubplotSpec],
        axis_subplot_id: Option<usize>,
    ) -> bool {
        if series.y_axis_index == axis_idx {
            return true;
        }
        let Some(subplot_id) = axis_subplot_id else {
            return false;
        };
        let Some(subplot) = specs.iter().find(|s| s.id == series.grid_index) else {
            return false;
        };
        if subplot.id != subplot_id {
            return false;
        }
        !subplot.y_axis_indices.contains(&series.y_axis_index)
            && subplot.y_axis_indices.first() == Some(&axis_idx)
    }

    /// 收集 Y 轴关联的所有 series 数据值
    ///
    /// 对于堆叠柱状图，还会计算每个 stack 组内各行的总值，
    /// 确保轴范围足以容纳堆叠后的总高度。
    fn collect_y_axis_range(&self, axis_idx: usize, specs: &[SubplotSpec]) -> (f64, f64) {
        let mut all_values: Vec<f64> = Vec::new();

        // 收集绑定到该轴的所有 series
        let mut bound_series: Vec<&SeriesSpec> = Vec::new();

        // 找到该轴所属的 subplot
        let axis_subplot_id = specs
            .iter()
            .find(|s| s.y_axis_indices.contains(&axis_idx))
            .map(|s| s.id);

        for series in self.series {
            // 检查该 series 是否属于当前处理的 subplot 之一
            let _spec = match specs.iter().find(|s| s.id == series.grid_index) {
                Some(s) => s,
                None => continue,
            };

            // 判断绑定关系：优先按 y_axis_index；未显式绑定到本 subplot 任一轴时才回退
            let is_bound = self.y_series_bound_to_axis(axis_idx, series, specs, axis_subplot_id);

            if !is_bound {
                continue;
            }

            all_values.extend(series.y_values());
            bound_series.push(series);
        }

        // 热力图：Y 轴范围由 distinct y 坐标数决定（而不是值列的范围）
        if bound_series
            .iter()
            .any(|s| matches!(s.chart_type(), super::ChartType::Heatmap))
        {
            let axis = &self.y_axes[axis_idx];
            let mut max_count = 0;
            for series in &bound_series {
                if !matches!(series.chart_type(), super::ChartType::Heatmap) {
                    continue;
                }
                let y_col = match &series.config {
                    super::SeriesConfig::Heatmap(c) => c.y_col.clone(),
                    _ => series.config.y_col_name().to_string(),
                };
                let count = if !axis.categories.is_empty() {
                    axis.categories.len()
                } else {
                    series
                        .data
                        .get_column(&y_col)
                        .map(count_distinct_values)
                        .unwrap_or(0)
                };
                if count > max_count {
                    max_count = count;
                }
            }
            let (lo, hi) = if axis.boundary_gap {
                (0.0, max_count as f64)
            } else {
                (0.0, (max_count.saturating_sub(1)) as f64)
            };
            return (lo, hi);
        }

        if all_values.is_empty() {
            return (0.0, 100.0);
        }

        let data_min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let mut data_max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // 检查是否有堆叠柱状图
        let has_stacked_bars = bound_series.iter().any(|s| s.stack.is_some());
        if has_stacked_bars {
            // 按 stack 名称分组
            use std::collections::HashMap;
            let mut stack_groups: HashMap<Option<String>, Vec<&SeriesSpec>> = HashMap::new();
            for s in &bound_series {
                stack_groups.entry(s.stack.clone()).or_default().push(s);
            }

            // 对每个有多于一个系列的 stack 组，计算每行总值
            for group in stack_groups.values() {
                if group.len() <= 1 {
                    continue;
                }

                // 找到最大行数
                let max_rows = group.iter().map(|s| s.data.row_count()).max().unwrap_or(0);
                if max_rows == 0 {
                    continue;
                }

                // 计算每行总值
                for row in 0..max_rows {
                    let mut row_total = 0.0;
                    for s in group {
                        let y_col = s.config.y_col_name();
                        if let Some(col) = s.data.get_column(y_col)
                            && let Some(v) = col.as_f64(row)
                        {
                            row_total += v;
                        }
                    }
                    if row_total > data_max {
                        data_max = row_total;
                    }
                }
            }
        }

        // 检查是否有堆叠面积图（Line/Stack 系列）
        let has_stacked_areas = bound_series
            .iter()
            .any(|s| s.stack.is_some() && matches!(s.chart_type(), super::ChartType::Line));
        if has_stacked_areas {
            use std::collections::HashMap;
            let mut stack_groups: HashMap<Option<String>, Vec<&SeriesSpec>> = HashMap::new();
            for s in &bound_series {
                if s.stack.is_some() && matches!(s.chart_type(), super::ChartType::Line) {
                    stack_groups.entry(s.stack.clone()).or_default().push(s);
                }
            }

            for group in stack_groups.values() {
                if group.len() <= 1 {
                    continue;
                }

                let max_rows = group.iter().map(|s| s.data.row_count()).max().unwrap_or(0);
                if max_rows == 0 {
                    continue;
                }

                for row in 0..max_rows {
                    let mut row_total = 0.0;
                    for s in group {
                        let y_col = s.config.y_col_name();
                        if let Some(col) = s.data.get_column(y_col)
                            && let Some(v) = col.as_f64(row)
                        {
                            row_total += v;
                        }
                    }
                    if row_total > data_max {
                        data_max = row_total;
                    }
                }
            }
        }

        // 零跨度数据不在这里提前扩展，交给 compute_final_range 统一按比例留白，
        // 避免"单数据点"场景被 include-zero 逻辑压到绘图区边缘。
        (data_min, data_max)
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_final_range(
        &self,
        user_min: Option<f64>,
        user_max: Option<f64>,
        data_min: f64,
        data_max: f64,
        axis_type: AxisType,
        category_count: usize,
        boundary_gap: bool,
        force_include_zero: bool,
    ) -> (f64, f64) {
        // Category 轴：使用计数
        if matches!(axis_type, AxisType::Category) {
            return if category_count > 0 {
                if boundary_gap {
                    (0.0, category_count as f64)
                } else {
                    (0.0, (category_count - 1) as f64)
                }
            } else {
                (0.0, 1.0)
            };
        }

        // Log 轴：在 log10 空间计算范围，返回 (log_min, log_max)。
        // 数据必须为正；对非正数据回退到 (0, log10(data_max)+1)。
        if matches!(axis_type, AxisType::Log) {
            return self.compute_log_range(user_min, user_max, data_min, data_max);
        }

        // Value 轴：结合用户指定 + 数据范围
        let range = data_max - data_min;

        // 零跨度数据（如单个数据点）：围绕数据值对称留白，不强制包含 0，
        // 否则 0 起点会把唯一的点压到绘图区上/下边缘。
        if range <= f64::EPSILON {
            let pad = (data_min.abs() * 0.05).max(1.0);
            let min = user_min.unwrap_or(data_min - pad);
            let max = user_max.unwrap_or(data_max + pad);
            return (min, max);
        }

        // 包含 0 的判断：
        // - 数据全正：非 K 线图默认从 0 起；但若数据量级远大于跨度（如 70M 附近的
        //   微小波动），从 0 起会把数据压到绘图区顶部，改为按数据范围留白。
        // - 全负对称处理；跨越 0 的数据必须包含 0。
        let should_include_zero = if data_min >= 0.0 {
            if force_include_zero {
                data_min <= range * 8.0
            } else {
                data_min < range * 0.2
            }
        } else if data_max <= 0.0 {
            if force_include_zero {
                data_max.abs() <= range * 8.0
            } else {
                data_max.abs() < range * 0.2
            }
        } else {
            true
        };

        // 数据全正时按 0 起点，否则围绕数据范围留 5% 空白
        let default_min = if should_include_zero && data_min >= 0.0 {
            0.0
        } else {
            data_min - range * 0.05
        };

        let default_max = if should_include_zero && data_max <= 0.0 {
            0.0
        } else {
            data_max + range * 0.05
        };

        let min = user_min.unwrap_or(default_min);
        let max = user_max.unwrap_or(default_max);

        (min, max)
    }

    /// 计算 Log 轴范围，返回 `(log10_min, log10_max)`。
    ///
    /// - 数据必须为正数（>0）；log 轴不支持 0/负数。
    /// - 自动向下取整到 10 的整数幂，保证刻度整齐（1, 10, 100 ...）。
    /// - 支持用户通过 `min`/`max` 显式指定数据范围（仍按 log10 归一化）。
    fn compute_log_range(
        &self,
        user_min: Option<f64>,
        user_max: Option<f64>,
        data_min: f64,
        data_max: f64,
    ) -> (f64, f64) {
        // 用户显式范围优先（作为实际值，取 log10）
        let lo = user_min.map(|v| v.max(f64::MIN_POSITIVE));
        let hi = user_max.map(|v| v.max(f64::MIN_POSITIVE));

        // 数据全无效（<=0）：回退到 (1, 10)
        if data_max <= 0.0 && hi.is_none() {
            return (0.0, 1.0);
        }

        // 确定有效的最小/最大数据值
        let valid: Vec<f64> = [data_min, data_max]
            .into_iter()
            .filter(|v| *v > 0.0)
            .collect();
        let data_lo = valid.iter().cloned().fold(f64::INFINITY, f64::min);
        let data_hi = valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let min_val = lo.unwrap_or(data_lo).max(f64::MIN_POSITIVE);
        let max_val = hi.unwrap_or(data_hi).max(min_val);

        // 取整到 10 的整数幂
        let log_min = min_val.log10().floor();
        let log_max = max_val.log10().ceil();

        // 至少一个数量级跨度，避免零跨度
        if log_max - log_min < 1.0 {
            return (log_min, log_min + 1.0);
        }
        (log_min, log_max)
    }
}

/// 统计一列数据中 distinct 坐标的数量（浮点 + 字符串），用于热力图轴范围。
fn count_distinct_values(col: &crate::pipeline::dataframe::Series) -> usize {
    use std::collections::HashSet;

    use crate::pipeline::dataframe::DataValue;

    let mut nums = HashSet::new();
    let mut strs = HashSet::new();
    for v in &col.data {
        match v {
            DataValue::Float(f) => {
                nums.insert(f.to_bits());
            }
            DataValue::Integer(i) => {
                nums.insert((*i as f64).to_bits());
            }
            DataValue::String(s) => {
                strs.insert(s.clone());
            }
            _ => {}
        }
    }
    nums.len() + strs.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::types::{AxisPosition, ItemStyleSpec, LineConfig, SeriesConfig};

    fn make_axis_spec(axis_type: AxisType, position: AxisPosition, grid_index: usize) -> AxisSpec {
        AxisSpec {
            axis_type,
            position,
            grid_index,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories: vec![],
            boundary_gap: true,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate: None,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }
    }

    #[test]
    fn test_basic_resolution() {
        let x_axes = vec![make_axis_spec(AxisType::Value, AxisPosition::Bottom, 0)];
        let y_axes = vec![make_axis_spec(AxisType::Value, AxisPosition::Left, 0)];

        let specs = vec![SubplotSpec {
            id: 0,
            bounds: Default::default(),
            series_indices: vec![0],
            x_axis_indices: vec![0],
            y_axis_indices: vec![0],
        }];

        let resolver = AxisBindingResolver::new(&x_axes, &y_axes, &[]);
        let ranges = resolver.resolve(&specs);

        assert_eq!(ranges.ranges.len(), 2);
    }

    fn final_range(data_min: f64, data_max: f64, force_include_zero: bool) -> (f64, f64) {
        let resolver = AxisBindingResolver::new(&[], &[], &[]);
        resolver.compute_final_range(
            None,
            None,
            data_min,
            data_max,
            AxisType::Value,
            0,
            true,
            force_include_zero,
        )
    }

    #[test]
    fn test_single_point_axis_range_centers_data() {
        // 单个数据点：围绕数值对称留白，而不是从 0 起把点压到顶部
        let (min, max) = final_range(70840845.0, 70840845.0, true);
        assert!(min > 0.0, "单点不应强制从 0 起，实际 min={}", min);
        assert!(max > 70840845.0);
        let pad = 70840845.0 * 0.05;
        assert!((min - (70840845.0 - pad)).abs() < 1e-6);
        assert!((max - (70840845.0 + pad)).abs() < 1e-6);
    }

    #[test]
    fn test_positive_data_still_includes_zero() {
        // 常规正数数据（量级与跨度同阶）：保持从 0 起
        let (min, max) = final_range(70.0, 200.0, true);
        assert_eq!(min, 0.0);
        assert!(max > 200.0);
    }

    #[test]
    fn test_high_magnitude_small_range_scales_to_data() {
        // 数据远离 0 且跨度很小：包含 0 会把数据压到顶部，应围绕数据范围缩放
        let (min, max) = final_range(1_000_000.0, 1_000_002.0, true);
        assert!(min > 0.0, "远离 0 的数据不应强制包含 0，实际 min={}", min);
        assert!(min < 1_000_000.0);
        assert!(max > 1_000_002.0);
    }

    #[test]
    fn test_user_range_overrides_auto() {
        let resolver = AxisBindingResolver::new(&[], &[], &[]);
        let (min, max) = resolver.compute_final_range(
            Some(0.0),
            Some(100.0),
            40.0,
            60.0,
            AxisType::Value,
            0,
            true,
            true,
        );
        assert_eq!((min, max), (0.0, 100.0));
    }

    fn make_value_series(
        name: &str,
        grid_index: usize,
        y_axis_index: usize,
        values: Vec<f64>,
    ) -> SeriesSpec {
        use crate::pipeline::dataframe::{DataFrame, DataValue, Series};
        let mut df = DataFrame::new();
        df.add_column(Series::new(
            "x",
            (0..values.len())
                .map(|i| DataValue::Float(i as f64))
                .collect(),
        ));
        df.add_column(Series::new(
            "y",
            values.into_iter().map(DataValue::Float).collect(),
        ));
        SeriesSpec {
            name: name.into(),
            data: df,
            grid_index,
            x_axis_index: 0,
            y_axis_index,
            stack: None,
            group_index: 0,
            sampling: None,
            item_style: ItemStyleSpec::default(),
            config: SeriesConfig::Line(LineConfig::default()),
        }
    }

    #[test]
    fn test_multi_axis_binding_respects_y_axis_index() {
        // 混合图：左右两个 y 轴，销量绑定左轴（0），增长率绑定右轴（1）
        let x_axes = vec![make_axis_spec(AxisType::Category, AxisPosition::Bottom, 0)];
        let y_axes = vec![
            make_axis_spec(AxisType::Value, AxisPosition::Left, 0),
            make_axis_spec(AxisType::Value, AxisPosition::Right, 0),
        ];
        let series = vec![
            make_value_series("销量", 0, 0, vec![120.0, 200.0, 150.0, 80.0, 70.0]),
            make_value_series("增长率", 0, 1, vec![10.0, 20.0, 15.0, 8.0, 7.0]),
        ];
        let specs = vec![SubplotSpec {
            id: 0,
            bounds: Default::default(),
            series_indices: vec![0, 1],
            x_axis_indices: vec![0],
            y_axis_indices: vec![0, 1],
        }];

        let resolver = AxisBindingResolver::new(&x_axes, &y_axes, &series);
        let ranges = resolver.resolve(&specs);

        let left = ranges.get_y_range(0).unwrap();
        let right = ranges.get_y_range(1).unwrap();
        // 左轴 max ≈ 200 + 130*5% = 206.5
        assert!((left.max - 206.5).abs() < 1.0, "左轴 max={}", left.max);
        // 右轴只应包含增长率：max ≈ 20 + 13*5% = 20.65（曾错误混入左轴数据）
        assert!(
            (right.max - 20.65).abs() < 1.0,
            "右轴 max={}，不应包含左轴系列",
            right.max
        );
    }
}
