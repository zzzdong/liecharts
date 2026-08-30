//! Decorator 阶段：渲染标题、图例、轴名称等装饰元素
//!
//! 职责：
//! - 接收 `&ChartSpec` + `&ColorContext` + `&Theme` + 布局信息
//! - 产生 `Vec<VisualElement>`
//! - 不修改任何管线状态，纯函数式渲染

mod axis_name;
mod legend;
mod title;

pub use axis_name::render_axis_name;
pub use legend::render_legend;
pub use title::render_title;

use crate::{
    pipeline::types::{ChartSpec, ColorContext, SubplotSpec},
    text::create_text_layout,
    theme::Theme,
    visual::VisualElement,
};

/// 计算文本布局（为所有未计算布局的 Text 执行真实文本排布）
///
/// 遍历所有 SceneNode，对 `layout: None` 的 `Element::Text` 调用 create_text_layout，
/// 并把文本块的纯文本写入 span。
///
/// 注意：`position` 保持「锚点」语义（canvas `fillText` 语义），水平对齐由
/// `style.align`、垂直对齐由 `style.baseline` 在渲染后端决定，这里**不做偏移烘焙**。
/// 若在此处把对齐偏移累加进 position，渲染后端会再次应用同样的偏移，导致
/// 右对齐/居中文本被平移两次（历史 bug：Y 轴标签左移一个文本宽、仪表盘
/// 中心数值偏离圆心）。
pub fn compute_text_layouts(elements: &mut [lievisual::scene::SceneNode]) {
    use lievisual::scene::Element;
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
            *layout = Some(std::sync::Arc::new(create_text_layout(
                &text,
                style,
                style.max_width,
            )));
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
) -> (Vec<VisualElement>, f64) {
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
/// 标题和图例，避免重叠。使用 theme 中的字体大小估算，不依赖文本测量。
pub fn estimate_header_height(spec: &ChartSpec, theme: &Theme) -> f64 {
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
    if let Some(legend) = &spec.legend
        && legend.show
    {
        if height > 0.0 {
            height += 16.0; // 标题和图例之间的间距
        }

        let legend_style = theme.get_legend_text_style();
        // 图例行高：symbol_size + 上下内边距
        let legend_height = legend_style.font_size * 1.4 + 16.0; // font + vertical padding
        height += legend_height;
    }

    // 最小值为 0，空标题/图例时返回 0
    height
}
