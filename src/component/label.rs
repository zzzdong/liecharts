use crate::model;
use crate::pipeline::LabelConfig;
use crate::text::{compute_text_offset, create_text_layout};
use crate::visual::{Color, StrokeStyle, TextAlign, TextBaseline, VisualElement};
use vello_cpu::kurbo::Point;

/// 通用标签组件，用于柱状图、折线图、散点图等
///
/// 特点：
/// - 简单的文本标签，显示在数据点旁边
/// - 支持多种位置（上、下、左、右、中心）
/// - 无需引导线
pub struct LabelComponent {
    /// 标签文本
    text: String,
    /// 标签位置（数据点坐标）
    position: Point,
    /// 字体配置（来自 model::TextStyle）
    font_config: model::TextStyle,
    /// 对齐方式
    align: TextAlign,
    /// 基线方式
    baseline: TextBaseline,
    /// 位置偏移（相对于数据点）
    offset: (f64, f64),
}

impl LabelComponent {
    /// 创建新的标签组件
    pub fn new(text: impl Into<String>, position: Point) -> Self {
        Self {
            text: text.into(),
            position,
            font_config: model::TextStyle {
                font_size: 12.0,
                font_family: "sans-serif".to_string(),
                color: Color::new(51, 51, 51),
                font_weight: crate::option::FontWeight::Named(
                    crate::option::FontWeightNamed::Normal,
                ),
                ..Default::default()
            },
            align: TextAlign::Center,
            baseline: TextBaseline::Middle,
            offset: (0.0, -15.0),
        }
    }

    /// 从 LabelConfig 创建标签组件
    pub fn with_config(text: impl Into<String>, position: Point, config: &LabelConfig) -> Self {
        let mut component = Self::new(text, position);
        component.font_config.font_size = config.font_size;
        component.font_config.font_family = config.font_family.clone();
        component.font_config.color = config.color;

        match config.position {
            crate::pipeline::LabelPosition::Top => {
                component.offset = (0.0, -10.0);
                component.align = TextAlign::Center;
                component.baseline = TextBaseline::Bottom;
            }
            crate::pipeline::LabelPosition::Bottom => {
                component.offset = (0.0, 10.0);
                component.align = TextAlign::Center;
                component.baseline = TextBaseline::Top;
            }
            crate::pipeline::LabelPosition::Left => {
                component.offset = (-10.0, 0.0);
                component.align = TextAlign::Right;
                component.baseline = TextBaseline::Middle;
            }
            crate::pipeline::LabelPosition::Right => {
                component.offset = (10.0, 0.0);
                component.align = TextAlign::Left;
                component.baseline = TextBaseline::Middle;
            }
            crate::pipeline::LabelPosition::Inside | crate::pipeline::LabelPosition::Center => {
                component.offset = (0.0, 0.0);
                component.align = TextAlign::Center;
                component.baseline = TextBaseline::Middle;
            }
            crate::pipeline::LabelPosition::Outside => {
                component.offset = (0.0, -10.0);
                component.align = TextAlign::Center;
                component.baseline = TextBaseline::Bottom;
            }
        }

        component
    }

    /// 设置字体配置
    pub fn with_font_config(mut self, config: model::TextStyle) -> Self {
        self.font_config = config;
        self
    }

    /// 设置对齐
    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// 设置基线
    pub fn with_baseline(mut self, baseline: TextBaseline) -> Self {
        self.baseline = baseline;
        self
    }

    /// 设置位置偏移
    pub fn with_offset(mut self, x: f64, y: f64) -> Self {
        self.offset = (x, y);
        self
    }

    /// 构建标签视觉元素
    pub fn build(&self) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        let final_x = self.position.x + self.offset.0;
        let final_y = self.position.y + self.offset.1;

        let layout = create_text_layout(&self.text, &self.font_config, None);
        let (x_offset, y_offset) = compute_text_offset(&layout, self.align, self.baseline);

        elements.push(VisualElement::TextRun {
            text: self.text.clone(),
            position: Point::new(final_x + x_offset, final_y + y_offset),
            style: crate::model::TextStyle {
                color: self.font_config.color,
                font_size: self.font_config.font_size,
                font_family: self.font_config.font_family.clone(),
                font_weight: self.font_config.font_weight,
                font_style: self.font_config.font_style,
                align: TextAlign::Left,
                vertical_align: TextBaseline::Top,
            },
            rotation: 0.0,
            max_width: None,
            layout: Some(layout),
        });

        elements
    }
}

/// 饼图引导线标签组件，专门用于饼图
pub struct PieLeaderLineLabel {
    text: String,
    center: Point,
    outer_radius: f64,
    mid_angle: f64,
    is_right_side: bool,
    text_width: f64,
    first_segment_length: f64,
    second_segment_length: f64,
    text_margin: f64,
    line_style: StrokeStyle,
    font_config: model::TextStyle,
    align: TextAlign,
    baseline: TextBaseline,
}

impl PieLeaderLineLabel {
    pub fn new(
        text: impl Into<String>,
        center: Point,
        outer_radius: f64,
        mid_angle: f64,
        is_right_side: bool,
        text_width: f64,
    ) -> Self {
        let text_align = if is_right_side {
            TextAlign::Left
        } else {
            TextAlign::Right
        };
        Self {
            text: text.into(),
            center,
            outer_radius,
            mid_angle,
            is_right_side,
            text_width,
            first_segment_length: 15.0,
            second_segment_length: 40.0,
            text_margin: 15.0,
            line_style: StrokeStyle {
                color: Color::new(160, 160, 160),
                width: 1.0,
            },
            font_config: model::TextStyle {
                font_size: 12.0,
                font_family: "sans-serif".to_string(),
                color: Color::new(51, 51, 51),
                font_weight: crate::option::FontWeight::Named(
                    crate::option::FontWeightNamed::Normal,
                ),
                ..Default::default()
            },
            align: text_align,
            baseline: TextBaseline::Middle,
        }
    }

    pub fn with_first_segment_length(mut self, length: f64) -> Self {
        self.first_segment_length = length;
        self
    }

    pub fn with_second_segment_length(mut self, length: f64) -> Self {
        self.second_segment_length = length;
        self
    }

    pub fn with_text_margin(mut self, margin: f64) -> Self {
        self.text_margin = margin;
        self
    }

    pub fn with_line_style(mut self, style: StrokeStyle) -> Self {
        self.line_style = style;
        self
    }

    pub fn with_font_config(mut self, config: model::TextStyle) -> Self {
        self.font_config = config;
        self
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn with_label_config(mut self, config: &LabelConfig) -> Self {
        self.font_config.font_size = config.font_size;
        self.font_config.font_family = config.font_family.clone();
        self.font_config.color = config.color;
        self
    }

    fn compute_positions(&self) -> (Point, Point, Point, Point) {
        let edge_x = self.center.x + self.outer_radius * self.mid_angle.cos();
        let edge_y = self.center.y + self.outer_radius * self.mid_angle.sin();
        let edge_point = Point::new(edge_x, edge_y);

        let turn_x =
            self.center.x + (self.outer_radius + self.first_segment_length) * self.mid_angle.cos();
        let turn_y =
            self.center.y + (self.outer_radius + self.first_segment_length) * self.mid_angle.sin();
        let turn_point = Point::new(turn_x, turn_y);

        let text_edge_x = if self.is_right_side {
            turn_x + self.second_segment_length
        } else {
            turn_x - self.second_segment_length
        };
        let text_edge_point = Point::new(text_edge_x, turn_y);

        let final_text_x = if self.is_right_side {
            text_edge_x + self.text_margin * 0.5
        } else {
            text_edge_x - self.text_margin * 0.5 - self.text_width
        };
        let final_text_point = Point::new(final_text_x, turn_y);

        (edge_point, turn_point, text_edge_point, final_text_point)
    }

    pub fn build(&self) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        let (edge_point, turn_point, text_edge_point, final_text_point) = self.compute_positions();

        elements.push(VisualElement::Line {
            start: edge_point,
            end: turn_point,
            style: self.line_style.clone(),
        });

        elements.push(VisualElement::Line {
            start: turn_point,
            end: text_edge_point,
            style: self.line_style.clone(),
        });

        let layout = create_text_layout(&self.text, &self.font_config, None);
        // final_text_point 已经是左边缘，x 偏移永远为 0；y 用 baseline 计算垂直居中
        let (_, y_offset) = compute_text_offset(&layout, TextAlign::Left, self.baseline);

        elements.push(VisualElement::TextRun {
            text: self.text.clone(),
            position: Point::new(final_text_point.x, final_text_point.y + y_offset),
            style: crate::model::TextStyle {
                color: self.font_config.color,
                font_size: self.font_config.font_size,
                font_family: self.font_config.font_family.clone(),
                font_weight: self.font_config.font_weight,
                font_style: self.font_config.font_style,
                align: TextAlign::Left,
                vertical_align: TextBaseline::Top,
            },
            rotation: 0.0,
            max_width: None,
            layout: Some(layout),
        });

        elements
    }

    pub fn build_line_only(&self) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        let (edge_point, turn_point, text_edge_point, _) = self.compute_positions();

        elements.push(VisualElement::Line {
            start: edge_point,
            end: turn_point,
            style: self.line_style.clone(),
        });

        elements.push(VisualElement::Line {
            start: turn_point,
            end: text_edge_point,
            style: self.line_style.clone(),
        });

        elements
    }

    pub fn build_text_only(&self) -> Vec<VisualElement> {
        let (_, _, _, final_text_point) = self.compute_positions();

        let layout = create_text_layout(&self.text, &self.font_config, None);
        let (_, y_offset) = compute_text_offset(&layout, TextAlign::Left, self.baseline);

        vec![VisualElement::TextRun {
            text: self.text.clone(),
            position: Point::new(final_text_point.x, final_text_point.y + y_offset),
            style: crate::model::TextStyle {
                color: self.font_config.color,
                font_size: self.font_config.font_size,
                font_family: self.font_config.font_family.clone(),
                font_weight: self.font_config.font_weight,
                font_style: self.font_config.font_style,
                align: self.align,
                vertical_align: self.baseline,
            },
            rotation: 0.0,
            max_width: None,
            layout: Some(layout),
        }]
    }
}

/// 饼图引导线标签构建器，用于批量创建标签
pub struct PieLeaderLineLabelBuilder {
    labels: Vec<PieLeaderLineLabel>,
}

impl PieLeaderLineLabelBuilder {
    /// 创建新的标签构建器
    pub fn new() -> Self {
        Self { labels: Vec::new() }
    }

    /// 添加一个标签
    pub fn add_label(&mut self, label: PieLeaderLineLabel) -> &mut Self {
        self.labels.push(label);
        self
    }

    /// 构建所有标签
    pub fn build(&self) -> Vec<VisualElement> {
        self.labels.iter().flat_map(|label| label.build()).collect()
    }

    /// 清空所有标签
    pub fn clear(&mut self) {
        self.labels.clear();
    }

    /// 获取标签数量
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

impl Default for PieLeaderLineLabelBuilder {
    fn default() -> Self {
        Self::new()
    }
}
