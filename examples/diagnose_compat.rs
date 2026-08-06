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

    // ── 23. 折线图 step 各种值 ──
    check(
        "line_step_start",
        r##"{ "series": [{"type":"line","data":[1,2,3],"step":"start"}] }"##,
    );
    check(
        "line_step_middle",
        r##"{ "series": [{"type":"line","data":[1,2,3],"step":"middle"}] }"##,
    );
    check(
        "line_step_end",
        r##"{ "series": [{"type":"line","data":[1,2,3],"step":"end"}] }"##,
    );
    check(
        "line_step_true",
        r##"{ "series": [{"type":"line","data":[1,2,3],"step":true} ] }"##,
    );
    check(
        "line_step_false",
        r##"{ "series": [{"type":"line","data":[1,2,3],"step":false} ] }"##,
    );

    // ── 24. 柱状图 barWidth/barGap/barCategoryGap 数值和百分比 ──
    check(
        "bar_width_number",
        r##"{ "series": [{"type":"bar","data":[1,2],"barWidth":20}] }"##,
    );
    check(
        "bar_width_percent",
        r##"{ "series": [{"type":"bar","data":[1,2],"barWidth":"60%"}] }"##,
    );
    check(
        "bar_gap_number",
        r##"{ "series": [{"type":"bar","data":[1,2],"barGap":"30%"}] }"##,
    );
    check(
        "bar_category_gap_percent",
        r##"{ "series": [{"type":"bar","data":[1,2],"barCategoryGap":"20%"}] }"##,
    );
    check(
        "bar_category_gap_number",
        r##"{ "series": [{"type":"bar","data":[1,2],"barCategoryGap":20}] }"##,
    );
    check(
        "bar_max_width_number",
        r##"{ "series": [{"type":"bar","data":[1,2],"barMaxWidth":50}] }"##,
    );
    check(
        "bar_min_width_number",
        r##"{ "series": [{"type":"bar","data":[1,2],"barMinWidth":5}] }"##,
    );

    // ── 25. 饼图 radius/center 各种格式 ──
    check(
        "pie_radius_single_number",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10}],"radius":50}] }"##,
    );
    check(
        "pie_radius_single_percent",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10}],"radius":"50%"}] }"##,
    );
    check(
        "pie_radius_array_numbers",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10}],"radius":[0,75]}] }"##,
    );
    check(
        "pie_radius_array_percents",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10}],"radius":["40%","70%"]}] }"##,
    );
    check(
        "pie_radius_array_mixed",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10}],"radius":[0,"70%"]}] }"##,
    );
    check(
        "pie_center_numbers",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10}],"center":[400,300]}] }"##,
    );
    check(
        "pie_center_percents",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10}],"center":["50%","50%"]}] }"##,
    );
    check(
        "pie_center_mixed",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10}],"center":["50%",300]}] }"##,
    );
    check(
        "pie_rose_area",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10}],"roseType":"area"}] }"##,
    );
    check(
        "pie_rose_radius",
        r##"{ "series": [{"type":"pie","data":[{"name":"a","value":10}],"roseType":"radius"}] }"##,
    );

    // ── 26. 仪表盘 center/radius 各种格式 ──
    check(
        "gauge_center_numbers",
        r##"{ "series": [{"type":"gauge","data":[{"value":50}],"center":[400,300]}] }"##,
    );
    check(
        "gauge_center_percents",
        r##"{ "series": [{"type":"gauge","data":[{"value":50}],"center":["50%","55%"]}] }"##,
    );
    check(
        "gauge_radius_number",
        r##"{ "series": [{"type":"gauge","data":[{"value":50}],"radius":150}] }"##,
    );
    check(
        "gauge_radius_percent",
        r##"{ "series": [{"type":"gauge","data":[{"value":50}],"radius":"75%"}] }"##,
    );
    check(
        "gauge_detail_formatter",
        r##"{ "series": [{"type":"gauge","data":[{"value":50}],"detail":{"formatter":"{value}%"}}] }"##,
    );

    // ── 27. 散点图 symbolSize 各种格式 ──
    check(
        "scatter_symbol_size_number",
        r##"{ "series": [{"type":"scatter","data":[[1,2],[3,4]],"symbolSize":10}] }"##,
    );
    check(
        "scatter_symbol_size_array",
        r##"{ "series": [{"type":"scatter","data":[[1,2],[3,4]],"symbolSize":[10,20]}] }"##,
    );
    check(
        "scatter_symbol_size_string",
        r##"{ "series": [{"type":"scatter","data":[[1,2],[3,4]],"symbolSize":"10"}] }"##,
    );

    // ── 28. 轴 min/max 字符串值 ──
    check(
        "axis_min_dataMin",
        r##"{ "xAxis": {"type":"value","min":"dataMin"} }"##,
    );
    check(
        "axis_max_dataMax",
        r##"{ "xAxis": {"type":"value","max":"dataMax"} }"##,
    );
    check(
        "axis_min_max_both_strings",
        r##"{ "xAxis": {"type":"value","min":"dataMin","max":"dataMax"} }"##,
    );
    check(
        "axis_min_number",
        r##"{ "xAxis": {"type":"value","min":0} }"##,
    );
    check(
        "axis_max_number",
        r##"{ "xAxis": {"type":"value","max":100} }"##,
    );

    // ── 29. 轴 boundaryGap 数组格式 ──
    check(
        "boundary_gap_bool_true",
        r##"{ "xAxis": {"type":"category","data":["a","b"],"boundaryGap":true} }"##,
    );
    check(
        "boundary_gap_bool_false",
        r##"{ "xAxis": {"type":"category","data":["a","b"],"boundaryGap":false} }"##,
    );
    check(
        "boundary_gap_array_percents",
        r##"{ "xAxis": {"type":"category","data":["a","b"],"boundaryGap":["20%","20%"]} }"##,
    );
    check(
        "boundary_gap_array_numbers",
        r##"{ "xAxis": {"type":"category","data":["a","b"],"boundaryGap":[0,0]} }"##,
    );

    // ── 30. 轴 data 数字数组 ──
    check(
        "axis_data_numbers",
        r##"{ "xAxis": {"type":"category","data":[1,2,3,4,5]} }"##,
    );
    check(
        "axis_data_mixed_numbers_strings",
        r##"{ "xAxis": {"type":"category","data":[1,"a",3]} }"##,
    );

    // ── 31. 多 grid 多轴布局 ──
    check(
        "multi_grid_multi_axis",
        r##"{
            "grid": [{"left":"10%","right":"55%"},{"left":"55%","right":"10%"}],
            "xAxis": [{"type":"category","data":["a","b"]},{"type":"value"}],
            "yAxis": [{"type":"value"},{"type":"category","data":["x","y"]}],
            "series": [
                {"type":"bar","data":[1,2],"xAxisIndex":0,"yAxisIndex":0},
                {"type":"line","data":[3,4],"xAxisIndex":1,"yAxisIndex":1}
            ]
        }"##,
    );

    // ── 32. 堆叠柱状图 ──
    check(
        "stacked_bar",
        r##"{
            "xAxis": {"type":"category","data":["Mon","Tue","Wed"]},
            "yAxis": {"type":"value"},
            "series": [
                {"type":"bar","name":"A","data":[1,2,3],"stack":"total"},
                {"type":"bar","name":"B","data":[4,5,6],"stack":"total"}
            ]
        }"##,
    );

    // ── 33. 堆叠面积图 ──
    check(
        "stacked_area",
        r##"{
            "xAxis": {"type":"category","data":["Mon","Tue","Wed"]},
            "yAxis": {"type":"value"},
            "series": [
                {"type":"line","name":"A","data":[1,2,3],"stack":"total","areaStyle":{}},
                {"type":"line","name":"B","data":[4,5,6],"stack":"total","areaStyle":{}}
            ]
        }"##,
    );

    // ── 34. 双 y 轴 ──
    check(
        "dual_y_axis",
        r##"{
            "xAxis": {"type":"category","data":["Mon","Tue","Wed"]},
            "yAxis": [{"type":"value","name":"温度","position":"left"},{"type":"value","name":"降水","position":"right"}],
            "series": [
                {"type":"line","name":"温度","data":[10,20,15],"yAxisIndex":0},
                {"type":"bar","name":"降水","data":[5,10,8],"yAxisIndex":1}
            ]
        }"##,
    );

    // ── 35. 横向柱状图 ──
    check(
        "horizontal_bar",
        r##"{
            "xAxis": {"type":"value"},
            "yAxis": {"type":"category","data":["A","B","C"]},
            "series": [{"type":"bar","data":[10,20,30]}]
        }"##,
    );

    // ── 36. markPoint 富字段 ──
    check(
        "rich_markpoint",
        r##"{
            "series": [{
                "type":"line","data":[1,2,3],
                "markPoint":{
                    "data":[
                        {"type":"max","name":"最大值"},
                        {"type":"min","name":"最小值"},
                        {"coord":[1,2],"name":"自定义","symbol":"pin","symbolSize":50}
                    ],
                    "symbol":"pin",
                    "symbolSize":50,
                    "label":{"show":true,"formatter":"{c}"}
                }
            }]
        }"##,
    );

    // ── 37. markArea ──
    check(
        "rich_markarea",
        r##"{
            "series": [{
                "type":"line","data":[1,2,3],
                "markArea":{
                    "data":[
                        [{"xAxis":"Mon"},{"xAxis":"Wed"}],
                        [{"yAxis":10,"itemStyle":{"color":"rgba(255,0,0,0.1)"}},{"yAxis":20}]
                    ]
                }
            }]
        }"##,
    );

    // ── 38. 雷达图 ──
    check(
        "radar_chart",
        r##"{
            "radar": {
                "indicator": [
                    {"name":"销售","max":100},
                    {"name":"管理","max":100},
                    {"name":"技术","max":100}
                ],
                "shape":"circle",
                "splitNumber":5,
                "center":["50%","50%"],
                "radius":"65%"
            },
            "series": [{"type":"radar","data":[{"value":[80,60,90],"name":"预算"}]}]
        }"##,
    );

    // ── 39. 极坐标柱状图 ──
    check(
        "polar_bar",
        r##"{
            "angleAxis": {"type":"category","data":["A","B","C"]},
            "radiusAxis": {"type":"value"},
            "polar": {},
            "series": [{"type":"bar","data":[10,20,30],"coordinateSystem":"polar"}]
        }"##,
    );

    // ── 40. 极坐标散点图 ──
    check(
        "polar_scatter",
        r##"{
            "angleAxis": {"type":"value"},
            "radiusAxis": {"type":"value"},
            "polar": {},
            "series": [{"type":"scatter","data":[[1,10],[2,20],[3,30]],"coordinateSystem":"polar"}]
        }"##,
    );

    // ── 41. K线图 ──
    check(
        "candlestick_chart",
        r##"{
            "xAxis": {"type":"category","data":["2021-01-01","2021-01-02","2021-01-03"]},
            "yAxis": {"type":"value"},
            "series": [{
                "type":"candlestick",
                "data":[
                    [20,34,10,38],
                    [40,35,30,50],
                    [31,38,33,44]
                ]
            }]
        }"##,
    );

    // ── 42. 箱线图 ──
    check(
        "boxplot_chart",
        r##"{
            "xAxis": {"type":"category","data":["A","B"]},
            "yAxis": {"type":"value"},
            "series": [{
                "type":"boxplot",
                "data":[
                    [850,900,950,980,1175],
                    [600,700,800,850,1000]
                ]
            }]
        }"##,
    );

    // ── 43. 表格 ──
    check(
        "table_chart",
        r##"{
            "series": [{
                "type":"table",
                "data":[
                    {"value":["Alice",25,"Engineer"]},
                    {"value":["Bob",30,"Designer"]}
                ],
                "header":["Name","Age","Job"]
            }]
        }"##,
    );

    // ── 44. dataZoom inside 类型 ──
    check(
        "datazoom_inside",
        r##"{
            "dataZoom": [{"type":"inside","xAxisIndex":[0,1],"start":0,"end":100}],
            "series": [{"type":"line","data":[1,2,3,4,5]}]
        }"##,
    );

    // ── 45. dataZoom 多类型组合 ──
    check(
        "datazoom_mixed",
        r##"{
            "dataZoom": [
                {"type":"slider","start":0,"end":50},
                {"type":"inside","start":0,"end":50}
            ],
            "series": [{"type":"line","data":[1,2,3,4,5]}]
        }"##,
    );

    // ── 46. visualMap 分段型 ──
    check(
        "visualmap_piecewise",
        r##"{
            "visualMap": {
                "type":"piecewise",
                "min":0,"max":100,
                "splitNumber":5,
                "inRange":{"color":["#50a3ba","#eac736","#d94e5d"]}
            },
            "series": [{"type":"scatter","data":[[1,2],[3,4]]}]
        }"##,
    );

    // ── 47. 颜色格式 ──
    check(
        "color_hex_short",
        r##"{ "color":"#f00","series":[{"type":"bar","data":[1]}] }"##,
    );
    check(
        "color_hex_long",
        r##"{ "color":"#ff0000","series":[{"type":"bar","data":[1]}] }"##,
    );
    check(
        "color_rgb",
        r##"{ "color":"rgb(255,0,0)","series":[{"type":"bar","data":[1]}] }"##,
    );
    check(
        "color_rgba",
        r##"{ "color":"rgba(255,0,0,0.5)","series":[{"type":"bar","data":[1]}] }"##,
    );
    check(
        "color_named",
        r##"{ "color":"red","series":[{"type":"bar","data":[1]}] }"##,
    );

    // ── 48. 线条样式 ──
    check(
        "line_style_solid",
        r##"{ "series":[{"type":"line","data":[1,2],"lineStyle":{"type":"solid"}}] }"##,
    );
    check(
        "line_style_dashed",
        r##"{ "series":[{"type":"line","data":[1,2],"lineStyle":{"type":"dashed"}}] }"##,
    );
    check(
        "line_style_dotted",
        r##"{ "series":[{"type":"line","data":[1,2],"lineStyle":{"type":"dotted"}}] }"##,
    );
    check(
        "line_style_width_number",
        r##"{ "series":[{"type":"line","data":[1,2],"lineStyle":{"width":3}}] }"##,
    );

    // ── 49. 面积样式 ──
    check(
        "area_style_object",
        r##"{ "series":[{"type":"line","data":[1,2],"areaStyle":{"color":"#5470c6","opacity":0.3}}] }"##,
    );
    check(
        "area_style_empty",
        r##"{ "series":[{"type":"line","data":[1,2],"areaStyle":{}}] }"##,
    );

    // ── 50. 强调/模糊/选中状态 ──
    check(
        "emphasis_state",
        r##"{
            "series": [{
                "type":"line","data":[1,2],
                "emphasis":{"focus":"series","itemStyle":{"borderWidth":2}},
                "blur":{"focus":"series"},
                "select":{"disabled":true}
            }]
        }"##,
    );

    // ── 51. 系列通用字段 ──
    check(
        "series_common_fields",
        r##"{
            "series": [{
                "type":"line","data":[1,2],
                "name":"test",
                "colorBy":"data",
                "legendHoverLink":true,
                "hoverAnimation":true,
                "zlevel":0,
                "z":2,
                "silent":false,
                "animation":true,
                "animationThreshold":2000,
                "animationDuration":1000,
                "animationEasing":"cubicOut",
                "animationDelay":0,
                "animationDurationUpdate":300,
                "animationEasingUpdate":"cubicOut",
                "animationDelayUpdate":0,
                "stateAnimation":{"duration":300,"easing":"cubicOut"}
            }]
        }"##,
    );

    // ── 52. 轴名称和样式 ──
    check(
        "axis_name_style",
        r##"{
            "xAxis": {
                "type":"value",
                "name":"温度 (°C)",
                "nameLocation":"middle",
                "nameGap":30,
                "nameTextStyle":{"color":"#333","fontSize":14,"fontWeight":"bold"}
            }
        }"##,
    );

    // ── 53. 轴刻度/标签富字段 ──
    check(
        "axis_tick_rich",
        r##"{
            "xAxis": {
                "type":"category",
                "data":["a","b","c"],
                "axisTick":{"show":true,"alignWithLabel":true,"interval":0,"inside":false,"length":5,"lineStyle":{"color":"#333","width":1}}
            }
        }"##,
    );
    check(
        "axis_label_rich",
        r##"{
            "xAxis": {
                "type":"category",
                "data":["a","b","c"],
                "axisLabel":{"show":true,"interval":0,"rotate":45,"margin":8,"color":"#333","fontSize":12,"formatter":"{value}°C"}
            }
        }"##,
    );

    // ── 54. 分割线/分割区域 ──
    check(
        "split_line_rich",
        r##"{
            "yAxis": {
                "type":"value",
                "splitLine":{"show":true,"interval":1,"lineStyle":{"color":"#ccc","type":"dashed","width":1}}
            }
        }"##,
    );
    check(
        "split_area_rich",
        r##"{
            "yAxis": {
                "type":"value",
                "splitArea":{"show":true,"areaStyle":{"color":["rgba(250,250,250,0.3)","rgba(200,200,200,0.3)"]}}
            }
        }"##,
    );

    // ── 55. 轴线样式 ──
    check(
        "axis_line_rich",
        r##"{
            "xAxis": {
                "type":"category",
                "data":["a","b"],
                "axisLine":{"show":true,"onZero":true,"onZeroAxisIndex":0,"symbol":["none","arrow"],"symbolSize":[10,15],"lineStyle":{"color":"#333","width":1,"type":"solid"}}
            }
        }"##,
    );

    // ── 56. 轴反转/对数轴 ──
    check(
        "axis_inverse_true",
        r##"{ "xAxis":{"type":"category","data":["a","b"],"inverse":true} }"##,
    );
    check(
        "axis_log_type",
        r##"{ "xAxis":{"type":"log","min":1,"max":1000,"logBase":10} }"##,
    );
    check("axis_time_type", r##"{ "xAxis":{"type":"time"} }"##);

    // ── 57. 饼图标签 ──
    check(
        "pie_label_rich",
        r##"{
            "series": [{
                "type":"pie",
                "data":[{"name":"a","value":10},{"name":"b","value":20}],
                "label":{"show":true,"position":"outside","formatter":"{b}: {d}%","color":"#333","fontSize":12},
                "labelLine":{"show":true,"length":15,"length2":10,"lineStyle":{"color":"#333","width":1}}
            }]
        }"##,
    );
    check(
        "pie_label_inside",
        r##"{
            "series": [{
                "type":"pie",
                "data":[{"name":"a","value":10}],
                "label":{"show":true,"position":"inside","formatter":"{b}"}
            }]
        }"##,
    );

    // ── 58. 仪表盘富字段 ──
    check(
        "gauge_rich",
        r##"{
            "series": [{
                "type":"gauge",
                "data":[{"value":50,"name":"完成率"}],
                "min":0,
                "max":100,
                "splitNumber":10,
                "axisLine":{"lineStyle":{"width":10,"color":[[0.3,"#67e0e3"],[0.7,"#37a2da"],[1,"#fd666d"]]}},
                "axisTick":{"show":true,"length":5},
                "axisLabel":{"show":true,"distance":10},
                "pointer":{"show":true,"length":"60%","width":5},
                "title":{"show":true,"offsetCenter":[0,"70%"],"fontSize":14},
                "detail":{"show":true,"offsetCenter":[0,"90%"],"fontSize":20,"formatter":"{value}%"}
            }]
        }"##,
    );

    // ── 59. 散点图数据格式 ──
    check(
        "scatter_data_with_name",
        r##"{ "series":[{"type":"scatter","data":[{"value":[1,2],"name":"A"},{"value":[3,4],"name":"B"}]}] }"##,
    );
    check(
        "scatter_data_with_symbol",
        r##"{ "series":[{"type":"scatter","data":[{"value":[1,2],"symbol":"triangle","symbolSize":20}]}] }"##,
    );

    // ── 60. 气泡图 ──
    check(
        "bubble_chart",
        r##"{
            "xAxis":{"type":"value"},
            "yAxis":{"type":"value"},
            "series":[{"type":"scatter","data":[[1,2,10],[3,4,20],[5,6,30]],"symbolSize":"data[2]"}]
        }"##,
    );

    // ── 61. 数据集 + 多个系列 ──
    check(
        "dataset_multi_series",
        r##"{
            "dataset":{
                "source":[
                    ["product","2015","2016","2017"],
                    ["Matcha",43.3,85.8,93.7],
                    ["Milk Tea",83.1,73.4,55.1],
                    ["Cheese Cocoa",86.4,65.2,82.5]
                ]
            },
            "xAxis":{"type":"category"},
            "yAxis":{},
            "series":[
                {"type":"bar","encode":{"x":"product","y":"2015"}},
                {"type":"bar","encode":{"x":"product","y":"2016"}},
                {"type":"bar","encode":{"x":"product","y":"2017"}}
            ]
        }"##,
    );

    // ── 62. 空数据系列 ──
    check(
        "series_empty_data",
        r##"{ "series":[{"type":"line","data":[]}] }"##,
    );
    check("series_no_data", r##"{ "series":[{"type":"line"}] }"##);

    // ── 63. 系列 ID 和名称 ──
    check(
        "series_id_name",
        r##"{ "series":[{"type":"line","id":"s1","name":"Series 1","data":[1,2,3]}] }"##,
    );

    // ── 64. tooltip 触发方式 ──
    check(
        "tooltip_trigger_axis",
        r##"{ "tooltip":{"trigger":"axis"},"series":[{"type":"line","data":[1,2]}] }"##,
    );
    check(
        "tooltip_trigger_item",
        r##"{ "tooltip":{"trigger":"item"},"series":[{"type":"pie","data":[{"name":"a","value":1}]}] }"##,
    );
    check(
        "tooltip_trigger_none",
        r##"{ "tooltip":{"trigger":"none"},"series":[{"type":"line","data":[1,2]}] }"##,
    );

    // ── 65. legend 位置和方向 ──
    check(
        "legend_left_right",
        r##"{ "legend":{"left":"right","data":["a"]} }"##,
    );
    check(
        "legend_orient_vertical",
        r##"{ "legend":{"orient":"vertical","left":"left","data":["a"]} }"##,
    );
    check("legend_show_false", r##"{ "legend":{"show":false} }"##);
    check(
        "legend_type_scroll",
        r##"{ "legend":{"type":"scroll","data":["a","b","c"]} }"##,
    );

    // ── 66. title 富文本 ──
    check(
        "title_rich_text",
        r##"{
            "title":{
                "text":"主标题",
                "subtext":"副标题",
                "link":"https://example.com",
                "target":"blank",
                "textStyle":{"color":"#333","fontSize":18,"fontWeight":"bold","fontFamily":"sans-serif"},
                "subtextStyle":{"color":"#aaa","fontSize":12}
            }
        }"##,
    );

    // ── 67. grid 富字段 ──
    check(
        "grid_rich",
        r##"{
            "grid":{
                "left":"10%","right":"10%","top":"15%","bottom":"15%",
                "containLabel":true,
                "show":true,
                "backgroundColor":"#fff",
                "borderColor":"#ccc",
                "borderWidth":1
            }
        }"##,
    );

    // ── 68. 轴标签格式化 ──
    check(
        "axis_label_formatter_function",
        r##"{ "xAxis":{"type":"value","axisLabel":{"formatter":"{value} °C"}} }"##,
    );

    // ── 69. 混合系列类型 ──
    check(
        "mixed_series_types",
        r##"{
            "xAxis":{"type":"category","data":["Mon","Tue","Wed"]},
            "yAxis":{"type":"value"},
            "series":[
                {"type":"bar","data":[10,20,30]},
                {"type":"line","data":[15,25,35]},
                {"type":"scatter","data":[12,22,32]}
            ]
        }"##,
    );

    // ── 70. 完整图表配置（模拟 LLM 输出）──
    check(
        "full_chart_config",
        r##"{
            "title":{"text":"销售统计","subtext":"2024年数据","left":"center"},
            "tooltip":{"trigger":"axis","axisPointer":{"type":"cross"}},
            "legend":{"data":["销售额","利润"],"top":"bottom"},
            "grid":{"left":"3%","right":"4%","bottom":"3%","containLabel":true},
            "xAxis":{"type":"category","data":["1月","2月","3月","4月","5月","6月"],"boundaryGap":false},
            "yAxis":{"type":"value","name":"金额(万元)","min":"dataMax"},
            "series":[
                {
                    "name":"销售额",
                    "type":"line",
                    "data":[820,932,901,934,1290,1330],
                    "smooth":true,
                    "lineStyle":{"width":2},
                    "areaStyle":{"opacity":0.3},
                    "emphasis":{"focus":"series"}
                },
                {
                    "name":"利润",
                    "type":"bar",
                    "data":[120,150,130,170,200,180],
                    "barWidth":"40%",
                    "itemStyle":{"color":"#91cc75"}
                }
            ]
        }"##,
    );

    // ── 71. 饼图完整配置 ──
    check(
        "full_pie_config",
        r##"{
            "title":{"text":"访问来源","left":"center"},
            "tooltip":{"trigger":"item","formatter":"{a} {b}: {c} ({d}%)"},
            "legend":{"orient":"vertical","left":"left","data":["直接访问","邮件营销","联盟广告"]},
            "series":[{
                "name":"访问来源",
                "type":"pie",
                "radius":["40%","70%"],
                "center":["50%","55%"],
                "avoidLabelOverlap":true,
                "itemStyle":{"borderRadius":10,"borderColor":"#fff","borderWidth":2},
                "label":{"show":true,"formatter":"{b}: {c}"},
                "emphasis":{"label":{"show":true,"fontSize":16,"fontWeight":"bold"}},
                "data":[
                    {"value":1048,"name":"直接访问"},
                    {"value":735,"name":"邮件营销"},
                    {"value":580,"name":"联盟广告"}
                ]
            }]
        }"##,
    );

    // ── 72. 仪表盘完整配置 ──
    check(
        "full_gauge_config",
        r##"{
            "series":[{
                "type":"gauge",
                "progress":{"show":true,"width":18},
                "axisLine":{"lineStyle":{"width":18}},
                "axisTick":{"show":false},
                "splitLine":{"length":15,"lineStyle":{"width":2,"color":"#999"}},
                "axisLabel":{"distance":25,"color":"#999","fontSize":12},
                "anchor":{"show":true,"showAbove":true,"size":20,"itemStyle":{"borderWidth":10}},
                "title":{"show":true},
                "detail":{"valueAnimation":true,"formatter":"{value}","fontSize":30,"offsetCenter":[0,"70%"]},
                "data":[{"value":70,"name":"完成率"}]
            }]
        }"##,
    );

    // ── 73. 散点图完整配置 ──
    check(
        "full_scatter_config",
        r##"{
            "xAxis":{"type":"value","splitLine":{"lineStyle":{"type":"dashed"}}},
            "yAxis":{"type":"value","splitLine":{"lineStyle":{"type":"dashed"}}},
            "series":[{
                "type":"scatter",
                "data":[[10.0,8.04],[8.07,6.95],[13.0,7.58],[9.05,8.81],[11.0,8.33]],
                "symbolSize":10,
                "itemStyle":{"color":"#5470c6"},
                "emphasis":{"focus":"self","itemStyle":{"borderColor":"#333","borderWidth":2}}
            }]
        }"##,
    );

    // ── 74. 数据缩放 + 区域缩放 ──
    check(
        "datazoom_slider_inside",
        r##"{
            "dataZoom":[
                {"type":"slider","start":10,"end":60,"xAxisIndex":[0]},
                {"type":"inside","start":10,"end":60,"xAxisIndex":[0]}
            ],
            "series":[{"type":"line","data":[1,2,3,4,5,6,7,8,9,10]}]
        }"##,
    );

    // ── 75. 视觉映射连续型 ──
    check(
        "visualmap_continuous_full",
        r##"{
            "visualMap":{
                "type":"continuous",
                "show":true,
                "min":0,
                "max":200,
                "range":[50,150],
                "calculable":true,
                "orient":"horizontal",
                "left":"center",
                "bottom":"5%",
                "inRange":{"color":["#bf444c","#d88273","#f6efa6"]},
                "textStyle":{"color":"#333"}
            },
            "series":[{"type":"scatter","data":[[1,2],[3,4]]}]
        }"##,
    );

    // ── 76. 极坐标系统 ──
    check(
        "polar_system_full",
        r##"{
            "angleAxis":{"type":"category","data":["周一","周二","周三","周四","周五","周六","周日"]},
            "radiusAxis":{"type":"value","min":0,"max":10},
            "polar":{"radius":["10%","80%"]},
            "series":[
                {"type":"bar","data":[1,2,3,4,5,6,7],"coordinateSystem":"polar","stack":"a"},
                {"type":"bar","data":[2,3,4,5,6,7,8],"coordinateSystem":"polar","stack":"a"}
            ]
        }"##,
    );

    // ── 77. 富文本标签 ──
    check(
        "rich_label_formatter",
        r##"{
            "series":[{
                "type":"pie",
                "data":[{"name":"a","value":10}],
                "label":{"show":true,"formatter":"{b|{b}}\n{c|{c}}","rich":{"b":{"fontSize":14,"fontWeight":"bold"},"c":{"fontSize":12,"color":"#999"}}}
            }]
        }"##,
    );

    // ── 78. 系列数据带样式 ──
    check(
        "data_with_item_style",
        r##"{
            "series":[{
                "type":"pie",
                "data":[
                    {"value":10,"name":"a","itemStyle":{"color":"#5470c6"}},
                    {"value":20,"name":"b","itemStyle":{"color":"#91cc75"}},
                    {"value":30,"name":"c","itemStyle":{"color":"#fac858"}}
                ]
            }]
        }"##,
    );

    // ── 79. 轴指针跨轴 ──
    check(
        "axis_pointer_cross",
        r##"{
            "tooltip":{"trigger":"axis","axisPointer":{"type":"cross","crossStyle":{"color":"#999","width":1,"type":"dashed"}}},
            "series":[{"type":"line","data":[1,2,3]}]
        }"##,
    );

    // ── 80. 系列动画配置 ──
    check(
        "series_animation_config",
        r##"{
            "series":[{
                "type":"line",
                "data":[1,2,3],
                "animation":true,
                "animationThreshold":2000,
                "animationDuration":1000,
                "animationEasing":"cubicOut",
                "animationDelay":0,
                "animationDurationUpdate":300,
                "animationEasingUpdate":"cubicOut",
                "animationDelayUpdate":0
            }]
        }"##,
    );

    // ═══════════════════════════════════════════════════════════════
    // 第 2 批：更多常用 ECharts 配置（81~120）
    // ═══════════════════════════════════════════════════════════════

    // ── 81. 饼图完整配置（minAngle/roseType/avoidLabelOverlap/startAngle） ──
    check(
        "pie_full_rose_config",
        r##"{
            "series":[{
                "type":"pie",
                "radius":["30%","70%"],
                "center":["50%","50%"],
                "roseType":"area",
                "startAngle":90,
                "endAngle":-270,
                "minAngle":5,
                "minShowLabelAngle":5,
                "avoidLabelOverlap":true,
                "stillShowZeroSum":true,
                "percentPrecision":2,
                "clockwise":true,
                "data":[
                    {"name":"a","value":10},
                    {"name":"b","value":20},
                    {"name":"c","value":30}
                ],
                "itemStyle":{"borderRadius":4,"borderColor":"#fff","borderWidth":2},
                "label":{"show":true,"position":"outside","formatter":"{b}: {d}%"},
                "labelLine":{"show":true,"length":10,"length2":20,"smooth":true},
                "emphasis":{"scale":true,"scaleSize":10,"focus":"self","itemStyle":{"shadowBlur":10,"shadowColor":"rgba(0,0,0,0.5)"}}
            }]
        }"##,
    );

    // ── 82. 折线图完整配置（smooth/connectNulls/clip/step end/stack） ──
    check(
        "line_full_config",
        r##"{
            "xAxis":{"type":"category","data":["A","B","C","D","E"],"boundaryGap":false},
            "yAxis":{"type":"value"},
            "series":[{
                "type":"line",
                "name":"访问量",
                "data":[820,932,901,934,1290],
                "stack":"total",
                "smooth":true,
                "smoothMonotone":"x",
                "connectNulls":true,
                "clip":true,
                "showSymbol":true,
                "showAllSymbol":false,
                "symbol":"circle",
                "symbolSize":8,
                "symbolRotate":0,
                "symbolKeepAspect":false,
                "hoverAnimation":true,
                "legendHoverLink":true,
                "lineStyle":{"width":2,"color":"#5470c6","type":"solid","cap":"round","join":"round"},
                "itemStyle":{"color":"#5470c6","borderColor":"#fff","borderWidth":1},
                "areaStyle":{"color":["rgba(84,112,198,0.5)","rgba(84,112,198,0.05)"],"origin":"start"},
                "emphasis":{"focus":"series","lineStyle":{"width":3}}
            }]
        }"##,
    );

    // ── 83. 柱状图完整配置（barMinHeight/roundCap/background/showBackground） ──
    check(
        "bar_full_config",
        r##"{
            "xAxis":{"type":"category","data":["Q1","Q2","Q3","Q4"]},
            "yAxis":{"type":"value","splitNumber":5},
            "series":[{
                "type":"bar",
                "name":"销售额",
                "data":[320,332,301,334],
                "barWidth":"60%",
                "barMaxWidth":50,
                "barMinHeight":0,
                "barMinAngle":0,
                "barGap":"30%",
                "barCategoryGap":"20%",
                "roundCap":false,
                "showBackground":true,
                "backgroundStyle":{"color":"rgba(180,180,180,0.2)","borderColor":null,"borderWidth":0,"borderRadius":4},
                "itemStyle":{"color":"#91cc75","borderRadius":[4,4,0,0],"borderColor":"#5a9e3a","borderWidth":1},
                "emphasis":{"focus":"series","itemStyle":{"color":"#7cb960"}}
            }]
        }"##,
    );

    // ── 84. 散点图/气泡图带 large, symbolSize 回调风格值 ──
    check(
        "scatter_large_symbol",
        r##"{
            "xAxis":{"type":"value","min":-100,"max":100},
            "yAxis":{"type":"value","min":-100,"max":100},
            "series":[{
                "type":"scatter",
                "data":[[28.61,39.48,37],[5.42,24.69,28],[76.84,80.13,52],[40.18,57.01,41]],
                "symbol":"circle",
                "symbolSize":[10,10],
                "symbolRotate":0,
                "large":true,
                "largeThreshold":2000,
                "legendHoverLink":true,
                "itemStyle":{"color":"#ee6666","borderColor":"#fff","borderWidth":1,"opacity":0.8},
                "emphasis":{"scale":true,"focus":"self"}
            }]
        }"##,
    );

    // ── 85. 雷达图详细配置（shape/circle/center/radius/indicator 带样式） ──
    check(
        "radar_full_config",
        r##"{
            "radar":{
                "center":["50%","57%"],
                "radius":"65%",
                "startAngle":90,
                "shape":"polygon",
                "splitNumber":5,
                "axisName":{"show":true,"formatter":"【{value}】","color":"#999","fontSize":12},
                "axisNameGap":15,
                "splitArea":{"show":true,"areaStyle":{"color":["rgba(114,172,209,0.2)","rgba(114,172,209,0.1)"],"shadowBlur":0}},
                "splitLine":{"show":true,"lineStyle":{"width":1,"type":"solid","color":"rgba(114,172,209,0.8)"}},
                "axisLine":{"show":true,"lineStyle":{"color":"rgba(114,172,209,0.8)"}},
                "axisTick":{"show":false},
                "indicator":[
                    {"name":"销售","max":100,"min":0,"color":"#333"},
                    {"name":"管理","max":100},
                    {"name":"技术","max":100},
                    {"name":"服务","max":100},
                    {"name":"协作","max":100},
                    {"name":"创新","max":100}
                ]
            },
            "series":[{
                "type":"radar",
                "symbol":"circle",
                "symbolSize":6,
                "data":[
                    {"value":[80,90,70,85,75,88],"name":"预算分配","areaStyle":{"color":"rgba(84,112,198,0.4)"},"lineStyle":{"width":2,"color":"#5470c6"},"itemStyle":{"color":"#5470c6"}},
                    {"value":[60,70,85,80,90,72],"name":"实际开销","areaStyle":{"color":"rgba(238,102,102,0.4)"},"lineStyle":{"width":2,"color":"#ee6666"},"itemStyle":{"color":"#ee6666"}}
                ]
            }]
        }"##,
    );

    // ── 86. Funnel 漏斗图完整配置（sort/gap/funnelAlign/label/labelLine） ──
    check(
        "funnel_full_config",
        r##"{
            "series":[{
                "type":"funnel",
                "name":"漏斗图",
                "left":"10%",
                "top":60,
                "bottom":60,
                "width":"80%",
                "min":0,
                "max":100,
                "minSize":"0%",
                "maxSize":"100%",
                "sort":"descending",
                "gap":2,
                "funnelAlign":"center",
                "orient":"vertical",
                "label":{"show":true,"position":"inside","formatter":"{b}: {c}","color":"#fff"},
                "labelLine":{"show":true,"length":10,"lineStyle":{"width":1,"type":"solid"}},
                "itemStyle":{"borderColor":"#fff","borderWidth":1},
                "emphasis":{"label":{"fontSize":16},"itemStyle":{"shadowBlur":20,"shadowColor":"rgba(0,0,0,0.5)"}},
                "data":[
                    {"value":100,"name":"展现"},
                    {"value":80,"name":"点击"},
                    {"value":60,"name":"访问"},
                    {"value":40,"name":"咨询"},
                    {"value":20,"name":"订单"}
                ]
            }]
        }"##,
    );

    // ── 87. Sankey 桑基图完整配置（nodes/links/nodeWidth/nodeGap/orient） ──
    check(
        "sankey_full_config",
        r##"{
            "series":[{
                "type":"sankey",
                "left":"5%",
                "right":"20%",
                "top":"10%",
                "bottom":"10%",
                "nodeWidth":20,
                "nodeGap":8,
                "nodeAlign":"justify",
                "layoutIterations":32,
                "orient":"horizontal",
                "draggable":true,
                "focusNodeAdjacency":"allEdges",
                "label":{"show":true,"position":"right","fontSize":12,"color":"#333"},
                "itemStyle":{"borderWidth":1,"borderColor":"#aaa"},
                "lineStyle":{"color":"gradient","curveness":0.5,"opacity":0.6},
                "emphasis":{"focus":"adjacency"},
                "data":[
                    {"name":"访问"},{"name":"咨询"},{"name":"订单"},
                    {"name":"直达"},{"name":"搜索引擎"},{"name":"邮件营销"}
                ],
                "links":[
                    {"source":"直达","target":"访问","value":100},
                    {"source":"搜索引擎","target":"访问","value":200},
                    {"source":"邮件营销","target":"访问","value":50},
                    {"source":"访问","target":"咨询","value":80},
                    {"source":"咨询","target":"订单","value":40}
                ]
            }]
        }"##,
    );

    // ── 88. Graph 关系图（力导向 layout/force/roam/edgeSymbol/categories） ──
    check(
        "graph_force_config",
        r##"{
            "series":[{
                "type":"graph",
                "layout":"force",
                "roam":true,
                "draggable":true,
                "focusNodeAdjacency":true,
                "categories":[{"name":"类目一","itemStyle":{"color":"#5470c6"}},{"name":"类目二","itemStyle":{"color":"#91cc75"}}],
                "center":["50%","50%"],
                "zoom":1,
                "edgeSymbol":["none","arrow"],
                "edgeSymbolSize":[4,10],
                "edgeLabel":{"show":true,"formatter":"{c}","fontSize":11},
                "label":{"show":true,"position":"right","fontSize":12,"color":"#333"},
                "itemStyle":{"borderColor":"#fff","borderWidth":1},
                "lineStyle":{"color":"#ccc","width":1,"curveness":0,"opacity":0.9},
                "force":{"repulsion":250,"gravity":0.1,"edgeLength":100,"layoutAnimation":true,"friction":0.6},
                "data":[
                    {"name":"节点1","category":0,"symbolSize":50,"value":10},
                    {"name":"节点2","category":0,"symbolSize":40,"value":8},
                    {"name":"节点3","category":1,"symbolSize":30,"value":5},
                    {"name":"节点4","category":1,"symbolSize":35,"value":7}
                ],
                "links":[
                    {"source":"节点1","target":"节点2","value":5,"lineStyle":{"width":5}},
                    {"source":"节点1","target":"节点3","value":3},
                    {"source":"节点2","target":"节点4","value":4},
                    {"source":"节点3","target":"节点4","value":2}
                ]
            }]
        }"##,
    );

    // ── 89. Tree 树图（orient/roam/initialTreeDepth/leaves/expandAndCollapse） ──
    check(
        "tree_full_config",
        r##"{
            "series":[{
                "type":"tree",
                "name":"树图",
                "top":"5%",
                "left":"20%",
                "bottom":"5%",
                "right":"15%",
                "layout":"orthogonal",
                "orient":"LR",
                "symbol":"emptyCircle",
                "symbolSize":7,
                "roam":true,
                "expandAndCollapse":true,
                "initialTreeDepth":2,
                "edgeShape":"curve",
                "edgeForkPosition":"63%",
                "lineStyle":{"width":1.5,"curveness":0.5,"color":"#ccc"},
                "label":{"show":true,"fontSize":12,"color":"#333","position":"left","verticalAlign":"middle","align":"right"},
                "leaves":{"label":{"show":true,"position":"right","verticalAlign":"middle","align":"left","color":"#2f4554"}},
                "emphasis":{"focus":"descendant"},
                "data":[{
                    "name":"根节点",
                    "children":[
                        {"name":"子节点1","children":[
                            {"name":"叶子1-1","value":10},
                            {"name":"叶子1-2","value":12}
                        ]},
                        {"name":"子节点2","children":[
                            {"name":"叶子2-1","value":8},
                            {"name":"叶子2-2","value":6}
                        ]}
                    ]
                }]
            }]
        }"##,
    );

    // ── 90. Treemap 矩形树图（width/height/roam/leafDepth/breadcrumb/levels） ──
    check(
        "treemap_full_config",
        r##"{
            "series":[{
                "type":"treemap",
                "left":"10%",
                "right":"10%",
                "top":"10%",
                "bottom":"10%",
                "width":null,
                "height":null,
                "sort":"desc",
                "squareRatio":1,
                "leafDepth":1,
                "drillDownIcon":"▶",
                "roam":true,
                "nodeClick":"zoomToNode",
                "zoomToNodeRatio":0.3136,
                "breadcrumb":{"show":true,"top":"bottom","left":"center","right":"center","height":22,"itemStyle":{"textStyle":{"color":"#333","fontSize":12},"height":22}},
                "label":{"show":true,"fontSize":12,"color":"#fff","formatter":"{b}\n{c}"},
                "upperLabel":{"show":true,"fontSize":12,"color":"#fff","height":20},
                "itemStyle":{"borderColor":"#fff","borderWidth":1,"gapWidth":1},
                "levels":[
                    {"itemStyle":{"borderColor":"#fff","borderWidth":0,"gapWidth":5}},
                    {"itemStyle":{"borderColor":"#555","borderWidth":5,"gapWidth":1},"emphasis":{"itemStyle":{"borderColor":"#333"}}},
                    {"colorSaturation":[0.35,0.6],"itemStyle":{"borderWidth":5,"gapWidth":1,"borderColorSaturation":0.6,"color":"#c23531"}}
                ],
                "data":[
                    {"name":"财经","value":100,"children":[
                        {"name":"股票","value":60},
                        {"name":"基金","value":40}
                    ]},
                    {"name":"科技","value":80,"children":[
                        {"name":"软件","value":50},
                        {"name":"硬件","value":30}
                    ]}
                ]
            }]
        }"##,
    );

    // ── 91. Sunburst 旭日图（sort/emphasis focus/radius/levels） ──
    check(
        "sunburst_full_config",
        r##"{
            "series":[{
                "type":"sunburst",
                "center":["50%","50%"],
                "radius":["10%","80%"],
                "sort":"desc",
                "startAngle":90,
                "clockwise":true,
                "minAngle":0,
                "avoidLabelOverlap":true,
                "nodeClick":"rootToNode",
                "zoomToNodeRatio":0.7225,
                "roam":true,
                "focusNodeAdjacency":true,
                "label":{"show":true,"rotate":"radial","fontSize":12,"color":"#fff"},
                "itemStyle":{"borderColor":"#fff","borderWidth":1,"borderRadius":0},
                "emphasis":{"focus":"ancestor","itemStyle":{"shadowBlur":10,"shadowColor":"rgba(0,0,0,0.5)"}},
                "levels":[
                    {},
                    {"r0":"15%","r":"40%","label":{"rotate":0},"itemStyle":{"borderWidth":2}},
                    {"r0":"40%","r":"70%","label":{"align":"right","rotate":"tangential"}},
                    {"r0":"70%","r":"72%","label":{"position":"outside","padding":3,"silent":false},"itemStyle":{"pointerEvents":"none"}}
                ],
                "data":[
                    {"name":"公司1","value":15,"children":[
                        {"name":"技术部","value":10,"children":[
                            {"name":"前端","value":4},{"name":"后端","value":6}
                        ]},
                        {"name":"产品部","value":5}
                    ]},
                    {"name":"公司2","value":10,"children":[
                        {"name":"设计部","value":6},
                        {"name":"市场部","value":4}
                    ]}
                ]
            }]
        }"##,
    );

    // ── 92. Calendar 日历图配置（range/cellSize/yearLabel/monthLabel/dayLabel） ──
    check(
        "calendar_heatmap",
        r##"{
            "calendar":{
                "top":120,
                "left":80,
                "right":40,
                "bottom":80,
                "cellSize":["auto",20],
                "range":"2024-01",
                "orient":"horizontal",
                "splitLine":{"show":true,"lineStyle":{"color":"#000","width":1,"type":"solid"}},
                "itemStyle":{"color":"#fff","borderWidth":1},
                "yearLabel":{"show":true,"fontSize":20,"color":"#333"},
                "monthLabel":{"show":true,"nameMap":"en","fontSize":12,"color":"#333"},
                "dayLabel":{"show":true,"firstDay":1,"nameMap":"en","fontSize":10,"color":"#999"}
            },
            "visualMap":{
                "min":0,"max":1000,
                "calculable":true,
                "orient":"horizontal",
                "left":"center",
                "top":"20",
                "inRange":{"color":["#ebedf0","#c6e48b","#7bc96f","#239a3b","#196127"]},
                "textStyle":{"color":"#333"}
            },
            "series":[{
                "type":"heatmap",
                "coordinateSystem":"calendar",
                "data":[["2024-01-01",120],["2024-01-05",450],["2024-01-10",800],["2024-01-15",200],["2024-01-20",650]]
            }]
        }"##,
    );

    // ── 93. Parallel 平行坐标（parallelAxisDefault 配置） ──
    check(
        "parallel_full_config",
        r##"{
            "parallel":{
                "left":"5%",
                "right":"10%",
                "top":80,
                "bottom":80,
                "width":null,
                "height":null,
                "layout":"horizontal",
                "axisExpandable":true,
                "axisExpandCenter":0,
                "axisExpandCount":4,
                "axisExpandWidth":50,
                "axisExpandTriggerOn":"click",
                "parallelAxisDefault":{
                    "type":"value",
                    "nameLocation":"end",
                    "nameGap":15,
                    "nameTextStyle":{"color":"#333","fontSize":12},
                    "inverse":false,
                    "splitNumber":5,
                    "axisLine":{"show":true,"lineStyle":{"color":"#aaa"}},
                    "axisTick":{"show":true,"length":6,"lineStyle":{"color":"#aaa"}},
                    "axisLabel":{"show":true,"color":"#666","fontSize":11},
                    "splitLine":{"show":true,"lineStyle":{"color":["#eee"]}},
                    "splitArea":{"show":false}
                }
            },
            "parallelAxis":[
                {"dim":0,"name":"价格","min":0,"max":100000,"inverse":true},
                {"dim":1,"name":"排量","min":0,"max":6},
                {"dim":2,"name":"油耗","min":0,"max":15},
                {"dim":3,"name":"功率","min":0,"max":300}
            ],
            "series":[{
                "type":"parallel",
                "coordinateSystem":"parallel",
                "name":"车系",
                "smooth":true,
                "lineStyle":{"width":2,"color":"#5470c6","opacity":0.7},
                "emphasis":{"lineStyle":{"width":3,"opacity":1}},
                "data":[
                    [12000,2.0,8.5,150],
                    [32000,3.0,10.2,220],
                    [80000,4.5,12.5,280]
                ]
            }]
        }"##,
    );

    // ── 94. ThemeRiver 主题河流 ──
    check(
        "themeriver_full",
        r##"{
            "singleAxis":{
                "top":50,
                "bottom":50,
                "left":"10%",
                "right":"10%",
                "type":"time",
                "boundaryGap":false,
                "axisLine":{"show":true,"lineStyle":{"color":"#999"}},
                "axisTick":{"show":false},
                "axisLabel":{"show":true,"color":"#666","fontSize":11},
                "splitLine":{"show":true,"lineStyle":{"color":"#eee","type":"dashed"}}
            },
            "tooltip":{"trigger":"axis"},
            "series":[{
                "type":"themeRiver",
                "boundaryGap":["10%","10%"],
                "singleAxisIndex":0,
                "label":{"show":true,"fontSize":12,"color":"#333"},
                "itemStyle":{"borderWidth":0,"opacity":0.8},
                "emphasis":{"itemStyle":{"shadowBlur":10,"shadowColor":"rgba(0,0,0,0.3)"}},
                "data":[
                    ["2024-01-01",10,"DQ"],
                    ["2024-01-02",15,"DQ"],
                    ["2024-01-03",20,"DQ"],
                    ["2024-01-01",8,"SS"],
                    ["2024-01-02",12,"SS"],
                    ["2024-01-03",16,"SS"],
                    ["2024-01-01",6,"QG"],
                    ["2024-01-02",10,"QG"],
                    ["2024-01-03",14,"QG"]
                ]
            }]
        }"##,
    );

    // ── 95. PictorialBar 象形柱图（symbol/symbolRepeat/symbolClip/symbolSize/position） ──
    check(
        "pictorialbar_full_config",
        r##"{
            "xAxis":{"type":"category","data":["巴西","印尼","中国","美国","德国"]},
            "yAxis":{"type":"value"},
            "series":[{
                "type":"pictorialBar",
                "name":"代表性柱",
                "symbolSize":["100%","100%"],
                "symbolRepeat":false,
                "symbolClip":true,
                "symbolOffset":[0,0],
                "symbolPosition":"start",
                "symbolRotate":0,
                "symbolBoundingData":"auto",
                "symbolPatternSize":400,
                "symbol":"rect",
                "itemStyle":{"borderColor":"#333","borderWidth":0},
                "label":{"show":true,"position":"top"},
                "emphasis":{"itemStyle":{"color":"#ff7f50"}},
                "data":[18203,23489,29034,104970,131744]
            }]
        }"##,
    );

    // ── 96. EffectScatter 涟漪散点图（rippleEffect/showEffectOn） ──
    check(
        "effectscatter_full_config",
        r##"{
            "xAxis":{"type":"value","min":0,"max":100},
            "yAxis":{"type":"value","min":0,"max":100},
            "series":[{
                "type":"effectScatter",
                "coordinateSystem":"cartesian2d",
                "showEffectOn":"render",
                "rippleEffect":{"period":4,"scale":4,"brushType":"stroke","number":2},
                "symbol":"circle",
                "symbolSize":12,
                "hoverAnimation":true,
                "legendHoverLink":true,
                "label":{"show":true,"position":"right","formatter":"{b}","fontSize":12},
                "itemStyle":{"color":"#ee6666","shadowBlur":10,"shadowColor":"#ee6666"},
                "data":[
                    {"value":[30,40],"name":"北京"},
                    {"value":[60,70],"name":"上海"},
                    {"value":[80,30],"name":"广州"},
                    {"value":[50,90],"name":"深圳"}
                ]
            }]
        }"##,
    );

    // ── 97. Candlestick/K 线图完整配置（itemStyle color/color0/borderColor 等） ──
    check(
        "candlestick_full_config",
        r##"{
            "xAxis":{"type":"category","data":["2024-01","2024-02","2024-03","2024-04","2024-05","2024-06"],"boundaryGap":true},
            "yAxis":{"type":"value","scale":true,"splitLine":{"show":true,"lineStyle":{"type":"dashed"}}},
            "series":[{
                "type":"candlestick",
                "name":"日K",
                "coordinateSystem":"cartesian2d",
                "barWidth":"70%",
                "barMinWidth":6,
                "barMaxWidth":30,
                "itemStyle":{
                    "color":"#ef5350",
                    "color0":"#26a69a",
                    "borderColor":"#ef5350",
                    "borderColor0":"#26a69a",
                    "borderWidth":1
                },
                "emphasis":{"focus":"series","itemStyle":{"borderWidth":2}},
                "data":[
                    [2000,2400,1800,2600],
                    [2400,2200,2000,2600],
                    [2200,2600,2100,2800],
                    [2600,2800,2500,3000],
                    [2800,2500,2400,3200],
                    [2500,2900,2300,3300]
                ]
            }]
        }"##,
    );

    // ── 98. Boxplot 箱形图完整配置（itemStyle style 颜色） ──
    check(
        "boxplot_full_config",
        r##"{
            "xAxis":{"type":"category","data":["Exp1","Exp2","Exp3","Exp4","Exp5"],"boundaryGap":true,"nameLocation":"middle"},
            "yAxis":{"type":"value","name":"数值","scale":true},
            "tooltip":{"trigger":"item","axisPointer":{"type":"shadow"}},
            "series":[{
                "type":"boxplot",
                "name":"实验",
                "coordinateSystem":"cartesian2d",
                "barWidth":"60%",
                "barMinWidth":10,
                "barMaxWidth":40,
                "layout":null,
                "itemStyle":{"color":"#fff","borderColor":"#5470c6","borderWidth":1.5},
                "emphasis":{"focus":"series"},
                "data":[
                    [655,850,940,980,1175],
                    [672.5,797.5,875,925,1037.5],
                    [730,860,930,1000,1110],
                    [678,797,865,923,1022],
                    [680,800,870,940,1080]
                ]
            }]
        }"##,
    );

    // ── 99. Gauge 仪表盘分段颜色（axisLine/axisTick/pointer/anchor/progress） ──
    check(
        "gauge_segments_full",
        r##"{
            "series":[{
                "type":"gauge",
                "name":"仪表盘",
                "center":["50%","60%"],
                "radius":"75%",
                "startAngle":210,
                "endAngle":-30,
                "min":0,
                "max":100,
                "splitNumber":10,
                "radiusAxisIndex":0,
                "clockwise":true,
                "axisLine":{"show":true,"roundCap":true,"lineStyle":{"width":20,"color":[[0.3,"#67e0e3"],[0.7,"#37a2da"],[1,"#fd666d"]]}},
                "progress":{"show":true,"width":20,"roundCap":true,"clip":true,"itemStyle":{"color":"#333"}},
                "axisTick":{"show":true,"length":8,"lineStyle":{"color":"#eee","width":1,"type":"solid"},"splitNumber":5},
                "splitLine":{"show":true,"length":15,"lineStyle":{"color":"#eee","width":2,"type":"solid"}},
                "axisLabel":{"show":true,"distance":30,"color":"#999","fontSize":12,"formatter":"{value}"},
                "pointer":{"show":true,"icon":null,"offsetCenter":[0,0],"length":"60%","width":6,"keepAspect":false,"itemStyle":{"color":"auto"}},
                "anchor":{"show":true,"showAbove":true,"size":20,"icon":"circle","offsetCenter":[0,0],"keepAspect":true,"itemStyle":{"borderWidth":10,"borderColor":"#fff","color":"#5470c6"}},
                "title":{"show":true,"offsetCenter":[0,"70%"],"color":"#999","fontSize":16,"fontWeight":"normal"},
                "detail":{"show":true,"valueAnimation":true,"formatter":"{value}%","color":"inherit","fontSize":30,"offsetCenter":[0,"90%"]},
                "data":[{"value":70,"name":"完成率"}]
            }]
        }"##,
    );

    // ── 100. dataZoom slider 完整配置（brushSelect/handleSize/showDetail/handleStyle） ──
    check(
        "datazoom_slider_full",
        r##"{
            "dataZoom":[{
                "type":"slider",
                "show":true,
                "orient":"horizontal",
                "xAxisIndex":[0],
                "start":10,
                "end":60,
                "width":null,
                "height":20,
                "left":"10%",
                "right":"10%",
                "top":"auto",
                "bottom":"20",
                "zoomLock":false,
                "realtime":true,
                "shrink":10,
                "disabled":false,
                "showDetail":true,
                "showDataShadow":"auto",
                "showHandle":true,
                "handleSize":"100%",
                "handleStyle":{"color":"#5470c6","borderColor":"#fff","borderWidth":1},
                "moveHandleSize":0,
                "brushSelect":true,
                "brushStyle":{"color":"rgba(84,112,198,0.3)","borderColor":"#5470c6","borderWidth":1},
                "preventDefaultMouseMove":true,
                "fillerColor":"rgba(84,112,198,0.2)",
                "borderColor":"#ddd",
                "borderRadius":4,
                "backgroundColor":"#f5f5f5",
                "dataBackground":{"lineStyle":{"color":"#2f4554","width":1,"type":"solid"},"areaStyle":{"color":"rgba(47,69,84,0.3)","opacity":0.3}},
                "selectedDataBackground":{"lineStyle":{"color":"#5470c6"},"areaStyle":{"color":"rgba(84,112,198,0.4)"}},
                "textStyle":{"color":"#999","fontSize":12}
            }],
            "xAxis":{"type":"category","data":["A","B","C","D","E","F","G","H","I","J"]},
            "yAxis":{"type":"value"},
            "series":[{"type":"bar","data":[1,2,3,4,5,6,7,8,9,10]}]
        }"##,
    );

    // ── 101. visualMap 分段型详细配置（pieces/text/itemWidth） ──
    check(
        "visualmap_piecewise_full",
        r##"{
            "visualMap":{
                "type":"piecewise",
                "show":true,
                "min":0,
                "max":100,
                "orient":"vertical",
                "left":"left",
                "top":"center",
                "splitNumber":5,
                "pieces":[
                    {"value":10,"label":"<=10 低","color":"#bee5f8"},
                    {"min":10,"max":50,"label":"10-50 中","color":"#74c6e8"},
                    {"min":50,"max":100,"label":">50 高","color":"#0077b6"}
                ],
                "category":null,
                "dimension":null,
                "seriesIndex":null,
                "hoverLink":true,
                "calculable":true,
                "inverse":false,
                "precision":0,
                "itemWidth":20,
                "itemHeight":14,
                "itemGap":10,
                "itemSymbol":"roundRect",
                "showLabel":true,
                "inRange":{"color":["#bee5f8","#74c6e8","#0077b6"],"colorAlpha":0.7},
                "outOfRange":{"color":"#ccc"},
                "controller":{"inRange":null,"outOfRange":null},
                "text":["高","低"],
                "textGap":10,
                "textStyle":{"color":"#333","fontSize":12}
            },
            "xAxis":{"type":"category","data":["a","b","c","d","e"]},
            "yAxis":{"type":"value"},
            "series":[{"type":"bar","data":[10,30,60,80,100]}]
        }"##,
    );

    // ── 102. Tooltip 详细配置（confine/enterable/triggerOn/formatter string） ──
    check(
        "tooltip_full_config",
        r##"{
            "tooltip":{
                "show":true,
                "showContent":true,
                "trigger":"axis",
                "triggerOn":"mousemove|click",
                "alwaysShowContent":false,
                "showDelay":0,
                "hideDelay":100,
                "enterable":false,
                "renderMode":"html",
                "confine":false,
                "appendToBody":false,
                "className":"echarts-tooltip",
                "order":"seriesAsc",
                "position":["50%","50%"],
                "formatter":"{b0}: {c0}<br/>{b1}: {c1}",
                "valueFormatter":null,
                "backgroundColor":"rgba(50,50,50,0.9)",
                "borderColor":"#333",
                "borderWidth":0,
                "padding":[5,10,5,10],
                "textStyle":{"color":"#fff","fontSize":14,"fontFamily":"sans-serif"},
                "extraCssText":"box-shadow: 0 0 3px rgba(0,0,0,0.3); border-radius: 4px;",
                "axisPointer":{
                    "type":"cross",
                    "snap":false,
                    "z":1,
                    "label":{"show":true,"color":"#fff","backgroundColor":"#5470c6","precision":2,"formatter":null,"padding":[2,4,2,4]},
                    "lineStyle":{"color":"#555","width":1,"type":"dashed"},
                    "crossStyle":{"color":"#999","width":1,"type":"dashed"},
                    "shadowStyle":{"color":"rgba(150,150,150,0.1)","opacity":0.3},
                    "triggerTooltip":true,
                    "triggerEmphasis":true
                }
            },
            "xAxis":{"type":"category","data":["A","B","C","D","E"]},
            "yAxis":{"type":"value"},
            "series":[
                {"type":"line","name":"S1","data":[120,132,101,134,90]},
                {"type":"line","name":"S2","data":[220,182,191,234,290]}
            ]
        }"##,
    );

    // ── 103. Legend scroll 型详细配置（pageIconSize/pageTextStyle/scrollDataIndex） ──
    check(
        "legend_scroll_full",
        r##"{
            "legend":{
                "type":"scroll",
                "show":true,
                "orient":"vertical",
                "left":"left",
                "top":"center",
                "align":"auto",
                "itemWidth":25,
                "itemHeight":14,
                "itemGap":10,
                "itemCheck":"auto",
                "symbolKeepAspect":true,
                "formatter":"{name}",
                "selectedMode":true,
                "inactiveColor":"#ccc",
                "inactiveBorderColor":"#ccc",
                "selected":{},
                "textStyle":{"color":"#333","fontSize":12,"fontFamily":"sans-serif"},
                "tooltip":{"show":false},
                "icon":"roundRect",
                "pageData":null,
                "pageIconSize":[15,15],
                "pageTextStyle":{"color":"#333","fontSize":12},
                "pageIconColor":"#2f4554",
                "pageIconInactiveColor":"#aaa",
                "pageIconBorderColor":"#2f4554",
                "pageIconInactiveBorderColor":"#aaa",
                "pageButtonPosition":"end",
                "pageFormatter":"{current}/{total}",
                "pageItemGap":6,
                "pageButtonItemGap":5,
                "pageGap":12,
                "scrollDataIndex":0,
                "animation":true,
                "animationDurationUpdate":800,
                "data":["邮件营销","联盟广告","视频广告","直接访问","搜索引擎","其他来源","推广活动","社交媒体"]
            },
            "series":[{
                "type":"pie",
                "data":[
                    {"name":"邮件营销","value":120},
                    {"name":"联盟广告","value":200},
                    {"name":"视频广告","value":150},
                    {"name":"直接访问","value":330},
                    {"name":"搜索引擎","value":400},
                    {"name":"其他来源","value":80},
                    {"name":"推广活动","value":250},
                    {"name":"社交媒体","value":320}
                ]
            }]
        }"##,
    );

    // ── 104. Title 详细配置（link/target/textAlign/verticalAlign/itemGap/border） ──
    check(
        "title_full_config",
        r##"{
            "title":{
                "show":true,
                "text":"主标题文本",
                "link":"https://example.com",
                "target":"blank",
                "subtext":"副标题辅助说明信息",
                "sublink":"https://sub.example.com",
                "subtarget":"blank",
                "left":"auto",
                "right":"auto",
                "top":"10",
                "bottom":"auto",
                "width":"auto",
                "height":"auto",
                "textAlign":"auto",
                "textVerticalAlign":"auto",
                "itemGap":10,
                "zlevel":0,
                "z":6,
                "backgroundColor":"transparent",
                "borderColor":"#ccc",
                "borderWidth":0,
                "borderRadius":4,
                "padding":[10,15,10,15],
                "shadowBlur":0,
                "shadowColor":"transparent",
                "shadowOffsetX":0,
                "shadowOffsetY":0,
                "textStyle":{
                    "color":"#333",
                    "fontStyle":"normal",
                    "fontWeight":"bold",
                    "fontFamily":"sans-serif",
                    "fontSize":18,
                    "lineHeight":20,
                    "width":null,
                    "height":null,
                    "textBorderColor":null,
                    "textBorderWidth":0,
                    "textShadowColor":"transparent",
                    "textShadowBlur":0,
                    "textShadowOffsetX":0,
                    "textShadowOffsetY":0,
                    "overflow":"none",
                    "ellipsis":"..."
                },
                "subtextStyle":{
                    "color":"#aaa",
                    "fontStyle":"normal",
                    "fontWeight":"normal",
                    "fontFamily":"sans-serif",
                    "fontSize":12,
                    "align":"left",
                    "verticalAlign":"top"
                },
                "rich":{}
            },
            "series":[{"type":"bar","data":[1,2,3]}]
        }"##,
    );

    // ── 105. Grid 详细配置（containLabel/shadow*） ──
    check(
        "grid_full_config",
        r##"{
            "grid":[
                {
                    "show":true,
                    "zlevel":0,
                    "z":2,
                    "left":"10%",
                    "right":"60%",
                    "top":80,
                    "bottom":"40%",
                    "width":"auto",
                    "height":"auto",
                    "containLabel":true,
                    "backgroundColor":"rgba(250,250,250,0.5)",
                    "borderColor":"#e5e5e5",
                    "borderWidth":1,
                    "borderRadius":4,
                    "shadowBlur":10,
                    "shadowColor":"rgba(0,0,0,0.1)",
                    "shadowOffsetX":5,
                    "shadowOffsetY":5,
                    "tooltip":{
                        "show":true,
                        "trigger":"axis",
                        "axisPointer":{"type":"shadow"}
                    }
                },
                {
                    "show":true,
                    "left":"60%",
                    "right":"10%",
                    "top":80,
                    "bottom":"40%",
                    "containLabel":true
                }
            ],
            "xAxis":[
                {"type":"category","gridIndex":0,"data":["A","B","C","D","E"]},
                {"type":"category","gridIndex":1,"data":["1","2","3","4","5"]}
            ],
            "yAxis":[
                {"type":"value","gridIndex":0},
                {"type":"value","gridIndex":1}
            ],
            "series":[
                {"type":"bar","xAxisIndex":0,"yAxisIndex":0,"data":[1,2,3,4,5]},
                {"type":"line","xAxisIndex":1,"yAxisIndex":1,"data":[5,4,3,2,1]}
            ]
        }"##,
    );

    // ── 106. MarkLine 完整配置（type min/max/average/symbol/silent/label） ──
    check(
        "markline_full_config",
        r##"{
            "xAxis":{"type":"category","data":["Mon","Tue","Wed","Thu","Fri","Sat","Sun"]},
            "yAxis":{"type":"value"},
            "series":[{
                "type":"line",
                "data":[820,932,901,934,1290,1330,1320],
                "markLine":{
                    "silent":false,
                    "symbol":["none","arrow"],
                    "symbolSize":[8,16],
                    "precision":2,
                    "animation":true,
                    "animationDuration":1000,
                    "animationEasing":"cubicOut",
                    "label":{"show":true,"position":"end","formatter":"{b}: {c}","color":"#333","fontSize":12},
                    "lineStyle":{"color":"#ee6666","width":1.5,"type":"solid","opacity":1,"curveness":0},
                    "emphasis":{"label":{"fontSize":14},"lineStyle":{"width":2}},
                    "data":[
                        {"type":"average","name":"平均值"},
                        {"type":"min","name":"最小值"},
                        {"type":"max","name":"最大值"},
                        {"name":"水平线","yAxis":1000},
                        {"name":"垂直线","xAxis":"Wed"},
                        [{"name":"斜线起点","coord":["Mon",820]},{"name":"斜线终点","coord":["Sun",1320]}]
                    ]
                }
            }]
        }"##,
    );

    // ── 107. MarkPoint 完整配置（type min/max/average/symbol/label position） ──
    check(
        "markpoint_full_config",
        r##"{
            "xAxis":{"type":"category","data":["A","B","C","D","E"]},
            "yAxis":{"type":"value"},
            "series":[{
                "type":"bar",
                "data":[320,332,301,334,390],
                "itemStyle":{"color":"#5470c6"},
                "markPoint":{
                    "symbol":"pin",
                    "symbolSize":50,
                    "symbolRotate":0,
                    "symbolKeepAspect":true,
                    "symbolOffset":[0,0],
                    "silent":false,
                    "animation":true,
                    "label":{"show":true,"position":"inside","color":"#fff","fontSize":12,"fontWeight":"bold","formatter":"{c}"},
                    "itemStyle":{"color":"#ee6666","borderColor":"#fff","borderWidth":1,"opacity":0.9},
                    "emphasis":{"label":{"fontSize":14}},
                    "data":[
                        {"type":"max","name":"最大值"},
                        {"type":"min","name":"最小值"},
                        {"type":"average","name":"均值"},
                        {"name":"自定义点","value":100,"xAxis":"C","yAxis":350,"symbolSize":80,"itemStyle":{"color":"#fac858"}}
                    ]
                }
            }]
        }"##,
    );

    // ── 108. 坐标轴 axisName* 完整配置（name/nameLocation/nameGap/nameTextStyle/nameTruncate） ──
    check(
        "axis_name_full_config",
        r##"{
            "xAxis":{
                "show":true,
                "position":"bottom",
                "type":"category",
                "name":"季度（Year Quarter）",
                "nameLocation":"middle",
                "nameGap":35,
                "nameRotate":0,
                "nameTextStyle":{"color":"#2f4554","fontSize":13,"fontWeight":"bold","fontFamily":"sans-serif","padding":[0,0,0,0]},
                "nameTruncate":{"maxWidth":180,"ellipsis":"..."},
                "inverse":false,
                "offset":0,
                "data":["Q1-春季","Q2-夏季","Q3-秋季","Q4-冬季"],
                "min":null,
                "max":null,
                "scale":false,
                "splitNumber":5,
                "boundaryGap":true,
                "minInterval":0,
                "maxInterval":null,
                "interval":null,
                "logBase":10,
                "silent":false,
                "triggerEvent":true,
                "axisLine":{"show":true,"onZero":true,"onZeroAxisIndex":0,"symbol":["none","arrow"],"symbolSize":[10,15],"lineStyle":{"color":"#333","width":1.5,"type":"solid"}}
            },
            "yAxis":{
                "show":true,
                "position":"left",
                "type":"value",
                "name":"销售额（万元）",
                "nameLocation":"middle",
                "nameGap":50,
                "nameRotate":90,
                "nameTextStyle":{"color":"#c23531","fontSize":14,"fontStyle":"italic"},
                "min":0,
                "max":500,
                "scale":true,
                "splitNumber":5
            },
            "series":[{"type":"bar","data":[120,200,150,180]}]
        }"##,
    );

    // ── 109. 混合系列（柱+折线+双Y轴+dataZoom+visualMap 全部） ──
    check(
        "mixed_all_components",
        r##"{
            "title":{"text":"混合组件看板","subtext":"多图元综合","left":"center"},
            "tooltip":{"trigger":"axis","axisPointer":{"type":"cross"}},
            "legend":{"data":["蒸发量","降水量","温度"],"top":30},
            "visualMap":{"show":true,"min":0,"max":40,"dimension":1,"seriesIndex":2,"inRange":{"symbolSize":[10,50],"color":["#50a3ba","#eac736","#d94e5d"]}},
            "dataZoom":[{"type":"slider","start":0,"end":100,"bottom":20,"height":15},{"type":"inside","start":0,"end":100}],
            "grid":{"left":"3%","right":"4%","bottom":"15%","top":"100","containLabel":true},
            "xAxis":{"type":"category","boundaryGap":true,"data":["1月","2月","3月","4月","5月","6月","7月","8月","9月","10月","11月","12月"]},
            "yAxis":[
                {"type":"value","name":"水量","position":"left","min":0,"max":300,"axisLabel":{"formatter":"{value} ml"}},
                {"type":"value","name":"温度","position":"right","min":-10,"max":40,"axisLabel":{"formatter":"{value} °C"}}
            ],
            "series":[
                {"type":"bar","name":"蒸发量","data":[2.0,4.9,7.0,23.2,25.6,76.7,135.6,162.2,32.6,20.0,6.4,3.3],"itemStyle":{"color":"#5470c6"}},
                {"type":"bar","name":"降水量","data":[2.6,5.9,9.0,26.4,28.7,70.7,175.6,182.2,48.7,18.8,6.0,2.3],"itemStyle":{"color":"#91cc75"}},
                {"type":"line","name":"温度","yAxisIndex":1,"data":[2,5,10,18,22,28,33,35,27,19,11,4],"itemStyle":{"color":"#ee6666"},"symbol":"circle","symbolSize":8}
            ]
        }"##,
    );

    // ── 110. dataset 多维 + transform filter ──
    check(
        "dataset_transform_filter",
        r##"{
            "dataset":[
                {
                    "source":[
                        ["product","2015","2016","2017"],
                        ["Matcha Latte",43.3,85.8,93.7],
                        ["Milk Tea",83.1,73.4,55.1],
                        ["Cheese Cocoa",86.4,65.2,82.5],
                        ["Walnut Brownie",72.4,53.9,39.1]
                    ]
                },
                {
                    "transform":{
                        "type":"filter",
                        "config":{"dimension":"product","value":"Milk Tea"}
                    },
                    "fromDatasetIndex":0
                },
                {
                    "transform":{
                        "type":"filter",
                        "config":{"and":[
                            {"dimension":"2015","gte":50},
                            {"dimension":"2017","gte":50}
                        ]}
                    },
                    "fromDatasetIndex":0
                }
            ],
            "xAxis":{"type":"category"},
            "yAxis":{"type":"value"},
            "series":[
                {"type":"bar","datasetIndex":1,"encode":{"x":"product","y":"2016"}},
                {"type":"bar","datasetIndex":2,"encode":{"x":"product","y":"2017"}}
            ]
        }"##,
    );

    println!("\nDone.");
}
