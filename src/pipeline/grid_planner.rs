use vello_cpu::kurbo::Rect;

use crate::pipeline::types::{AxisSpec, GridSpec, SeriesSpec, SubplotSpec};

/// 纯数学画布切分器
///
/// 职责：根据 grid 配置和画布尺寸，计算每个 subplot 的像素边界。
/// **完全不接触系列数据、轴标签、文本测量、刻度计算**。
///
/// header_height 参数告诉 planner 顶部有多少空间被标题/图例占据，
/// 确保 subplot 不会与这些装饰元素重叠。
pub struct GridPlanner<'a> {
    total_width: u32,
    total_height: u32,
    header_height: f64,
    grids: &'a [GridSpec],
    series: &'a [SeriesSpec],
    x_axes: &'a [AxisSpec],
    y_axes: &'a [AxisSpec],
}

impl<'a> GridPlanner<'a> {
    pub fn new(
        width: u32,
        height: u32,
        header_height: f64,
        grids: &'a [GridSpec],
        series: &'a [SeriesSpec],
        x_axes: &'a [AxisSpec],
        y_axes: &'a [AxisSpec],
    ) -> Self {
        Self {
            total_width: width,
            total_height: height,
            header_height: header_height.max(0.0),
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

    /// 无 grid 配置时的默认区域
    ///
    /// left/right/bottom: 留 60px 边距供轴标签和名称使用
    /// top: 使用 header_height（若无标题/图例则回退 60px）
    fn default_bounds(&self) -> Rect {
        let total_w = self.total_width as f64;
        let total_h = self.total_height as f64;
        let margin = 60.0;
        let top = self.header_height.max(margin);
        Rect::new(margin, top, total_w - margin, total_h - margin)
    }

    /// 根据 GridSpec 计算 subplot 像素边界
    ///
    /// - 当用户显式指定 left/right/top/bottom 时，直接使用
    /// - 当值为 None（auto）时：
    ///   - top 使用 header_height（标题/图例空间）
    ///   - 若 contain_label=true，left/bottom 使用更大默认值以容纳轴标签
    ///   - 否则使用标准边距
    ///
    /// 注意：用户指定的 bottom 值被视为子图整体（含坐标轴标签）的底部边距，
    /// 因此计算图表区域时会额外减去标签占用的空间（约 28px），
    /// 确保坐标轴标签不会被画布底部截断。
    fn resolve_position(&self, grid: &GridSpec) -> Rect {
        let total_w = self.total_width as f64;
        let total_h = self.total_height as f64;

        // 根据 contain_label 决定默认边距
        // contain_label=true 时，边距需要足够容纳轴刻度标签
        let default_left = if grid.contain_label { 70.0 } else { 60.0 };
        let default_right = if grid.contain_label { 50.0 } else { 60.0 };
        let default_bottom = if grid.contain_label { 60.0 } else { 60.0 };

        let left = grid.left.unwrap_or(default_left);
        let right = grid.right.unwrap_or(default_right);
        let top = grid.top.unwrap_or(self.header_height.max(40.0));
        let bottom = grid.bottom.unwrap_or(default_bottom);

        // 当用户显式设置了边距值时，额外添加标签留白空间：
        // - bottom: X 轴刻度标签在 bounds.y1 + 14px 处，约占用 28px 高度
        // - left: Y 轴刻度标签在 bounds.x0 - 8px 处，数字标签约 35px 宽，留 50px
        // - right: 右侧 Y 轴刻度标签在 bounds.x1 + 8px 处，留 40px
        const LABEL_BOTTOM_PADDING: f64 = 28.0;
        const LABEL_LEFT_PADDING: f64 = 50.0;
        const LABEL_RIGHT_PADDING: f64 = 40.0;

        let effective_bottom = if grid.bottom.is_some() {
            bottom + LABEL_BOTTOM_PADDING
        } else {
            bottom
        };
        let effective_left = if grid.left.is_some() {
            left + LABEL_LEFT_PADDING
        } else {
            left
        };
        let effective_right = if grid.right.is_some() {
            right + LABEL_RIGHT_PADDING
        } else {
            right
        };

        let width = total_w - effective_left - effective_right;
        let height = total_h - top - effective_bottom;

        Rect::new(effective_left, top, effective_left + width, top + height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{
        dataframe::{DataFrame, DataValue, Series},
        types::*,
    };

    fn make_series(name: &str, chart_type: ChartType, grid_index: usize) -> SeriesSpec {
        use crate::pipeline::dataframe::{DataFrame, DataValue, Series};
        let mut df = DataFrame::new();
        df.add_column(Series::new("x", vec![DataValue::Float(0.0)]));
        df.add_column(Series::new("y", vec![DataValue::Float(0.0)]));
        SeriesSpec {
            name: name.to_string(),
            chart_type,
            data: df,
            grid_index,
            x_axis_index: 0,
            y_axis_index: 0,
            stack: None,
            group_index: 0,
            sampling: None,
            item_style: ItemStyleSpec::default(),
            config: SeriesConfig::Line(LineConfig::default()),
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
        let planner = GridPlanner::new(800, 600, 100.0, &grids, &[], &[], &[]);
        let specs = planner.plan();

        assert_eq!(specs.len(), 1);
        let s = &specs[0];
        assert_eq!(s.id, 0);
        assert!(s.bounds.width() > 0.0);
        assert!(s.bounds.height() > 0.0);
        assert!(s.bounds.x0 >= 60.0);
        assert!(s.bounds.y0 >= 100.0); // header_height
    }

    #[test]
    fn test_single_grid_no_header() {
        let grids = vec![];
        let planner = GridPlanner::new(800, 600, 0.0, &grids, &[], &[], &[]);
        let specs = planner.plan();

        let s = &specs[0];
        // top 回退到 60.0（因为 0.0 < margin 60.0）
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
        let planner = GridPlanner::new(800, 600, 100.0, &grids, &series, &[], &[]);
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
        let planner = GridPlanner::new(800, 600, 100.0, &grids, &series, &[], &[]);
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
        let planner = GridPlanner::new(800, 600, 100.0, &grids, &[], &x_axes, &y_axes);
        let specs = planner.plan();

        assert_eq!(specs[0].x_axis_indices, vec![0]);
        assert_eq!(specs[1].x_axis_indices, vec![1]);
        assert_eq!(specs[0].y_axis_indices, vec![0]);
        assert!(specs[1].y_axis_indices.is_empty());
    }

    #[test]
    fn test_contain_label_increases_margins() {
        let grids = vec![GridSpec {
            left: None,
            right: None,
            top: None,
            bottom: None,
            contain_label: true,
        }];
        let planner = GridPlanner::new(800, 600, 100.0, &grids, &[], &[], &[]);
        let specs = planner.plan();
        let s = &specs[0];

        // contain_label=true 时 left 默认 70，right 默认 50，bottom 默认 60
        assert!((s.bounds.x0 - 70.0).abs() < 1.0);
        assert!((s.bounds.x1 - 750.0).abs() < 1.0); // 800 - 50
        assert!((s.bounds.y0 - 100.0).abs() < 1.0); // header_height
        assert!((s.bounds.y1 - 540.0).abs() < 1.0); // 600 - 60
    }
}
