//! line/bar 值标签（label.show / position / color / formatter）回归测试。
//!
//! 覆盖的语义约定：
//! - 默认 `position = Top` 表示**值端外侧**：正值柱在柱顶上方、负值柱在柱底下方；
//!   折线在数据点上方。历史实现按"柱高 > 25px"自动切换内/外，忽略 position 与负值。
//! - `Inside` 表示值端内侧（柱内），柱高不足容纳文字时自动回退柱外，避免溢出。
//! - 颜色未显式配置时跟随语义默认（柱外/折线 = 系列色，柱内 = 白字）；
//!   显式 `label_color` 必须生效。

use liecharts::Color;
use liecharts::api::*;
use liecharts::pipeline::types::ValueLabelPos;

/// 提取 SVG 中所有 `<text>` 元素的 (内容, 属性串)。
fn texts(svg: &str) -> Vec<(String, String)> {
    svg.split("<text")
        .skip(1)
        .filter_map(|seg| {
            let (attrs, rest) = seg.split_once('>')?;
            let content = rest.split("</text>").next()?;
            Some((content.trim().to_string(), attrs.to_string()))
        })
        .collect()
}

fn parse_attr(attrs: &str, key: &str) -> Option<f64> {
    attrs
        .split(&format!("{key}=\""))
        .nth(1)
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse().ok())
}

/// 取值标签所在 `<text>` 的完整属性串。
///
/// 同一文本可能同时是 Y 轴刻度标签（右对齐 `text-anchor="end"`），
/// 而 line/bar 值标签恒为居中 `text-anchor="middle"`，据此区分。
fn text_attrs(svg: &str, content: &str) -> Option<String> {
    texts(svg)
        .into_iter()
        .find(|(c, a)| c == content && a.contains("text-anchor=\"middle\""))
        .map(|(_, a)| a)
}

/// 取值标签的 `y` 属性值。
fn text_y(svg: &str, content: &str) -> Option<f64> {
    text_attrs(svg, content).and_then(|a| parse_attr(&a, "y"))
}

fn bar_chart(configure: impl FnOnce(Bar) -> Bar) -> String {
    let bar = configure(
        Bar::new()
            .data(dataframe!(
                "cat" => ["A", "B", "C"],
                "val" => [120.0, 60.0, 200.0],
            ))
            .name("销量")
            .x("cat")
            .y("val"),
    );
    Chart::new(800, 600).add_bar(bar).render_svg().unwrap()
}

fn line_chart(configure: impl FnOnce(Line) -> Line) -> String {
    let line = configure(
        Line::new()
            .data(dataframe!(
                "cat" => ["A", "B", "C"],
                "val" => [120.0, 60.0, 200.0],
            ))
            .name("销量")
            .x("cat")
            .y("val"),
    );
    Chart::new(800, 600).add_line(line).render_svg().unwrap()
}

/// 统计 SVG 中内容等于 `content` 的 `<text>` 数量。
///
/// 不能只判断"是否出现"：值 120/200 同时也可能是 Y 轴刻度标签文本，
/// 故用**数量差**来判定值标签是否被渲染。
fn count_texts(svg: &str, content: &str) -> usize {
    texts(svg)
        .iter()
        .filter(|(c, a)| c == content && a.contains("text-anchor=\"middle\""))
        .count()
}

#[test]
fn labels_hidden_by_default() {
    let off = bar_chart(|b| b);
    let on = bar_chart(|b| b.label_show(true));

    // 开启后每个值多出恰好 1 个文本（即值标签本身）
    for v in ["120", "60", "200"] {
        assert_eq!(
            count_texts(&on, v),
            count_texts(&off, v) + 1,
            "开启 label 应使 {v} 的文本数 +1（默认不渲染值标签）"
        );
    }
}

#[test]
fn bar_labels_shown_when_enabled() {
    let svg = bar_chart(|b| b.label_show(true));
    for v in ["120", "60", "200"] {
        assert!(
            texts(&svg).iter().any(|(c, _)| c == v),
            "开启 label 后应渲染值标签 {v}"
        );
    }
}

#[test]
fn line_labels_shown_when_enabled() {
    let svg = line_chart(|l| l.label_show(true));
    for v in ["120", "60", "200"] {
        assert!(
            texts(&svg).iter().any(|(c, _)| c == v),
            "折线开启 label 后应渲染值标签 {v}"
        );
    }
}

#[test]
fn bar_top_label_sits_outside_above_bar() {
    // 关键回归：高柱（200）在 Top 语义下也必须落在柱顶**外侧**，
    // 而非旧实现的"高柱自动塞进柱内"。
    let svg = bar_chart(|b| b.label_show(true).label_position(ValueLabelPos::Top));
    let tall = text_y(&svg, "200").expect("应存在 200 标签");
    let short = text_y(&svg, "60").expect("应存在 60 标签");

    // 值越大柱越高 → 柱顶 y 越小 → 外侧标签 y 也越小
    assert!(
        tall < short,
        "Top 标签应贴各自柱顶外侧：200 的 y({tall}) 应小于 60 的 y({short})"
    );
}

#[test]
fn bar_inside_label_is_below_top_label() {
    let top = bar_chart(|b| b.label_show(true).label_position(ValueLabelPos::Top));
    let inside = bar_chart(|b| b.label_show(true).label_position(ValueLabelPos::Inside));

    let y_top = text_y(&top, "200").expect("Top 应存在 200 标签");
    let y_inside = text_y(&inside, "200").expect("Inside 应存在 200 标签");

    // Inside 在柱顶内侧（更靠下），Top 在柱顶外侧（更靠上）
    assert!(
        y_inside > y_top,
        "Inside 标签({y_inside}) 应位于 Top 标签({y_top}) 下方（柱内 vs 柱外）"
    );
}

#[test]
fn line_bottom_label_is_below_top_label() {
    let top = line_chart(|l| l.label_show(true).label_position(ValueLabelPos::Top));
    let bottom = line_chart(|l| l.label_show(true).label_position(ValueLabelPos::Bottom));

    let y_top = text_y(&top, "120").expect("Top 应存在 120 标签");
    let y_bottom = text_y(&bottom, "120").expect("Bottom 应存在 120 标签");

    assert!(
        y_bottom > y_top,
        "Bottom 标签({y_bottom}) 应位于数据点下方，Top({y_top}) 位于上方"
    );
}

#[test]
fn line_inside_falls_back_to_top() {
    // 折线无"柱内"概念，Inside 必须退化为 Top（与 ECharts 降级行为一致）
    let top = line_chart(|l| l.label_show(true).label_position(ValueLabelPos::Top));
    let inside = line_chart(|l| l.label_show(true).label_position(ValueLabelPos::Inside));

    assert_eq!(
        text_y(&top, "120"),
        text_y(&inside, "120"),
        "折线 Inside 应与 Top 位置一致（回退）"
    );
}

#[test]
fn explicit_label_color_is_applied() {
    let svg = bar_chart(|b| {
        b.label_show(true)
            .label_position(ValueLabelPos::Top)
            .label_color(Color::rgb(255, 0, 0))
    });
    let attrs = text_attrs(&svg, "200").expect("应存在 200 标签");
    assert!(
        attrs.contains("255, 0, 0") || attrs.to_lowercase().contains("#ff0000"),
        "显式 label_color 应生效，实际属性: {attrs}"
    );
}

#[test]
fn label_formatter_is_applied() {
    let svg = bar_chart(|b| b.label_show(true).label_formatter("{c} 件"));
    assert!(
        texts(&svg).iter().any(|(c, _)| c == "200 件"),
        "label_formatter 模板应生效，期望出现 '200 件'"
    );
}
