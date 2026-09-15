//! ECharts 兼容性缺陷修复的回归测试。
//!
//! 每个用例对应一个真实缺陷（详见 `docs/` 兼容性调研报告的复核结论）：
//! - `markLine.symbolSize` 数字/数组不再解析失败
//! - `lineStyle.type: dashed/dotted` 真的产出虚线
//! - `itemStyle.borderRadius` 真的产出圆角柱
//! - `markPoint` 真的渲染
//! - `dataZoom` 真的裁剪窗口
//! - `dataset`（缺省 datasetIndex / 对象数组 / dimensions / 自动表头）真的生效
//! - `series.xAxisIndex/yAxisIndex` 真的选中 subplot；`grid.width/height` 真的生效
//! - `type:"category"` 无 `data` 时从系列数据推导类目
//! - `angleAxis.data` 的极坐标类目名不再退化成 `Item N`
//! - `radar.indicator[].max` 参与顶点半径换算；`series[].label` 渲染数值
//! - 时间轴刻度落在时间边界（不出现重复日期）

mod common;
use common::*;

use liecharts::{
    ChartBuilder, Fill, SceneNode,
    option::ChartOption,
    pipeline::{compat::chart_option_to_chart_spec, types::GridEdge},
};
use lievisual::kurbo::{PathEl, Rect, Shape};
use lievisual::scene::Element;

/// 渲染一段 ECharts JSON 为结构化场景 IR。
fn render_json(json: &str, w: u32, h: u32) -> Vec<SceneNode> {
    ChartBuilder::from_option_json(json)
        .expect("JSON 应可解析")
        .with_theme(liecharts::theme::Theme::echarts())
        .build(w, h)
        .expect("图表应可构建")
        .collect_visual_elements()
        .expect("应可收集场景元素")
}

fn spec_of(json: &str) -> liecharts::pipeline::types::ChartSpec {
    let option: ChartOption = serde_json::from_str(json).expect("JSON 应可解析");
    chart_option_to_chart_spec(&option, 800, 500)
}

/// 收集所有文本内容
fn all_texts(nodes: &[SceneNode]) -> Vec<String> {
    texts(nodes).into_iter().map(|(t, _, _)| t).collect()
}

// ── 1. markLine.symbolSize 容错 ────────────────────────────────────────

#[test]
fn mark_line_symbol_size_accepts_number_and_array() {
    // 回归：`symbolSize: 8` 曾报 `invalid type: integer 8, expected a sequence`
    for size in ["8", "[8, 12]"] {
        let json = format!(
            r#"{{"xAxis":{{"type":"category","data":["a","b","c"]}},
                 "yAxis":{{"type":"value"}},
                 "series":[{{"type":"line","data":[10,20,15],
                   "markLine":{{"symbolSize":{size},"data":[{{"type":"average","name":"平均值"}}]}}}}]}}"#
        );
        let nodes = render_json(&json, 800, 500);
        assert!(
            all_texts(&nodes).iter().any(|t| t.starts_with("平均值")),
            "symbolSize={size} 时 markLine 标签应渲染"
        );
    }
}

// ── 2. 虚线 ───────────────────────────────────────────────────────────

#[test]
fn line_style_type_dashed_and_dotted_emit_dash_array() {
    // 回归：`lineStyle.type` 被解析但从未消费，SVG 里没有 stroke-dasharray
    for (ty, expect) in [("dashed", vec![6.0, 4.0]), ("dotted", vec![1.5, 3.0])] {
        let json = format!(
            r#"{{"xAxis":{{"type":"category","data":["a","b","c"]}},"yAxis":{{"type":"value"}},
                 "series":[{{"type":"line","lineStyle":{{"type":"{ty}"}},"data":[10,30,20]}}]}}"#
        );
        let nodes = render_json(&json, 800, 500);
        let dashed: Vec<Vec<f64>> = elements_of(&nodes, |e| matches!(e, Element::Path { .. }))
            .into_iter()
            .filter_map(|(e, _)| match e {
                Element::Path { style, .. } => style.stroke.as_ref().map(|s| s.dash_array.clone()),
                _ => None,
            })
            .filter(|d| !d.is_empty())
            .collect();
        assert_eq!(dashed.len(), 1, "{ty}: 应恰好有一条虚线");
        assert_eq!(dashed[0], expect, "{ty}: 虚线段长应符合预期");
    }
}

#[test]
fn solid_line_has_no_dash_array() {
    let nodes = render_json(
        r#"{"xAxis":{"type":"category","data":["a","b"]},"yAxis":{"type":"value"},
            "series":[{"type":"line","data":[10,20]}]}"#,
        800,
        500,
    );
    let any_dash = elements_of(&nodes, |e| matches!(e, Element::Path { .. }))
        .into_iter()
        .any(|(e, _)| match e {
            Element::Path { style, .. } => style
                .stroke
                .as_ref()
                .is_some_and(|s| !s.dash_array.is_empty()),
            _ => false,
        });
    assert!(!any_dash, "实线不应带 dash_array");
}

// ── 3. 柱体圆角 ───────────────────────────────────────────────────────

#[test]
fn bar_border_radius_emits_rounded_rect() {
    // 回归：`itemStyle.borderRadius` 字段不存在，被静默忽略
    for radius in ["8", "[4,4,0,0]"] {
        let json = format!(
            r#"{{"xAxis":{{"type":"category","data":["a","b"]}},"yAxis":{{"type":"value"}},
                 "series":[{{"type":"bar","itemStyle":{{"borderRadius":{radius}}},"data":[10,20]}}]}}"#
        );
        let nodes = render_json(&json, 800, 500);
        let radii: Vec<f64> = elements_of(&nodes, |e| matches!(e, Element::RoundedRect { .. }))
            .into_iter()
            .filter_map(|(e, _)| match e {
                Element::RoundedRect { radius, .. } => Some(*radius),
                _ => None,
            })
            .collect();
        assert_eq!(
            radii.len(),
            2,
            "borderRadius={radius} 时两根柱都应是圆角矩形"
        );
        assert!(radii.iter().all(|r| *r > 0.0), "圆角半径应大于 0");
    }
}

#[test]
fn stacked_bar_border_radius_applies_to_grouped_bars() {
    let nodes = render_json(
        r#"{"xAxis":{"type":"category","data":["a","b"]},"yAxis":{"type":"value"},
            "series":[{"type":"bar","stack":"t","itemStyle":{"borderRadius":6},"data":[30,60]},
                      {"type":"bar","stack":"t","data":[70,40]}]}"#,
        800,
        500,
    );
    let count = elements_of(&nodes, |e| matches!(e, Element::RoundedRect { .. })).len();
    assert_eq!(count, 4, "堆叠柱的 4 段都应带圆角");
}

// ── 4. markPoint ─────────────────────────────────────────────────────

#[test]
fn mark_point_renders_symbol_and_label() {
    let nodes = render_json(
        r#"{"xAxis":{"type":"category","data":["a","b","c"]},"yAxis":{"type":"value"},
            "series":[{"type":"line","data":[10,20,15],
              "markPoint":{"data":[{"type":"max","name":"最大"},{"type":"min"}]}}]}"#,
        800,
        500,
    );
    let t = all_texts(&nodes);
    assert!(
        t.iter().any(|s| s == "最大: 20"),
        "应渲染最大值标注点：{t:?}"
    );
    assert!(
        t.iter().any(|s| s == "最小值: 10"),
        "应渲染最小值标注点：{t:?}"
    );
}

// ── 5. dataZoom ──────────────────────────────────────────────────────

#[test]
fn data_zoom_trims_visible_window() {
    // 回归：dataZoom 曾被完全忽略，8 个类目全部显示
    let spec = spec_of(
        r#"{"dataZoom":[{"start":25,"end":75}],
            "xAxis":{"type":"category","data":["a","b","c","d","e","f","g","h"]},
            "yAxis":{"type":"value"},
            "series":[{"type":"line","data":[1,5,3,8,2,6,4,7]}]}"#,
    );
    assert_eq!(spec.x_axes[0].categories, vec!["c", "d", "e", "f"]);
    assert_eq!(spec.series[0].data.row_count(), 4, "数据行应同步裁剪");
}

#[test]
fn data_zoom_full_range_keeps_all() {
    let spec = spec_of(
        r#"{"dataZoom":[{"start":0,"end":100}],
            "xAxis":{"type":"category","data":["a","b"]},
            "yAxis":{"type":"value"},
            "series":[{"type":"bar","data":[1,2]}]}"#,
    );
    assert_eq!(spec.x_axes[0].categories, vec!["a", "b"]);
    assert_eq!(spec.series[0].data.row_count(), 2);
}

// ── 6. dataset ───────────────────────────────────────────────────────

#[test]
fn dataset_without_dataset_index_uses_first() {
    // 回归：ECharts 的 `datasetIndex` 缺省为 0，此前必须显式指定，
    // 否则回落空 `series.data` → `Missing column: x` 渲染失败
    let json = r#"{"dataset":{"source":[["a",10],["b",20]]},
                   "xAxis":{"type":"category"},"yAxis":{"type":"value"},
                   "series":[{"type":"bar"}]}"#;
    let nodes = render_json(json, 800, 500);
    let t = all_texts(&nodes);
    assert!(
        t.contains(&"a".to_string()) && t.contains(&"b".to_string()),
        "{t:?}"
    );
}

#[test]
fn dataset_object_rows_and_dimensions_are_supported() {
    // 对象数组（ECharts 常见写法）
    let t = all_texts(&render_json(
        r#"{"dataset":{"source":[{"x":"a","y":10},{"x":"b","y":20}]},
            "xAxis":{"type":"category"},"yAxis":{"type":"value"},
            "series":[{"type":"bar","encode":{"x":"x","y":"y"}}]}"#,
        800,
        500,
    ));
    assert!(
        t.contains(&"a".to_string()),
        "对象数组 dataset 应可用：{t:?}"
    );

    // dimensions 命名列
    let t = all_texts(&render_json(
        r#"{"dataset":{"dimensions":["x","y"],"source":[["a",10],["b",20]]},
            "xAxis":{"type":"category"},"yAxis":{"type":"value"},
            "series":[{"type":"bar","encode":{"x":"x","y":"y"}}]}"#,
        800,
        500,
    ));
    assert!(
        t.contains(&"a".to_string()),
        "dimensions 应可为列命名：{t:?}"
    );
}

#[test]
fn dataset_header_is_auto_detected() {
    // 回归：`sourceHeader` 缺省一律当表头，导致 `[["a",10],…]` 丢掉首行数据
    let spec = spec_of(
        r#"{"dataset":{"source":[["a",10],["b",20]]},
            "xAxis":{"type":"category"},"yAxis":{"type":"value"},
            "series":[{"type":"bar"}]}"#,
    );
    assert_eq!(
        spec.series[0].data.row_count(),
        2,
        "混合型首行不应被当作表头"
    );

    // 全字符串首行 → 视为表头
    let spec = spec_of(
        r#"{"dataset":{"source":[["product","count"],["A",10],["B",20]]},
            "xAxis":{"type":"category"},"yAxis":{"type":"value"},
            "series":[{"type":"bar"}]}"#,
    );
    assert_eq!(
        spec.series[0].data.row_count(),
        2,
        "全字符串首行应被当作表头"
    );
}

// ── 7. grid / 轴索引 ─────────────────────────────────────────────────

#[test]
fn series_axis_index_selects_subplot() {
    // 回归：line/bar 的 `x_axis_index` 被硬编码为 0，两个子图会叠在一起
    let spec = spec_of(
        r#"{"grid":[{},{"left":"55%","right":"5%"}],
            "xAxis":[{"type":"category","data":["a","b"]},
                     {"type":"category","gridIndex":1,"data":["c","d"]}],
            "yAxis":[{"type":"value"},{"type":"value","gridIndex":1}],
            "series":[{"type":"bar","data":[1,2]},
                      {"type":"line","xAxisIndex":1,"yAxisIndex":1,"data":[3,4]}]}"#,
    );
    assert_eq!(spec.series[0].grid_index, 0);
    assert_eq!(spec.series[1].grid_index, 1, "xAxisIndex 应反推出 subplot");
    assert_eq!(spec.series[1].x_axis_index, 1);
    assert_eq!(spec.series[1].y_axis_index, 1);
}

#[test]
fn grid_width_and_height_are_honoured() {
    // 回归：`grid.width/height` 字段不存在，被静默忽略
    let spec = spec_of(
        r#"{"grid":[{},{"left":"55%","width":"40%","top":"10%","height":"60%"}],
            "xAxis":[{"type":"category","data":["a"]},{"type":"category","gridIndex":1,"data":["c"]}],
            "yAxis":[{"type":"value"},{"type":"value","gridIndex":1}],
            "series":[{"type":"bar","gridIndex":0,"data":[1]},
                      {"type":"bar","gridIndex":1,"data":[2]}]}"#,
    );
    assert_eq!(spec.grids[1].width, Some(GridEdge::Pct(40.0)));
    assert_eq!(spec.grids[1].height, Some(GridEdge::Pct(60.0)));
}

// ── 8. 类目轴从数据推导 ───────────────────────────────────────────────

#[test]
fn category_axis_derives_categories_from_series_data() {
    let spec = spec_of(
        r#"{"xAxis":{"type":"category"},"yAxis":{"type":"value"},
            "series":[{"type":"line","data":[["a",10],["b",20],["c",15]]}]}"#,
    );
    assert_eq!(spec.x_axes[0].categories, vec!["a", "b", "c"]);
}

#[test]
fn horizontal_bar_does_not_fake_categories_from_numeric_column() {
    // 横向柱的类目列是数值索引，不能伪造成 "0"/"1"
    let spec = spec_of(
        r#"{"xAxis":{"type":"value"},"yAxis":{"type":"category"},
            "series":[{"type":"bar","data":[10,20]}]}"#,
    );
    assert!(
        spec.y_axes[0].categories.is_empty(),
        "数值列不应被当成类目名：{:?}",
        spec.y_axes[0].categories
    );
}

// ── 9. 极坐标类目名 ──────────────────────────────────────────────────

#[test]
fn polar_bar_uses_angle_axis_category_names() {
    // 回归：极坐标类目名完全丢失，渲染成 `Item 0 / Item 1 / …`
    let nodes = render_json(
        r#"{"angleAxis":{"type":"category","data":["东","南","西"]},
            "radiusAxis":{},"polar":{},
            "series":[{"type":"bar","coordinateSystem":"polar","data":[10,20,30]}]}"#,
        800,
        500,
    );
    let t = all_texts(&nodes);
    for name in ["东", "南", "西"] {
        assert!(
            t.iter().any(|s| s == name),
            "极坐标应显示类目名 {name}：{t:?}"
        );
    }
    assert!(
        !t.iter().any(|s| s.starts_with("Item ")),
        "不应再出现 Item N：{t:?}"
    );
}

// ── 10. 雷达图 ───────────────────────────────────────────────────────

#[test]
fn radar_label_renders_indicator_values() {
    let nodes = render_json(
        r#"{"radar":{"indicator":[{"name":"A","max":100},{"name":"B","max":100},{"name":"C","max":100}]},
            "series":[{"type":"radar","label":{"show":true},"data":[{"value":[80,60,90],"name":"设备A"}]}]}"#,
        800,
        500,
    );
    let t = all_texts(&nodes);
    for v in ["80", "60", "90"] {
        assert!(t.iter().any(|s| s == v), "雷达数值标签应含 {v}：{t:?}");
    }
}

#[test]
fn radar_uses_per_indicator_max() {
    // 每个维度各自归一化：值等于各自的 max 时，三个顶点都应落在最外圈
    let nodes = render_json(
        r#"{"radar":{"indicator":[{"name":"A","max":200},{"name":"B","max":100},{"name":"C","max":50}]},
            "series":[{"type":"radar","data":[{"value":[200,100,50],"name":"x"}]}]}"#,
        800,
        500,
    );
    let all: Vec<(bool, Rect)> = paths(&nodes)
        .into_iter()
        .map(|(p, style, _)| (style.fill.is_some(), p.bounding_box()))
        .collect();
    let filled: Vec<Rect> = all.iter().filter(|(f, _)| *f).map(|(_, bb)| *bb).collect();
    assert_eq!(filled.len(), 1, "应只有一个数据多边形（填充）");
    // 未填充里最大的那个就是最外圈网格
    let outer = all
        .iter()
        .filter(|(f, _)| !*f)
        .map(|(_, bb)| *bb)
        .max_by(|a, b| a.width().partial_cmp(&b.width()).unwrap())
        .expect("应有网格多边形");
    let (d, o) = (filled[0], outer);
    assert!(
        (d.width() - o.width()).abs() < 0.5 && (d.height() - o.height()).abs() < 0.5,
        "值达到各维 max 时数据多边形应贴合最外圈：data={d:?} outer={o:?}"
    );
}

// ── 11. 时间轴刻度 ───────────────────────────────────────────────────

#[test]
fn time_axis_labels_are_distinct_and_cover_data_dates() {
    // 回归：epoch 秒直接做十进制取整 → 刻度落在 02:06/16:00… 舍入到日出现
    // `08-01/08-01/08-02/08-02`，真实日期 08-03 永远没有刻度
    let nodes = render_json(
        r#"{"xAxis":{"type":"time"},"yAxis":{"type":"value"},
            "series":[{"type":"line","data":[["2026-08-01",100],["2026-08-02",150],["2026-08-03",120]]}]}"#,
        800,
        500,
    );
    let labels: Vec<String> = all_texts(&nodes)
        .into_iter()
        .filter(|t| t.starts_with("2026-"))
        .collect();
    assert!(!labels.is_empty(), "时间轴应渲染日期标签");
    let mut uniq = labels.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), labels.len(), "时间轴刻度不应重复：{labels:?}");
    assert!(
        labels.iter().any(|t| t.starts_with("2026-08-03")),
        "应出现数据日期 08-03：{labels:?}"
    );
}

#[test]
fn time_axis_sub_day_labels_do_not_shift_the_date() {
    // 回归：标签日期用 `round(t/86400)` 计算，12:00 会进位成次日
    // （`08-01 12:00` 被标成 `08-02 12:00`）
    let nodes = render_json(
        r#"{"xAxis":{"type":"time"},"yAxis":{"type":"value"},
            "series":[{"type":"line","data":[["2026-08-01 00:00:00",1],
                                              ["2026-08-01 06:00:00",5],
                                              ["2026-08-01 12:00:00",3],
                                              ["2026-08-01 18:00:00",7]]}]}"#,
        800,
        500,
    );
    let labels: Vec<String> = all_texts(&nodes)
        .into_iter()
        .filter(|t| t.starts_with("2026-"))
        .collect();
    assert!(!labels.is_empty(), "应渲染时间标签");
    assert!(
        labels.iter().all(|t| t.starts_with("2026-08-01")),
        "18 小时跨度内所有刻度都应落在 08-01：{labels:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 第二轮修复：枚举兜底 / 默认行为 / 轴装饰 / 柱几何 / 系列配色
// ═══════════════════════════════════════════════════════════════════

/// ECharts 合法但 liecharts 未建模的枚举值，必须**降级为忽略**而不是整图解析失败。
#[test]
fn lenient_enums_never_break_parsing() {
    let snippets = [
        r#"{"series":[{"type":"bar","data":[1],"label":{"show":true,"position":"insideTopLeft"}}]}"#,
        r#"{"series":[{"type":"bar","data":[1],"label":{"show":true,"position":"insideBottomRight"}}]}"#,
        r#"{"series":[{"type":"line","symbol":"image://x.png","data":[1,2]}]}"#,
        r#"{"series":[{"type":"line","symbol":"path://M0,0L1,1","data":[1,2]}]}"#,
        r#"{"series":[{"type":"line","symbol":"inherit","data":[1,2]}]}"#,
        r#"{"series":[{"type":"line","sampling":"lttb","data":[1,2]}]}"#,
        r#"{"series":[{"type":"pie","labelLine":{"smooth":0.3},"data":[{"name":"a","value":1}]}]}"#,
        r#"{"series":[{"type":"bar","label":{"show":true,"fontWeight":500},"data":[1]}]}"#,
        r#"{"series":[{"type":"line","lineStyle":{"type":[5,5]},"data":[1,2]}]}"#,
        r#"{"series":[{"type":"bar","itemStyle":{"borderType":"dashed"},"data":[1]}]}"#,
    ];
    for json in snippets {
        let parsed = ChartBuilder::from_option_json(json);
        assert!(
            parsed.is_ok(),
            "应可解析（不因未建模枚举值而失败）：{json} — {:?}",
            parsed.as_ref().err()
        );
        // 解析通过即可；部分片段结构不完整，渲染失败不算回归
        let _ = ChartBuilder::from_option_json(json)
            .unwrap()
            .with_theme(liecharts::theme::Theme::echarts())
            .build(400, 300);
    }
}

/// 饼图 `label.show` 默认 **true**（ECharts 语义），line/bar 默认 false。
#[test]
fn pie_label_defaults_to_shown_and_bar_to_hidden() {
    let pie = render_json(
        r#"{"series":[{"type":"pie","data":[{"name":"a","value":1},{"name":"b","value":2}]}]}"#,
        600,
        400,
    );
    let t = all_texts(&pie);
    assert!(
        t.iter().any(|s| s.contains("33.3")),
        "饼图应默认显示百分比标签：{t:?}"
    );

    let bar = render_json(
        r#"{"xAxis":{"type":"category","data":["a","b"]},"yAxis":{"type":"value"},
            "series":[{"type":"bar","data":[1234,5678]}]}"#,
        600,
        400,
    );
    let t = all_texts(&bar);
    assert!(
        !t.iter().any(|s| s == "1234" || s == "5678"),
        "柱状图默认不显示数值标签：{t:?}"
    );
}

/// 标题默认**水平居中**（ECharts 6 起 `title.left` 默认 `'center'`，v5 为贴左），
/// `left:'left'` / `left:20` 生效，`show:false` 不渲染。
#[test]
fn title_position_and_show_are_honoured() {
    let tail = r#","xAxis":{"type":"category","data":["a"]},"yAxis":{"type":"value"},
                 "series":[{"type":"line","data":[1]}]}"#;

    let x_of_title = |json: &str| -> Option<f64> {
        let nodes = render_json(json, 800, 500);
        texts(&nodes)
            .into_iter()
            .find(|(t, _, _)| t == "T")
            .map(|(_, x, _)| x)
    };

    let d = x_of_title(&format!(r#"{{"title":{{"text":"T"}}{tail}"#)).expect("应有标题");
    let c = x_of_title(&format!(
        r#"{{"title":{{"text":"T","left":"center"}}{tail}"#
    ))
    .expect("应有标题");
    let le =
        x_of_title(&format!(r#"{{"title":{{"text":"T","left":"left"}}{tail}"#)).expect("应有标题");
    let l = x_of_title(&format!(r#"{{"title":{{"text":"T","left":20}}{tail}"#)).expect("应有标题");
    let hidden = x_of_title(&format!(r#"{{"title":{{"text":"T","show":false}}{tail}"#));

    assert!(
        (d - c).abs() < 1.0,
        "默认标题应与 left:center 同位置（居中），实际 default={d} center={c}"
    );
    assert!(d > 200.0, "默认标题应居中，实际 x={d}");
    assert!(le < 5.0, "left:'left' 应贴左，实际 x={le}");
    assert!((l - 20.0).abs() < 1.0, "left:20 应生效，实际 x={l}");
    assert!(hidden.is_none(), "show:false 时不应渲染标题");
}

fn count_lines(json: &str) -> usize {
    elements_of(&render_json(json, 800, 500), |e| {
        matches!(e, Element::Line { .. })
    })
    .len()
}

/// 轴装饰开关（`axisTick.show` / `splitLine.show`）真的生效：
/// 此前 `AxisSpec` 里存了 `axis_line_show`/`split_line_show` 但渲染器从不读。
#[test]
fn axis_tick_and_split_line_show_are_honoured() {
    let base = r#"{"xAxis":{"type":"category","data":["a","b"]},"yAxis":{"type":"value"},
                   "series":[{"type":"line","data":[10,20]}]}"#;
    let no_tick = r#"{"xAxis":{"type":"category","data":["a","b"],"axisTick":{"show":false}},
                      "yAxis":{"type":"value"},"series":[{"type":"line","data":[10,20]}]}"#;
    let no_grid = r#"{"xAxis":{"type":"category","data":["a","b"]},
                      "yAxis":{"type":"value","splitLine":{"show":false}},
                      "series":[{"type":"line","data":[10,20]}]}"#;

    let (b, t, g) = (
        count_lines(base),
        count_lines(no_tick),
        count_lines(no_grid),
    );
    assert!(
        t < b,
        "axisTick.show:false 应减少刻度线：base={b} no_tick={t}"
    );
    assert!(
        g < b,
        "splitLine.show:false 应减少网格线：base={b} no_grid={g}"
    );
}

/// 类目轴默认**不画**分隔线（ECharts：分隔线默认数值轴显示、类目轴不显示）。
#[test]
fn category_axis_has_no_split_line_by_default() {
    // 类目轴若有分隔线，会多出 n+1 条竖线
    let with = count_lines(
        r#"{"xAxis":{"type":"category","data":["a","b"],"splitLine":{"show":true}},
            "yAxis":{"type":"value"},"series":[{"type":"line","data":[10,20]}]}"#,
    );
    let without = count_lines(
        r#"{"xAxis":{"type":"category","data":["a","b"]},"yAxis":{"type":"value"},
            "series":[{"type":"line","data":[10,20]}]}"#,
    );
    assert!(
        without < with,
        "类目轴默认不应画分隔线：默认={without} 显式开启={with}"
    );
}

/// 图例 `orient` / `itemWidth` / `itemHeight`：垂直排列成单列，符号框按配置尺寸。
#[test]
fn legend_orient_vertical_and_item_size() {
    let json = r#"{"xAxis":{"type":"category","data":["x"]},"yAxis":{"type":"value"},
        "legend":{"data":["a","b"],"orient":"vertical","itemWidth":40,"itemHeight":20},
        "series":[{"name":"a","type":"bar","data":[1]},
                  {"name":"b","type":"bar","data":[2]}]}"#;
    let nodes = render_json(json, 800, 500);

    // 柱状系列的图例符号 = 40×20 的矩形（`itemWidth` × `itemHeight`）
    let boxes: Vec<Rect> = rects(&nodes)
        .into_iter()
        .filter(|(r, _)| (r.width() - 40.0).abs() < 0.1 && (r.height() - 20.0).abs() < 0.1)
        .map(|(r, _)| r)
        .collect();
    assert_eq!(boxes.len(), 2, "两个系列应各有一个 40×20 的图例符号框");
    assert!(
        (boxes[0].x0 - boxes[1].x0).abs() < 0.1,
        "垂直排列时各项左缘应对齐：{:?} vs {:?}",
        boxes[0],
        boxes[1]
    );
    assert!(
        (boxes[0].y0 - boxes[1].y0).abs() > 20.0,
        "垂直排列时各项应各占一行"
    );

    // 水平（默认）：同一行内 y 相同、x 递增
    let horizontal = render_json(
        r#"{"xAxis":{"type":"category","data":["x"]},"yAxis":{"type":"value"},
            "legend":{"data":["a","b"]},
            "series":[{"name":"a","type":"bar","data":[1]},
                      {"name":"b","type":"bar","data":[2]}]}"#,
        800,
        500,
    );
    let h_boxes: Vec<Rect> = rects(&horizontal)
        .into_iter()
        .filter(|(r, _)| (r.width() - 25.0).abs() < 0.1 && (r.height() - 14.0).abs() < 0.1)
        .map(|(r, _)| r)
        .collect();
    assert_eq!(h_boxes.len(), 2, "默认水平排列应有 2 个 25×14 符号框");
    assert!(
        (h_boxes[0].y0 - h_boxes[1].y0).abs() < 0.1,
        "水平排列各项同一行"
    );
    assert!(h_boxes[1].x0 > h_boxes[0].x0, "水平排列各项 x 递增");
}

/// 图例符号形状跟随系列类型：折线 → 线段 + 中点标记，饼图 → 圆，散点 → 圆。
#[test]
fn legend_symbol_shape_follows_chart_type() {
    let line = render_json(
        r#"{"legend":{"data":["s1"]},
            "xAxis":{"type":"category","data":["x"]},"yAxis":{"type":"value"},
            "series":[{"name":"s1","type":"line","data":[1]}]}"#,
        800,
        500,
    );
    assert!(
        rects(&line)
            .iter()
            .any(|(r, _)| (r.width() - 25.0).abs() < 0.1 && r.height() < 5.0),
        "折线系列的图例符号应为线段（宽 itemWidth、细高）"
    );
    assert!(
        circles(&line).iter().any(|(_, r, _)| *r > 1.0),
        "折线系列的图例符号应带中点标记"
    );

    let pie = render_json(
        r#"{"legend":{"data":["甲"]},
            "series":[{"type":"pie","data":[{"name":"甲","value":1}]}]}"#,
        800,
        500,
    );
    // 饼图符号 = 圆，直径取符号框短边 `itemHeight`(14) → 半径 7
    assert!(
        circles(&pie).iter().any(|(_, r, _)| (*r - 7.0).abs() < 0.1),
        "饼图系列的图例符号应为直径 14 的圆"
    );
}

/// 图例色块必须与它标注的系列**同名匹配**。
///
/// 回归：图例曾按**项下标**取色，`legend.data` 重排（`["s3","s1"]`）或只列子集时，
/// 色块与所标注的系列对不上（表现为图例颜色与实际线/柱颜色不一致）。
#[test]
fn legend_symbol_colors_follow_series_names() {
    fn hex_norm(s: &str) -> String {
        s.trim_start_matches('#').to_ascii_lowercase()
    }

    // 折线系列的图例符号 = 线段（`itemWidth` 25 × 线宽 2.5）+ 中点标记
    let symbols = |json: &str| -> Vec<String> {
        rects(&render_json(json, 800, 500))
            .into_iter()
            .filter(|(r, _)| (r.width() - 25.0).abs() < 0.1 && r.height() < 5.0)
            .filter_map(|(_, s)| match s.fill.as_ref() {
                Some(Fill::Solid(c)) => Some(solid_color(c)),
                _ => None,
            })
            .collect()
    };

    let palette = liecharts::theme::Theme::echarts().color;
    let got = symbols(
        r#"{"xAxis":{"type":"category","data":["x"]},"yAxis":{"type":"value"},
            "legend":{"data":["s3","s1"]},
            "series":[{"name":"s1","type":"line","data":[1]},
                      {"name":"s2","type":"line","data":[2]},
                      {"name":"s3","type":"line","data":[3]}]}"#,
    );
    assert_eq!(got.len(), 2, "应有两个图例色块，实际 {got:?}");
    assert_eq!(
        hex_norm(&got[0]),
        hex_norm(&palette[2]),
        "s3 的图例色应等于第 3 个系列色"
    );
    assert_eq!(
        hex_norm(&got[1]),
        hex_norm(&palette[0]),
        "s1 的图例色应等于第 1 个系列色"
    );

    // 只列子集（跳过 s2）同样按名称对齐
    let subset = symbols(
        r#"{"xAxis":{"type":"category","data":["x"]},"yAxis":{"type":"value"},
            "legend":{"data":["s3"]},
            "series":[{"name":"s1","type":"line","data":[1]},
                      {"name":"s2","type":"line","data":[2]},
                      {"name":"s3","type":"line","data":[3]}]}"#,
    );
    assert_eq!(subset.len(), 1);
    assert_eq!(
        hex_norm(&subset[0]),
        hex_norm(&palette[2]),
        "子集图例也应取 s3 的系列色"
    );
}

/// `axisLabel.interval` 显式抽稀生效。
#[test]
fn axis_label_interval_is_honoured() {
    let nodes = render_json(
        r#"{"xAxis":{"type":"category","data":["a","b","c","d","e"],"axisLabel":{"interval":2}},
            "yAxis":{"type":"value"},"series":[{"type":"line","data":[1,2,3,4,5]}]}"#,
        800,
        500,
    );
    let cat: Vec<String> = all_texts(&nodes)
        .into_iter()
        .filter(|t| ["a", "b", "c", "d", "e"].contains(&t.as_str()))
        .collect();
    assert_eq!(cat, vec!["a", "c", "e"], "interval:2 应每 3 个显示一个");
}

fn bar_rects(json: &str) -> Vec<(Rect, liecharts::FillStrokeStyle)> {
    // 过滤掉画布/绘图区背景（宽高接近画布）与图例色块（10×10）
    rects(&render_json(json, 800, 500))
        .into_iter()
        .filter(|(r, _)| r.width() < 700.0 && r.height() < 450.0 && r.height() > 15.0)
        .map(|(r, s)| (r, s.clone()))
        .collect()
}

/// 柱宽默认占类目槽宽 **80%**（ECharts `barCategoryGap:'20%'`），
/// 多系列时按 `barGap:'30%'` 留出组内间距。
#[test]
fn bar_default_width_and_gap_match_echarts() {
    let one = bar_rects(
        r#"{"xAxis":{"type":"category","data":["a","b","c"]},"yAxis":{"type":"value"},
            "series":[{"type":"bar","data":[10,20,30]}]}"#,
    );
    assert_eq!(one.len(), 3, "应有 3 根柱");
    // v6 默认 grid：left 15%(120) / right 10%(80) → 绘图区 x=[120,720]，
    // 3 个类目 → 槽宽 200；barCategoryGap 20% → 柱宽 160
    let w = one[0].0.width();
    assert!(
        (w - 160.0).abs() < 1.5,
        "单系列默认柱宽应为槽宽 80%（≈160），实际 {w}"
    );

    let two = bar_rects(
        r#"{"xAxis":{"type":"category","data":["a","b","c"]},"yAxis":{"type":"value"},
            "series":[{"name":"s1","type":"bar","data":[10,20,30]},
                      {"name":"s2","type":"bar","data":[5,10,15]}]}"#,
    );
    assert_eq!(two.len(), 6, "应有 6 根柱");
    // 同组内两根柱之间应留出 barGap（30% 柱宽）
    let gap = two[3].0.x0 - two[0].0.x1;
    assert!(gap > 10.0, "两系列应留出 barGap 间距，实际 {gap}");
}

/// `barWidth` 数字是**像素**（历史：被当成百分比除以 100）。
#[test]
fn bar_width_number_means_pixels() {
    let bars = bar_rects(
        r#"{"xAxis":{"type":"category","data":["a","b","c"]},"yAxis":{"type":"value"},
            "series":[{"type":"bar","barWidth":40,"data":[10,20,30]}]}"#,
    );
    assert!(
        (bars[0].0.width() - 40.0).abs() < 0.5,
        "barWidth:40 应为 40px，实际 {}",
        bars[0].0.width()
    );
}

/// `barMaxWidth` 生效。
#[test]
fn bar_max_width_is_honoured() {
    let bars = bar_rects(
        r#"{"xAxis":{"type":"category","data":["a","b","c"]},"yAxis":{"type":"value"},
            "series":[{"type":"bar","barMaxWidth":20,"data":[10,20,30]}]}"#,
    );
    assert!(
        bars[0].0.width() <= 20.5,
        "barMaxWidth:20 应限制柱宽，实际 {}",
        bars[0].0.width()
    );
}

/// `showBackground` 画出值轴全幅背景柱。
#[test]
fn bar_show_background_draws_background_bars() {
    let on = bar_rects(
        r#"{"xAxis":{"type":"category","data":["a","b","c"]},"yAxis":{"type":"value"},
            "series":[{"type":"bar","showBackground":true,"data":[10,20,30]}]}"#,
    );
    let off = bar_rects(
        r#"{"xAxis":{"type":"category","data":["a","b","c"]},"yAxis":{"type":"value"},
            "series":[{"type":"bar","data":[10,20,30]}]}"#,
    );
    assert_eq!(off.len(), 3);
    assert_eq!(on.len(), 6, "showBackground 应额外画出 3 条背景柱");
}

/// `series.itemStyle.color` / `lineStyle.color` 覆盖调色板
/// （历史：只有 boxplot 读取，其余图表一律走调色板）。
#[test]
fn series_item_style_color_overrides_palette() {
    let bars = bar_rects(
        r##"{"xAxis":{"type":"category","data":["a","b"]},"yAxis":{"type":"value"},
            "series":[{"type":"bar","itemStyle":{"color":"#ff0000"},"data":[10,20]}]}"##,
    );
    let red = liecharts::Color::rgb(255, 0, 0);
    for (r, style) in &bars {
        assert_eq!(
            style.fill,
            Some(Fill::Solid(red)),
            "柱体应使用 itemStyle.color：rect={r:?}"
        );
    }
}

/// null 数据点：默认在 null 处**断开**折线且不画符号；
/// `connectNulls:true` 时连成一条。
#[test]
fn null_data_points_break_line_unless_connect_nulls() {
    let count_move_to = |json: &str| -> usize {
        let nodes = render_json(json, 800, 500);
        paths(&nodes)
            .into_iter()
            .map(|(p, _, _)| {
                p.elements()
                    .iter()
                    .filter(|e| matches!(e, PathEl::MoveTo(_)))
                    .count()
            })
            .sum()
    };

    let broken = count_move_to(
        r#"{"xAxis":{"type":"category","data":["a","b","c","d","e"]},"yAxis":{"type":"value"},
            "series":[{"type":"line","data":[10,20,null,30,40]}]}"#,
    );
    let connected = count_move_to(
        r#"{"xAxis":{"type":"category","data":["a","b","c","d","e"]},"yAxis":{"type":"value"},
            "series":[{"type":"line","connectNulls":true,"data":[10,20,null,30,40]}]}"#,
    );
    assert!(
        broken > connected,
        "默认应在 null 处断开（子路径更多）：broken={broken} connected={connected}"
    );
}

/// 热力图 `series[].label.show` 渲染单元格数值。
#[test]
fn heatmap_label_renders_cell_values() {
    let nodes = render_json(
        r#"{"xAxis":{"type":"category","data":["a"]},"yAxis":{"type":"category","data":["x"]},
            "series":[{"type":"heatmap","label":{"show":true},"data":[[0,0,42]]}]}"#,
        600,
        400,
    );
    let t = all_texts(&nodes);
    assert!(t.iter().any(|s| s == "42"), "热力图应显示单元格数值：{t:?}");
}

/// 非笛卡尔坐标系的 heatmap（`calendar`）不再整图报错。
#[test]
fn heatmap_non_cartesian_coordinate_system_does_not_fail() {
    let nodes = render_json(
        r#"{"calendar":{"range":"2026-01"},
            "series":[{"type":"heatmap","coordinateSystem":"calendar",
                       "data":[["2026-01-01",1]]}]}"#,
        600,
        400,
    );
    // 不支持的坐标系 → 静默跳过（与未知 series type 一致），不应 panic/报错
    assert!(
        ChartBuilder::from_option_json(
            r#"{"calendar":{"range":"2026-01"},"series":[{"type":"heatmap",
               "coordinateSystem":"calendar","data":[["2026-01-01",1]]}]}"#
        )
        .is_ok()
    );
    let _ = nodes;
}

/// K 线 `itemStyle.color/color0` 生效（历史：完全忽略，只能用主题色）。
#[test]
fn candlestick_item_style_colors_are_applied() {
    let nodes = render_json(
        r##"{"xAxis":{"type":"category","data":["d1"]},"yAxis":{"type":"value"},
            "series":[{"type":"candlestick",
                       "itemStyle":{"color":"#ff0000","color0":"#00ff00"},
                       "data":[[20,34,10,38]]}]}"##,
        800,
        500,
    );
    let red = liecharts::Color::rgb(255, 0, 0);
    let filled: Vec<_> = rects(&nodes)
        .into_iter()
        .filter(|(r, _)| r.width() < 700.0 && r.height() < 450.0)
        .collect();
    assert_eq!(filled.len(), 1, "应有一根 K 线实体");
    assert_eq!(
        filled[0].1.fill,
        Some(Fill::Solid(red)),
        "涨（阳线）应使用 itemStyle.color 且实心填充"
    );
}

/// 饼图 `labelLine.show:false` 不画引导线。
#[test]
fn pie_label_line_show_false_removes_leader_lines() {
    let count_stroked_paths = |json: &str| -> usize {
        paths(&render_json(json, 600, 400))
            .into_iter()
            .filter(|(_, s, _)| s.stroke.is_some())
            .count()
    };
    let on = count_stroked_paths(
        r#"{"series":[{"type":"pie","label":{"show":true},
            "data":[{"name":"a","value":1},{"name":"b","value":2}]}]}"#,
    );
    let off = count_stroked_paths(
        r#"{"series":[{"type":"pie","label":{"show":true},"labelLine":{"show":false},
            "data":[{"name":"a","value":1},{"name":"b","value":2}]}]}"#,
    );
    assert!(
        off < on,
        "labelLine.show:false 应去掉引导线：on={on} off={off}"
    );
}

/// 玫瑰图 `roseType:'radius'` 按数值缩放扇区半径。
#[test]
fn pie_rose_type_scales_sector_radius() {
    let json = |rose: &str| -> String {
        format!(
            r#"{{"series":[{{"type":"pie","roseType":"{rose}","radius":["0%","70%"],
                "data":[{{"name":"a","value":1}},{{"name":"b","value":4}}]}}]}}"#
        )
    };
    // 取**最小**扇区的包围盒：玫瑰图会按数值缩缩小值扇区的半径，
    // 而整体包围盒由大值扇区决定（对 roseType 不敏感）
    let min_sector_extent = |rose: &str| -> f64 {
        let nodes = render_json(&json(rose), 600, 400);
        let mut sizes: Vec<f64> = paths(&nodes)
            .into_iter()
            .filter(|(_, s, _)| s.fill.is_some())
            .map(|(p, _, _)| {
                let b = p.bounding_box();
                b.width().max(b.height())
            })
            .collect();
        sizes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sizes.first().copied().unwrap_or(0.0)
    };

    let plain = min_sector_extent("");
    let rose = min_sector_extent("radius");
    assert!(rose > 0.0, "玫瑰图应正常渲染，实际 {rose}");
    assert!(
        rose < plain * 0.7,
        "roseType 应明显缩小小值扇区半径：rose={rose} plain={plain}"
    );
}
