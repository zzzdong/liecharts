//! 组件基础模块 - 提供系列组件的通用功能
//!
//! 该模块提供 SeriesComponentBase，封装了所有系列组件共用的功能：
//! - Grid 信息获取
//! - 颜色管理
//! - 上下文创建

use crate::layout::{GridLayoutInfo, LayoutOutput};
use crate::model::ResolvedOption;
use crate::visual::Color;

/// 系列组件基类 - 提供通用功能
#[derive(Debug, Clone)]
pub struct SeriesComponentBase {
    pub series_index: usize,
    pub grid_index: usize,
}

impl SeriesComponentBase {
    /// 创建新的系列组件基类
    pub fn new(series_index: usize, grid_index: usize) -> Self {
        Self {
            series_index,
            grid_index,
        }
    }

    /// 获取 grid 信息
    ///
    /// 根据 grid_index 从布局输出中查找对应的 grid 信息
    pub fn get_grid_info<'a>(&self, layout: &'a LayoutOutput) -> Option<&'a GridLayoutInfo> {
        layout.grids.iter().find(|g| g.grid_index == self.grid_index)
    }

    /// 获取系列颜色
    ///
    /// 优先使用 item_color，如果没有则使用主题色轮
    pub fn get_series_color(&self, resolved: &ResolvedOption, item_color: Option<Color>) -> Color {
        item_color.unwrap_or_else(|| {
            resolved.colors
                .get(self.series_index % resolved.colors.len())
                .copied()
                .unwrap_or_else(|| Color::new(0, 0, 0))
        })
    }

    /// 获取系列颜色（带透明度）
    pub fn get_series_color_with_opacity(
        &self,
        resolved: &ResolvedOption,
        item_color: Option<Color>,
        opacity: f64,
    ) -> Color {
        let mut color = self.get_series_color(resolved, item_color);
        color.a = (color.a as f64 * opacity).clamp(0.0, 255.0) as u8;
        color
    }

    /// 创建边框样式
    pub fn create_border_stroke(
        &self,
        border_color: Option<Color>,
        border_width: f64,
        _fill_color: Color,
    ) -> Option<crate::visual::Stroke> {
        border_color.map(|c| crate::visual::Stroke {
            color: c,
            width: border_width,
        })
    }
}

/// 系列组件扩展 trait
///
/// 为具体系列组件提供统一的接口
pub trait SeriesComponentExt {
    /// 获取基础信息
    fn base(&self) -> &SeriesComponentBase;

    /// 检查数据是否为空
    fn is_empty(&self) -> bool;

    /// 获取 grid 索引
    fn grid_index(&self) -> usize {
        self.base().grid_index
    }

    /// 获取系列索引
    fn series_index(&self) -> usize {
        self.base().series_index
    }

    /// 获取 grid 信息
    fn get_grid_info<'a>(&self, layout: &'a LayoutOutput) -> Option<&'a GridLayoutInfo> {
        self.base().get_grid_info(layout)
    }

    /// 获取系列颜色
    fn get_color(&self, resolved: &ResolvedOption, item_color: Option<Color>) -> Color {
        self.base().get_series_color(resolved, item_color)
    }
}
