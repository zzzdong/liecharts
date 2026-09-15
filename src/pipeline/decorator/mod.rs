//! Decorator 阶段：渲染标题、图例、轴名称等装饰元素
//!
//! 职责：
//! - 接收 `&ChartSpec` + `&ColorContext` + `&Theme` + 布局信息
//! - 产生 `Vec<SceneNode>`
//! - 不修改任何管线状态，纯函数式渲染

mod axis_name;
mod legend;
mod title;

pub use axis_name::render_axis_name;
use legend::LegendVerticalZone;
pub use legend::measure_legend_layout;
pub use legend::render_legend;
use lievisual::text::measure_text;
pub use title::render_title;
use title::{TITLE_ITEM_GAP, TITLE_TOP_MARGIN};

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
    let (title_elems, title_height) = render_title(spec, width, height, theme, colors);
    all_elements.extend(title_elems);

    // 2. 图例（贴画布底部，与标题高度无关）
    all_elements.extend(render_legend(spec, width, height, colors, theme));

    // 3. 轴名称
    all_elements.extend(render_axis_name(spec, width, height, specs, colors, theme));

    (all_elements, title_height)
}

/// 估计**标题**占用的顶部空间高度（像素）
///
/// 在 GridPlanner 之前调用，确保 subplot 的 top margin 足够容纳标题，
/// 避免重叠。v6 起图例默认贴画布底部，不再占用头部空间（见
/// [`estimate_footer_height`]）。
pub fn estimate_header_height(spec: &ChartSpec, theme: &Theme) -> f64 {
    let mut height = 0.0;

    // 标题占用（顶部留白与主/副标题间距均与 `render_title` 同源）
    if let Some(title) = &spec.title {
        let theme_title_style = theme.get_title_text_style();
        let subtitle_style = theme.get_subtitle_text_style();

        // v6 `title.top` = tokens.size.m(15px)
        height += TITLE_TOP_MARGIN;

        // 主标题高度（基于 font_size + 行距）
        if title.text.is_some() {
            height += theme_title_style.font_size * 1.4;
        }

        // 副标题高度（含 v6 `title.itemGap` 默认 10px 的间距）
        if title.subtext.is_some() {
            height += title.item_gap.unwrap_or(TITLE_ITEM_GAP) + subtitle_style.font_size * 1.4;
        }
    }

    // 图例被显式锚定在顶部时（`legend.top`）也占用头部空间，取两者下缘的较大值
    if let Some(legend) = &spec.legend
        && legend::legend_vertical_zone(legend) == LegendVerticalZone::Top
        && let Some(layout) = legend::measure_legend_layout(spec, spec.width, theme)
        && layout.rows > 0
    {
        let block_top = legend::legend_block_top(legend, layout.total_height, spec.height as f64);
        height = height.max(block_top + layout.total_height);
    }

    // 最小值为 0，空标题时返回 0
    height
}

/// 图例块与绘图区之间的呼吸间距（图例贴底时计入底部预留）
const LEGEND_PLOT_GAP: f64 = 8.0;

/// 估计**图例**占用的底部空间高度（像素）
///
/// v6 起 `legend` 默认位于画布底部居中（`bottom: tokens.size.m`），因此它消耗的
/// 是底部空间而非头部空间：这里返回「图例块高度 + 其下留白 + 与绘图区间距」。
/// 单行图例时该值小于 v6 的 `grid.bottom`（80px）默认值，两者取 max 后默认输出
/// 与 v6 一致；多行（换行）图例才会超过 80px，从而避免与绘图区重叠。
///
/// 图例被显式配置在顶部时改由 [`estimate_header_height`] 预留；配置在画布中部
/// 时不做预留（与 ECharts 一致：中部图例允许与绘图区重叠）。
pub fn estimate_footer_height(spec: &ChartSpec, theme: &Theme, width: f64) -> f64 {
    let Some(legend) = &spec.legend else {
        return 0.0;
    };
    if legend::legend_vertical_zone(legend) != LegendVerticalZone::Bottom {
        return 0.0;
    }
    match legend::measure_legend_layout(spec, width as u32, theme) {
        Some(layout) if layout.rows > 0 => {
            let canvas_h = spec.height as f64;
            let block_top = legend::legend_block_top(legend, layout.total_height, canvas_h);
            let below = (canvas_h - block_top - layout.total_height).max(0.0);
            layout.total_height + below + LEGEND_PLOT_GAP
        }
        _ => 0.0,
    }
}
