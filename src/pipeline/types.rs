use vello_cpu::kurbo::Rect;

use crate::{
    option::{AxisPosition, ChartOption},
    text::create_text_layout,
    visual::{Color, TextStyle, VisualElement},
};

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
    pub position: AxisPosition,
    pub min: f64,
    pub max: f64,
    pub is_user_defined: bool,
    pub tick_count_hint: Option<usize>,
}

impl ResolvedAxisRange {
    pub fn is_y_axis(&self) -> bool {
        matches!(self.position, AxisPosition::Left | AxisPosition::Right)
    }
}

/// AxisBindingResolver 的输出：所有轴的解析结果集合
#[derive(Debug, Clone)]
pub struct ResolvedAxisRanges {
    pub ranges: Vec<ResolvedAxisRange>,
}

impl ResolvedAxisRanges {
    pub fn get_x_range(&self, axis_index: usize) -> Option<&ResolvedAxisRange> {
        self.ranges
            .iter()
            .find(|r| !r.is_y_axis() && r.axis_index == axis_index)
    }

    pub fn get_y_range(&self, axis_index: usize) -> Option<&ResolvedAxisRange> {
        self.ranges
            .iter()
            .find(|r| r.is_y_axis() && r.axis_index == axis_index)
    }
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
    // 新增颜色字段
    pub border_color: Color,         // 边框/描边颜色
    pub text_color: Color,           // 主要文字颜色
    pub text_secondary_color: Color, // 次要文字颜色
    pub up_color: Color,             // 涨/正值颜色（K线图等）
    pub down_color: Color,           // 跌/负值颜色（K线图等）
    pub table_header_bg: Color,      // 表格表头背景
    pub table_row_even_bg: Color,    // 表格偶数行背景
    pub table_row_odd_bg: Color,     // 表格奇数行背景
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
            border_color: Color::new(255, 255, 255),
            text_color: Color::new(51, 51, 51),
            text_secondary_color: Color::new(102, 102, 102),
            up_color: Color::new(234, 85, 67),
            down_color: Color::new(80, 170, 94),
            table_header_bg: Color::new(220, 220, 220),
            table_row_even_bg: Color::new(248, 248, 248),
            table_row_odd_bg: Color::new(255, 255, 255),
        }
    }
}

impl ColorContext {
    /// 获取指定索引的系列颜色，支持回退到 palette
    pub fn get_series_color(&self, index: usize) -> Color {
        self.series_colors
            .get(index)
            .copied()
            .or_else(|| self.palette.get(index).copied())
            .unwrap_or_else(|| {
                // 回退到默认调色板
                let default_colors = [
                    Color::new(80, 112, 221), // 蓝色
                    Color::new(182, 214, 52), // 绿色
                    Color::new(234, 85, 67),  // 红色
                    Color::new(255, 193, 7),  // 黄色
                    Color::new(156, 39, 176), // 紫色
                    Color::new(0, 188, 212),  // 青色
                    Color::new(255, 87, 34),  // 橙色
                    Color::new(96, 125, 139), // 蓝灰色
                ];
                default_colors
                    .get(index % default_colors.len())
                    .copied()
                    .unwrap_or(Color::new(80, 112, 221))
            })
    }

    /// 获取数据点颜色（用于饼图、散点等按数据点着色的图表）
    pub fn get_data_color(&self, index: usize) -> Color {
        self.palette
            .get(index)
            .copied()
            .unwrap_or_else(|| self.get_series_color(index))
    }

    /// 获取默认颜色（第一个系列颜色，或第一个调色板颜色）
    pub fn get_default_color(&self) -> Color {
        self.series_colors
            .first()
            .copied()
            .or_else(|| self.palette.first().copied())
            .unwrap_or(Color::new(80, 112, 221))
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

/// 文本测量工具
///
/// 使用 parley 进行真实的文本排版和测量。
/// 注意：LayoutContext 内部已有缓存机制，无需额外缓存。
#[derive(Debug, Clone, Default)]
pub struct TextMeasurer;

impl TextMeasurer {
    pub fn new() -> Self {
        Self
    }

    /// 测量指定文本在给定字体样式下的宽度和高度
    ///
    /// 使用 parley 进行真实的文本排版，而非简单估算。
    pub fn measure(&mut self, text: &str, style: &TextStyle) -> (f64, f64) {
        let layout = create_text_layout(text, style, None);
        (layout.width() as f64, layout.height() as f64)
    }

    /// 测量文本，支持最大宽度限制（自动换行）
    ///
    /// LayoutContext 内部会缓存布局结果。
    pub fn measure_with_max_width(
        &mut self,
        text: &str,
        style: &TextStyle,
        max_width: f64,
    ) -> (f64, f64) {
        let layout = create_text_layout(text, style, Some(max_width));
        (layout.width() as f64, layout.height() as f64)
    }
}
