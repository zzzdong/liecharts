//! 布局元素 - Title、Legend、Axis、Grid 的布局实现

use vello_cpu::kurbo::{Rect, Size};

use super::{LayoutResult, Layoutable, SizeConstraint, measure_text_size, resolve_position};
use crate::{
    model::{Axis, TextStyle},
    option::AxisPosition,
    text::layout_text,
    visual::TextAlign,
};

/// 标题布局元素
pub struct TitleLayout {
    text: String,
    subtext: Option<String>,
    text_style: TextStyle,
    subtext_style: Option<TextStyle>,
    left: crate::model::Position,
    top: crate::model::Position,
    result: Option<LayoutResult>,
}

impl TitleLayout {
    pub fn new(
        text: String,
        subtext: Option<String>,
        text_style: TextStyle,
        subtext_style: Option<TextStyle>,
        left: crate::model::Position,
        top: crate::model::Position,
    ) -> Self {
        Self {
            text,
            subtext,
            text_style,
            subtext_style,
            left,
            top,
            result: None,
        }
    }

    fn calculate_desired_size(&self) -> Size {
        let layout = if let Some(ref subtext) = self.subtext {
            let sub_style = self.subtext_style.clone().unwrap_or_else(|| TextStyle {
                font_size: 12.0,
                ..Default::default()
            });
            let main_with_newline = format!("{}\n", self.text);
            layout_text(
                &[
                    (&main_with_newline, &self.text_style),
                    (subtext, &sub_style),
                ],
                None,
                TextAlign::Center,
            )
        } else {
            layout_text(&[(&self.text, &self.text_style)], None, TextAlign::Center)
        };
        Size::new(layout.width() as f64, layout.height() as f64)
    }
}

impl Layoutable for TitleLayout {
    fn measure(&mut self, constraint: SizeConstraint) -> Size {
        let desired = self.calculate_desired_size();
        let size = constraint.constrain(desired);
        self.result = Some(LayoutResult::new(size));
        size
    }

    fn arrange(&mut self, bounds: Rect) {
        if let Some(ref mut result) = self.result {
            let x = resolve_position(&self.left, bounds.width(), result.desired_size.width);
            let max_y = (bounds.height() - result.desired_size.height).max(0.0);
            let y =
                resolve_position(&self.top, bounds.height(), result.desired_size.height).min(max_y);

            result.bounds = Rect::new(
                bounds.x0 + x,
                bounds.y0 + y,
                bounds.x0 + x + result.desired_size.width,
                bounds.y0 + y + result.desired_size.height,
            );
        }
    }

    fn layout_result(&self) -> Option<&LayoutResult> {
        self.result.as_ref()
    }
}

/// 图例布局元素
pub struct LegendLayout {
    data: Vec<String>,
    orient: crate::model::Orient,
    left: crate::model::Position,
    top: crate::model::Position,
    item_heights: Vec<f64>,
    item_widths: Vec<f64>,
    wrap_cols: usize,
    result: Option<LayoutResult>,
}

impl LegendLayout {
    pub fn new(
        data: Vec<String>,
        orient: crate::model::Orient,
        left: crate::model::Position,
        top: crate::model::Position,
        text_style: TextStyle,
        symbol_size: f64,
        item_height: f64,
    ) -> Self {
        // 预计算每项宽度和高度（与渲染端 LegendComponent 保持一致）
        // 宽度 = 色块 + 间距(5) + 文本宽度 + 右边距(10)
        // 高度取所有项中最大的文本高度，确保横向同行时行高统一
        let mut max_text_height = 0.0f64;
        let item_widths: Vec<f64> = data
            .iter()
            .map(|s| {
                let size = measure_text_size(s, &text_style);
                let text_h = size.height + 8.0;
                if text_h > max_text_height {
                    max_text_height = text_h;
                }
                symbol_size + 5.0 + size.width + 10.0
            })
            .collect();
        let item_height = item_height.max(max_text_height);
        let item_heights: Vec<f64> = data.iter().map(|_| item_height).collect();

        Self {
            data,
            orient,
            left,
            top,
            item_heights,
            item_widths,
            wrap_cols: 0,
            result: None,
        }
    }

    /// 获取每项宽度（用于渲染时对齐）
    pub fn item_widths(&self) -> &[f64] {
        &self.item_widths
    }

    /// 获取每项高度
    pub fn item_heights(&self) -> &[f64] {
        &self.item_heights
    }

    /// 获取换行后的列数
    pub fn wrap_cols(&self) -> usize {
        self.wrap_cols
    }

    /// 计算水平布局换行后的行数
    fn calc_wrap_rows(&self, max_width: f64) -> (usize, Vec<f64>) {
        if self.data.is_empty() {
            return (0, Vec::new());
        }
        let mut rows = 1;
        let mut col_widths = Vec::new();
        let mut current_row: Vec<f64> = Vec::new();
        let mut current_row_width = 0.0;

        for &w in &self.item_widths {
            let gap = if current_row.is_empty() { 0.0 } else { 10.0 };
            if current_row.is_empty() {
                current_row.push(w);
                current_row_width = w;
            } else if current_row_width + gap + w <= max_width {
                current_row.push(w);
                current_row_width += gap + w;
            } else {
                col_widths.push(current_row.iter().copied().fold(0.0f64, f64::max));
                current_row = vec![w];
                current_row_width = w;
                rows += 1;
            }
        }
        if !current_row.is_empty() {
            col_widths.push(current_row.iter().copied().fold(0.0f64, f64::max));
        }
        (rows, col_widths)
    }

    fn calculate_desired_size(&self, max_width: f64) -> Size {
        if self.data.is_empty() {
            return Size::new(0.0, 0.0);
        }

        match self.orient {
            crate::model::Orient::Horizontal => {
                if max_width.is_infinite() {
                    // 不换行：所有项目在一行
                    let total_width: f64 = self.item_widths.iter().sum::<f64>()
                        + 10.0 * (self.data.len().saturating_sub(1)) as f64;
                    let height = self.item_heights[0];
                    Size::new(total_width, height)
                } else {
                    // 换行：计算行数
                    let (rows, _) = self.calc_wrap_rows(max_width);
                    let row_height = self.item_heights[0];
                    Size::new(max_width, rows as f64 * row_height)
                }
            }
            crate::model::Orient::Vertical => {
                let width = self.item_widths.iter().copied().fold(0.0f64, f64::max);
                let height: f64 = self.item_heights.iter().sum::<f64>()
                    + 5.0 * (self.data.len().saturating_sub(1)) as f64;
                Size::new(width, height)
            }
        }
    }
}

impl Layoutable for LegendLayout {
    fn measure(&mut self, constraint: SizeConstraint) -> Size {
        let desired = self.calculate_desired_size(constraint.max_width);
        let size = constraint.constrain(desired);
        // 记录换行列数，供渲染时使用
        if self.orient == crate::model::Orient::Horizontal && constraint.max_width < f64::INFINITY {
            let (_, col_widths) = self.calc_wrap_rows(constraint.max_width);
            self.wrap_cols = col_widths.len();
        } else {
            self.wrap_cols = self.data.len();
        }
        self.result = Some(LayoutResult::new(size));
        size
    }

    fn arrange(&mut self, bounds: Rect) {
        if let Some(ref mut result) = self.result {
            let x = resolve_position(&self.left, bounds.width(), result.desired_size.width);
            let max_y = (bounds.height() - result.desired_size.height).max(0.0);
            let y =
                resolve_position(&self.top, bounds.height(), result.desired_size.height).min(max_y);

            result.bounds = Rect::new(
                bounds.x0 + x,
                bounds.y0 + y,
                bounds.x0 + x + result.desired_size.width,
                bounds.y0 + y + result.desired_size.height,
            );
        }
    }

    fn layout_result(&self) -> Option<&LayoutResult> {
        self.result.as_ref()
    }
}

/// 坐标轴布局元素
///
/// 持有 `model::Axis` 配置，在 measure 阶段计算标签和名称尺寸，
/// 通过 `total_outside_extent()` 统一计算轴线外侧的总占用尺寸。
pub struct AxisLayout {
    config: Axis,
    /// 标签区域在垂直于轴线方向的最大尺寸（垂直轴=宽度，水平轴=高度）
    label_extent: f64,
    result: Option<LayoutResult>,
}

impl AxisLayout {
    pub fn new(config: Axis) -> Self {
        // 预测量标签最大尺寸
        let label_extent = match config.position {
            AxisPosition::Left | AxisPosition::Right => config
                .data
                .as_ref()
                .map(|d| {
                    d.iter()
                        .map(|s| {
                            measure_text_size(
                                s,
                                &TextStyle {
                                    font_size: config.axis_label.font_size,
                                    font_family: config.axis_label.font_family.clone(),
                                    ..Default::default()
                                },
                            )
                            .width
                        })
                        .fold(0.0, f64::max)
                })
                .unwrap_or_else(|| {
                    // 数值轴：用 "9999" 估算最宽4位数标签
                    measure_text_size(
                        "9999",
                        &TextStyle {
                            font_size: config.axis_label.font_size,
                            font_family: config.axis_label.font_family.clone(),
                            ..Default::default()
                        },
                    )
                    .width
                    .max(30.0)
                }),
            AxisPosition::Bottom | AxisPosition::Top => config
                .data
                .as_ref()
                .map(|d| {
                    d.iter()
                        .map(|s| {
                            measure_text_size(
                                s,
                                &TextStyle {
                                    font_size: config.axis_label.font_size,
                                    font_family: config.axis_label.font_family.clone(),
                                    ..Default::default()
                                },
                            )
                            .height
                        })
                        .fold(0.0, f64::max)
                })
                .unwrap_or(config.axis_label.font_size * 2.0),
        };

        Self {
            config,
            label_extent,
            result: None,
        }
    }

    /// 计算轴线外侧方向的总占用尺寸（垂直于轴线）
    ///
    /// 统一公式：tick_length + label_padding + label_extent
    /// 名称不参与布局计算（与 ECharts 行为一致），仅用于渲染定位。
    fn total_outside_extent(&self) -> f64 {
        self.config.tick_length + self.config.label_padding + self.label_extent
    }
}

impl Layoutable for AxisLayout {
    fn measure(&mut self, constraint: SizeConstraint) -> Size {
        let total = self.total_outside_extent();
        let size = match self.config.position {
            AxisPosition::Bottom | AxisPosition::Top => Size::new(
                constraint.max_width,
                total.clamp(constraint.min_height, constraint.max_height),
            ),
            AxisPosition::Left | AxisPosition::Right => Size::new(
                total.clamp(constraint.min_width, constraint.max_width),
                constraint.max_height,
            ),
        };
        self.result = Some(LayoutResult::new(size));
        size
    }

    fn arrange(&mut self, bounds: Rect) {
        if let Some(ref mut result) = self.result {
            result.bounds = bounds;
        }
    }

    fn layout_result(&self) -> Option<&LayoutResult> {
        self.result.as_ref()
    }

    fn axis_position(&self) -> Option<AxisPosition> {
        Some(self.config.position)
    }
}

/// 网格布局元素
pub struct GridLayout {
    result: Option<LayoutResult>,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl GridLayout {
    pub fn new() -> Self {
        Self { result: None }
    }
}

impl Layoutable for GridLayout {
    fn measure(&mut self, constraint: SizeConstraint) -> Size {
        // 网格占据所有可用空间
        let size = Size::new(constraint.max_width, constraint.max_height);
        self.result = Some(LayoutResult::new(size));
        size
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
