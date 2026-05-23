use std::collections::HashMap;

use vello_cpu::kurbo::Rect;

use crate::model::TextStyle;
use crate::option::ChartOption;
use crate::visual::{Color, VisualElement};

/// GridPlanner 的输出：一个 subplot 的完整分配信息
#[derive(Debug, Clone)]
pub struct SubplotSpec {
    pub id: usize,
    pub bounds: Rect,
    pub series_indices: Vec<usize>,
    pub x_axis_indices: Vec<usize>,
    pub y_axis_indices: Vec<usize>,
}

/// AxisBindingResolver 的输出：单个轴实例的解析结果
#[derive(Debug, Clone)]
pub struct ResolvedAxisRange {
    pub axis_index: usize,
    pub min: f64,
    pub max: f64,
    pub is_user_defined: bool,
    pub tick_count_hint: Option<usize>,
}

/// AxisBindingResolver 的输出：所有轴的解析结果集合
#[derive(Debug, Clone)]
pub struct ResolvedAxisRanges {
    pub ranges: Vec<ResolvedAxisRange>,
}

/// ColorAssigner 的输出：颜色上下文
#[derive(Debug, Clone)]
pub struct ColorContext {
    pub palette: Vec<Color>,
    pub background: Color,
    pub series_colors: Vec<Color>,
    pub axis_line_color: Color,
    pub axis_label_color: Color,
    pub grid_line_color: Color,
}

impl Default for ColorContext {
    fn default() -> Self {
        Self {
            palette: Vec::new(),
            background: Color::new(255, 255, 255),
            series_colors: Vec::new(),
            axis_line_color: Color::new(200, 200, 200),
            axis_label_color: Color::new(50, 50, 50),
            grid_line_color: Color::new(230, 230, 230),
        }
    }
}

/// DataProcessor 的输入
pub struct DataProcessorInput<'a> {
    pub spec: &'a SubplotSpec,
    pub option: &'a ChartOption,
    pub colors: &'a ColorContext,
    pub axis_ranges: &'a ResolvedAxisRanges,
    pub text_measurer: &'a mut TextMeasurer,
}

/// DataProcessor 的输出
#[derive(Debug, Clone)]
pub struct SubplotVisualData {
    pub series_elements: Vec<VisualElement>,
    pub axis_elements: Vec<VisualElement>,
    pub grid_lines: Vec<VisualElement>,
}

/// 文本测量缓存
#[derive(Debug, Clone)]
pub struct TextMeasurer {
    cache: HashMap<String, (f64, f64)>,
}

impl TextMeasurer {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// 测量指定文本在给定字体样式下的宽度和高度
    pub fn measure(&mut self, text: &str, style: &TextStyle) -> (f64, f64) {
        let key = format!("{}|{}|0", text, style.font_size);
        if let Some(cached) = self.cache.get(&key) {
            return *cached;
        }
        // 简单估算：每个字符约 font_size * 0.6 宽，行高约 font_size * 1.2
        let char_width = style.font_size * 0.6;
        let width = text.len() as f64 * char_width;
        let height = style.font_size * 1.2;
        let result = (width, height);
        self.cache.insert(key, result);
        result
    }
}

impl Default for TextMeasurer {
    fn default() -> Self {
        Self::new()
    }
}