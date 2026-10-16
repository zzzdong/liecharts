//! 字体「用户显式根 + rem」模式回归测试（Q2）。
//!
//! 解析优先级：**组件显式 > 全局根 × 系数 > 主题令牌**
//!
//! - 全局 `option.textStyle.fontSize` → rem 根（覆盖主题 `body_size` 作 1rem），
//!   各类型按系数换算：标题 1.5×、副标题 1.0×、表体 1.0×、辅助 0.833×；
//! - 组件显式 `fontSize`（`title.textStyle` / `axisLabel` / `series[].label`）→
//!   绝对优先，不参与 rem 缩放；
//! - 均未指定 → 主题令牌（默认根 12：标题 18 / 轴标签 12 / 值标签 12）。
//!
//! ⚠️ 回归重点（Default 陷阱）：用户写 `axisLabel: {}` / `textStyle: {}` 这类
//! **空对象**时，serde 会对缺失字段套 `Default`。若 `Default` 预填
//! `font_size: Some(12.0)`，会被误判为"显式指定 12px"，导致全局根无法向下
//! 传播（轴标签永远卡在 12）。正确语义是 `None`（= 继承）。

use liecharts::SceneNode;
use liecharts::theme::{DesignTokens, TextTokens, Theme};
use lievisual::scene::Element;
use lievisual::text::TextAlign;

/// 扁平化场景，产出所有文本元素（内容, 字号, 对齐）。
fn texts(nodes: &[SceneNode]) -> Vec<(String, f64, TextAlign)> {
    fn walk<'a>(nodes: &'a [SceneNode], out: &mut Vec<(&'a Element, i32)>) {
        for n in nodes {
            if let Some(element) = n.element() {
                out.push((element, n.z_index));
            } else if let Some(children) = n.children() {
                walk(children, out);
            }
        }
    }
    let mut all = Vec::new();
    walk(nodes, &mut all);
    all.into_iter()
        .filter_map(|(e, _)| match e {
            Element::Text { spans, style, .. } => {
                let content: String = spans.iter().map(|s| s.text.clone()).collect();
                Some((content, style.font_size, style.align))
            }
            _ => None,
        })
        .collect()
}

/// 取指定内容文本的字号（要求唯一，否则 panic 并以对齐方式消歧）。
fn font_size_of(nodes: &[SceneNode], content: &str) -> f64 {
    let matches: Vec<f64> = texts(nodes)
        .into_iter()
        .filter(|(c, _, _)| c == content)
        .map(|(_, size, _)| size)
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "文本 {:?} 应恰好出现一次以断言字号，实际 {:?}",
        content,
        matches
    );
    matches[0]
}

/// 取指定内容、指定对齐文本的字号（用于消歧轴标签 vs 值标签）。
fn font_size_of_aligned(nodes: &[SceneNode], content: &str, align: TextAlign) -> f64 {
    let matches: Vec<f64> = texts(nodes)
        .into_iter()
        .filter(|(c, _, a)| c == content && *a == align)
        .map(|(_, size, _)| size)
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "文本 {:?}(align={:?}) 应恰好出现一次，实际 {:?}",
        content,
        align,
        matches
    );
    matches[0]
}

/// 基础图：标题 + 类目 X 轴 + 数值 Y 轴 + 开标签的柱状系列。
///
/// `text_style` / `title_style` / `axis_label` / `series_label` 以 JSON 片段
/// 注入，便于构造「显式 / 空对象 / 缺省」三种形态。
fn chart_json(text_style: &str, title_style: &str, axis_label: &str, series_label: &str) -> String {
    format!(
        r#"{{
  "title": {{ "text": "总标题"{title_style} }},
  "textStyle": {text_style},
  "xAxis": {{ "type": "category", "data": ["甲", "乙", "丙"]{axis_label} }},
  "yAxis": {{ "type": "value" }},
  "series": [
    {{ "type": "bar", "name": "销量", "data": [1, 2, 3], "label": {{ "show": true{series_label} }} }}
  ]
}}"#
    )
}

fn render_json(json: &str) -> Vec<SceneNode> {
    let chart = liecharts::ChartBuilder::from_option_json(json)
        .expect("JSON 应可解析")
        .build(800, 500)
        .expect("应可构建");
    chart.collect_visual_elements().expect("应可渲染")
}

/// 均未显式指定 → 主题令牌（默认根 12）：标题 18 / 轴标签 12 / 值标签 12。
#[test]
fn theme_tokens_apply_when_nothing_specified() {
    let json = chart_json("{}", "", "", "");
    let nodes = render_json(&json);

    assert_eq!(font_size_of(&nodes, "总标题"), 18.0, "标题 = 1.5rem × 12");
    assert_eq!(font_size_of(&nodes, "甲"), 12.0, "轴标签 = 1rem × 12");
    // 值标签与 Y 轴刻度可能同文本，用对齐消歧（值标签居中）
    assert_eq!(
        font_size_of_aligned(&nodes, "2", TextAlign::Center),
        12.0,
        "值标签 = 1rem × 12"
    );
}

/// 全局 `option.textStyle.fontSize` 成为 rem 根，整图文字等比缩放。
#[test]
fn user_text_style_font_size_becomes_rem_root() {
    let json = chart_json(r#"{"fontSize": 20}"#, "", "", "");
    let nodes = render_json(&json);

    assert_eq!(font_size_of(&nodes, "总标题"), 30.0, "标题 = 1.5rem × 20");
    assert_eq!(font_size_of(&nodes, "甲"), 20.0, "轴标签 = 1rem × 20");
    assert_eq!(
        font_size_of_aligned(&nodes, "2", TextAlign::Center),
        20.0,
        "值标签 = 1rem × 20"
    );
}

/// 组件显式 fontSize 绝对优先，不参与 rem 缩放。
#[test]
fn explicit_component_font_size_beats_global_root() {
    let json = chart_json(
        r#"{"fontSize": 20}"#,
        r#", "textStyle": {"fontSize": 10}"#,
        r#", "axisLabel": {"fontSize": 9}"#,
        r#", "fontSize": 15"#,
    );
    let nodes = render_json(&json);

    assert_eq!(font_size_of(&nodes, "总标题"), 10.0, "标题显式 10 优先");
    assert_eq!(font_size_of(&nodes, "甲"), 9.0, "轴标签显式 9 优先");
    assert_eq!(
        font_size_of_aligned(&nodes, "2", TextAlign::Center),
        15.0,
        "值标签显式 15 优先"
    );
}

/// Default 陷阱回归：空样式对象（`{}`）不得把字号钉在默认 12，
/// 全局根必须继续向下传播。
#[test]
fn empty_style_objects_do_not_pin_default_twelve() {
    let json = chart_json(
        r#"{"fontSize": 24}"#,
        r#", "textStyle": {}"#,
        r#", "axisLabel": {}"#,
        r#""#,
    );
    let nodes = render_json(&json);

    assert_eq!(
        font_size_of(&nodes, "总标题"),
        36.0,
        "textStyle:{{}} 是空对象而非显式 12，标题应跟随全局根 24 × 1.5"
    );
    assert_eq!(
        font_size_of(&nodes, "甲"),
        24.0,
        "axisLabel:{{}} 是空对象而非显式 12，轴标签应跟随全局根 24"
    );
    assert_eq!(
        font_size_of_aligned(&nodes, "2", TextAlign::Center),
        24.0,
        "label:{{}} 是空对象而非显式 12，值标签应跟随全局根 24"
    );
}

/// 未给全局根时，自定义主题的 `body_size` 作为主题令牌兜底。
#[test]
fn custom_theme_body_size_used_without_user_root() {
    let mut tokens = DesignTokens::echarts_v6();
    tokens.text = TextTokens::rem(14.0);
    let theme = Theme::from_tokens("root14", tokens);

    let json = chart_json("{}", "", "", "");
    let chart = liecharts::ChartBuilder::from_option_json(&json)
        .expect("JSON 应可解析")
        .with_theme(theme)
        .build(800, 500)
        .expect("应可构建");
    let nodes = chart.collect_visual_elements().expect("应可渲染");

    assert_eq!(font_size_of(&nodes, "总标题"), 21.0, "标题 = 1.5rem × 14");
    assert_eq!(font_size_of(&nodes, "甲"), 14.0, "轴标签 = 1rem × 14");
}

/// 表格排版走主题令牌：表头 1.5rem、表体 1rem（默认根 12 → 18 / 12）。
#[test]
fn table_text_follows_theme_tokens() {
    use liecharts::api::*;

    let chart = Chart::new(800, 500)
        .title(Title::new("数据表"))
        .add_table(Table::new().data(dataframe!(
            "产品" => ["iPhone 15", "Galaxy S24"],
            "销量" => [1200, 980],
        )));
    let nodes = chart.build().expect("应可构建");

    assert_eq!(font_size_of(&nodes, "产品"), 18.0, "表头 = 1.5rem × 12");
    assert_eq!(font_size_of(&nodes, "iPhone 15"), 12.0, "表体 = 1rem × 12");
}

/// `TextTokens::rescale_root` 只换根、保留主题自身系数。
#[test]
fn rescale_root_preserves_theme_coefficients() {
    let mut tokens = TextTokens::rem(12.0);
    tokens.rescale_root(20.0);
    assert_eq!(tokens.body_size, 20.0);
    assert_eq!(tokens.title_size, 30.0);
    assert_eq!(tokens.subtitle_size, 20.0);
    assert!((tokens.caption_size - 20.0 * 10.0 / 12.0).abs() < 1e-9);

    // 自定义系数（如标题 2.0×）不被标准系数冲掉
    let mut custom = TextTokens::rem(10.0);
    custom.title_rem = 2.0;
    custom.rescale_root(15.0);
    assert_eq!(custom.title_size, 30.0, "标题系数 2.0 应保留");
}

// ── 图例文本（legend.textStyle.fontSize） ──────────────────────────────

/// 双系列折线图（图例显示系列名）。
fn two_series_json(text_style: &str, legend_style: &str) -> String {
    format!(
        r#"{{
  "textStyle": {text_style},
  "legend": {{ {legend_style} }},
  "xAxis": {{ "type": "category", "data": ["甲", "乙", "丙"] }},
  "yAxis": {{ "type": "value" }},
  "series": [
    {{ "type": "line", "name": "系列A", "data": [1, 2, 3] }},
    {{ "type": "line", "name": "系列B", "data": [3, 2, 1] }}
  ]
}}"#
    )
}

/// 图例文本跟随 rem 根（×1.0），显式 `legend.textStyle.fontSize` 绝对优先。
#[test]
fn legend_text_size_follows_root_and_explicit() {
    // 全局根 20 → 图例 20（1.0×）
    let nodes = render_json(&two_series_json(r#"{"fontSize": 20}"#, r#""data": []"#));
    assert_eq!(font_size_of(&nodes, "系列A"), 20.0, "图例 = 1rem × 20");
    assert_eq!(font_size_of(&nodes, "系列B"), 20.0);

    // 缺省 → 主题令牌 12
    let nodes = render_json(&two_series_json("{}", r#""data": []"#));
    assert_eq!(font_size_of(&nodes, "系列A"), 12.0, "图例 = 1rem × 12");

    // 显式 22 + 根 20 → 22（不参与缩放）
    let nodes = render_json(&two_series_json(
        r#"{"fontSize": 20}"#,
        r#""data": [], "textStyle": {"fontSize": 22}"#,
    ));
    assert_eq!(font_size_of(&nodes, "系列A"), 22.0, "图例显式 22 优先");
}

// ── 仪表盘（gauge axisLabel / title / detail 的 fontSize） ──────────────

/// 仪表盘图：无图表标题，系列名同时是 gauge 标题文本。
fn gauge_json(text_style: &str, series_extra: &str) -> String {
    format!(
        r#"{{
  "textStyle": {text_style},
  "series": [
    {{ "type": "gauge", "name": "完成率", "min": 0, "max": 100,
       "splitNumber": 2, "data": [{{ "value": 75.5 }}]{series_extra} }}
  ]
}}"#
    )
}

/// 缺省时保持历史观感：刻度 12 / 标题 16 / 数值 30（= 根 12 的 1.0 / 4/3 / 2.5×）。
#[test]
fn gauge_defaults_match_historical_sizes() {
    let nodes = render_json(&gauge_json("{}", ""));

    assert_eq!(font_size_of(&nodes, "0"), 12.0, "刻度标签 = 1rem");
    assert_eq!(font_size_of(&nodes, "50"), 12.0);
    assert_eq!(font_size_of(&nodes, "100"), 12.0);
    assert_eq!(font_size_of(&nodes, "完成率"), 16.0, "标题 = 根 × 4/3");
    assert_eq!(font_size_of(&nodes, "75.5%"), 30.0, "数值 = 根 × 2.5");
}

/// 全局根缩放仪表盘三处文字；组件显式 fontSize 绝对优先。
#[test]
fn gauge_sizes_follow_root_and_explicit() {
    // 根 20 → 刻度 20 / 标题 20×4/3 / 数值 20×2.5 = 50
    let nodes = render_json(&gauge_json(r#"{"fontSize": 20}"#, ""));
    assert_eq!(font_size_of(&nodes, "50"), 20.0, "刻度 = 1rem × 20");
    assert!(
        (font_size_of(&nodes, "完成率") - 20.0 * 4.0 / 3.0).abs() < 1e-9,
        "标题 = 根 × 4/3"
    );
    assert_eq!(font_size_of(&nodes, "75.5%"), 50.0, "数值 = 根 × 2.5");

    // 显式覆盖（axisLabel 7 / title 13 / detail 44）优先于根
    let nodes = render_json(&gauge_json(
        r#"{"fontSize": 20}"#,
        r#", "axisLabel": {"fontSize": 7}, "title": {"fontSize": 13}, "detail": {"fontSize": 44}"#,
    ));
    assert_eq!(font_size_of(&nodes, "50"), 7.0, "刻度显式 7 优先");
    assert_eq!(font_size_of(&nodes, "完成率"), 13.0, "标题显式 13 优先");
    assert_eq!(font_size_of(&nodes, "75.5%"), 44.0, "数值显式 44 优先");
}

// ── 标注（markLine / markPoint 标签） ──────────────────────────────────

/// 带 markLine（平均值）与 markPoint（最大值）的柱状图。
fn mark_json(text_style: &str, mark_line: &str, mark_point: &str) -> String {
    format!(
        r#"{{
  "textStyle": {text_style},
  "xAxis": {{ "type": "category", "data": ["甲", "乙", "丙"] }},
  "yAxis": {{ "type": "value" }},
  "series": [
    {{ "type": "bar", "name": "销量", "data": [1, 2, 3],
       "markLine": {mark_line}, "markPoint": {mark_point} }}
  ]
}}"#
    )
}

/// 标注标签属辅助文字：缺省 = 0.833rem（10px，替代原硬编码 11），随根缩放；
/// `label.fontSize` 显式优先，数据级覆盖系列级。
#[test]
fn mark_labels_follow_caption_token_and_explicit() {
    // 缺省 → 辅助令牌 0.833rem × 12 = 10
    let nodes = render_json(&mark_json(
        "{}",
        r#"{"data": [{"type": "average"}]}"#,
        r#"{"data": [{"type": "max"}]}"#,
    ));
    assert_eq!(
        font_size_of(&nodes, "平均值: 2"),
        10.0,
        "markLine 标签 = 0.833rem × 12"
    );
    assert_eq!(
        font_size_of(&nodes, "最大值: 3"),
        10.0,
        "markPoint 标签同口径"
    );

    // 根 20 → 辅助 = 20 × 0.833 ≈ 16.667
    let nodes = render_json(&mark_json(
        r#"{"fontSize": 20}"#,
        r#"{"data": [{"type": "average"}]}"#,
        r#"{"data": [{"type": "max"}]}"#,
    ));
    assert!(
        (font_size_of(&nodes, "平均值: 2") - 20.0 * 10.0 / 12.0).abs() < 1e-9,
        "markLine 标签 = 0.833rem × 20"
    );
    assert!(
        (font_size_of(&nodes, "最大值: 3") - 20.0 * 10.0 / 12.0).abs() < 1e-9,
        "markPoint 标签 = 0.833rem × 20"
    );

    // 系列级显式 13 + 数据级显式 14（数据级覆盖系列级）
    let nodes = render_json(&mark_json(
        r#"{"fontSize": 20}"#,
        r#"{"label": {"fontSize": 13}, "data": [{"type": "average", "label": {"fontSize": 14}}]}"#,
        r#"{"data": [{"type": "max"}]}"#,
    ));
    assert_eq!(
        font_size_of(&nodes, "平均值: 2"),
        14.0,
        "数据级 label.fontSize 覆盖系列级"
    );
    assert_eq!(
        font_size_of(&nodes, "最大值: 3"),
        20.0 * 10.0 / 12.0,
        "markPoint 未显式 → 跟随辅助令牌"
    );
}

// ── api Builder 模式：`.body_font_size()` 也须支持设置 rem 根 ────────────

/// `Chart::new(...).body_font_size(n)` 等价于 JSON `option.textStyle.fontSize`，
/// 整图文字按主题系数等比缩放。
#[test]
fn body_font_size_sets_rem_root_on_builder() {
    use liecharts::api::*;

    let df = dataframe!(
        "cat" => ["甲", "乙", "丙"],
        "v" => [1.0, 2.0, 3.0],
    );
    let line = || {
        Line::new()
            .name("v")
            .x("cat")
            .y("v")
            .data(df.clone())
            .label_show(true)
    };

    // 默认（不设 body_font_size）→ 标题 18 / 轴标签 12 / 值标签 12
    let nodes = Chart::new(800, 500)
        .title(Title::new("总标题"))
        .add_line(line())
        .build()
        .expect("应可构建");
    assert_eq!(
        font_size_of(&nodes, "总标题"),
        18.0,
        "默认标题 = 1.5rem × 12"
    );
    assert_eq!(font_size_of(&nodes, "甲"), 12.0, "默认轴标签 = 1rem × 12");
    assert_eq!(
        font_size_of_aligned(&nodes, "2", TextAlign::Center),
        12.0,
        "默认值标签 = 1rem × 12"
    );

    // body_font_size(20) → 标题 30 / 轴标签 20 / 值标签 20
    let nodes = Chart::new(800, 500)
        .title(Title::new("总标题"))
        .body_font_size(20.0)
        .add_line(line())
        .build()
        .expect("应可构建");
    assert_eq!(font_size_of(&nodes, "总标题"), 30.0, "标题 = 1.5rem × 20");
    assert_eq!(font_size_of(&nodes, "甲"), 20.0, "轴标签 = 1rem × 20");
    assert_eq!(
        font_size_of_aligned(&nodes, "2", TextAlign::Center),
        20.0,
        "值标签 = 1rem × 20"
    );
}
