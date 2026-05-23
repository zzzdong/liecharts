use vello_cpu::kurbo::Rect;

use crate::new_pipeline::types::SubplotSpec;
use crate::option::{ChartOption, GridOption, PositionOption, SeriesOption};

/// 纯数学画布切分器
///
/// 职责：仅根据 option.grid 配置和画布尺寸，计算每个 subplot 的像素边界。
/// **完全不接触系列数据、轴标签、文本测量、刻度计算**。
pub struct GridPlanner {
    total_width: u32,
    total_height: u32,
    option: ChartOption,
}

impl GridPlanner {
    pub fn new(width: u32, height: u32, option: &ChartOption) -> Self {
        Self {
            total_width: width,
            total_height: height,
            option: option.clone(),
        }
    }

    /// 执行画布切分，返回每个 subplot 的分配结果
    ///
    /// 算法：
    /// - 如果 grid 配置为空，创建一个默认 grid 填满画布（预留 60px 边距）
    /// - 否则，按 ECharts 布局规则解析每个 GridOption：
    ///   · left/right/top/bottom：支持像素值或百分比
    /// - 同时解析每个 series 的 grid_index 绑定，以及轴的 grid_index 绑定
    pub fn plan(&self) -> Vec<SubplotSpec> {
        let grids = if self.option.grid.is_empty() {
            vec![GridOption::default()]
        } else {
            self.option.grid.clone()
        };

        let mut specs: Vec<SubplotSpec> = grids
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
            .collect();

        // 绑定 series 到 grid
        for (series_idx, series) in self.option.series.iter().enumerate() {
            let grid_idx = series_grid_index(series).unwrap_or(0);
            if grid_idx < specs.len() {
                specs[grid_idx].series_indices.push(series_idx);
            }
        }

        // 绑定 xAxis 到 grid
        for (axis_idx, axis) in self.option.x_axis.iter().enumerate() {
            let grid_idx = axis.grid_index.unwrap_or(0);
            if grid_idx < specs.len() {
                specs[grid_idx].x_axis_indices.push(axis_idx);
            }
        }

        // 绑定 yAxis 到 grid
        for (axis_idx, axis) in self.option.y_axis.iter().enumerate() {
            let grid_idx = axis.grid_index.unwrap_or(0);
            if grid_idx < specs.len() {
                specs[grid_idx].y_axis_indices.push(axis_idx);
            }
        }

        specs
    }

    fn resolve_position(&self, grid: &GridOption) -> Rect {
        let total_w = self.total_width as f64;
        let total_h = self.total_height as f64;

        let left = resolve_dimension(grid.left.as_ref(), total_w, 60.0);
        let right = resolve_dimension(grid.right.as_ref(), total_w, 60.0);
        let top = resolve_dimension(grid.top.as_ref(), total_h, 60.0);
        let bottom = resolve_dimension(grid.bottom.as_ref(), total_h, 60.0);

        let width = total_w - left - right;
        let height = total_h - top - bottom;

        Rect::new(left, top, left + width, top + height)
    }
}

/// 从 SeriesOption 中提取 grid_index
fn series_grid_index(series: &SeriesOption) -> Option<usize> {
    match series {
        SeriesOption::Line(s) => s.grid_index,
        SeriesOption::Bar(s) => s.grid_index,
        SeriesOption::Candlestick(s) => s.grid_index,
        SeriesOption::Pie(s) => s.grid_index,
        SeriesOption::Scatter(s) => s.grid_index,
        SeriesOption::Radar(_) => None,
        SeriesOption::PolarBar(_) => None,
        SeriesOption::PolarScatter(_) => None,
        SeriesOption::Bubble(_) => None,
        SeriesOption::Gauge(_) => None,
        SeriesOption::Table(_) => None,
    }
}

/// 解析维度值，支持像素值和百分比
fn resolve_dimension(pos: Option<&PositionOption>, total: f64, default: f64) -> f64 {
    pos.map(|p| match p {
        PositionOption::Pixel(v) => *v,
        PositionOption::Percent(pct) => total * pct / 100.0,
        _ => default,
    })
    .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::option::*;

    fn make_option(grids: Vec<GridOption>, series: Vec<SeriesOption>) -> ChartOption {
        ChartOption {
            grid: grids,
            series,
            x_axis: vec![AxisOption::category()],
            y_axis: vec![AxisOption::value()],
            ..Default::default()
        }
    }

    #[test]
    fn test_single_grid_default() {
        let option = make_option(vec![], vec![]);
        let planner = GridPlanner::new(800, 600, &option);
        let specs = planner.plan();

        assert_eq!(specs.len(), 1);
        let s = &specs[0];
        assert_eq!(s.id, 0);
        assert!(s.bounds.width() > 0.0);
        assert!(s.bounds.height() > 0.0);
        // 默认边距 60px
        assert!(s.bounds.x0 >= 60.0);
        assert!(s.bounds.y0 >= 60.0);
    }

    #[test]
    fn test_two_grids_horizontal() {
        let grids = vec![
            GridOption {
                left: Some(PositionOption::Pixel(0.0)),
                top: Some(PositionOption::Pixel(0.0)),
                right: Some(PositionOption::Percent(50.0)),
                bottom: Some(PositionOption::Pixel(0.0)),
                contain_label: None,
            },
            GridOption {
                left: Some(PositionOption::Percent(50.0)),
                top: Some(PositionOption::Pixel(0.0)),
                right: Some(PositionOption::Pixel(0.0)),
                bottom: Some(PositionOption::Pixel(0.0)),
                contain_label: None,
            },
        ];
        let series = vec![
            SeriesOption::Bar(BarSeriesOption {
                name: Some("S1".into()),
                grid_index: Some(0),
                ..Default::default()
            }),
            SeriesOption::Bar(BarSeriesOption {
                name: Some("S2".into()),
                grid_index: Some(1),
                ..Default::default()
            }),
        ];
        let option = make_option(grids, series);
        let planner = GridPlanner::new(800, 600, &option);
        let specs = planner.plan();

        assert_eq!(specs.len(), 2);

        // grid[0]: 左侧 0~400
        assert!((specs[0].bounds.x1 - 400.0).abs() < 1.0);
        // grid[1]: 右侧 400~800
        assert!((specs[1].bounds.x0 - 400.0).abs() < 1.0);
        assert!((specs[1].bounds.x1 - 800.0).abs() < 1.0);

        // series 绑定
        assert_eq!(specs[0].series_indices, vec![0]);
        assert_eq!(specs[1].series_indices, vec![1]);
    }

    #[test]
    fn test_series_binding_to_grid() {
        let grids = vec![
            GridOption::default(),
            GridOption::default(),
        ];
        let series = vec![
            SeriesOption::Line(LineSeriesOption {
                name: Some("Line1".into()),
                grid_index: Some(0),
                ..Default::default()
            }),
            SeriesOption::Pie(PieSeriesOption {
                name: Some("Pie1".into()),
                grid_index: Some(1),
                ..Default::default()
            }),
            SeriesOption::Bar(BarSeriesOption {
                name: Some("Bar1".into()),
                grid_index: Some(0),
                ..Default::default()
            }),
        ];
        let option = make_option(grids, series);
        let planner = GridPlanner::new(800, 600, &option);
        let specs = planner.plan();

        assert_eq!(specs[0].series_indices, vec![0, 2]); // Line1 + Bar1 → grid[0]
        assert_eq!(specs[1].series_indices, vec![1]); // Pie1 → grid[1]
    }

    #[test]
    fn test_axis_binding_to_grid() {
        let option = ChartOption {
            grid: vec![
                GridOption::default(),
                GridOption::default(),
            ],
            x_axis: vec![
                AxisOption { grid_index: Some(0), ..AxisOption::category() },
                AxisOption { grid_index: Some(1), ..AxisOption::value() },
            ],
            y_axis: vec![
                AxisOption { grid_index: Some(0), ..AxisOption::value() },
            ],
            ..Default::default()
        };
        let planner = GridPlanner::new(800, 600, &option);
        let specs = planner.plan();

        assert_eq!(specs[0].x_axis_indices, vec![0]);
        assert_eq!(specs[1].x_axis_indices, vec![1]);
        assert_eq!(specs[0].y_axis_indices, vec![0]);
        assert!(specs[1].y_axis_indices.is_empty());
    }
}