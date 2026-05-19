//! 布局系统 V2 - 采用 Measure-Arrange 模式
//!
//! 核心设计原则：
//! 1. 分离测量和排列两个阶段
//! 2. 父容器决定子元素的位置和大小
//! 3. 子元素报告自己的期望尺寸
//! 4. 支持百分比和绝对定位

use vello_cpu::kurbo::{Point, Rect, Size};

use crate::{model, text::create_text_layout};

pub mod elements;
pub mod engine;
pub mod grid_manager;
pub mod table_layout;

// 重新导出常用类型
pub use elements::{AxisLayout, GridLayout, LegendLayout, TitleLayout};
pub use engine::{
    AxisArea, ChartLayout, GridLayoutInfo, LayoutEngine, LayoutOutput, SubplotLayout,
};
pub use grid_manager::{GridDefinition, GridManager, GridRect};
pub use table_layout::{ColumnConfig, TableLayout, TableLayoutElement};

pub use crate::option::AxisPosition;

/// 数据坐标系 - 统一数据值与像素坐标的映射
/// 确保坐标轴刻度和数据系列使用相同的坐标系
/// 支持多Y轴，每个系列可以绑定到不同的Y轴
#[derive(Debug, Clone)]
pub struct DataCoordinateSystem {
    pub x_range: (f64, f64),
    pub y_ranges: Vec<(f64, f64)>, // 支持多Y轴
    pub plot_bounds: Rect,
    pub is_category_x: bool,
    pub category_count: usize,
}

impl DataCoordinateSystem {
    pub fn new(
        plot_bounds: Rect,
        x_range: (f64, f64),
        y_ranges: Vec<(f64, f64)>,
        is_category_x: bool,
        _is_category_y: bool,
        category_count: usize,
    ) -> Self {
        Self {
            x_range,
            y_ranges,
            plot_bounds,
            is_category_x,
            category_count,
        }
    }

    /// 获取指定Y轴的范围，如果不存在则返回默认范围
    pub fn get_y_range(&self, y_axis_index: usize) -> (f64, f64) {
        self.y_ranges
            .get(y_axis_index)
            .copied()
            .or_else(|| self.y_ranges.first().copied())
            .unwrap_or((0.0, 100.0))
    }

    pub fn data_to_pixel(&self, data_x: f64, data_y: f64, y_axis_index: usize) -> Point {
        let x = self.x_to_pixel(data_x);
        let y = self.y_to_pixel(data_y, y_axis_index);
        Point::new(x, y)
    }

    pub fn x_to_pixel(&self, data_x: f64) -> f64 {
        self.plot_bounds.x0
            + (data_x - self.x_range.0) / (self.x_range.1 - self.x_range.0)
                * self.plot_bounds.width()
    }

    /// 将数值型 X 值映射到像素坐标（用于散点图等数值 X 轴图表）
    pub fn x_value_to_pixel(&self, value: f64) -> f64 {
        // 直接使用数值范围映射，不考虑类目索引
        self.plot_bounds.x0
            + (value - self.x_range.0) / (self.x_range.1 - self.x_range.0)
                * self.plot_bounds.width()
    }

    pub fn y_to_pixel(&self, data_y: f64, y_axis_index: usize) -> f64 {
        let y_range = self.get_y_range(y_axis_index);
        self.plot_bounds.y1
            - (data_y - y_range.0) / (y_range.1 - y_range.0) * self.plot_bounds.height()
    }

    /// 兼容旧代码，默认使用第一个Y轴
    pub fn y_to_pixel_default(&self, data_y: f64) -> f64 {
        self.y_to_pixel(data_y, 0)
    }

    pub fn category_width(&self) -> f64 {
        if self.category_count > 0 {
            self.plot_bounds.width() / self.category_count as f64
        } else {
            0.0
        }
    }
}

impl Default for DataCoordinateSystem {
    fn default() -> Self {
        Self {
            x_range: (0.0, 1.0),
            y_ranges: vec![(0.0, 100.0)],
            plot_bounds: Rect::new(0.0, 0.0, 1.0, 1.0),
            is_category_x: true,
            category_count: 0,
        }
    }
}

/// 尺寸约束 - 父容器对子组件的尺寸限制
#[derive(Debug, Clone, Copy)]
pub struct SizeConstraint {
    pub min_width: f64,
    pub max_width: f64,
    pub min_height: f64,
    pub max_height: f64,
}

impl SizeConstraint {
    pub fn fixed(width: f64, height: f64) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: height,
            max_height: height,
        }
    }

    pub fn unlimited() -> Self {
        Self {
            min_width: 0.0,
            max_width: f64::INFINITY,
            min_height: 0.0,
            max_height: f64::INFINITY,
        }
    }

    pub fn horizontal_unlimited(height: f64) -> Self {
        Self {
            min_width: 0.0,
            max_width: f64::INFINITY,
            min_height: height,
            max_height: height,
        }
    }

    pub fn vertical_unlimited(width: f64) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: 0.0,
            max_height: f64::INFINITY,
        }
    }

    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.min_width, self.max_width),
            size.height.clamp(self.min_height, self.max_height),
        )
    }
}

/// 布局结果
#[derive(Debug, Clone, Copy)]
pub struct LayoutResult {
    pub desired_size: Size,
    pub bounds: Rect,
}

impl LayoutResult {
    pub fn new(desired_size: Size) -> Self {
        Self {
            desired_size,
            bounds: Rect::from_origin_size(Point::ZERO, desired_size),
        }
    }

    pub fn with_bounds(desired_size: Size, bounds: Rect) -> Self {
        Self {
            desired_size,
            bounds,
        }
    }
}

/// 可布局元素 trait
pub trait Layoutable {
    /// 测量阶段 - 计算期望尺寸
    fn measure(&mut self, constraint: SizeConstraint) -> Size;

    /// 排列阶段 - 设置最终位置
    fn arrange(&mut self, bounds: Rect);

    /// 获取布局结果
    fn layout_result(&self) -> Option<&LayoutResult>;

    /// 如果是坐标轴，返回其位置（Left/Right/Bottom/Top）
    /// 非轴元素返回 None
    fn axis_position(&self) -> Option<AxisPosition> {
        None
    }
}

/// 布局上下文
#[derive(Debug, Clone)]
pub struct LayoutContext {
    pub chart_width: f64,
    pub chart_height: f64,
    pub padding: f64,
    pub spacing: f64,
}

impl LayoutContext {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            chart_width: width,
            chart_height: height,
            padding: 10.0,
            spacing: 5.0,
        }
    }
}

/// 解析位置
pub fn resolve_position(
    position: &crate::model::Position,
    container_size: f64,
    element_size: f64,
) -> f64 {
    use crate::model::Position;

    match position {
        Position::Auto => 0.0, // 自动布局，默认从起点开始
        Position::Left(v) | Position::Top(v) => *v,
        Position::Right(v) | Position::Bottom(v) => container_size - element_size - *v,
        Position::Center => (container_size - element_size) / 2.0,
        Position::Value(v) => *v,
        Position::Percent(p) => (container_size - element_size) * p / 100.0,
    }
}

/// 使用 TextStyle 测量文本尺寸
pub fn measure_text_size(text: &str, style: &model::TextStyle) -> Size {
    let layout = create_text_layout(text, style, None);
    Size::new(layout.width() as f64, layout.height() as f64)
}
