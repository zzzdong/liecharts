//! 诊断 ECharts JSON 兼容性问题
//!
//! 测试常见的 LLM 输出场景，找出解析失败的点。
use liecharts::option::ChartOption;

fn check(name: &str, json: &str) {
    match serde_json::from_str::<ChartOption>(json) {
        Ok(_) => println!("[OK]   {}", name),
        Err(e) => println!("[FAIL] {}  --  {}", name, e),
    }
}

fn main() {
    // ── 1. 未知 series 类型（LLM 经常输出）──
    check(
        "heatmap",
        r##"{ "series": [ {"type":"heatmap","data":[[0,0,1]]} ] }"##,
    );
    check(
        "funnel",
        r##"{ "series": [ {"type":"funnel","data":[{ "name":"a","value":10 }]} ] }"##,
    );
    check(
        "treemap",
        r##"{ "series": [ {"type":"treemap","data":[]} ] }"##,
    );
    check(
        "sunburst",
        r##"{ "series": [ {"type":"sunburst","data":[]} ] }"##,
    );
    check(
        "sankey",
        r##"{ "series": [ {"type":"sankey","data":[],"links":[]} ] }"##,
    );
    check("graph", r##"{ "series": [ {"type":"graph","data":[]} ] }"##);
    check("tree", r##"{ "series": [ {"type":"tree","data":[]} ] }"##);
    check(
        "boxplot",
        r##"{ "series": [ {"type":"boxplot","data":[]} ] }"##,
    );
    check(
        "effectScatter",
        r##"{ "series": [ {"type":"effectScatter","data":[]} ] }"##,
    );
    check(
        "pictorialBar",
        r##"{ "series": [ {"type":"pictorialBar","data":[]} ] }"##,
    );
    check(
        "parallel",
        r##"{ "series": [ {"type":"parallel","data":[]} ] }"##,
    );
    check(
        "themeRiver",
        r##"{ "series": [ {"type":"themeRiver","data":[]} ] }"##,
    );
    check(
        "custom",
        r##"{ "series": [ {"type":"custom","renderItem":{}} ] }"##,
    );

    // ── 2. 顶层未知组件 ──
    check(
        "toolbox",
        r##"{ "toolbox": {"feature":{"saveAsImage":{}}} }"##,
    );
    check("polar", r##"{ "polar": {}, "series": [{"type":"bar"}] }"##);
    check("geo", r##"{ "geo": {"map":"china"} }"##);
    check("singleAxis", r##"{ "singleAxis": {} }"##);
    check("parallelAxis", r##"{ "parallelAxis": [{}] }"##);
    check("calendar", r##"{ "calendar": {} }"##);
    check("graphic", r##"{ "graphic": {"type":"text"} }"##);
    check("aria", r##"{ "aria": {"enabled":true} }"##);
    check("axisPointer_top", r##"{ "axisPointer": {} }"##);

    // ── 3. dataset + encode（ECharts 5 推荐用法）──
    check(
        "dataset_with_encode",
        r##"{
            "dataset": {
                "source": [
                    ["product","2015","2016","2017"],
                    ["Matcha Latte", 43.3, 85.8, 93.7],
                    ["Milk Tea", 83.1, 73.4, 55.1]
                ]
            },
            "xAxis": {"type":"category"},
            "yAxis": {},
            "series": [
                {"type":"bar","datasetIndex":0,"encode":{"x":"product","y":"2015"}}
            ]
        }"##,
    );

    // ── 4. 富 tooltip ──
    check(
        "rich_tooltip",
        r##"{
            "tooltip": {
                "trigger":"axis",
                "axisPointer":{"type":"cross","label":{"backgroundColor":"#6a7985"}},
                "formatter": "{a}: {c}"
            }
        }"##,
    );

    // ── 5. 富 legend ──
    check(
        "rich_legend",
        r##"{
            "legend": {
                "type":"scroll",
                "top":"top",
                "feature":{},
                "data":["a","b"],
                "textStyle":{"color":"#333"},
                "pageButtonItemGap":5
            }
        }"##,
    );

    // ── 6. 富 series itemStyle ──
    check(
        "series_with_rich_fields",
        r##"{
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
        }"##,
    );

    // ── 7. 富 axis ──
    check(
        "rich_axis",
        r##"{
            "xAxis": {
                "type":"category",
                "data":["a","b"],
                "axisLine":{"onZero":true,"symbol":["none","arrow"]},
                "axisTick":{"alignWithLabel":true},
                "axisLabel":{"rotate":30,"interval":0,"formatter":"{value}"},
                "splitLine":{"show":false},
                "splitArea":{"show":false},
                "minInterval":0,
                "maxInterval":null,
                "interval":1,
                "boundaryGap":false,
                "inverse":false
            }
        }"##,
    );

    // ── 8. 富 title ──
    check(
        "rich_title",
        r##"{
            "title": {
                "text":"a","subtext":"b",
                "left":"center","top":"top",
                "backgroundColor":"#fff",
                "borderColor":"#ccc",
                "borderWidth":1,
                "padding":[5,10],
                "itemGap":10,
                "textStyle":{"color":"#333","fontStyle":"italic","fontWeight":"bold","fontSize":18},
                "subtextStyle":{"color":"#aaa","fontSize":12}
            }
        }"##,
    );

    // ── 9. 富 grid ──
    check(
        "rich_grid",
        r##"{
            "grid": {
                "left":"10%","right":"10%","top":"15%","bottom":"15%",
                "containLabel":true,
                "backgroundColor":"#fff",
                "borderColor":"#ccc",
                "borderWidth":1,
                "show":true,
                "z":2,
                "tooltip":{"show":true}
            }
        }"##,
    );

    // ── 10. label 富字段 ──
    check(
        "rich_label",
        r##"{
            "series": [{
                "type":"bar",
                "data":[1,2,3],
                "label":{"show":true,"position":"top","formatter":"{c}","color":"auto","fontFamily":"sans-serif"}
            }]
        }"##,
    );

    // ── 11. dataZoom 富字段 ──
    check(
        "rich_datazoom",
        r##"{
            "dataZoom": [{
                "type":"slider",
                "show":true,
                "realtime":true,
                "orient":"horizontal",
                "filterMode":"filter",
                "throttle":100,
                "start":0,
                "end":100,
                "zoomLock":false,
                "left":"center",
                "handleSize":"100%",
                "handleStyle":{"color":"#fff","borderColor":"#000"},
                "moveHandleSize":7,
                "emphasis":{"handleStyle":{"borderColor":"#1e90ff"}},
                "textStyle":{"color":"#6a7985"},
                "dataBackground":{"lineStyle":{"color":"#d2d2d2"},"areaStyle":{"color":"#d2d2d2"}},
                "selectedDataBackground":{"lineStyle":{"color":"#333"},"areaStyle":{"color":"#333"}}
            }]
        }"##,
    );

    // ── 12. visualMap 富字段 ──
    check(
        "rich_visualmap",
        r##"{
            "visualMap": {
                "type":"continuous",
                "min":0,"max":100,
                "calculable":true,
                "orient":"horizontal",
                "left":"center",
                "bottom":"5%",
                "inRange":{"color":["#50a3ba","#eac736","#d94e5d"]},
                "text":["High","Low"],
                "realtime":true
            }
        }"##,
    );

    // ── 13. MarkLine 富字段 ──
    check(
        "rich_markline",
        r##"{
            "series": [{
                "type":"line",
                "data":[1,2,3],
                "markLine":{
                    "data":[{"type":"average", "name":"Avg"}],
                    "lineStyle":{"type":"solid"},
                    "label":{"show":true},
                    "symbol":["none","none"]
                }
            }]
        }"##,
    );

    // ── 14. 单值 vs 数组 (grid/axis) ──
    check("single_grid", r##"{ "grid": {"left":"10%"} }"##);
    check("single_xaxis", r##"{ "xAxis": {"type":"value"} }"##);
    check("array_grid", r##"{ "grid": [{"left":"10%"}] }"##);
    check("array_xaxis", r##"{ "xAxis": [{"type":"value"}] }"##);

    // ── 15. 边界 case: 空对象 ──
    check("empty_object", r##"{ }"##);
    check("null_field", r##"{ "title": null }"##);

    // ── 16. series data 各种格式 ──
    check(
        "data_numbers",
        r##"{ "series": [{"type":"line","data":[1,2,3]}] }"##,
    );
    check(
        "data_xy_arrays",
        r##"{ "series": [{"type":"scatter","data":[[1,2],[3,4]]}] }"##,
    );
    check(
        "data_named_objs",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10},{"name":"b","value":20}]}] }"##,
    );
    check(
        "data_value_array_objs",
        r##"{ "series": [{"type":"scatter","data":[{"value":[1,2],"name":"x"}]}] }"##,
    );
    check(
        "data_mixed",
        r##"{ "series": [{"type":"line","data":[1,null,3,{"value":5}]}] }"##,
    );

    // ── 17. color 单值 vs 数组 ──
    check(
        "color_single_string",
        r##"{ "color": "#5470c6", "series": [{"type":"bar","data":[1,2]}] }"##,
    );
    check(
        "color_array",
        r##"{ "color": ["#5470c6","#91cc75"], "series": [{"type":"bar","data":[1,2]}] }"##,
    );

    // ── 18. axisLabel/axisLine 富字段 ──
    check(
        "axis_label_array_color",
        r##"{ "xAxis": {"axisLabel":{"color":["#333","#666"]}} }"##,
    );
    check(
        "axis_label_string_int",
        r##"{ "xAxis": {"axisLabel":{"interval":"auto"}} }"##,
    );

    // ── 19. 富 legend.data: 含对象数组 ──
    check(
        "legend_data_with_objects",
        r##"{ "legend": {"data":[{"name":"a","icon":"rect"},{"name":"b"}]} }"##,
    );

    // ── 20. 富 itemStyle: linear gradient ──
    check(
        "item_style_gradient",
        r##"{
            "series": [{
                "type":"bar",
                "data":[1,2,3],
                "itemStyle":{"color":{"type":"linear","x":0,"y":0,"x2":0,"y2":1,"colorStops":[{"offset":0,"color":"#5470c6"},{"offset":1,"color":"#91cc75"}]}}
            }]
        }"##,
    );

    // ── 21. 富 line series: lineStyle 富字段 ──
    check(
        "line_style_shadow",
        r##"{
            "series": [{
                "type":"line","data":[1,2,3],
                "lineStyle":{"color":"#5470c6","width":2,"type":"solid","shadowBlur":10,"shadowColor":"rgba(0,0,0,0.5)","opacity":0.8,"cap":"round","join":"round"}
            }]
        }"##,
    );

    // ── 22. axisPointer 富字段 ──
    check(
        "rich_axis_pointer",
        r##"{
            "tooltip":{"axisPointer":{"type":"shadow","snap":true,"z":50,"label":{"margin":4,"padding":5}}},
            "series":[{"type":"bar","data":[1,2]}]
        }"##,
    );

    println!("\nDone.");
}
