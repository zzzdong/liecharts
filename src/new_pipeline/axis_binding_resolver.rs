use crate::new_pipeline::types::{ResolvedAxisRange, ResolvedAxisRanges, SubplotSpec};
use crate::option::{AxisType, ChartOption};

/// 轴绑定解析器
///
/// 职责：
/// - 解析 xAxisIndex / yAxisIndex 为具体的轴配置
/// - 识别哪些 subplot 共用同一个轴实例
/// - 对每个轴实例，收集所有关联 subplot 的数据范围
/// - 结合用户指定的 min/max，计算出最终轴范围
pub struct AxisBindingResolver<'a> {
    option: &'a ChartOption,
}

impl<'a> AxisBindingResolver<'a> {
    pub fn new(option: &'a ChartOption) -> Self {
        Self { option }
    }

    /// 解析所有 subplot 的轴绑定关系，输出协调后的轴范围
    pub fn resolve(&self, specs: &[SubplotSpec]) -> ResolvedAxisRanges {
        let mut ranges = Vec::new();

        // 处理 x 轴
        for axis_idx in 0..self.option.x_axis.len() {
            let axis = &self.option.x_axis[axis_idx];

            // 收集该轴在各 subplot 下关联的 series 数据
            let (data_min, data_max) = self.collect_x_axis_range(axis_idx, specs, axis);

            let (resolved_min, resolved_max) = self.compute_final_range(
                axis.min,
                axis.max,
                data_min,
                data_max,
                axis.axis_type,
                axis.data.as_ref().map(|d| d.len()).unwrap_or(0),
                axis.boundary_gap.unwrap_or(true),
            );

            ranges.push(ResolvedAxisRange {
                axis_index: axis_idx,
                min: resolved_min,
                max: resolved_max,
                is_user_defined: axis.min.is_some() || axis.max.is_some(),
                tick_count_hint: None,
            });
        }

        // 处理 y 轴
        for axis_idx in 0..self.option.y_axis.len() {
            let axis = &self.option.y_axis[axis_idx];

            let (data_min, data_max) = self.collect_y_axis_range(axis_idx, specs, axis);

            let (resolved_min, resolved_max) = self.compute_final_range(
                axis.min,
                axis.max,
                data_min,
                data_max,
                axis.axis_type,
                axis.data.as_ref().map(|d| d.len()).unwrap_or(0),
                axis.boundary_gap.unwrap_or(true),
            );

            ranges.push(ResolvedAxisRange {
                axis_index: axis_idx,
                min: resolved_min,
                max: resolved_max,
                is_user_defined: axis.min.is_some() || axis.max.is_some(),
                tick_count_hint: None,
            });
        }

        ResolvedAxisRanges { ranges }
    }

    /// 收集 X 轴关联的所有 series 数据值
    fn collect_x_axis_range(
        &self,
        axis_idx: usize,
        specs: &[SubplotSpec],
        axis: &crate::option::AxisOption,
    ) -> (f64, f64) {
        let grids_with_axis: Vec<usize> = specs
            .iter()
            .filter(|s| s.x_axis_indices.contains(&axis_idx))
            .map(|s| s.id)
            .collect();

        if grids_with_axis.is_empty() {
            return (0.0, 0.0);
        }

        let mut all_values: Vec<f64> = Vec::new();
        let mut all_x_values: Vec<f64> = Vec::new();

        for series in &self.option.series {
            let series_grid_idx = series_grid_index(series).unwrap_or(0);
            if !grids_with_axis.contains(&series_grid_idx) {
                continue;
            }
            let (vals, x_vals) = extract_series_values(series);
            all_values.extend(vals);
            all_x_values.extend(x_vals);
        }

        let values = if matches!(axis.axis_type, Some(AxisType::Value)) && !all_x_values.is_empty()
        {
            all_x_values
        } else if matches!(axis.axis_type, Some(AxisType::Value)) {
            all_values
        } else {
            all_values
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
    fn collect_y_axis_range(
        &self,
        axis_idx: usize,
        specs: &[SubplotSpec],
        _axis: &crate::option::AxisOption,
    ) -> (f64, f64) {
        let grids_with_axis: Vec<usize> = specs
            .iter()
            .filter(|s| s.y_axis_indices.contains(&axis_idx))
            .map(|s| s.id)
            .collect();

        if grids_with_axis.is_empty() {
            return (0.0, 0.0);
        }

        let mut all_values: Vec<f64> = Vec::new();

        for series in &self.option.series {
            let series_grid_idx = series_grid_index(series).unwrap_or(0);
            if !grids_with_axis.contains(&series_grid_idx) {
                continue;
            }
            let series_y_axis_idx = series_y_axis_index(series).unwrap_or(0);
            if series_y_axis_idx != axis_idx {
                continue;
            }
            let (vals, _) = extract_series_values(series);
            all_values.extend(vals);
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
        axis_type: Option<AxisType>,
        category_count: usize,
        boundary_gap: bool,
    ) -> (f64, f64) {
        // Category 轴：使用计数
        if matches!(axis_type, Some(AxisType::Category)) {
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

        // Value 轴：结合用户指定 + 数据范围，加 5% 余量
        let range = data_max - data_min;
        let default_min = if range > 0.0 {
            data_min - range * 0.05
        } else {
            data_min - 1.0
        };
        let default_max = if range > 0.0 {
            data_max + range * 0.05
        } else {
            data_max + 1.0
        };

        let min = user_min.unwrap_or(if data_min >= 0.0 { 0.0_f64.max(default_min) } else { default_min });
        let max = user_max.unwrap_or(default_max);

        (min, max)
    }
}

fn series_grid_index(series: &crate::option::SeriesOption) -> Option<usize> {
    match series {
        crate::option::SeriesOption::Line(s) => s.grid_index,
        crate::option::SeriesOption::Bar(s) => s.grid_index,
        crate::option::SeriesOption::Candlestick(s) => s.grid_index,
        crate::option::SeriesOption::Pie(s) => s.grid_index,
        crate::option::SeriesOption::Scatter(s) => s.grid_index,
        crate::option::SeriesOption::Radar(_) => None,
        crate::option::SeriesOption::PolarBar(_) => None,
        crate::option::SeriesOption::PolarScatter(_) => None,
        crate::option::SeriesOption::Bubble(_) => None,
        crate::option::SeriesOption::Gauge(_) => None,
        crate::option::SeriesOption::Table(_) => None,
    }
}

fn series_y_axis_index(series: &crate::option::SeriesOption) -> Option<usize> {
    match series {
        crate::option::SeriesOption::Line(s) => s.y_axis_index,
        crate::option::SeriesOption::Bar(s) => s.y_axis_index,
        crate::option::SeriesOption::Candlestick(s) => s.y_axis_index,
        crate::option::SeriesOption::Scatter(s) => s.y_axis_index,
        _ => None,
    }
}

fn extract_series_values(series: &crate::option::SeriesOption) -> (Vec<f64>, Vec<f64>) {
    match series {
        crate::option::SeriesOption::Line(s) => {
            let vals: Vec<f64> = s.data.iter().map(extract_value).collect();
            let x_vals: Vec<f64> = s.data.iter().filter_map(extract_x_value).collect();
            (vals, x_vals)
        }
        crate::option::SeriesOption::Bar(s) => {
            let vals: Vec<f64> = s.data.iter().map(extract_value).collect();
            let x_vals: Vec<f64> = s.data.iter().filter_map(extract_x_value).collect();
            (vals, x_vals)
        }
        crate::option::SeriesOption::Scatter(s) => {
            let y_vals: Vec<f64> = s.data.iter().map(|d| match d {
                crate::option::DataPoint::XY(_, y) => *y,
                crate::option::DataPoint::Value(v) => *v,
                crate::option::DataPoint::Named(_, v) => *v,
            }).collect();
            let x_vals: Vec<f64> = s.data.iter().map(|d| match d {
                crate::option::DataPoint::XY(x, _) => *x,
                crate::option::DataPoint::Value(_) => 0.0,
                crate::option::DataPoint::Named(_, _) => 0.0,
            }).collect();
            (y_vals, x_vals)
        }
        crate::option::SeriesOption::Candlestick(s) => {
            let vals: Vec<f64> = s
                .data
                .iter()
                .flat_map(|d| vec![d.open, d.close, d.low, d.high])
                .collect();
            (vals, Vec::new())
        }
        crate::option::SeriesOption::Pie(s) => {
            let vals: Vec<f64> = s.data.iter().map(extract_value).collect();
            (vals, Vec::new())
        }
        _ => (Vec::new(), Vec::new()),
    }
}

fn extract_value(d: &crate::option::DataPoint) -> f64 {
    match d {
        crate::option::DataPoint::Value(v) => *v,
        crate::option::DataPoint::Named(_, v) => *v,
        crate::option::DataPoint::XY(_, y) => *y,
    }
}

fn extract_x_value(d: &crate::option::DataPoint) -> Option<f64> {
    match d {
        crate::option::DataPoint::XY(x, _) => Some(*x),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::option::*;
    use vello_cpu::kurbo::Rect;

    fn make_spec(grid_index: usize, x_axis_idx: usize, y_axis_idx: usize) -> SubplotSpec {
        SubplotSpec {
            id: grid_index,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            series_indices: vec![grid_index],
            x_axis_indices: vec![x_axis_idx],
            y_axis_indices: vec![y_axis_idx],
        }
    }

    #[test]
    fn test_value_axis_from_data() {
        let option = ChartOption {
            x_axis: vec![AxisOption::category()],
            y_axis: vec![AxisOption::value()],
            series: vec![
                SeriesOption::Line(LineSeriesOption {
                    name: Some("Line1".into()),
                    data: vec![
                        DataPoint::Named("A".into(), 10.0),
                        DataPoint::Named("B".into(), 50.0),
                        DataPoint::Named("C".into(), 30.0),
                    ],
                    ..Default::default()
                }),
            ],
            ..Default::default()
        };
        let resolver = AxisBindingResolver::new(&option);

        // 手动构造 spec——grid_index=0 绑定 series[0]
        let specs = vec![SubplotSpec {
            id: 0,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            series_indices: vec![0],
            x_axis_indices: vec![0],
            y_axis_indices: vec![0],
        }];
        let result = resolver.resolve(&specs);

        // xAxis[0]: category → (0, count)
        assert_eq!(result.ranges[0].axis_index, 0);
        // yAxis[0]: value → (10 - 5% range, 50 + 5% range)
        assert_eq!(result.ranges[1].axis_index, 0);
        assert!(result.ranges[1].min < 10.0);
        assert!(result.ranges[1].max > 50.0);
    }

    #[test]
    fn test_category_axis_range() {
        let option = ChartOption {
            x_axis: vec![AxisOption::category()
                .data(vec!["A", "B", "C", "D"])],
            y_axis: vec![AxisOption::value()],
            series: vec![
                SeriesOption::Bar(BarSeriesOption {
                    name: Some("Bar1".into()),
                    data: vec![
                        DataPoint::Named("A".into(), 10.0),
                        DataPoint::Named("B".into(), 20.0),
                        DataPoint::Named("C".into(), 30.0),
                        DataPoint::Named("D".into(), 40.0),
                    ],
                    ..Default::default()
                }),
            ],
            ..Default::default()
        };
        let resolver = AxisBindingResolver::new(&option);
        let specs = vec![make_spec(0, 0, 0)];
        let result = resolver.resolve(&specs);

        // xAxis: category, 4 items, boundary_gap=true → (0, 4)
        assert!((result.ranges[0].min - 0.0).abs() < 0.01);
        assert!((result.ranges[0].max - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_user_defined_axis_range() {
        let option = ChartOption {
            x_axis: vec![AxisOption::category()],
            y_axis: vec![AxisOption::value().min(0.0).max(200.0)],
            series: vec![
                SeriesOption::Scatter(ScatterSeriesOption {
                    name: Some("Sct1".into()),
                    data: vec![
                        DataPoint::XY(1.0, 10.0),
                        DataPoint::XY(2.0, 50.0),
                    ],
                    ..Default::default()
                }),
            ],
            ..Default::default()
        };
        let resolver = AxisBindingResolver::new(&option);
        let specs = vec![make_spec(0, 0, 0)];
        let result = resolver.resolve(&specs);

        // yAxis[0]: user-defined min=0, max=200
        assert!((result.ranges[1].min - 0.0).abs() < 0.01);
        assert!((result.ranges[1].max - 200.0).abs() < 0.01);
        assert!(result.ranges[1].is_user_defined);
    }

    #[test]
    fn test_shared_axis_across_grids() {
        // grid[0] 和 grid[1] 共用 yAxis[0]
        let option = ChartOption {
            grid: vec![GridOption::default(), GridOption::default()],
            x_axis: vec![AxisOption::category(), AxisOption::category()],
            y_axis: vec![AxisOption::value()], // 单一 y 轴实例，两个 grid 共用
            series: vec![
                SeriesOption::Bar(BarSeriesOption {
                    name: Some("Bar1".into()),
                    data: vec![DataPoint::Named("A".into(), 10.0)],
                    grid_index: Some(0),
                    ..Default::default()
                }),
                SeriesOption::Bar(BarSeriesOption {
                    name: Some("Bar2".into()),
                    data: vec![DataPoint::Named("B".into(), 100.0)],
                    grid_index: Some(1),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        };
        let resolver = AxisBindingResolver::new(&option);
        let specs = vec![
            SubplotSpec {
                id: 0,
                bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
                series_indices: vec![0],
                x_axis_indices: vec![0],
                y_axis_indices: vec![0],
            },
            SubplotSpec {
                id: 1,
                bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
                series_indices: vec![1],
                x_axis_indices: vec![1],
                y_axis_indices: vec![0],
            },
        ];
        let result = resolver.resolve(&specs);

        let y0 = &result.ranges[2]; // x[0], x[1], y[0] = indices 0,1,2
        assert_eq!(y0.axis_index, 0);
        // 应覆盖两个 grid 的数据范围：10~100
        assert!(y0.min <= 10.0);
        assert!(y0.max >= 100.0);
    }
}