//! 标题纵向定位回归测试。
//!
//! 回归背景：标题文本样式此前沿用 `TextStyle::new` 的默认基线
//! （`TextBaseline::Alphabetic`），而渲染后端把 `position.y` 解释为**基线**。
//! 于是 `TITLE_TOP_MARGIN`（15px，对齐 ECharts v6 `title.top`）被上标高
//! （ascent）吃掉，标题墨迹顶部实际落在画布顶边附近（≈1px），视觉上
//! "贴着图片顶边、没有 margin"。
//!
//! 修复后标题（含副标题）以 `TextBaseline::Top` 为锚点，`position.y` 即
//! 文本块顶边，15px 留白才真正生效。

mod common;

use common::*;
use liecharts::api::{Axis, Chart, Line, Title};
use liecharts::compute_text_offset;
use lievisual::scene::Element;
use lievisual::text::TextBaseline;

/// 标题默认顶部留白（与 `title.rs` 的 `TITLE_TOP_MARGIN` 同源，
/// 对齐 ECharts v6 `title.top` = `tokens.size.m` = 15px）。
const TITLE_TOP_MARGIN: f64 = 15.0;

/// 构建一个带标题的最小折线图。
fn title_chart() -> Vec<liecharts::SceneNode> {
    Chart::new(800, 500)
        .title(Title::new("标题"))
        .data(liecharts::dataframe!(
            "cat" => ["a", "b", "c"],
            "v" => [1.0, 2.0, 3.0],
        ))
        .x_axis(Axis::category().data(["a", "b", "c"]))
        .y_axis(Axis::value())
        .add_line(Line::new().name("v").x("cat").y("v"))
        .build()
        .expect("应可构建")
}

/// 标题（含副标题）必须以文本块顶边为锚点，且默认顶部留白真实生效。
#[test]
fn title_top_margin_is_not_eaten_by_baseline() {
    let nodes = title_chart();
    let mut all = Vec::new();
    flatten(&nodes, &mut all);

    let (position, style, layout) = all
        .iter()
        .find_map(|(e, _)| match e {
            Element::Text {
                spans,
                position,
                style,
                layout,
            } => {
                let content: String = spans.iter().map(|s| s.text.clone()).collect();
                (content == "标题").then(|| (*position, style.clone(), layout.clone()))
            }
            _ => None,
        })
        .expect("应渲染标题「标题」");

    // 关键回归点：标题必须以文本块顶边为锚点，而非默认的字母基线。
    assert_eq!(
        style.baseline,
        TextBaseline::Top,
        "标题应以 TextBaseline::Top 为锚点，否则 position.y 会被当成基线"
    );
    assert!(
        (position.y - TITLE_TOP_MARGIN).abs() < 0.5,
        "标题顶边应位于 title.top 默认 {TITLE_TOP_MARGIN}px 处，实际 {}",
        position.y
    );

    // 用渲染后端同款语义（compute_text_offset）把锚点换算成文本块顶边：
    // Top 基线时纵向偏移为 0（块顶边 == position.y）；若退回 Alphabetic，
    // 块顶边会被上标高顶到 ~1px。
    let layout = layout.expect("标题节点应携带预排版结果");
    let (_, dy) = compute_text_offset(&layout, style.align, style.baseline);
    let block_top = position.y + dy;
    assert!(
        block_top >= TITLE_TOP_MARGIN - 1.0,
        "标题块顶边距画布顶部过小（{block_top}px），疑似基线语义回退导致贴顶"
    );
}
