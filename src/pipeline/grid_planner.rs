use vello_cpu::kurbo::Rect;

use crate::pipeline::types::{AxisPosition, AxisSpec, AxisType, GridSpec, SeriesSpec, SubplotSpec};

/// 纯数学画布切分器
///
/// 职责：根据 grid 配置和画布尺寸，计算每个 subplot 的像素边界，
/// 并将 series/axis 按 grid_index 绑定到对应的 subplot。
/// **完全不接触系列数据、轴标签、文本测量、刻度计算**。
///
/// header_height 参数告诉 planner 顶部有多少空间被标题/图例占据，
/// 确保 subplot 不会与这些装饰元素重叠。
pub struct GridPlanner<'a> {
    total_width: u32,
    total_height: u32,
    header_height: f64,
    grids: &'a [GridSpec],
}

impl<'a> GridPlanner<'a> {
    pub fn new(
        width: u32,
        height: u32,
        header_height: f64,
        grids: &'a [GridSpec],
    ) -> Self {
        Self {
            total_width: width,
            total_height: height,
            header_height: header_height.max(0.0),
            grids,
        }
    }

    /// 执行画布切分，返回每个 subplot 的分配结果
    ///
    /// `series`、`x_axes`、`y_axes` 按各自的 grid_index 绑定到对应 subplot。
    pub fn plan(
        &self,
        series: &[SeriesSpec],
        x_axes: &[AxisSpec],
        y_axes: &[AxisSpec],
    ) -> Vec<SubplotSpec> {
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
        for (series_idx, series) in series.iter().enumerate() {
            let grid_idx = series.grid_index;
            if grid_idx < specs.len() {
                specs[grid_idx].series_indices.push(series_idx);
            }
        }

        // 绑定 xAxis 到 grid
        for (axis_idx, axis) in x_axes.iter().enumerate() {
            let grid_idx = axis.grid_index;
            if grid_idx < specs.len() {
                specs[grid_idx].x_axis_indices.push(axis_idx);
            }
        }

        // 绑定 yAxis 到 grid
        for (axis_idx, axis) in y_axes.iter().enumerate() {
            let grid_idx = axis.grid_index;
            if grid_idx < specs.len() {
                specs[grid_idx].y_axis_indices.push(axis_idx);
            }
        }

        // 根据坐标轴标签的实际占用空间自适应调整 subplot 边界，
        // 避免密集/长文本标签（尤其旋转后）超出画布或被截断。
        self.adjust_label_margins(&mut specs, x_axes, y_axes);

        specs
    }

    /// 根据坐标轴标签尺寸自适应放大边距。
    ///
    /// 与 `CartesianAxisRenderer` 共用同一套旋转决策：
    /// - X 轴标签横向放不下时自动旋转（45°/90°），按旋转后的投影高度预留底部空间
    /// - Y 轴标签按宽度预留左侧/右侧空间
    ///
    /// 文本尺寸使用启发式估算（见 `axis_label::estimate_text_size`），无需字体引擎。
    fn adjust_label_margins(
        &self,
        specs: &mut [SubplotSpec],
        x_axes: &[AxisSpec],
        y_axes: &[AxisSpec],
    ) {
        use crate::pipeline::axis_label::{auto_rotate, estimate_text_size, rotated_bounds};

        const FONT_SIZE: f64 = 11.0;
        const X_LABEL_GAP: f64 = 14.0; // 锚点距坐标轴的距离
        const Y_LABEL_GAP: f64 = 8.0; // 锚点距坐标轴的距离
        const LABEL_PAD: f64 = 4.0; // 额外安全边距
        const MIN_PLOT_W: f64 = 50.0;
        const MIN_PLOT_H: f64 = 40.0;
        const VALUE_TICK_ESTIMATE: &str = "1234.5";

        let total_w = self.total_width as f64;
        let total_h = self.total_height as f64;

        for spec in specs.iter_mut() {
            let mut grow_bottom: f64 = 0.0;
            let mut grow_top: f64 = 0.0;
            let mut grow_left: f64 = 0.0;
            let mut grow_right: f64 = 0.0;

            // ── X 轴：按旋转后的投影高度预留顶部/底部空间 ──
            for &axis_idx in &spec.x_axis_indices {
                let Some(axis) = x_axes.get(axis_idx) else {
                    continue;
                };
                if !axis.label_show {
                    continue;
                }
                let labels: Vec<String> = if axis.axis_type == AxisType::Category {
                    axis.categories.clone()
                } else {
                    vec![VALUE_TICK_ESTIMATE.to_string(); 5]
                };
                if labels.is_empty() {
                    continue;
                }
                let n = labels.len();
                let slot_w = spec.bounds.width() / n as f64;
                let (max_w, max_h) = labels
                    .iter()
                    .map(|l| estimate_text_size(l, FONT_SIZE))
                    .fold((0.0_f64, 0.0_f64), |acc, s| {
                        (acc.0.max(s.0), acc.1.max(s.1))
                    });
                let rotation = axis
                    .label_rotate
                    .map(|deg| deg.to_radians())
                    .unwrap_or_else(|| auto_rotate(max_w, max_h, slot_w));
                let (_, rotated_h) = rotated_bounds(max_w, max_h, rotation);
                let needed = X_LABEL_GAP + rotated_h + LABEL_PAD;
                if axis.position == AxisPosition::Top {
                    // 顶部 X 轴：标签在绘图区上方，且不能侵入标题/图例占用的头部空间，
                    // 可用空间 = 绘图区上缘到画布顶部的距离减去 header_height
                    let current = (spec.bounds.y0 - self.header_height).max(0.0);
                    if needed > current {
                        grow_top = grow_top.max(needed - current);
                    }
                } else {
                    let current = total_h - spec.bounds.y1;
                    if needed > current {
                        grow_bottom = grow_bottom.max(needed - current);
                    }
                }
            }

            // ── Y 轴：按标签宽度预留左侧/右侧空间 ──
            for &axis_idx in &spec.y_axis_indices {
                let Some(axis) = y_axes.get(axis_idx) else {
                    continue;
                };
                if !axis.label_show {
                    continue;
                }
                let labels: Vec<String> = if axis.axis_type == AxisType::Category {
                    axis.categories.clone()
                } else {
                    vec![VALUE_TICK_ESTIMATE.to_string(); 5]
                };
                if labels.is_empty() {
                    continue;
                }
                let (max_w, max_h) = labels
                    .iter()
                    .map(|l| estimate_text_size(l, FONT_SIZE))
                    .fold((0.0_f64, 0.0_f64), |acc, s| {
                        (acc.0.max(s.0), acc.1.max(s.1))
                    });
                // Y 轴不自动旋转，仅尊重用户配置
                let rotation = axis
                    .label_rotate
                    .map(|deg| deg.to_radians())
                    .unwrap_or(0.0);
                let (rotated_w, _) = rotated_bounds(max_w, max_h, rotation);
                let needed = Y_LABEL_GAP + rotated_w + LABEL_PAD;
                let is_right = axis.position == AxisPosition::Right;
                if is_right {
                    let current = total_w - spec.bounds.x1;
                    if needed > current {
                        grow_right = grow_right.max(needed - current);
                    }
                } else {
                    let current = spec.bounds.x0;
                    if needed > current {
                        grow_left = grow_left.max(needed - current);
                    }
                }
            }

            // 应用增长（保留最小绘图尺寸）
            if grow_left > 0.0 || grow_right > 0.0 {
                let new_x0 = spec.bounds.x0 + grow_left;
                let new_x1 = spec.bounds.x1 - grow_right;
                if new_x1 - new_x0 >= MIN_PLOT_W {
                    spec.bounds.x0 = new_x0;
                    spec.bounds.x1 = new_x1;
                }
            }
            if grow_bottom > 0.0 {
                let new_y1 = spec.bounds.y1 - grow_bottom;
                if new_y1 - spec.bounds.y0 >= MIN_PLOT_H {
                    spec.bounds.y1 = new_y1;
                }
            }
            if grow_top > 0.0 {
                let new_y0 = spec.bounds.y0 + grow_top;
                if spec.bounds.y1 - new_y0 >= MIN_PLOT_H {
                    spec.bounds.y0 = new_y0;
                }
            }
        }
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
        let default_bottom = 60.0;

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
    use crate::pipeline::types::*;

    fn make_series(name: &str, grid_index: usize) -> SeriesSpec {
        use crate::pipeline::dataframe::{DataFrame, DataValue, Series};
        let mut df = DataFrame::new();
        df.add_column(Series::new("x", vec![DataValue::Float(0.0)]));
        df.add_column(Series::new("y", vec![DataValue::Float(0.0)]));
        SeriesSpec {
            name: name.to_string(),
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
        let planner = GridPlanner::new(800, 600, 100.0, &grids);
        let specs = planner.plan(&[], &[], &[]);

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
        let planner = GridPlanner::new(800, 600, 0.0, &grids);
        let specs = planner.plan(&[], &[], &[]);

        let s = &specs[0];
        // top 回退到 60.0（因为 0.0 < margin 60.0）
        assert!(s.bounds.y0 >= 60.0);
    }

    #[test]
    fn test_dense_long_category_labels_grow_bottom_margin() {
        let grids = make_grids(1);
        // 30 个长日期标签，slot 宽度远小于标签宽度 → 自动旋转 90°，底部边距增大
        let x_axes = vec![AxisSpec {
            axis_type: AxisType::Category,
            position: AxisPosition::Bottom,
            grid_index: 0,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories: (0..30)
                .map(|i| format!("2024-01-{:02}", i + 1))
                .collect(),
            boundary_gap: true,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate: None,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }];
        let planner = GridPlanner::new(800, 600, 60.0, &grids);
        let specs = planner.plan(&[], &x_axes, &[]);

        // 默认 bounds.y1 = 600 - 60 = 540；密集长标签应使绘图区向上收缩
        assert!(
            specs[0].bounds.y1 < 540.0,
            "旋转标签后底部边距应增大，实际 y1={}",
            specs[0].bounds.y1
        );
    }

    #[test]
    fn test_top_axis_dense_labels_grow_top_margin() {
        let grids = make_grids(1);
        // 顶部 X 轴 + 密集长标签 → 上边距增大（绘图区下移）
        let x_axes = vec![AxisSpec {
            axis_type: AxisType::Category,
            position: AxisPosition::Top,
            grid_index: 0,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories: (0..30)
                .map(|i| format!("2024-01-{:02}", i + 1))
                .collect(),
            boundary_gap: true,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate: None,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }];
        let planner = GridPlanner::new(800, 600, 60.0, &grids);
        let specs = planner.plan(&[], &x_axes, &[]);

        // 默认 bounds.y0 = 60（header_height=60）；
        // 顶部密集长标签需要 14 + 投影高 + 4 ≈ 78px 空间，
        // 且不能侵入头部（header_height）区域 → 绘图区应明显下移
        assert!(
            specs[0].bounds.y0 > 60.0 + 60.0,
            "顶部旋转标签后上边距应增大，实际 y0={}",
            specs[0].bounds.y0
        );
    }

    #[test]
    fn test_short_labels_keep_default_margins() {
        let grids = make_grids(1);
        // 3 个短标签，横向能放下 → 不旋转、不额外预留
        let x_axes = vec![AxisSpec {
            axis_type: AxisType::Category,
            position: AxisPosition::Bottom,
            grid_index: 0,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories: vec!["一".into(), "二".into(), "三".into()],
            boundary_gap: true,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate: None,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }];
        let planner = GridPlanner::new(800, 600, 60.0, &grids);
        let specs = planner.plan(&[], &x_axes, &[]);

        assert!((specs[0].bounds.y1 - 540.0).abs() < 1e-6);
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
            make_series("S1", 0),
            make_series("S2", 1),
        ];
        let planner = GridPlanner::new(800, 600, 100.0, &grids);
        let specs = planner.plan(&series, &[], &[]);

        assert_eq!(specs.len(), 2);
        // Grid 0: left=0 → effective_left=50, right=400 → effective_right=440, width=310, x1=360
        assert!((specs[0].bounds.x1 - 360.0).abs() < 1.0);
        // Grid 1: left=400 → effective_left=450, right=0 → effective_right=40, x0=450, x1=760
        assert!((specs[1].bounds.x0 - 450.0).abs() < 1.0);
        assert!((specs[1].bounds.x1 - 760.0).abs() < 1.0);

        assert_eq!(specs[0].series_indices, vec![0]);
        assert_eq!(specs[1].series_indices, vec![1]);
    }

    #[test]
    fn test_series_binding_to_grid() {
        let grids = make_grids(2);
        let series = vec![
            make_series("Line1", 0),
            make_series("Pie1", 1),
            make_series("Bar1", 0),
        ];
        let planner = GridPlanner::new(800, 600, 100.0, &grids);
        let specs = planner.plan(&series, &[], &[]);

        assert_eq!(specs[0].series_indices, vec![0, 2]);
        assert_eq!(specs[1].series_indices, vec![1]);
    }

    #[test]
    fn test_axis_binding_to_grid() {
        let grids = make_grids(2);
        let x_axes = vec![
            AxisSpec {
                axis_type: AxisType::Category,
                position: AxisPosition::Bottom,
                grid_index: 0,
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
            },
            AxisSpec {
                axis_type: AxisType::Value,
                position: AxisPosition::Bottom,
                grid_index: 1,
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
            },
        ];
        let y_axes = vec![AxisSpec {
            axis_type: AxisType::Value,
            position: AxisPosition::Left,
            grid_index: 0,
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
        }];
        let planner = GridPlanner::new(800, 600, 100.0, &grids);
        let specs = planner.plan(&[], &x_axes, &y_axes);

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
        let planner = GridPlanner::new(800, 600, 100.0, &grids);
        let specs = planner.plan(&[], &[], &[]);
        let s = &specs[0];

        // contain_label=true 时 left 默认 70，right 默认 50，bottom 默认 60
        assert!((s.bounds.x0 - 70.0).abs() < 1.0);
        assert!((s.bounds.x1 - 750.0).abs() < 1.0); // 800 - 50
        assert!((s.bounds.y0 - 100.0).abs() < 1.0); // header_height
        assert!((s.bounds.y1 - 540.0).abs() < 1.0); // 600 - 60
    }
}
