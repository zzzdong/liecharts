use vello_cpu::kurbo::Rect;

use crate::pipeline::types::{AxisSpec, GridSpec, SeriesSpec, SubplotSpec};

/// 纯数学画布切分器
///
/// 职责：仅根据 grid 配置和画布尺寸，计算每个 subplot 的像素边界。
/// **完全不接触系列数据、轴标签、文本测量、刻度计算**。
pub struct GridPlanner<'a> {
    total_width: u32,
    total_height: u32,
    grids: &'a [GridSpec],
    series: &'a [SeriesSpec],
    x_axes: &'a [AxisSpec],
    y_axes: &'a [AxisSpec],
}

impl<'a> GridPlanner<'a> {
    pub fn new(
        width: u32,
        height: u32,
        grids: &'a [GridSpec],
        series: &'a [SeriesSpec],
        x_axes: &'a [AxisSpec],
        y_axes: &'a [AxisSpec],
    ) -> Self {
        Self {
            total_width: width,
            total_height: height,
            grids,
            series,
            x_axes,
            y_axes,
        }
    }

    /// 执行画布切分，返回每个 subplot 的分配结果
    pub fn plan(&self) -> Vec<SubplotSpec> {
        let specs = if self.grids.is_empty() {
            vec![SubplotSpec {
                id: 0,
                bounds: self.default_bounds(),
                series_indices: Vec::new(),
                x_axis_indices: Vec::new(),
                y_axis_indices: Vec::new(),
            }]
        } else {
            self.grids
                .iter()
                .enumerate()
                .map(|(idx, grid)| {
                    let bounds = self.resolve_position(grid);
                    SubplotSpec {
                        id: idx,
                        bounds,
                        series_indices: Vec::new(),
                        x_axis_indices: Vec::new(),
                        y_axis_indices: Vec::new(),
                    }
                })
                .collect()
        };

        let mut specs = specs;

        // 绑定 series 到 grid
        for (series_idx, series) in self.series.iter().enumerate() {
            let grid_idx = series.grid_index;
            if grid_idx < specs.len() {
                specs[grid_idx].series_indices.push(series_idx);
            }
        }

        // 绑定 xAxis 到 grid
        for (axis_idx, axis) in self.x_axes.iter().enumerate() {
            let grid_idx = axis.grid_index;
            if grid_idx < specs.len() {
                specs[grid_idx].x_axis_indices.push(axis_idx);
            }
        }

        // 绑定 yAxis 到 grid
        for (axis_idx, axis) in self.y_axes.iter().enumerate() {
            let grid_idx = axis.grid_index;
            if grid_idx < specs.len() {
                specs[grid_idx].y_axis_indices.push(axis_idx);
            }
        }

        specs
    }

    fn default_bounds(&self) -> Rect {
        let total_w = self.total_width as f64;
        let total_h = self.total_height as f64;
        let margin = 60.0;
        Rect::new(margin, margin, total_w - margin, total_h - margin)
    }

    fn resolve_position(&self, grid: &GridSpec) -> Rect {
        let total_w = self.total_width as f64;
        let total_h = self.total_height as f64;

        let left = grid.left.unwrap_or(60.0);
        let right = grid.right.unwrap_or(60.0);
        let top = grid.top.unwrap_or(60.0);
        let bottom = grid.bottom.unwrap_or(60.0);

        let width = total_w - left - right;
        let height = total_h - top - bottom;

        Rect::new(left, top, left + width, top + height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pipeline::{
            dataframe::{DataFrame, DataValue, Series},
            types::*,
        },
    };

    fn make_series(name: &str, chart_type: ChartType, grid_index: usize) -> SeriesSpec {
        let mut df = DataFrame::new();
        df.add_column(Series::new("x", vec![DataValue::Float(0.0)]));
        df.add_column(Series::new("y", vec![DataValue::Float(0.0)]));
        SeriesSpec {
            name: name.to_string(),
            chart_type,
            data: df,
            x_col: "x".into(),
            y_col: "y".into(),
            grid_index,
            x_axis_index: 0,
            y_axis_index: 0,
            stack: None,
            group_index: 0,
            sampling: None,
            smooth: false,
            item_style: ItemStyleSpec::default(),
            ..Default::default()
        }
    }

    fn make_grids(count: usize) -> Vec<GridSpec> {
        (0..count)
            .map(|_| GridSpec {
                left: None,
                right: None,
                top: None,
                bottom: None,
                contain_label: false,
            })
            .collect()
    }

    #[test]
    fn test_single_grid_default() {
        let grids = vec![];
        let planner = GridPlanner::new(800, 600, &grids, &[], &[], &[]);
        let specs = planner.plan();

        assert_eq!(specs.len(), 1);
        let s = &specs[0];
        assert_eq!(s.id, 0);
        assert!(s.bounds.width() > 0.0);
        assert!(s.bounds.height() > 0.0);
        assert!(s.bounds.x0 >= 60.0);
        assert!(s.bounds.y0 >= 60.0);
    }

    #[test]
    fn test_two_grids_horizontal() {
        let grids = vec![
            GridSpec {
                left: Some(0.0),
                top: Some(0.0),
                right: Some(400.0), // width = 800 - 400 - 0 = 400
                bottom: Some(0.0),
                contain_label: false,
            },
            GridSpec {
                left: Some(400.0),
                top: Some(0.0),
                right: Some(0.0),
                bottom: Some(0.0),
                contain_label: false,
            },
        ];
        let series = vec![
            make_series("S1", ChartType::Bar, 0),
            make_series("S2", ChartType::Bar, 1),
        ];
        let planner = GridPlanner::new(800, 600, &grids, &series, &[], &[]);
        let specs = planner.plan();

        assert_eq!(specs.len(), 2);
        assert!((specs[0].bounds.x1 - 400.0).abs() < 1.0);
        assert!((specs[1].bounds.x0 - 400.0).abs() < 1.0);
        assert!((specs[1].bounds.x1 - 800.0).abs() < 1.0);

        assert_eq!(specs[0].series_indices, vec![0]);
        assert_eq!(specs[1].series_indices, vec![1]);
    }

    #[test]
    fn test_series_binding_to_grid() {
        let grids = make_grids(2);
        let series = vec![
            make_series("Line1", ChartType::Line, 0),
            make_series("Pie1", ChartType::Pie, 1),
            make_series("Bar1", ChartType::Bar, 0),
        ];
        let planner = GridPlanner::new(800, 600, &grids, &series, &[], &[]);
        let specs = planner.plan();

        assert_eq!(specs[0].series_indices, vec![0, 2]);
        assert_eq!(specs[1].series_indices, vec![1]);
    }

    #[test]
    fn test_axis_binding_to_grid() {
        let grids = make_grids(2);
        let x_axes = vec![
            AxisSpec {
                grid_index: 0,
                ..AxisSpec {
                    axis_type: AxisType::Category,
                    position: AxisPosition::Bottom,
                    grid_index: 0,
                    min: None,
                    max: None,
                    name: None,
                    categories: vec![],
                    boundary_gap: true,
                }
            },
            AxisSpec {
                grid_index: 1,
                ..AxisSpec {
                    axis_type: AxisType::Value,
                    position: AxisPosition::Bottom,
                    grid_index: 1,
                    min: None,
                    max: None,
                    name: None,
                    categories: vec![],
                    boundary_gap: true,
                }
            },
        ];
        let y_axes = vec![AxisSpec {
            grid_index: 0,
            ..AxisSpec {
                axis_type: AxisType::Value,
                position: AxisPosition::Left,
                grid_index: 0,
                min: None,
                max: None,
                name: None,
                categories: vec![],
                boundary_gap: true,
            }
        }];
        let planner = GridPlanner::new(800, 600, &grids, &[], &x_axes, &y_axes);
        let specs = planner.plan();

        assert_eq!(specs[0].x_axis_indices, vec![0]);
        assert_eq!(specs[1].x_axis_indices, vec![1]);
        assert_eq!(specs[0].y_axis_indices, vec![0]);
        assert!(specs[1].y_axis_indices.is_empty());
    }
}
