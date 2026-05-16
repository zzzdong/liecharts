use std::cell::RefCell;
use parley::style::{FontFamily, StyleProperty};
use parley::{FontContext, LayoutContext, Alignment, AlignmentOptions};
use vello_cpu::color::{AlphaColor, Srgb};
use crate::model;
use crate::visual::{TextAlign, TextBaseline, Color};

/// 文本布局包装类型
pub type TextLayout = parley::Layout<TextColor>;

thread_local! {
    /// 全局字体上下文 - 线程本地存储
    pub static FONT_CONTEXT: RefCell<FontContext> = RefCell::new(FontContext::default());
    /// 全局布局上下文 - 线程本地存储
    pub static LAYOUT_CONTEXT: RefCell<LayoutContext<TextColor>> = RefCell::new(LayoutContext::default());
}

/// 访问字体上下文的便捷函数
pub fn with_font_context<R, F: FnOnce(&mut FontContext) -> R>(f: F) -> R {
    FONT_CONTEXT.with(|cx| f(&mut cx.borrow_mut()))
}

/// 访问布局上下文的便捷函数
pub fn with_layout_context<R, F: FnOnce(&mut LayoutContext<TextColor>) -> R>(f: F) -> R {
    LAYOUT_CONTEXT.with(|cx| f(&mut cx.borrow_mut()))
}

/// 同时访问两个上下文的便捷函数
pub fn with_text_contexts<R, F: FnOnce(&mut FontContext, &mut LayoutContext<TextColor>) -> R>(f: F) -> R {
    FONT_CONTEXT.with(|font_cx| {
        LAYOUT_CONTEXT.with(|layout_cx| {
            f(&mut font_cx.borrow_mut(), &mut layout_cx.borrow_mut())
        })
    })
}

/// 文本颜色包装器
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextColor(pub AlphaColor<Srgb>);

impl TextColor {
    pub const BLACK: Self = Self(AlphaColor::BLACK);
    pub const WHITE: Self = Self(AlphaColor::WHITE);
    pub const RED: Self = Self(AlphaColor::from_rgb8(255, 0, 0));
    pub const GREEN: Self = Self(AlphaColor::from_rgb8(0, 128, 0));
    pub const BLUE: Self = Self(AlphaColor::from_rgb8(0, 0, 255));

    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(AlphaColor::from_rgba8(r, g, b, a))
    }

    pub fn inner(&self) -> AlphaColor<Srgb> {
        self.0
    }
}

impl Default for TextColor {
    fn default() -> Self {
        Self(AlphaColor::BLACK)
    }
}

/// 将 Shape 的 Color 转换为 TextColor
fn color_to_text_color(color: &Color) -> TextColor {
    TextColor::from_rgba8(color.r, color.g, color.b, color.a)
}

/// 创建文本布局
/// 
/// 使用 parley 以 **左对齐** 排版文本，返回布局以获取自然宽度/高度。
/// 组件的对齐（居中、右对齐等）应在拿到 layout 尺寸后手动计算位置偏移。
pub fn create_text_layout(
    text: &str,
    font_config: &model::TextStyle,
    max_width: Option<f64>,
) -> TextLayout {
    with_text_contexts(|font_cx, layout_cx| {
        create_text_layout_with_contexts(text, font_config, max_width, font_cx, layout_cx)
    })
}

/// 使用指定的上下文创建文本布局
pub fn create_text_layout_with_contexts(
    text: &str,
    style: &model::TextStyle,
    max_width: Option<f64>,
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext<TextColor>,
) -> TextLayout {
    // 创建布局构建器
    let mut builder = layout_cx.ranged_builder(font_cx, text, 1.0, true);

    // 应用样式
    let font_stack = FontFamily::named(&style.font_family);
    builder.push_default(StyleProperty::FontFamily(font_stack));
    builder.push_default(StyleProperty::FontSize(style.font_size as f32));
    builder.push_default(StyleProperty::Brush(color_to_text_color(&style.color)));

    // 构建布局
    let mut layout = builder.build(text);

    // 断行
    layout.break_all_lines(max_width.map(|w| w as f32));

    // 始终左对齐：parley 不做居中/右对齐，组件的对齐由 compute_text_offset 或手动计算实现
    layout.align(Alignment::Start, AlignmentOptions::default());

    layout
}

/// 创建双行文本布局（标题 + 副标题，可配置不同字号与字体）
///
/// 使用 parley 的 RangedBuilder 在同一布局中应用不同样式，
/// 通过 `\n` 分隔双行，行间距由 parley 精确计算。
pub fn create_two_line_layout(
    line1: &str,
    line1_font_size: f64,
    line1_font_family: &str,
    line2: Option<&str>,
    line2_font_size: f64,
    line2_font_family: Option<&str>,
) -> TextLayout {
    with_text_contexts(|font_cx, layout_cx| {
        let combined = if let Some(l2) = line2 {
            format!("{}\n{}", line1, l2)
        } else {
            line1.to_string()
        };

        let mut builder = layout_cx.ranged_builder(font_cx, &combined, 1.0, true);

        let font_stack = FontFamily::named(line1_font_family);
        builder.push_default(StyleProperty::FontFamily(font_stack));
        builder.push_default(StyleProperty::FontSize(line1_font_size as f32));

        if let Some(l2) = line2 {
            let sub_start = line1.len() + 1; // +1 跳过 '\n'
            let sub_end = sub_start + l2.len();

            // 如果副标题字体不同，单独设置
            if let Some(sub_family) = line2_font_family.filter(|f| *f != line1_font_family) {
                builder.push(
                    StyleProperty::FontFamily(FontFamily::named(sub_family)),
                    sub_start..sub_end,
                );
            }
            builder.push(
                StyleProperty::FontSize(line2_font_size as f32),
                sub_start..sub_end,
            );
        }

        let mut layout = builder.build(&combined);
        layout.break_all_lines(None);
        layout
    })
}

/// 从锚点到文本块左上角的偏移量
///
/// 根据期望的对齐/基线方式，计算从锚点坐标到文本块左上角的偏移。
/// 组件使用范式：
///
/// ```ignore
/// let layout = create_text_layout(text, &font, max_width);
/// let (x_off, y_off) = compute_text_offset(&layout, align, baseline);
/// let position = Point::new(anchor.x + x_off, anchor.y + y_off);
///
/// // TextRun 始终用 Left/Top（position 已是左上角）
/// VisualElement::TextRun {
///     position,
///     align: TextAlign::Left,
///     baseline: TextBaseline::Top,
///     ...
/// }
/// ```
///
/// 例如：
/// - 锚点=单元格中心，align=Center → x_off = -width/2，position = 左边缘
/// - 锚点=文本中心，baseline=Middle → y_off = -height/2，position = 上边缘
pub fn compute_text_offset(
    layout: &TextLayout,
    align: TextAlign,
    baseline: TextBaseline,
) -> (f64, f64) {
    let layout_width = layout.width() as f64;
    let layout_height = layout.height() as f64;

    let x_offset = match align {
        TextAlign::Left => 0.0,
        TextAlign::Center => -layout_width / 2.0,
        TextAlign::Right => -layout_width,
    };

    let y_offset = match baseline {
        TextBaseline::Top => 0.0,
        TextBaseline::Middle => -layout_height / 2.0,
        TextBaseline::Bottom => -layout_height,
        TextBaseline::Alphabetic => {
            // 对于基线对齐，使用第一行的 ascent
            let first_line = layout.lines().next();
            if let Some(line) = first_line {
                let line_metrics = line.metrics();
                -line_metrics.ascent as f64
            } else {
                -layout_height * 0.8
            }
        }
    };

    (x_offset, y_offset)
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
        let style = model::TextStyle {
            font_size,
            font_family: font_family.to_string(),
            color: *color,
            font_weight: crate::option::FontWeight::Named(crate::option::FontWeightNamed::Normal),
            ..Default::default()
        };
        let layout = create_text_layout(text, &style, max_width);
        (layout.width() as f64, layout.height() as f64)
    }
}

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}
