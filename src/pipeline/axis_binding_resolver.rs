use crate::pipeline::types::{
    AxisSpec, AxisType, ResolvedAxisRange, ResolvedAxisRanges, SeriesSpec, SubplotSpec,
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
            let (data_min, data_max) = self.collect_x_axis_range(axis_idx, specs);
            
            // Category 轴：collect_x_axis_range 已经返回正确范围
            let (resolved_min, resolved_max) = if matches!(axis.axis_type, AxisType::Category) {
                (data_min, data_max)
            } else {
                eprintln!("DEBUG: calling compute_final_range with data_min={}, data_max={}", data_min, data_max);
                let result = self.compute_final_range(
                    axis.min,
                    axis.max,
                    data_min,
                    data_max,
                    axis.axis_type,
                    axis.categories.len(),
                    axis.boundary_gap,
                );
                eprintln!("DEBUG: compute_final_range result={:?}", result);
                result
            };

            ranges.push(ResolvedAxisRange {
                axis_index: axis_idx,
                position: axis.position,
                axis_type: axis.axis_type.clone(),
                min: resolved_min,
                max: resolved_max,
                is_user_defined: axis.min.is_some() || axis.max.is_some(),
                tick_count_hint: None,
            });
        }

        // 处理 y 轴
        for axis_idx in 0..self.y_axes.len() {
            let axis = &self.y_axes[axis_idx];
            let (data_min, data_max) = self.collect_y_axis_range(axis_idx, specs);
            let (resolved_min, resolved_max) = self.compute_final_range(
                axis.min,
                axis.max,
                data_min,
                data_max,
                axis.axis_type,
                axis.categories.len(),
                axis.boundary_gap,
            );

            ranges.push(ResolvedAxisRange {
                axis_index: axis_idx,
                position: axis.position,
                axis_type: axis.axis_type.clone(),
                min: resolved_min,
                max: resolved_max,
                is_user_defined: axis.min.is_some() || axis.max.is_some(),
                tick_count_hint: None,
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

        eprintln!("DEBUG collect_x_axis_range: axis_idx={}, grids_with_axis={:?}, specs={:?}", axis_idx, grids_with_axis, specs);

        if grids_with_axis.is_empty() {
            return (0.0, 0.0);
        }

        let axis = &self.x_axes[axis_idx];

        // Category 轴：根据数据点数量计算范围
        if matches!(axis.axis_type, AxisType::Category) {
            let mut max_count = 0;
            for series in self.series {
                if !grids_with_axis.contains(&series.grid_index) {
                    continue;
                }
                let count = series.data.row_count();
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
                return (0.0, (max_count - 1).max(0) as f64);
            }
        }

        let mut all_y: Vec<f64> = Vec::new();
        let mut all_x: Vec<f64> = Vec::new();
        let mut bound_series: Vec<&SeriesSpec> = Vec::new();

        for series in self.series {
            eprintln!("DEBUG: series.grid_index={}, grids_with_axis={:?}", series.grid_index, grids_with_axis);
            if !grids_with_axis.contains(&series.grid_index) {
                eprintln!("DEBUG: series skipped");
                continue;
            }
            all_y.extend(series.y_values());
            all_x.extend(series.x_values());
            eprintln!("DEBUG: series added, stack={:?}", series.stack);
            bound_series.push(series);
        }

        // 对于数值轴，如果 x_values 为空（字符串列），则使用 y_values
        let values = if matches!(axis.axis_type, AxisType::Value) && !all_x.is_empty() {
            all_x
        } else if matches!(axis.axis_type, AxisType::Value) {
            all_y
        } else {
            all_y
        };

        if values.is_empty() {
            return (0.0, 100.0);
        }

        let data_min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let mut data_max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // 检查是否有堆叠（用于横向柱状图，其值在 X 轴上）
        eprintln!("DEBUG: axis_type={:?}, bound_series.len={}", axis.axis_type, bound_series.len());
        if matches!(axis.axis_type, AxisType::Value) {
            let has_stacked_bars = bound_series.iter().any(|s| s.stack.is_some());
            eprintln!("DEBUG: has_stacked_bars={}", has_stacked_bars);
            if has_stacked_bars {
                use std::collections::HashMap;
                let mut stack_groups: HashMap<Option<String>, Vec<&SeriesSpec>> = HashMap::new();
                for s in &bound_series {
                    stack_groups.entry(s.stack.clone()).or_default().push(s);
                }

                for (_, group) in &stack_groups {
                    if group.len() <= 1 {
                        continue;
                    }

                    let max_rows = group.iter().map(|s| s.data.row_count()).max().unwrap_or(0);
                    if max_rows == 0 {
                        continue;
                    }

                    // 对于数值 X 轴上的堆叠（横向柱状图），使用 y_col 作为值列
                    for row in 0..max_rows {
                        let mut row_total = 0.0;
                        for s in group {
                            if let Some(col) = s.data.get_column(s.config.y_col_name()) {
                                if let Some(v) = col.as_f64(row) {
                                    row_total += v;
                                }
                            }
                        }
                        eprintln!("DEBUG: row={}, row_total={}, data_max={}", row, row_total, data_max);
                        if row_total > data_max {
                            data_max = row_total;
                        }
                    }
                }
            }
        }

        (data_min, data_max)
    }

    /// 收集 Y 轴关联的所有 series 数据值
    ///
    /// 对于堆叠柱状图，还会计算每个 stack 组内各行的总值，
    /// 确保轴范围足以容纳堆叠后的总高度。
    fn collect_y_axis_range(&self, axis_idx: usize, specs: &[SubplotSpec]) -> (f64, f64) {
        let mut all_values: Vec<f64> = Vec::new();

        // 收集绑定到该轴的所有 series
        let mut bound_series: Vec<&SeriesSpec> = Vec::new();

        for series in self.series {
            // 检查该 series 是否属于当前处理的 subplot 之一
            let _spec = match specs.iter().find(|s| s.id == series.grid_index) {
                Some(s) => s,
                None => continue,
            };

            // 直接使用 series 的 y_axis_index 来判断绑定关系
            if series.y_axis_index != axis_idx {
                continue;
            }

            all_values.extend(series.y_values());
            bound_series.push(series);
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
            for (_, group) in &stack_groups {
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
                        if let Some(col) = s.data.get_column(y_col) {
                            if let Some(v) = col.as_f64(row) {
                                row_total += v;
                            }
                        }
                    }
                    if row_total > data_max {
                        data_max = row_total;
                    }
                }
            }
        }

        if (data_min - data_max).abs() < f64::EPSILON {
            (data_min - 10.0, data_max + 10.0)
        } else {
            (data_min, data_max)
        }
    }

    fn compute_final_range(
        &self,
        user_min: Option<f64>,
        user_max: Option<f64>,
        data_min: f64,
        data_max: f64,
        axis_type: AxisType,
        category_count: usize,
        boundary_gap: bool,
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

        // Value 轴：结合用户指定 + 数据范围
        let range = data_max - data_min;

        let should_include_zero = if data_min >= 0.0 {
            data_min < range * 0.2
        } else if data_max <= 0.0 {
            data_max.abs() < range * 0.2
        } else {
            true
        };

        let default_min = if range > 0.0 {
            if should_include_zero && data_min >= 0.0 {
                0.0
            } else {
                data_min - range * 0.05
            }
        } else {
            data_min - 1.0
        };

        let default_max = if range > 0.0 {
            if should_include_zero && data_max <= 0.0 {
                0.0
            } else {
                data_max + range * 0.05
            }
        } else {
            data_max + 1.0
        };

        let min = user_min.unwrap_or(default_min);
        let max = user_max.unwrap_or(default_max);

        (min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::types::AxisPosition;

    fn make_axis_spec(axis_type: AxisType, position: AxisPosition, grid_index: usize) -> AxisSpec {
        AxisSpec {
            axis_type,
            position,
            grid_index,
            min: None,
            max: None,
            name: None,
            categories: vec![],
            boundary_gap: true,
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
}
