use lievisual::text::{RichSpan, TextAlign, TextBaseline, TextStyle, measure_text as lv_measure_text};

/// 默认字体栈：优先使用 CJK 字体，再回退到通用 sans-serif。
///
/// 用于未显式指定字体时的排版回退，避免 ASCII 数字/标点被过宽字体撑开。
pub const DEFAULT_FONT_STACK: &str = "Noto Sans CJK SC, sans-serif";

use crate::error::ChartError;
use lievisual::Color;

/// 文本布局包装类型（委托 lievisual）。
pub type TextLayout = lievisual::text::TextLayout;

/// 字体来源（委托 lievisual）。
pub use lievisual::text::FontSource;

/// 注册自定义字体到全局字体上下文。
///
/// 加载后的字体可以通过 `font_family` 名称在图表的文本样式中使用。
///
/// # 示例
///
/// ```no_run
/// // 从内存加载（例如从 CDN 下载的字节）
/// # let font_bytes = vec![0u8; 1024];
/// liecharts::text::register_font(liecharts::text::FontSource::Memory(font_bytes), Some("MyFont")).unwrap();
/// ```
pub fn register_font(
    source: FontSource,
    family_name_override: Option<&str>,
) -> crate::error::Result<()> {
    lievisual::text::register_font(source, family_name_override).map_err(ChartError::FontLoadError)
}

/// 解析用户配置的 `font_family`：空或 `sans-serif` 时回退到 [`DEFAULT_FONT_STACK`]。
fn resolve_font_family(font_family: &str) -> String {
    let trimmed = font_family.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("sans-serif") {
        DEFAULT_FONT_STACK.to_string()
    } else {
        trimmed.to_string()
    }
}

/// 把 liecharts 的 `TextStyle` 转成 lievisual 的 `TextStyle`，并解析字体栈。
fn to_lv_style(style: &TextStyle) -> lievisual::text::TextStyle {
    let mut s = style.clone();
    s.font_family = resolve_font_family(&style.font_family);
    s
}

/// 创建文本布局（左对齐排版，返回布局以获取自然宽度/高度）。
///
/// 组件的对齐（居中、右对齐等）应在拿到 layout 尺寸后手动计算位置偏移。
pub fn create_text_layout(
    text: &str,
    style: &TextStyle,
    max_width: Option<f64>,
) -> TextLayout {
    let lv_style = to_lv_style(style);
    let measure = lv_measure_text(&[RichSpan::new(text.to_string(), lv_style)], max_width);
    (*measure.layout).clone()
}

/// 从锚点到文本块左上角的偏移量。
///
/// 根据期望的对齐/基线方式，计算从锚点坐标到文本块左上角的偏移。
pub fn compute_text_offset(
    layout: &TextLayout,
    align: TextAlign,
    baseline: TextBaseline,
) -> (f64, f64) {
    lievisual::text::compute_text_offset(layout, align, baseline)
}

/// 将多段不同样式的文本合并在一个 TextLayout 中。
///
/// 每段文本可以有自己的 TextStyle（字体、字号、颜色）。所有文本按顺序直接拼接。
/// 需要换行时，请在文本段中自行包含 `\n`。
///
/// # 参数
/// - `texts`: 文本段列表，每项为 `(文本内容, 文本样式)`。至少包含一段。
/// - `max_width`: 最大行宽，`None` 表示不断行。
/// - `align`: 多行对齐方式。`Left`、`Center` 或 `Right`。
pub fn layout_text(
    texts: &[(&str, &TextStyle)],
    max_width: Option<f64>,
    align: TextAlign,
) -> TextLayout {
    if texts.is_empty() {
        return create_text_layout("", &TextStyle::new(Color::BLACK, 12.0, "sans-serif"), max_width);
    }
    let spans: Vec<RichSpan> = texts
        .iter()
        .map(|(t, s)| RichSpan::new(t.to_string(), to_lv_style(s)))
        .collect();
    // 用首段 align 对齐整体
    let measure = lv_measure_text(&spans, max_width);
    let _ = align; // 多行对齐已在排版时按 spans[0].style.align 应用
    (*measure.layout).clone()
}

/// 文本布局引擎
pub struct TextEngine;

impl TextEngine {
    pub fn new() -> Self {
        Self
    }

    /// 计算文本布局尺寸（宽度和高度）
    pub fn measure_text(
        text: &str,
        font_size: f64,
        font_family: &str,
        color: &Color,
        max_width: Option<f64>,
    ) -> (f64, f64) {
        let style = TextStyle::new(*color, font_size, font_family);
        let layout = create_text_layout(text, &style, max_width);
        (layout.width, layout.height)
    }
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}
