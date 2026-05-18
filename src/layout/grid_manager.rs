//! Grid 布局管理器 - 负责计算所有 grid 的位置和大小
//!
//! 核心职责：
//! 1. 解析 Position 配置为绝对坐标
//! 2. 处理 grid 之间的冲突和重叠
//! 3. 支持表格内容的最小尺寸约束
//! 4. 提供自动网格布局功能

use super::LayoutContext;
use crate::model::{Grid, Position, TableSeries};
use vello_cpu::kurbo::{Rect, Size};

/// Grid 定义 - 包含配置和计算后的布局信息
#[derive(Debug, Clone)]
pub struct GridDefinition {
    pub index: usize,
    pub config: Grid,
    /// 最小尺寸要求（由内容决定，如表格的行数）
    pub min_size: Size,
    /// 是否自动调整大小以适应内容
    pub auto_resize: bool,
}

impl GridDefinition {
    pub fn new(index: usize, config: Grid) -> Self {
        Self {
            index,
            config,
            min_size: Size::new(100.0, 100.0), // 默认最小尺寸
            auto_resize: false,
        }
    }

    /// 从表格系列创建，应用表格的尺寸约束
    pub fn with_table_series(mut self, table: &TableSeries) -> Self {
        // 计算表格所需的最小高度
        let header_height = if table.header_config.show {
            table.header_config.height
        } else {
            0.0
        };
        let body_height = if table.body_config.show {
            table.data.len() as f64 * table.body_config.row_height
        } else {
            0.0
        };
        let padding = 16.0; // 8px * 2

        self.min_size = Size::new(
            200.0, // 表格最小宽度
            header_height + body_height + padding,
        );
        self.auto_resize = table.auto_fit_grid;
        self
    }
}

/// Grid 布局结果
#[derive(Debug, Clone, Copy)]
pub struct GridRect {
    pub index: usize,
    /// Grid 的外边界（包含坐标轴空间）
    pub bounds: Rect,
    /// Grid 的内边界（实际绘图区域）
    pub inner_bounds: Rect,
}

/// Grid 布局管理器
pub struct GridManager {
    context: LayoutContext,
}

impl GridManager {
    pub fn new(context: LayoutContext) -> Self {
        Self { context }
    }

    /// 计算所有 grid 的布局
    ///
    /// 算法步骤：
    /// 1. 解析所有 Position 为绝对坐标
    /// 2. 应用最小尺寸约束
    /// 3. 检测冲突并调整（简单策略：按顺序排列）
    /// 4. 返回最终布局
    pub fn compute_layout(&self, grids: &[GridDefinition], container: Rect) -> Vec<GridRect> {
        if grids.is_empty() {
            return Vec::new();
        }

        // 第一步：计算每个 grid 的原始边界
        let mut raw_rects: Vec<(usize, Rect)> = grids
            .iter()
            .map(|grid| {
                let bounds = self.calculate_grid_bounds(grid, container);
                (grid.index, bounds)
            })
            .collect();

        // 第二步：应用最小尺寸约束
        for (i, grid) in grids.iter().enumerate() {
            if grid.auto_resize {
                let (_, ref mut bounds) = raw_rects[i];
                let min_width = grid.min_size.width;
                let min_height = grid.min_size.height;

                if bounds.width() < min_width {
                    let extra = min_width - bounds.width();
                    bounds.x1 += extra;
                }
                if bounds.height() < min_height {
                    let extra = min_height - bounds.height();
                    bounds.y1 += extra;
                }
            }
        }

        // 第三步：检测并解决冲突
        // 简化策略：如果 grid 重叠，按索引顺序调整
        self.resolve_conflicts(&mut raw_rects, container);

        // 第四步：构建最终布局
        raw_rects
            .into_iter()
            .map(|(index, bounds)| {
                let inner_bounds = self.calculate_inner_bounds(bounds, container);
                GridRect {
                    index,
                    bounds,
                    inner_bounds,
                }
            })
            .collect()
    }

    /// 计算单个 grid 的边界
    fn calculate_grid_bounds(&self, grid: &GridDefinition, container: Rect) -> Rect {
        let chart_width = container.width();
        let chart_height = container.height();

        // 解析 left
        let left = match &grid.config.left {
            Position::Value(v) => container.x0 + v,
            Position::Percent(pct) => container.x0 + chart_width * pct / 100.0,
            Position::Auto => container.x0 + self.context.padding,
            _ => container.x0 + self.context.padding,
        };

        // 解析 right
        let right = match &grid.config.right {
            Position::Value(v) => container.x1 - v,
            Position::Percent(pct) => container.x1 - chart_width * pct / 100.0,
            Position::Auto => container.x1 - self.context.padding,
            _ => container.x1 - self.context.padding,
        };

        // 解析 top
        let top = match &grid.config.top {
            Position::Value(v) => container.y0 + v,
            Position::Percent(pct) => container.y0 + chart_height * pct / 100.0,
            Position::Auto => container.y0 + self.context.padding,
            _ => container.y0 + self.context.padding,
        };

        // 解析 bottom
        let bottom = match &grid.config.bottom {
            Position::Value(v) => container.y1 - v,
            Position::Percent(pct) => container.y1 - chart_height * pct / 100.0,
            Position::Auto => container.y1 - self.context.padding,
            _ => container.y1 - self.context.padding,
        };

        // 确保边界有效
        let left = left.min(right - 50.0).max(container.x0);
        let right = right.max(left + 50.0).min(container.x1);
        let top = top.min(bottom - 50.0).max(container.y0);
        let bottom = bottom.max(top + 50.0).min(container.y1);

        Rect::new(left, top, right, bottom)
    }

    /// 计算 grid 的内边界（减去 padding）
    fn calculate_inner_bounds(&self, bounds: Rect, _container: Rect) -> Rect {
        // 内边界暂时等于外边界，坐标轴布局会在后续步骤中调整
        // 这里预留一些空间给坐标轴标签
        let padding = 5.0;
        Rect::new(
            bounds.x0 + padding,
            bounds.y0 + padding,
            bounds.x1 - padding,
            bounds.y1 - padding,
        )
    }

    /// 解决 grid 之间的冲突
    ///
    /// 当前策略：检测重叠，如果重叠则按索引顺序向下/向右推移
    fn resolve_conflicts(&self, rects: &mut [(usize, Rect)], container: Rect) {
        let n = rects.len();
        if n <= 1 {
            return;
        }

        // 简单的冲突解决：按索引排序后，确保不重叠
        // 实际项目中可能需要更复杂的算法
        for i in 0..n {
            for j in (i + 1)..n {
                let rect_i = rects[i].1;
                let rect_j = rects[j].1;

                if Self::rects_overlap(rect_i, rect_j) {
                    // 简单策略：将后面的 grid 向下移动
                    let overlap_y = rect_i.y1 - rect_j.y0;
                    rects[j].1.y0 += overlap_y + self.context.spacing;
                    rects[j].1.y1 += overlap_y + self.context.spacing;

                    // 如果超出容器底部，改为向右移动
                    if rects[j].1.y1 > container.y1 {
                        rects[j].1.y0 = container.y0 + self.context.padding;
                        rects[j].1.y1 -= overlap_y + self.context.spacing;

                        let overlap_x = rect_i.x1 - rect_j.x0;
                        rects[j].1.x0 += overlap_x + self.context.spacing;
                        rects[j].1.x1 += overlap_x + self.context.spacing;
                    }
                }
            }
        }
    }

    /// 检查两个矩形是否重叠
    fn rects_overlap(a: Rect, b: Rect) -> bool {
        a.x0 < b.x1 && a.x1 > b.x0 && a.y0 < b.y1 && a.y1 > b.y0
    }

    /// 自动网格布局 - 当用户未指定位置时
    ///
    /// 将容器等分为 rows x cols 的网格
    pub fn auto_layout(
        &self,
        grids: &[GridDefinition],
        container: Rect,
        rows: usize,
        cols: usize,
        gap: f64,
    ) -> Vec<GridRect> {
        if grids.is_empty() || rows == 0 || cols == 0 {
            return Vec::new();
        }

        let available_width = container.width() - self.context.padding * 2.0;
        let available_height = container.height() - self.context.padding * 2.0;

        let cell_width = (available_width - gap * (cols - 1) as f64) / cols as f64;
        let cell_height = (available_height - gap * (rows - 1) as f64) / rows as f64;

        grids
            .iter()
            .enumerate()
            .map(|(i, grid)| {
                let row = i / cols;
                let col = i % cols;

                let x0 = container.x0 + self.context.padding + col as f64 * (cell_width + gap);
                let y0 = container.y0 + self.context.padding + row as f64 * (cell_height + gap);
                let x1 = x0 + cell_width;
                let y1 = y0 + cell_height;

                let bounds = Rect::new(x0, y0, x1, y1);
                let inner_bounds = self.calculate_inner_bounds(bounds, container);

                GridRect {
                    index: grid.index,
                    bounds,
                    inner_bounds,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_grid(left: Position, right: Position, top: Position, bottom: Position) -> Grid {
        Grid {
            left,
            right,
            top,
            bottom,
            contain_label: false,
        }
    }

    fn create_test_context() -> LayoutContext {
        LayoutContext::new(800.0, 600.0)
    }

    #[test]
    fn test_calculate_grid_bounds() {
        let context = create_test_context();
        let manager = GridManager::new(context);

        let grid = GridDefinition::new(
            0,
            create_test_grid(
                Position::Value(10.0),
                Position::Value(10.0),
                Position::Value(10.0),
                Position::Value(10.0),
            ),
        );

        let container = Rect::new(0.0, 0.0, 800.0, 600.0);
        let bounds = manager.calculate_grid_bounds(&grid, container);

        assert_eq!(bounds.x0, 10.0);
        assert_eq!(bounds.x1, 790.0);
        assert_eq!(bounds.y0, 10.0);
        assert_eq!(bounds.y1, 590.0);
    }

    #[test]
    fn test_auto_layout() {
        let context = create_test_context();
        let manager = GridManager::new(context);

        let grids = vec![
            GridDefinition::new(
                0,
                create_test_grid(
                    Position::Auto,
                    Position::Auto,
                    Position::Auto,
                    Position::Auto,
                ),
            ),
            GridDefinition::new(
                1,
                create_test_grid(
                    Position::Auto,
                    Position::Auto,
                    Position::Auto,
                    Position::Auto,
                ),
            ),
            GridDefinition::new(
                2,
                create_test_grid(
                    Position::Auto,
                    Position::Auto,
                    Position::Auto,
                    Position::Auto,
                ),
            ),
            GridDefinition::new(
                3,
                create_test_grid(
                    Position::Auto,
                    Position::Auto,
                    Position::Auto,
                    Position::Auto,
                ),
            ),
        ];

        let container = Rect::new(0.0, 0.0, 800.0, 600.0);
        let layout = manager.auto_layout(&grids, container, 2, 2, 10.0);

        assert_eq!(layout.len(), 4);
        // 检查第一个 grid 的位置
        assert_eq!(layout[0].bounds.x0, 10.0);
        assert_eq!(layout[0].bounds.y0, 10.0);
    }
}
