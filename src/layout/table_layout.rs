//! 表格布局模块 - 专门处理 TableSeries 的布局计算
//!
//! 表格与普通图表系列不同：
//! 1. 需要精确计算行高和列宽
//! 2. 可能需要滚动（内容超出 grid 时）
//! 3. 表头和表体需要分别处理

use super::{LayoutResult, Layoutable, SizeConstraint, measure_text_size};
use crate::model::{TableSeries, TextStyle};
use vello_cpu::kurbo::{Rect, Size};

/// 列配置
#[derive(Debug, Clone)]
pub struct ColumnConfig {
    pub index: usize,
    pub name: String,
    /// 固定宽度（None 表示自适应）
    pub fixed_width: Option<f64>,
    /// 最小宽度
    pub min_width: f64,
    /// 实际计算后的宽度
    pub computed_width: f64,
}

/// 表格布局计算器
#[derive(Debug, Clone)]
pub struct TableLayout {
    pub columns: Vec<ColumnConfig>,
    pub row_count: usize,
    pub header_height: f64,
    pub row_height: f64,
    pub padding: f64,
    pub header_style: TextStyle,
    pub body_style: TextStyle,
}

impl TableLayout {
    /// 从 TableSeries 创建布局计算器
    pub fn from_series(series: &TableSeries) -> Self {
        let columns: Vec<ColumnConfig> = series
            .columns
            .iter()
            .enumerate()
            .map(|(i, name)| ColumnConfig {
                index: i,
                name: name.clone(),
                fixed_width: None,
                min_width: 50.0, // 默认最小列宽
                computed_width: 0.0,
            })
            .collect();

        Self {
            columns,
            row_count: series.data.len(),
            header_height: series.header_config.height,
            row_height: series.body_config.row_height,
            padding: 8.0,
            header_style: series.header_config.style.clone(),
            body_style: series.body_config.style.clone(),
        }
    }

    /// 计算表格所需的最小尺寸
    pub fn measure(&self, available_width: f64) -> Size {
        // 计算总宽度
        let total_width = if self.columns.is_empty() {
            0.0
        } else {
            // 计算每列的最小宽度
            let col_widths = self.calc_column_widths(available_width - self.padding * 2.0);
            col_widths.iter().sum::<f64>() + self.padding * 2.0
        };

        // 计算总高度
        let header_height = if self.row_count > 0 {
            self.header_height
        } else {
            0.0
        };
        let body_height = self.row_count as f64 * self.row_height;
        let total_height = header_height + body_height + self.padding * 2.0;

        Size::new(total_width, total_height)
    }

    /// 计算列宽分配
    ///
    /// 策略：
    /// 1. 有固定宽度的列使用固定宽度
    /// 2. 剩余空间按内容比例分配给自适应列
    /// 3. 每列至少满足最小宽度
    pub fn calc_column_widths(&self, total_width: f64) -> Vec<f64> {
        if self.columns.is_empty() {
            return Vec::new();
        }

        let col_count = self.columns.len();

        // 计算每列的内容宽度
        let content_widths: Vec<f64> = self
            .columns
            .iter()
            .map(|col| {
                // 测量列名文本宽度
                let name_width = measure_text_size(&col.name, &self.header_style).width;
                name_width.max(col.min_width)
            })
            .collect();

        // 计算固定列总宽度
        let fixed_total: f64 = self
            .columns
            .iter()
            .filter_map(|col| col.fixed_width)
            .sum();

        let fixed_count = self.columns.iter().filter(|c| c.fixed_width.is_some()).count();
        let flexible_count = col_count - fixed_count;

        if flexible_count == 0 {
            // 所有列都是固定宽度
            return self
                .columns
                .iter()
                .map(|col| col.fixed_width.unwrap_or(0.0))
                .collect();
        }

        // 计算自适应列的可用空间
        let flexible_available = (total_width - fixed_total).max(0.0);
        let flexible_content_total: f64 = self
            .columns
            .iter()
            .enumerate()
            .filter(|(_, col)| col.fixed_width.is_none())
            .map(|(i, _)| content_widths[i])
            .sum();

        // 分配宽度
        self.columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                if let Some(fixed) = col.fixed_width {
                    fixed
                } else {
                    // 按内容比例分配
                    if flexible_content_total > 0.0 {
                        let ratio = content_widths[i] / flexible_content_total;
                        (flexible_available * ratio).max(col.min_width)
                    } else {
                        flexible_available / flexible_count as f64
                    }
                }
            })
            .collect()
    }

    /// 获取单元格位置
    pub fn get_cell_bounds(&self, row: usize, col: usize, table_bounds: Rect) -> Option<Rect> {
        if col >= self.columns.len() {
            return None;
        }

        let col_widths = self.calc_column_widths(table_bounds.width());
        let x_offset: f64 = col_widths.iter().take(col).sum();

        let cell_x = table_bounds.x0 + self.padding + x_offset;
        let cell_width = col_widths.get(col).copied().unwrap_or(0.0);

        let (cell_y, cell_height) = if row == 0 && self.header_height > 0.0 {
            // 表头行
            (table_bounds.y0 + self.padding, self.header_height)
        } else {
            // 数据行
            let data_row = if self.header_height > 0.0 { row - 1 } else { row };
            let y = table_bounds.y0 + self.padding + self.header_height + data_row as f64 * self.row_height;
            (y, self.row_height)
        };

        Some(Rect::new(
            cell_x,
            cell_y,
            cell_x + cell_width,
            cell_y + cell_height,
        ))
    }

    /// 获取表头边界
    pub fn get_header_bounds(&self, table_bounds: Rect) -> Option<Rect> {
        if self.header_height == 0.0 {
            return None;
        }

        Some(Rect::new(
            table_bounds.x0 + self.padding,
            table_bounds.y0 + self.padding,
            table_bounds.x1 - self.padding,
            table_bounds.y0 + self.padding + self.header_height,
        ))
    }

    /// 获取数据行边界
    pub fn get_row_bounds(&self, row: usize, table_bounds: Rect) -> Option<Rect> {
        if row >= self.row_count {
            return None;
        }

        let y = table_bounds.y0 + self.padding + self.header_height + row as f64 * self.row_height;

        Some(Rect::new(
            table_bounds.x0 + self.padding,
            y,
            table_bounds.x1 - self.padding,
            y + self.row_height,
        ))
    }
}

/// 表格布局元素 - 实现 Layoutable trait
pub struct TableLayoutElement {
    layout: TableLayout,
    result: Option<LayoutResult>,
}

impl TableLayoutElement {
    pub fn new(series: &TableSeries) -> Self {
        Self {
            layout: TableLayout::from_series(series),
            result: None,
        }
    }

    /// 获取表格布局计算器
    pub fn layout(&self) -> &TableLayout {
        &self.layout
    }
}

impl Layoutable for TableLayoutElement {
    fn measure(&mut self, constraint: SizeConstraint) -> Size {
        let size = self.layout.measure(constraint.max_width);

        // 确保满足约束
        let constrained_size = Size::new(
            size.width.clamp(constraint.min_width, constraint.max_width),
            size.height.clamp(constraint.min_height, constraint.max_height),
        );

        self.result = Some(LayoutResult::new(constrained_size));
        constrained_size
    }

    fn arrange(&mut self, bounds: Rect) {
        if let Some(ref mut result) = self.result {
            result.bounds = bounds;
        }
    }

    fn layout_result(&self) -> Option<&LayoutResult> {
        self.result.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{TableHeaderConfig, TableBodyConfig, TextStyle};
    use crate::visual::{Color, TextAlign};

    fn create_test_series() -> TableSeries {
        TableSeries {
            name: "Test Table".to_string(),
            data: vec![
                vec![serde_json::json!("Row1Col1"), serde_json::json!("Row1Col2")],
                vec![serde_json::json!("Row2Col1"), serde_json::json!("Row2Col2")],
            ],
            columns: vec!["Column 1".to_string(), "Column 2".to_string()],
            header_config: TableHeaderConfig {
                show: true,
                height: 40.0,
                style: TextStyle::default(),
                background_color: Color::new(248, 248, 248),
                align: TextAlign::Center,
            },
            body_config: TableBodyConfig {
                show: true,
                row_height: 32.0,
                style: TextStyle::default(),
                even_row_background_color: Color::new(255, 255, 255),
                odd_row_background_color: Color::new(250, 250, 250),
                align: TextAlign::Center,
            },
            grid_index: 0,
            auto_fit_grid: false,
        }
    }

    #[test]
    fn test_table_layout_measure() {
        let series = create_test_series();
        let layout = TableLayout::from_series(&series);

        let size = layout.measure(400.0);

        // 高度 = padding*2 + header + rows*row_height
        // = 16 + 40 + 2*32 = 120
        assert!(size.height >= 120.0);
        assert!(size.width >= 100.0); // 至少有两列
    }

    #[test]
    fn test_calc_column_widths() {
        let series = create_test_series();
        let layout = TableLayout::from_series(&series);

        let widths = layout.calc_column_widths(400.0);

        assert_eq!(widths.len(), 2);
        assert!(widths[0] > 0.0);
        assert!(widths[1] > 0.0);
    }
}
