//! 系列渲染上下文 - 封装系列渲染所需的所有信息
//!
//! 该模块提供 SeriesContext，作为系列渲染的统一入口，
//! 封装了渲染过程中需要的所有上下文信息。

use vello_cpu::kurbo::{Point, Rect};

use crate::{
    layout::{DataCoordinateSystem, GridLayoutInfo, LayoutOutput},
    model::ChartModel,
    visual::Color,
};

/// 系列渲染上下文
///
/// 封装了系列渲染所需的所有信息，避免在渲染函数中传递多个参数
#[derive(Debug, Clone)]
pub struct SeriesContext<'a> {
    /// 系列索引
    pub series_index: usize,
    /// Grid 索引
    pub grid_index: usize,
    /// 解析后的配置
    pub resolved: &'a ChartModel,
    /// 布局输出
    pub layout: &'a LayoutOutput,
    /// Grid 布局信息
    pub grid_info: &'a GridLayoutInfo,
    /// 数据坐标系
    pub coord: &'a DataCoordinateSystem,
}

impl<'a> SeriesContext<'a> {
    /// 创建新的渲染上下文
    ///
    /// 需要提供所有必需的上下文信息
    pub fn new(
        series_index: usize,
        grid_index: usize,
        resolved: &'a ChartModel,
        layout: &'a LayoutOutput,
        grid_info: &'a GridLayoutInfo,
    ) -> Self {
        Self {
            series_index,
            grid_index,
            resolved,
            layout,
            grid_info,
            coord: &grid_info.data_coord,
        }
    }

    /// 尝试从布局中创建上下文
    ///
    /// 如果找不到对应的 grid，返回 None
    pub fn try_new(
        series_index: usize,
        grid_index: usize,
        resolved: &'a ChartModel,
        layout: &'a LayoutOutput,
    ) -> Option<Self> {
        let grid_info = layout.grids.iter().find(|g| g.grid_index == grid_index)?;
        Some(Self::new(
            series_index,
            grid_index,
            resolved,
            layout,
            grid_info,
        ))
    }

    /// 获取绘图区域边界
    pub fn plot_bounds(&self) -> Rect {
        self.grid_info.grid_inner_bbox
    }

    /// 获取系列颜色
    ///
    /// 优先使用 item_color，如果没有则使用主题色轮
    pub fn get_series_color(&self, item_color: Option<Color>) -> Color {
        item_color.unwrap_or_else(|| {
            self.resolved
                .colors
                .get(self.series_index % self.resolved.colors.len())
                .copied()
                .unwrap_or_else(|| Color::new(0, 0, 0))
        })
    }

    /// 获取系列颜色（带透明度）
    pub fn get_series_color_with_opacity(&self, item_color: Option<Color>, opacity: f64) -> Color {
        let mut color = self.get_series_color(item_color);
        color.a = (color.a as f64 * opacity).clamp(0.0, 255.0) as u8;
        color
    }

    /// 计算极坐标的中心点
    ///
    /// 默认使用绘图区域的中心
    pub fn get_polar_center(&self) -> Point {
        let bounds = self.plot_bounds();
        Point::new(
            bounds.x0 + bounds.width() * 0.5,
            bounds.y0 + bounds.height() * 0.5,
        )
    }

    /// 计算极坐标的最大半径
    ///
    /// 使用绘图区域宽高的较小值的一半
    pub fn get_polar_max_radius(&self) -> f64 {
        let bounds = self.plot_bounds();
        bounds.width().min(bounds.height()) * 0.5
    }

    /// 获取类别宽度
    ///
    /// 用于柱状图、蜡烛图等需要计算类别宽度的图表
    pub fn category_width(&self) -> f64 {
        self.coord.category_width()
    }

    /// 将数据坐标转换为像素坐标
    pub fn data_to_pixel(&self, x: f64, y: f64, y_axis_index: usize) -> Point {
        self.coord.data_to_pixel(x, y, y_axis_index)
    }
}

/// 系列渲染器 trait
///
/// 所有数据系列组件都应实现此 trait
pub trait SeriesRenderer {
    /// 检查数据是否为空
    fn is_empty(&self) -> bool;

    /// 获取 grid 索引
    fn grid_index(&self) -> usize;

    /// 获取系列索引
    fn series_index(&self) -> usize;

    /// 渲染为视觉元素
    fn render(&self, ctx: &SeriesContext) -> Vec<crate::visual::VisualElement>;
}

/// 笛卡尔坐标系列渲染器
///
/// 用于标准 X/Y 轴图表（柱状图、折线图、散点图等）
pub trait CartesianRenderer: SeriesRenderer {
    /// 获取 Y 轴索引
    fn y_axis_index(&self) -> usize;

    /// 数据是否为空
    fn is_data_empty(&self) -> bool;

    /// 执行笛卡尔坐标渲染
    fn render_cartesian(&self, ctx: &SeriesContext) -> Vec<crate::visual::VisualElement>;
}

/// 极坐标系列渲染器
///
/// 用于饼图、雷达图、极坐标柱状图等
pub trait PolarRenderer: SeriesRenderer {
    /// 获取中心点（百分比）
    fn center_percent(&self) -> (f64, f64) {
        (50.0, 50.0) // 默认中心
    }

    /// 获取半径（百分比）
    fn radius_percent(&self) -> (f64, f64) {
        (0.0, 75.0) // 默认半径范围
    }

    /// 执行极坐标渲染
    fn render_polar(
        &self,
        ctx: &SeriesContext,
        center: Point,
        max_radius: f64,
    ) -> Vec<crate::visual::VisualElement>;
}
