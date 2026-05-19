//! 渲染器工具模块 - 提供通用的渲染辅助函数
//!
//! 该模块提供了一系列辅助函数，用于简化系列组件的渲染逻辑。

use vello_cpu::kurbo::Point;

use crate::{
    component::context::{CartesianRenderer, PolarRenderer, SeriesContext},
    visual::{Color, Stroke, VisualElement},
};

/// 标准笛卡尔坐标渲染流程
///
/// 适用于使用 Pipeline 模式的图表（柱状图、折线图、散点图等）
pub fn render_cartesian_pipeline<R: CartesianRenderer>(
    renderer: &R,
    ctx: &SeriesContext,
) -> Vec<VisualElement> {
    if renderer.is_data_empty() {
        return Vec::new();
    }

    // 委托给具体实现
    renderer.render_cartesian(ctx)
}

/// 标准极坐标渲染流程
///
/// 适用于饼图、雷达图、极坐标图表等
pub fn render_polar_pipeline<R: PolarRenderer>(
    renderer: &R,
    ctx: &SeriesContext,
) -> Vec<VisualElement> {
    if renderer.is_empty() {
        return Vec::new();
    }

    let plot_bounds = ctx.plot_bounds();

    // 计算中心点和半径
    let center_percent = renderer.center_percent();
    let center = Point::new(
        plot_bounds.x0 + plot_bounds.width() * center_percent.0 / 100.0,
        plot_bounds.y0 + plot_bounds.height() * center_percent.1 / 100.0,
    );

    let radius_percent = renderer.radius_percent();
    let max_radius = ctx.get_polar_max_radius() * radius_percent.1 / 100.0;

    // 委托给具体实现
    renderer.render_polar(ctx, center, max_radius)
}

/// 创建系列样式
#[derive(Debug, Clone)]
pub struct SeriesStyle {
    pub color: Color,
    pub stroke: Option<Stroke>,
    pub fill: Option<Color>,
}

impl SeriesStyle {
    /// 创建新的系列样式
    pub fn new(color: Color) -> Self {
        Self {
            color,
            stroke: None,
            fill: Some(color),
        }
    }

    /// 设置描边
    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// 设置填充
    pub fn with_fill(mut self, fill: Color) -> Self {
        self.fill = Some(fill);
        self
    }

    /// 设置无边框
    pub fn without_stroke(mut self) -> Self {
        self.stroke = None;
        self
    }

    /// 设置无填充
    pub fn without_fill(mut self) -> Self {
        self.fill = None;
        self
    }
}

/// 标签配置
#[derive(Debug, Clone)]
pub struct LabelConfig {
    pub show: bool,
    pub position: LabelPosition,
    pub color: Color,
    pub font_size: f64,
    pub font_family: String,
    pub formatter: Option<String>,
}

impl Default for LabelConfig {
    fn default() -> Self {
        Self {
            show: false,
            position: LabelPosition::Top,
            color: Color::new(0, 0, 0),
            font_size: 12.0,
            font_family: "sans-serif".to_string(),
            formatter: None,
        }
    }
}

impl LabelConfig {
    /// 创建启用的标签配置
    pub fn enabled(color: Color, font_size: f64) -> Self {
        Self {
            show: true,
            color,
            font_size,
            ..Default::default()
        }
    }

    /// 设置位置
    pub fn with_position(mut self, position: LabelPosition) -> Self {
        self.position = position;
        self
    }

    /// 设置格式化器
    pub fn with_formatter(mut self, formatter: String) -> Self {
        self.formatter = Some(formatter);
        self
    }
}

/// 标签位置
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelPosition {
    Top,
    Inside,
    Bottom,
    Left,
    Right,
    Outside,
    Center,
}

/// 从 model::Label 创建 LabelConfig
pub fn create_label_config(label: &Option<crate::model::Label>) -> LabelConfig {
    match label {
        Some(l) if l.show => LabelConfig {
            show: true,
            position: match l.position {
                crate::model::LabelPosition::Top => LabelPosition::Top,
                crate::model::LabelPosition::Inside => LabelPosition::Inside,
                crate::model::LabelPosition::Bottom => LabelPosition::Bottom,
                crate::model::LabelPosition::Left => LabelPosition::Left,
                crate::model::LabelPosition::Right => LabelPosition::Right,
                crate::model::LabelPosition::Outside => LabelPosition::Outside,
                _ => LabelPosition::Top,
            },
            color: l.color,
            font_size: l.font_size,
            font_family: l.font_family.clone(),
            formatter: l.formatter.clone(),
        },
        _ => LabelConfig::default(),
    }
}

/// 网格辅助函数
pub mod grid_utils {
    use super::*;

    /// 计算类别中心 X 坐标
    pub fn calc_category_center_x(ctx: &SeriesContext, index: usize) -> f64 {
        ctx.coord.x_to_pixel(index as f64 + 0.5)
    }

    /// 计算 Y 坐标
    pub fn calc_y_pixel(ctx: &SeriesContext, value: f64, y_axis_index: usize) -> f64 {
        ctx.coord.y_to_pixel(value, y_axis_index)
    }

    /// 计算柱状图宽度
    pub fn calc_bar_width(ctx: &SeriesContext, ratio: f64) -> f64 {
        ctx.category_width() * ratio
    }
}

/// 颜色辅助函数
pub mod color_utils {
    use super::*;

    /// 应用透明度到颜色
    pub fn apply_opacity(color: Color, opacity: f64) -> Color {
        let mut result = color;
        result.a = (result.a as f64 * opacity).clamp(0.0, 255.0) as u8;
        result
    }

    /// 获取对比色（用于文本）
    pub fn get_contrast_color(background: Color) -> Color {
        // 简单的亮度计算
        let luminance = (0.299 * background.r as f64
            + 0.587 * background.g as f64
            + 0.114 * background.b as f64)
            / 255.0;

        if luminance > 0.5 {
            Color::new(0, 0, 0) // 深色背景用黑色文字
        } else {
            Color::new(255, 255, 255) // 浅色背景用白色文字
        }
    }
}
