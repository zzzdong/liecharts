//! 布局引擎 - 实现 Measure-Arrange 布局流程
//!
//! V2 版本改进：
//! 1. 集成 GridManager 统一管理多 grid 布局
//! 2. 支持表格内容的最小尺寸约束
//! 3. 改进坐标轴与 grid 的空间分配

use super::{
    DataCoordinateSystem, GridDefinition, GridManager, GridRect, LayoutContext, Layoutable,
    SizeConstraint,
};
use crate::option::AxisPosition;
use vello_cpu::kurbo::Rect;

/// 坐标轴区域信息
#[derive(Debug, Clone)]
pub struct AxisArea {
    pub axis_bbox: Rect,
    pub label_bbox: Rect,
    pub position: AxisPosition,
    /// 关联的网格区域（用于数值轴计算刻度位置）
    pub grid_bbox: Rect,
}

/// 单个子图的布局信息
#[derive(Debug, Clone)]
pub struct GridLayoutInfo {
    pub grid_index: usize,
    pub grid_bbox: Rect,
    pub grid_inner_bbox: Rect,
    pub x_axis_areas: Vec<AxisArea>,
    pub y_axis_areas: Vec<AxisArea>,
    pub data_coord: DataCoordinateSystem,
}

/// 布局输出结果
#[derive(Debug, Clone)]
pub struct LayoutOutput {
    pub title_bbox: Option<Rect>,
    pub legend_bbox: Option<Rect>,
    pub grids: Vec<GridLayoutInfo>,
}

/// 单个子图的布局结构
pub struct SubplotLayout {
    pub grid_index: usize,
    pub grid: Box<dyn Layoutable>,
    pub x_axes: Vec<Box<dyn Layoutable>>,
    pub y_axes: Vec<Box<dyn Layoutable>>,
    /// 子图的位置配置（从 Grid 解析）
    pub left: crate::model::Position,
    pub right: crate::model::Position,
    pub top: crate::model::Position,
    pub bottom: crate::model::Position,
}

/// 图表布局结构
pub struct ChartLayout {
    pub title: Option<Box<dyn Layoutable>>,
    pub legend: Option<Box<dyn Layoutable>>,
    pub subplots: Vec<SubplotLayout>,
}

/// 布局引擎
pub struct LayoutEngine {
    context: LayoutContext,
    grid_manager: GridManager,
}

impl LayoutEngine {
    pub fn new(context: LayoutContext) -> Self {
        let grid_manager = GridManager::new(context.clone());
        Self {
            context,
            grid_manager,
        }
    }

    /// 执行完整布局流程，返回 LayoutOutput
    ///
    /// 新的布局流程：
    /// 1. Measure 阶段：测量标题、图例、坐标轴
    /// 2. Grid 布局：使用 GridManager 计算所有 grid 的位置
    /// 3. Arrange 阶段：在 grid 内排列坐标轴和内容
    pub fn layout(&mut self, chart_layout: &mut ChartLayout) -> LayoutOutput {
        // Measure 阶段
        self.measure(chart_layout);

        // 准备 chart bounds
        let chart_bounds = Rect::new(
            0.0,
            0.0,
            self.context.chart_width,
            self.context.chart_height,
        );

        // 使用 GridManager 计算 grid 布局
        let grid_definitions: Vec<GridDefinition> = chart_layout
            .subplots
            .iter()
            .map(|subplot| {
                GridDefinition::new(
                    subplot.grid_index,
                    crate::model::Grid {
                        left: subplot.left.clone(),
                        right: subplot.right.clone(),
                        top: subplot.top.clone(),
                        bottom: subplot.bottom.clone(),
                        contain_label: false,
                    },
                )
            })
            .collect();

        let grid_rects = self
            .grid_manager
            .compute_layout(&grid_definitions, chart_bounds);

        // Arrange 阶段
        self.arrange_with_grids(chart_layout, chart_bounds, &grid_rects)
    }

    /// Measure 阶段 - 自底向上计算期望尺寸
    fn measure(&mut self, chart_layout: &mut ChartLayout) {
        // 1. 测量标题
        if let Some(ref mut title) = chart_layout.title {
            let constraint = SizeConstraint {
                min_width: 0.0,
                max_width: f64::INFINITY,
                min_height: 0.0,
                max_height: self.context.chart_height * 0.3,
            };
            title.measure(constraint);
        }

        // 2. 测量图例
        if let Some(ref mut legend) = chart_layout.legend {
            let constraint = SizeConstraint {
                min_width: 0.0,
                max_width: f64::INFINITY,
                min_height: 0.0,
                max_height: self.context.chart_height * 0.2,
            };
            legend.measure(constraint);
        }

        // 3. 测量每个子图的坐标轴和网格
        for subplot in &mut chart_layout.subplots {
            for axis in &mut subplot.x_axes {
                let constraint = SizeConstraint::vertical_unlimited(self.context.chart_width);
                axis.measure(constraint);
            }

            for axis in &mut subplot.y_axes {
                let constraint = SizeConstraint::horizontal_unlimited(self.context.chart_height);
                axis.measure(constraint);
            }

            let constraint = SizeConstraint::unlimited();
            subplot.grid.measure(constraint);
        }
    }

    /// Arrange 阶段 - 使用预计算的 grid 位置进行排列
    fn arrange_with_grids(
        &mut self,
        chart_layout: &mut ChartLayout,
        chart_bounds: Rect,
        grid_rects: &[GridRect],
    ) -> LayoutOutput {
        let mut current_y = chart_bounds.y0 + self.context.padding;

        // 1. 排列标题
        let mut has_title = false;
        if let Some(ref mut title) = chart_layout.title
            && let Some(result) = title.layout_result()
        {
            let title_height = result.desired_size.height;
            let title_bounds = Rect::new(
                chart_bounds.x0 + self.context.padding,
                current_y,
                chart_bounds.x1 - self.context.padding,
                current_y + title_height,
            );
            title.arrange(title_bounds);
            current_y += title_height;
            has_title = true;
        }

        // 2. 排列图例
        let mut has_legend = false;
        if let Some(ref mut legend) = chart_layout.legend
            && let Some(result) = legend.layout_result()
        {
            let height = result.desired_size.height;
            if has_title {
                current_y += self.context.spacing;
            }
            let legend_bounds = Rect::new(
                chart_bounds.x0 + self.context.padding,
                current_y,
                chart_bounds.x1 - self.context.padding,
                current_y + height,
            );
            legend.arrange(legend_bounds);
            current_y += height;
            has_legend = true;
        }

        // 计算主绘图区域起始位置
        if has_title || has_legend {
            current_y += self.context.spacing;
        }

        // 3. 为每个子图计算布局，使用 GridManager 提供的 grid 位置
        let mut grids = Vec::new();

        for (subplot, grid_rect) in chart_layout.subplots.iter_mut().zip(grid_rects.iter()) {
            let grid_info = self.arrange_subplot_with_grid(subplot, grid_rect, current_y);
            grids.push(grid_info);
        }

        LayoutOutput {
            title_bbox: chart_layout
                .title
                .as_ref()
                .and_then(|t| t.layout_result().map(|r| r.bounds)),
            legend_bbox: chart_layout
                .legend
                .as_ref()
                .and_then(|l| l.layout_result().map(|r| r.bounds)),
            grids,
        }
    }

    /// 排列单个子图 - 使用预计算的 grid 位置
    fn arrange_subplot_with_grid(
        &mut self,
        subplot: &mut SubplotLayout,
        grid_rect: &GridRect,
        plot_top: f64,
    ) -> GridLayoutInfo {
        // 使用 GridManager 计算好的 grid 边界，并考虑标题/图例的偏移
        let mut grid_bounds = grid_rect.bounds;
        if grid_bounds.y0 < plot_top {
            let offset = plot_top - grid_bounds.y0;
            grid_bounds = Rect::new(
                grid_bounds.x0,
                plot_top,
                grid_bounds.x1,
                grid_bounds.y1 + offset,
            );
        }

        // 2. 计算 inner bounds（实际绘图区域）
        // 当 contain_label=false 时，inner_bounds 等于 grid_bounds
        // 坐标轴排列在 grid 边缘，可能超出边界
        let inner_bounds = grid_bounds;

        // 3. 排列 Y 轴
        // 坐标轴从 grid 边缘向外排列（ECharts 行为：containLabel=false 时标签可能超出容器）
        let mut current_left_width = 0.0;
        let mut current_right_width = 0.0;
        let mut y_axis_areas = Vec::new();

        for (i, axis) in subplot.y_axes.iter_mut().enumerate() {
            if let Some(result) = axis.layout_result() {
                let axis_width = result.desired_size.width;
                let is_right = i % 2 == 1;

                let (axis_bounds, position) = if is_right {
                    // 右侧轴：从 grid 右边缘向外排列
                    let bounds = Rect::new(
                        grid_bounds.x1 + current_right_width,
                        grid_bounds.y0,
                        grid_bounds.x1 + current_right_width + axis_width,
                        grid_bounds.y1,
                    );
                    current_right_width += axis_width;
                    (bounds, AxisPosition::Right)
                } else {
                    // 左侧轴：从 grid 左边缘向外排列
                    let bounds = Rect::new(
                        grid_bounds.x0 - axis_width - current_left_width,
                        grid_bounds.y0,
                        grid_bounds.x0 - current_left_width,
                        grid_bounds.y1,
                    );
                    current_left_width += axis_width;
                    (bounds, AxisPosition::Left)
                };

                axis.arrange(axis_bounds);

                y_axis_areas.push(AxisArea {
                    axis_bbox: axis_bounds,
                    label_bbox: axis_bounds,
                    position,
                    grid_bbox: inner_bounds,
                });
            }
        }

        // 4. 排列 X 轴
        // 坐标轴从 grid 边缘向外排列（ECharts 行为：containLabel=false 时标签可能超出容器）
        let mut current_bottom_height = 0.0;
        let mut current_top_height = 0.0;
        let mut x_axis_areas = Vec::new();

        for (i, axis) in subplot.x_axes.iter_mut().enumerate() {
            if let Some(result) = axis.layout_result() {
                let axis_height = result.desired_size.height;
                let is_top = i % 2 == 1;

                let (axis_bounds, position) = if is_top {
                    // 顶部轴：从 grid 上边缘向外排列
                    let bounds = Rect::new(
                        grid_bounds.x0,
                        grid_bounds.y0 - axis_height - current_top_height,
                        grid_bounds.x1,
                        grid_bounds.y0 - current_top_height,
                    );
                    current_top_height += axis_height;
                    (bounds, AxisPosition::Top)
                } else {
                    // 底部轴：从 grid 下边缘向外排列
                    let bounds = Rect::new(
                        grid_bounds.x0,
                        grid_bounds.y1 + current_bottom_height,
                        grid_bounds.x1,
                        grid_bounds.y1 + current_bottom_height + axis_height,
                    );
                    current_bottom_height += axis_height;
                    (bounds, AxisPosition::Bottom)
                };

                axis.arrange(axis_bounds);

                x_axis_areas.push(AxisArea {
                    axis_bbox: axis_bounds,
                    label_bbox: axis_bounds,
                    position,
                    grid_bbox: inner_bounds,
                });
            }
        }

        // 5. 排列网格（使用 inner_bounds 作为实际绘图区域）
        subplot.grid.arrange(inner_bounds);

        GridLayoutInfo {
            grid_index: subplot.grid_index,
            grid_bbox: grid_bounds,
            grid_inner_bbox: inner_bounds,
            x_axis_areas,
            y_axis_areas,
            data_coord: DataCoordinateSystem::default(),
        }
    }
}
