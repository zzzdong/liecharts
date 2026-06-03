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

        if grids_with_axis.is_empty() {
            return (0.0, 0.0);
        }

        let axis = &self.x_axes[axis_idx];
        let mut all_y: Vec<f64> = Vec::new();
        let mut all_x: Vec<f64> = Vec::new();

        for series in self.series {
            if !grids_with_axis.contains(&series.grid_index) {
                continue;
            }
            all_y.extend(series.y_values());
            all_x.extend(series.x_values());
        }

        let values = if matches!(axis.axis_type, AxisType::Value) && !all_x.is_empty() {
            all_x
        } else if matches!(axis.axis_type, AxisType::Value) {
            all_y
        } else {
            all_y
        };

        if values.is_empty() {
            (0.0, 100.0)
        } else {
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            (min, max)
        }
    }

    /// 收集 Y 轴关联的所有 series 数据值
    fn collect_y_axis_range(&self, axis_idx: usize, specs: &[SubplotSpec]) -> (f64, f64) {
        let mut all_values: Vec<f64> = Vec::new();

        for series in self.series {
            let spec = match specs.iter().find(|s| s.id == series.grid_index) {
                Some(s) => s,
                None => continue,
            };

            let effective_y_axis = spec
                .y_axis_indices
                .first()
                .copied()
                .unwrap_or(series.y_axis_index);

            if effective_y_axis != axis_idx {
                continue;
            }

            all_values.extend(series.y_values());
        }

        if all_values.is_empty() {
            (0.0, 100.0)
        } else {
            let min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            if (min - max).abs() < f64::EPSILON {
                (min - 10.0, max + 10.0)
            } else {
                (min, max)
            }
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
