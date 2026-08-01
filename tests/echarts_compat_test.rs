//! ECharts 兼容性集成测试
//!
//! 验证从 ECharts JSON 配置文件到 ChartOption 的解析，
//! 以及到 ChartSpec 的转换是否正常。

use liecharts::option::{ChartOption, SeriesOption, TooltipTrigger};
use liecharts::pipeline::compat::chart_option_to_chart_spec;

/// 读取 JSON 文件内容
fn read_json(name: &str) -> String {
    let path = format!("site/examples/{}.json", name);
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("无法读取文件: {}", path))
}

/// 解析 JSON 为 ChartOption
fn parse_json(name: &str) -> ChartOption {
    let json = read_json(name);
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("无法解析 JSON: {} - {}", name, e))
}

/// 判断 SeriesOption 的类型名称
fn series_type_name(s: &SeriesOption) -> &'static str {
    match s {
        SeriesOption::Line(_) => "line",
        SeriesOption::Bar(_) => "bar",
        SeriesOption::Pie(_) => "pie",
        SeriesOption::Scatter(_) => "scatter",
        SeriesOption::Radar(_) => "radar",
        SeriesOption::PolarBar(_) => "polarBar",
        SeriesOption::PolarScatter(_) => "polarScatter",
        SeriesOption::Bubble(_) => "bubble",
        SeriesOption::Gauge(_) => "gauge",
        SeriesOption::Candlestick(_) => "candlestick",
        SeriesOption::Boxplot(_) => "boxplot",
        SeriesOption::Table(_) => "table",
        SeriesOption::Unknown => "unknown",
    }
}

/// 解析并转换为 ChartSpec
fn parse_and_convert(name: &str) {
    let option: ChartOption = parse_json(name);
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert!(!spec.series.is_empty(), "{}: 转换后的 series 为空", name);
}

// ── 各图表类型测试 ──

#[test]
fn test_line_chart() {
    let option = parse_json("line");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("月度趋势图"));
    assert_eq!(option.series.len(), 2);
    assert_eq!(series_type_name(&option.series[0]), "line");
    assert_eq!(series_type_name(&option.series[1]), "line");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 2);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_bar_chart() {
    let option = parse_json("bar");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("月度销售数据"));
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "bar");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_pie_chart() {
    let option = parse_json("pie");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("访问来源"));
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "pie");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
}

#[test]
fn test_radar_chart() {
    let option = parse_json("radar");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("产品能力雷达图"));
    assert_eq!(option.series.len(), 2);
    assert_eq!(series_type_name(&option.series[0]), "radar");
    assert_eq!(series_type_name(&option.series[1]), "radar");
    assert!(option.radar.is_some());
    let radar = option.radar.as_ref().unwrap();
    assert!(radar.indicator.is_some());
    assert_eq!(radar.indicator.as_ref().unwrap().len(), 5);
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 2);
}

#[test]
fn test_scatter_chart() {
    let option = parse_json("scatter");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("散点图示例"));
    assert_eq!(option.series.len(), 2);
    assert_eq!(series_type_name(&option.series[0]), "scatter");
    assert_eq!(series_type_name(&option.series[1]), "scatter");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 2);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_gauge_chart() {
    let option = parse_json("gauge");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("仪表盘示例"));
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "gauge");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
}

#[test]
fn test_bubble_chart() {
    let option = parse_json("bubble");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("编程语言数据分析"));
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "bubble");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_polar_bar_chart() {
    let option = parse_json("polar_bar");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("极坐标柱状图"));
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "polarBar");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
}

#[test]
fn test_polar_scatter_chart() {
    let option = parse_json("polar_scatter");
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "polarScatter");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
}

#[test]
fn test_candlestick_chart() {
    let option = parse_json("candlestick");
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "candlestick");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_boxplot_chart() {
    let option = parse_json("boxplot");
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "boxplot");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
    // 验证五元数组被正确解析为 5 行数据
    let series = &spec.series[0];
    assert_eq!(series.data.row_count(), 5);
    assert!(series.data.get_column("min").is_some());
    assert!(series.data.get_column("q1").is_some());
    assert!(series.data.get_column("median").is_some());
    assert!(series.data.get_column("q3").is_some());
    assert!(series.data.get_column("max").is_some());
}

#[test]
fn test_area_chart() {
    let option = parse_json("area");
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "line");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_stacked_area_chart() {
    let option = parse_json("stacked_area");
    assert!(option.series.len() >= 2);
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert!(spec.series.len() >= 2);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_dual_y_axis_chart() {
    let option = parse_json("dual_y_axis");
    assert!(option.series.len() >= 2);
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert!(spec.series.len() >= 2);
    assert!(spec.y_axes.len() >= 2);
}

#[test]
fn test_mixed_chart() {
    let option = parse_json("mixed");
    assert!(option.series.len() >= 2);
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert!(spec.series.len() >= 2);
    assert_eq!(spec.x_axes.len(), 1);
    assert!(spec.y_axes.len() >= 2);
}

#[test]
fn test_table_chart() {
    let option = parse_json("table");
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "table");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
}

// ── 增强配置测试（tooltip、legend、visualMap、dataZoom 等） ──

#[test]
fn test_line_with_tooltip_and_mark() {
    let option = parse_json("line_with_tooltip_and_mark");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("带标注的折线图"));
    assert!(option.tooltip.is_some());
    let tooltip = option.tooltip.as_ref().unwrap();
    assert!(matches!(tooltip.trigger.as_ref().unwrap(), TooltipTrigger::Axis));
    assert!(tooltip.axis_pointer.is_some());
    assert!(option.legend.is_some());
    assert_eq!(option.series.len(), 2);
    assert_eq!(series_type_name(&option.series[0]), "line");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 2);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_bar_with_visual_map() {
    let option = parse_json("bar_with_visual_map");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("带 visualMap 的柱状图"));
    assert!(option.tooltip.is_some());
    assert!(option.visual_map.is_some());
    let vm_slice = option.visual_map.as_ref().unwrap().as_slice();
    assert!(!vm_slice.is_empty());
    let vm = &vm_slice[0];
    assert_eq!(vm.min, Some(0.0));
    assert_eq!(vm.max, Some(300.0));
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "bar");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_pie_rose() {
    let option = parse_json("pie_rose");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("南丁格尔玫瑰图"));
    assert!(option.tooltip.is_some());
    assert!(option.legend.is_some());
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "pie");
    if let SeriesOption::Pie(pie) = &option.series[0] {
        assert_eq!(pie.rose_type.as_deref(), Some("radius"));
        assert!(pie.radius.is_some());
    } else {
        panic!("应为 pie 类型");
    }
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
}

#[test]
fn test_stacked_bar() {
    let option = parse_json("stacked_bar");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("堆叠柱状图"));
    assert!(option.tooltip.is_some());
    assert!(option.legend.is_some());
    assert_eq!(option.series.len(), 3);
    for s in &option.series {
        assert_eq!(series_type_name(s), "bar");
    }
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 3);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_area_smooth() {
    let option = parse_json("area_smooth");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("平滑面积图"));
    assert!(option.tooltip.is_some());
    assert!(option.legend.is_some());
    assert_eq!(option.series.len(), 3);
    for s in &option.series {
        assert_eq!(series_type_name(s), "line");
    }
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 3);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_radar_multi() {
    let option = parse_json("radar_multi");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("多雷达对比图"));
    assert!(option.tooltip.is_some());
    assert!(option.legend.is_some());
    assert!(option.radar.is_some());
    let radar = option.radar.as_ref().unwrap();
    assert_eq!(radar.indicator.as_ref().unwrap().len(), 6);
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "radar");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
}

#[test]
fn test_scatter_datazoom() {
    let option = parse_json("scatter_datazoom");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("散点图 - 身高体重分布"));
    assert!(option.tooltip.is_some());
    assert!(option.legend.is_some());
    assert!(option.visual_map.is_some());
    assert!(option.data_zoom.is_some());
    let dz = option.data_zoom.as_ref().unwrap();
    assert_eq!(dz.len(), 2);
    assert_eq!(option.series.len(), 2);
    assert_eq!(series_type_name(&option.series[0]), "scatter");
    assert_eq!(series_type_name(&option.series[1]), "scatter");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 2);
    assert_eq!(spec.x_axes.len(), 1);
    assert_eq!(spec.y_axes.len(), 1);
}

#[test]
fn test_gauge_detailed() {
    let option = parse_json("gauge_detailed");
    assert_eq!(option.title.as_ref().unwrap().text.as_deref(), Some("仪表盘 - 多指标"));
    assert!(option.tooltip.is_some());
    assert_eq!(option.series.len(), 1);
    assert_eq!(series_type_name(&option.series[0]), "gauge");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
}

// ── 批量兼容性测试 ──

const ALL_CHART_FILES: &[&str] = &[
    "line", "bar", "pie", "radar", "scatter", "gauge", "bubble",
    "polar_bar", "polar_scatter", "candlestick", "boxplot", "area", "stacked_area",
    "dual_y_axis", "mixed", "table",
    "line_with_tooltip_and_mark", "bar_with_visual_map", "pie_rose",
    "stacked_bar", "area_smooth", "radar_multi", "scatter_datazoom",
    "gauge_detailed", "polymorphic_fields",
];

#[test]
fn test_all_charts_parse() {
    for name in ALL_CHART_FILES {
        let option: ChartOption = parse_json(name);
        assert!(!option.series.is_empty(), "{}: series 为空", name);
    }
}

#[test]
fn test_all_charts_convert() {
    for name in ALL_CHART_FILES {
        parse_and_convert(name);
    }
}

/// 测试 dataset 数据源 + encode 映射
#[test]
fn test_dataset_with_encode() {
    let json = r#"{
        "dataset": {
            "source": [
                ["product", "2015", "2016", "2017"],
                ["Matcha Latte", 43.3, 85.8, 93.7],
                ["Milk Tea", 83.1, 73.4, 55.1],
                ["Cheese Cocoa", 86.4, 65.2, 82.5],
                ["Walnut Brownie", 72.4, 53.9, 39.1]
            ]
        },
        "xAxis": { "type": "category" },
        "yAxis": { "type": "value" },
        "series": [
            { "type": "bar", "name": "2015", "datasetIndex": 0, "encode": { "x": 0, "y": 1 } },
            { "type": "bar", "name": "2016", "datasetIndex": 0, "encode": { "x": 0, "y": 2 } },
            { "type": "bar", "name": "2017", "datasetIndex": 0, "encode": { "x": 0, "y": 3 } }
        ]
    }"#;

    let option: ChartOption = serde_json::from_str(json).expect("dataset JSON 解析失败");
    assert!(option.dataset.is_some(), "dataset 字段应存在");
    assert_eq!(option.dataset.as_ref().unwrap().as_slice().len(), 1);

    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 3, "应有 3 个系列");

    // 验证每个系列的数据来自 dataset
    for (i, series) in spec.series.iter().enumerate() {
        assert!(
            series.data.row_count() > 0,
            "系列 {} 的数据不应为空",
            i
        );
        // 验证 x 列有 4 个品类名
        let x_col = series.data.get_column("x").unwrap();
        assert_eq!(x_col.len(), 4, "系列 {} 应有 4 行数据", i);
    }

    // 验证第一系列数据：2015 年
    let s0 = &spec.series[0];
    let x_vals: Vec<String> = (0..s0.data.row_count())
        .filter_map(|i| s0.data.get_column("x").and_then(|c| c.as_string(i)))
        .collect();
    assert_eq!(x_vals, vec!["Matcha Latte", "Milk Tea", "Cheese Cocoa", "Walnut Brownie"]);

    let y_vals: Vec<f64> = (0..s0.data.row_count())
        .filter_map(|i| s0.data.get_column("y").and_then(|c| c.as_f64(i)))
        .collect();
    assert_eq!(y_vals, vec![43.3, 83.1, 86.4, 72.4]);
}

/// 测试 dataset 无 header（source_header: false）的情况
#[test]
fn test_dataset_no_header() {
    let json = r#"{
        "dataset": {
            "sourceHeader": false,
            "source": [
                ["Jan", 120],
                ["Feb", 200],
                ["Mar", 150],
                ["Apr", 80]
            ]
        },
        "xAxis": { "type": "category" },
        "yAxis": { "type": "value" },
        "series": [
            { "type": "bar", "name": "Sales", "datasetIndex": 0, "encode": { "x": 0, "y": 1 } }
        ]
    }"#;

    let option: ChartOption = serde_json::from_str(json).expect("无 header 的 dataset JSON 解析失败");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
    let series = &spec.series[0];
    assert_eq!(series.data.row_count(), 4);

    // 列名应为 column0, column1
    let x_vals: Vec<String> = (0..series.data.row_count())
        .filter_map(|i| series.data.get_column("x").and_then(|c| c.as_string(i)))
        .collect();
    assert_eq!(x_vals, vec!["Jan", "Feb", "Mar", "Apr"]);
}

/// 测试 dataset 使用 encode.value 和 encode.itemName（饼图场景）
#[test]
fn test_dataset_with_pie_encode() {
    let json = r#"{
        "dataset": {
            "source": [
                ["category", "value"],
                ["A", 30],
                ["B", 50],
                ["C", 20]
            ]
        },
        "series": [
            {
                "type": "pie",
                "name": "Pie",
                "datasetIndex": 0,
                "encode": { "itemName": 0, "value": 1 }
            }
        ]
    }"#;

    let option: ChartOption = serde_json::from_str(json).expect("饼图 dataset JSON 解析失败");
    let spec = chart_option_to_chart_spec(&option, 800, 600);
    assert_eq!(spec.series.len(), 1);
    let series = &spec.series[0];
    assert_eq!(series.data.row_count(), 3);

    // 验证 itemName 列映射到 x
    let x_vals: Vec<String> = (0..series.data.row_count())
        .filter_map(|i| series.data.get_column("x").and_then(|c| c.as_string(i)))
        .collect();
    assert_eq!(x_vals, vec!["A", "B", "C"]);

    // 验证 value 列映射到 y
    let y_vals: Vec<f64> = (0..series.data.row_count())
        .filter_map(|i| series.data.get_column("y").and_then(|c| c.as_f64(i)))
        .collect();
    assert_eq!(y_vals, vec![30.0, 50.0, 20.0]);
}

// ═══════════════════════════════════════════════════════════════════
// 容错回归测试 — 验证 ECharts JSON 输入不报错
// ═══════════════════════════════════════════════════════════════════
// 这些测试对应 examples/diagnose_compat.rs 中识别出的失败模式。
// 目标：任意 LLM 输出的 ECharts JSON 都能解析成功，并尽可能渲染。

mod tolerance_tests {
    use super::*;
    use liecharts::option::LegendDataItem;

    /// 12 种未知 series 类型都应该被解析为 `SeriesOption::Unknown`，而不是报错。
    /// （boxplot 已被实现为正式支持的类型，故从列表中移除。）
    #[test]
    fn test_tolerates_unknown_series_types() {
        for ty in [
            "heatmap", "funnel", "treemap", "sunburst", "sankey", "graph", "tree",
            "effectScatter", "pictorialBar", "parallel", "themeRiver", "custom",
        ] {
            let json = format!(
                r#"{{ "series": [ {{ "type": "{}", "data": [] }} ] }}"#,
                ty
            );
            let opt: ChartOption = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("未知 series 类型 {} 不应报错: {}", ty, e));
            assert!(
                matches!(opt.series[0], SeriesOption::Unknown),
                "type={} 应解析为 Unknown",
                ty
            );
        }
    }

    /// dataset + encode 使用字符串列名（ECharts 5 推荐用法）。
    #[test]
    fn test_tolerates_dataset_string_encode() {
        let json = r#"{
            "dataset": {
                "source": [
                    ["product","2015","2016"],
                    ["Matcha Latte", 43.3, 85.8],
                    ["Milk Tea", 83.1, 73.4]
                ]
            },
            "xAxis": {"type":"category"},
            "yAxis": {},
            "series": [
                {"type":"bar","datasetIndex":0,"encode":{"x":"product","y":"2015"}}
            ]
        }"#;
        let opt: ChartOption =
            serde_json::from_str(json).expect("dataset 字符串 encode 应能解析");
        let spec = chart_option_to_chart_spec(&opt, 800, 600);
        assert_eq!(spec.series.len(), 1);
        assert_eq!(spec.series[0].data.row_count(), 2);
    }

    /// color 数组包含 "auto"、渐变对象和 CSS 关键字 —— 都不应报错。
    #[test]
    fn test_tolerates_color_auto_and_gradient() {
        let json = r##"{
            "color": ["auto", {"type":"linear","colorStops":[{"offset":0,"color":"#5470c6"},{"offset":1,"color":"#91cc75"}]}, "red"],
            "series": [{"type":"bar","data":[1,2,3]}]
        }"##;
        let opt: ChartOption =
            serde_json::from_str(json).expect("color auto/gradient/keyword 应能解析");
        assert!(opt.color.is_some());
    }

    /// axisLine.symbol 可以是字符串数组 `["none", "arrow"]`。
    #[test]
    fn test_tolerates_axis_line_symbol_array() {
        let json = r#"{
            "xAxis": {
                "type":"category",
                "data":["a","b"],
                "axisLine":{"onZero":true,"symbol":["none","arrow"]}
            }
        }"#;
        let opt: ChartOption =
            serde_json::from_str(json).expect("axisLine.symbol 数组应能解析");
        // 转换为 ChartSpec 也不应报错
        let _spec = chart_option_to_chart_spec(&opt, 800, 600);
    }

    /// legend.data 可以混合字符串和 `{name, icon}` 对象。
    #[test]
    fn test_tolerates_legend_data_objects() {
        let json = r#"{
            "legend": {
                "data":[{"name":"a","icon":"rect"},{"name":"b"},"c"]
            }
        }"#;
        let opt: ChartOption =
            serde_json::from_str(json).expect("legend.data 含对象应能解析");
        let data = opt.legend.unwrap().data.unwrap();
        assert_eq!(data.len(), 3);
        // {"name":"a","icon":"rect"} 和 {"name":"b"} 都是对象，"c" 是字符串
        assert!(matches!(data[0], LegendDataItem::Object { .. }));
        assert!(matches!(data[1], LegendDataItem::Object { .. }));
        assert!(matches!(data[2], LegendDataItem::Str(_)));
        // 名称提取
        assert_eq!(data[0].name(), "a");
        assert_eq!(data[1].name(), "b");
        assert_eq!(data[2].name(), "c");
    }

    /// series.data 支持 null、`{value:5}`、`{value:[x,y]}` 等混合形式。
    #[test]
    fn test_tolerates_datapoint_null_and_array_value() {
        let json = r#"{
            "series": [
                {"type":"line","data":[1, null, 3, {"value":5}, {"value":[6,7],"name":"x"}]}
            ]
        }"#;
        let opt: ChartOption =
            serde_json::from_str(json).expect("含 null / 数组 value 的 series.data 应能解析");
        assert_eq!(opt.series.len(), 1);
    }

    /// dataZoom.handleSize 可以是百分比字符串 `"100%"`。
    #[test]
    fn test_tolerates_handle_size_percent() {
        let json = r#"{
            "dataZoom": [{
                "type":"slider",
                "handleSize":"100%",
                "start":0,
                "end":100
            }]
        }"#;
        let opt: ChartOption =
            serde_json::from_str(json).expect("dataZoom.handleSize 百分比应能解析");
        assert!(opt.data_zoom.is_some());
    }

    /// axisLabel.interval 可以是字符串 `"auto"` 或数字。
    #[test]
    fn test_tolerates_axis_label_interval_auto() {
        let json = r#"{
            "xAxis": {
                "type":"category",
                "data":["a","b","c"],
                "axisLabel":{"interval":"auto"}
            }
        }"#;
        let opt: ChartOption =
            serde_json::from_str(json).expect("axisLabel.interval='auto' 应能解析");
        let _spec = chart_option_to_chart_spec(&opt, 800, 600);
    }

    /// ChartOption.color 可以是单个字符串而非数组。
    #[test]
    fn test_color_field_accepts_single_string() {
        let json = r##"{ "color": "#c23531", "series": [{"type":"bar","data":[1,2]}] }"##;
        let opt: ChartOption =
            serde_json::from_str(json).expect("color 单值字符串应能解析");
        assert!(opt.color.is_some());
        let _spec = chart_option_to_chart_spec(&opt, 800, 600);
    }

    /// 顶层未知字段（toolbox/polar/geo/graphic/aria/axisPointer/calendar 等）应被静默忽略。
    #[test]
    fn test_unknown_top_level_fields_ignored() {
        let json = r#"{
            "toolbox": {"feature":{"saveAsImage":{}}},
            "polar": {},
            "geo": {"map":"china"},
            "singleAxis": {},
            "parallelAxis": [{}],
            "calendar": {},
            "graphic": {"type":"text"},
            "aria": {"enabled":true},
            "axisPointer": {},
            "series": [{"type":"bar","data":[1,2]}]
        }"#;
        let opt: ChartOption =
            serde_json::from_str(json).expect("顶层未知字段应被忽略不报错");
        assert_eq!(opt.series.len(), 1);
    }

    /// 富 line series 含 emphasis/blur/select/universalTransition/animation 等 ECharts 5 字段。
    /// `animation` 字段已应用 `LenientBool`，可接受 bool 或 "true"/"false" 字符串。
    #[test]
    fn test_tolerates_rich_line_series_fields() {
        let json = r#"{
            "series": [{
                "type":"line",
                "name":"a",
                "data":[1,2,3],
                "emphasis": {"focus":"series"},
                "blur": {"focus":"series"},
                "select": {"focus":"series"},
                "colorBy":"series",
                "universalTransition": {"enabled":true},
                "dimensions":[{"name":"x"}],
                "clip":true,
                "progressive":200,
                "progressiveThreshold":1000,
                "id":"s1",
                "animation":"true",
                "animationEasing":"cubicOut",
                "animationDelay":0,
                "animationDurationUpdate":300,
                "animationEasingUpdate":"cubicOut",
                "animationDelayUpdate":0
            }]
        }"#;
        let opt: ChartOption = serde_json::from_str(json)
            .expect("富 line series 字段应能解析（含 animation:\"true\" 字符串 bool）");
        assert_eq!(opt.series.len(), 1);
    }

    /// 字符串 bool 字段（如 `"animation":"true"`）应被 LenientBool 接受。
    /// LLM 偶发输出字符串形式的 bool，不应让解析失败。
    #[test]
    fn test_tolerates_string_bool_in_animation() {
        // 字符串 "true"
        let json = r#"{ "series": [{"type":"bar","data":[1,2],"animation":"true"}] }"#;
        let opt: ChartOption =
            serde_json::from_str(json).expect("animation:\"true\" 应能解析");
        match &opt.series[0] {
            SeriesOption::Bar(b) => {
                let anim = b.animation.as_ref().expect("animation 字段应被设置");
                assert!(anim.0, "字符串 \"true\" 应解析为 true");
            }
            _ => panic!("期望 Bar 系列"),
        }

        // 字符串 "false"
        let json_false = r#"{ "series": [{"type":"bar","data":[1,2],"animation":"false"}] }"#;
        let opt: ChartOption =
            serde_json::from_str(json_false).expect("animation:\"false\" 应能解析");
        match &opt.series[0] {
            SeriesOption::Bar(b) => {
                let anim = b.animation.as_ref().expect("animation 字段应被设置");
                assert!(!anim.0, "字符串 \"false\" 应解析为 false");
            }
            _ => panic!("期望 Bar 系列"),
        }

        // 真 bool true
        let json_bool = r#"{ "series": [{"type":"bar","data":[1,2],"animation":true}] }"#;
        let opt: ChartOption =
            serde_json::from_str(json_bool).expect("animation:true 应能解析");
        match &opt.series[0] {
            SeriesOption::Bar(b) => {
                let anim = b.animation.as_ref().expect("animation 字段应被设置");
                assert!(anim.0, "真 bool true 应解析为 true");
            }
            _ => panic!("期望 Bar 系列"),
        }
    }
}
