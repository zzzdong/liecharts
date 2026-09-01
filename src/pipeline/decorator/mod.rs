//! Decorator 阶段：渲染标题、图例、轴名称等装饰元素
//!
//! 职责：
//! - 接收 `&ChartSpec` + `&ColorContext` + `&Theme` + 布局信息
//! - 产生 `Vec<SceneNode>`
//! - 不修改任何管线状态，纯函数式渲染

mod axis_name;
mod legend;
mod title;

use legend::LegendLayout;
pub use axis_name::render_axis_name;
pub use legend::measure_legend_layout;
pub use legend::render_legend;
use lievisual::text::measure_text;
pub use title::render_title;

use crate::{
    SceneNode,
    pipeline::types::{ChartSpec, ColorContext, SubplotSpec},
    theme::{DEFAULT_FONT_STACK, Theme},
};

/// 计算文本布局（为所有未计算布局的 Text 执行真实文本排布）
///
/// 遍历所有 SceneNode，对 `layout: None` 的 `Element::Text` 调用 `lievisual::text::measure_text` 排版，
/// 并把文本块的纯文本写入 span。
///
/// 注意：`position` 保持「锚点」语义（canvas `fillText` 语义），水平对齐由
/// `style.align`、垂直对齐由 `style.baseline` 在渲染后端决定，这里**不做偏移烘焙**。
/// 若在此处把对齐偏移累加进 position，渲染后端会再次应用同样的偏移，导致
/// 右对齐/居中文本被平移两次（历史 bug：Y 轴标签左移一个文本宽、仪表盘
/// 中心数值偏离圆心）。
pub fn compute_text_layouts(elements: &mut [lievisual::scene::SceneNode]) {
    use lievisual::{scene::Element, text::RichSpan};
    for node in elements.iter_mut() {
        if let Element::Text {
            spans,
            style,
            layout,
            ..
        } = &mut node.element
            && layout.is_none()
        {
            // 拼接纯文本（单 span 最常见）
            let text: String = spans.iter().map(|s| s.text.clone()).collect();
            let mut lv_style = style.clone();
            if lv_style.font_family.trim().is_empty()
                || lv_style
                    .font_family
                    .trim()
                    .eq_ignore_ascii_case("sans-serif")
            {
                lv_style.font_family = DEFAULT_FONT_STACK.to_string();
            }
            *layout = Some(std::sync::Arc::new(
                (*measure_text(&[RichSpan::new(text, lv_style)], style.max_width).layout).clone(),
            ));
        }
    }
}

/// 渲染所有装饰元素（标题、图例、轴名称）
///
/// 按固定顺序渲染，确保 z-index 正确：
/// 标题 → 图例 → 轴名称
pub fn render_all_decorators(
    spec: &ChartSpec,
    width: u32,
    height: u32,
    specs: &[SubplotSpec],
    colors: &ColorContext,
    theme: &Theme,
) -> (Vec<SceneNode>, f64) {
    let mut all_elements = Vec::new();

    // 1. 标题
    let (title_elems, title_height) = render_title(spec, width, theme, colors);
    all_elements.extend(title_elems);

    // 2. 图例（依赖标题高度确定 Y 位置）
    all_elements.extend(render_legend(spec, width, colors, theme, title_height));

    // 3. 轴名称
    all_elements.extend(render_axis_name(spec, width, height, specs, colors, theme));

    (all_elements, title_height)
}

/// 估计标题和图例占用的顶部空间高度（像素）
///
/// 在 GridPlanner 之前调用，确保 subplot 的 top margin 足够容纳
/// 标题和图例，避免重叠。
///
/// P1 起图例高度按**真实换行行数**计算（与 `render_legend` 共用
/// `wrap_legend_rows` 分行逻辑与常量）：图例项总宽超出画布时换行，
/// 行数计入顶部预留，避免图例与绘图区重叠。宽度过小无法度量时
/// 按单行兜底。
pub fn estimate_header_height(spec: &ChartSpec, theme: &Theme, width: f64) -> f64 {
    let mut height = 0.0;

    // 标题占用
    if let Some(title) = &spec.title {
        let title_style = theme.get_title_text_style();
        let subtitle_style = theme.get_subtitle_text_style();

        // 标题顶部内边距 24px
        height += 24.0;

        // 主标题高度（基于 font_size + 行距）
        if title.text.is_some() {
            height += title_style.font_size * 1.4;
        }

        // 副标题高度
        if title.subtext.is_some() {
            height += subtitle_style.font_size * 1.4 + 2.0; // 2px 间距
        }
    }

    // 图例占用（在标题下方，有 16px 间距）
    let legend_layout = legend::measure_legend_layout(spec, width as u32, theme)
        .unwrap_or_else(|| {
            // 无图例或 auto 单行兜底：保持旧行高估算
            let legend_style = theme.get_legend_text_style();
            let has_legend = spec
                .legend
                .as_ref()
                .is_some_and(|l| l.show && !l.data.is_empty());
            if has_legend {
                LegendLayout {
                    rows: 1,
                    row_height: legend_style.font_size * 1.4 + 16.0,
                    total_height: legend_style.font_size * 1.4 + 16.0,
                }
            } else {
                LegendLayout {
                    rows: 0,
                    row_height: 0.0,
                    total_height: 0.0,
                }
            }
        });

    if legend_layout.rows > 0 {
        if height > 0.0 {
            height += 16.0; // 标题和图例之间的间距
        }
        height += legend_layout.total_height;
    }

    // 最小值为 0，空标题/图例时返回 0
    height
}
