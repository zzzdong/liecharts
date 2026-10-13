//! 饼图输出断言测试。
//!
//! 对应 `docs/svg_chart_checklist.md` 中"饼图"相关检查：
//! - 扇区数量与数据项一致
//! - 扇区面积/角度与数值成正比（数值最大的扇区占最大角度）
//! - 扇区颜色各不相同，与主题调色板一致
//! - 数据标签完整、位置在画布内
//!
//! 说明：lievisual 将饼图扇区渲染为 `Path` 元素（非 `Pie` 图元），
//! 因此用 `solid_filled_paths` 提取实心填充的扇区路径。

mod common;
use common::*;

const W: f64 = 800.0;
const H: f64 = 600.0;

/// 扇区数量应与饼图数据项数一致（5 项）。
#[test]
fn pie_sector_count_matches_data() {
    let nodes = render("pie", 800, 600);
    let sectors = solid_filled_paths(&nodes);
    assert_eq!(
        sectors.len(),
        5,
        "饼图应有 5 个扇区，实际 {} 个: {:?}",
        sectors.len(),
        sectors
    );
}

/// 扇区颜色应各不相同（5 个扇区 5 种颜色）。
#[test]
fn pie_sector_colors_distinct() {
    let nodes = render("pie", 800, 600);
    let sectors = solid_filled_paths(&nodes);
    let colors: std::collections::HashSet<String> =
        sectors.iter().map(|(_, c)| c.clone()).collect();
    assert_eq!(
        colors.len(),
        5,
        "饼图扇区应有 5 种不同颜色，实际 {} 种: {:?}",
        colors.len(),
        sectors.iter().map(|(_, c)| c.clone()).collect::<Vec<_>>()
    );
}

/// 所有扇区边界必须在画布内。
#[test]
fn pie_sectors_in_canvas() {
    let nodes = render("pie", 800, 600);
    let sectors = solid_filled_paths(&nodes);
    let pts: Vec<(f64, f64)> = sectors
        .iter()
        .flat_map(|(bb, _)| vec![(bb.x0, bb.y0), (bb.x1, bb.y1)])
        .collect();
    assert_all_points_in_canvas(&pts, W, H, 2.0);
}

/// 数值最大的扇区（搜索引擎=1548，占 60%）应占据最大的角度/面积。
///
/// 由于每个扇区都从圆心出发，面积与"角度/360°"成正比。最大的扇区
/// 应跨越超过 90°（搜索引起点角应超过圆周的 1/4）。
#[test]
fn pie_largest_sector_has_largest_area() {
    let nodes = render("pie", 800, 600);
    let sectors = solid_filled_paths(&nodes);
    // 找垂直范围最大的扇区（占比大的扇区在 y 方向上跨越更大）
    let biggest = sectors
        .iter()
        .max_by(|(a, _), (b, _)| {
            let ah = a.y1 - a.y0;
            let bh = b.y1 - b.y0;
            ah.partial_cmp(&bh).unwrap()
        })
        .unwrap();
    // 最大扇区应跨越超过 1/4 圆周对应的垂直范围（圆心在 y≈328.9）
    let span = biggest.0.y1 - biggest.0.y0;
    assert!(
        span > 100.0,
        "最大扇区（搜索引擎）应跨越较大垂直范围，实际 {:?}",
        biggest.0
    );
}

/// 5 个数据标签都应渲染（直接访问/邮件营销/联盟广告/视频广告/搜索引擎）。
#[test]
fn pie_labels_all_present() {
    let nodes = render("pie", 800, 600);
    let all_texts: Vec<String> = texts(&nodes).iter().map(|(t, _, _)| t.clone()).collect();
    for name in ["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"] {
        assert!(
            all_texts.iter().any(|t| t.contains(name)),
            "饼图缺少数据标签 '{}'。实际文本: {:?}",
            name,
            all_texts
        );
    }
}

/// 标签位置必须在画布内。
#[test]
fn pie_labels_in_canvas() {
    let nodes = render("pie", 800, 600);
    let pts: Vec<(f64, f64)> = texts(&nodes).iter().map(|(_, x, y)| (*x, *y)).collect();
    assert_all_points_in_canvas(&pts, W, H, 2.0);
}

/// 饼图扇区应闭合（渲染为完整区域而非开放线）。
#[test]
fn pie_sectors_closed() {
    let nodes = render("pie", 800, 600);
    let closed_count = paths(&nodes)
        .iter()
        .filter(|(_, _, closed)| *closed)
        .count();
    assert!(
        closed_count >= 5,
        "应有至少 5 个闭合路径（扇区），实际闭合 {} 个",
        closed_count
    );
}

/// ECharts 对齐回归（`examples/json_config.rs` 场景）：
///
/// 1. **饼图不参与 grid**：`radius: ['0%','70%']` 的半径 = 70% × min(画布宽,高)/2，
///    圆心在画布正中（`center: ['50%','50%']`），不被竖排图例的底部预留压缩。
///    历史实现把饼塞进默认 grid 并 clamp 到 grid 内接半径——8 项竖排图例把
///    grid 压到 206px 高，配置的 210px 被砍到 104px（官方同图为 263.2px / 70%）。
/// 2. **竖向图例行距** = max(itemHeight, 字号) + itemGap = 14 + 8 = 22px/档
///    （官方 v6 SVG 竖向图例实测同值；历史实现是 38.8px/项）。
#[test]
fn pie_geometry_and_vertical_legend_pitch_match_echarts() {
    let json = r#"
{
  "title": {"text": "安全事件类型分布"},
  "legend": {"orient": "vertical", "left": "left"},
  "series": [{
    "name": "安全事件类型分布",
    "type": "pie",
    "radius": ["0%", "70%"],
    "data": [
      {"name": "运维监控", "value": 67403},
      {"name": "拒绝服务", "value": 5764},
      {"name": "数据泄露", "value": 4155},
      {"name": "可疑行为", "value": 2458},
      {"name": "登录操作", "value": 1536},
      {"name": "漏洞攻击", "value": 141},
      {"name": "web攻击", "value": 103},
      {"name": "网络行为", "value": 27}
    ]
  }]
}
"#;
    let chart = liecharts::ChartBuilder::from_option_json(json)
        .unwrap()
        .build(800, 600)
        .unwrap();
    let svg = chart.render_svg().unwrap();

    // 半径/圆心：首扇区从 12 点钟方向起点 (cx, cy − r) 出发，路径以 "L400,300"
    // （圆心连线）收口。70% × min(800,600)/2 = 210 → 起点 (400, 90)。
    assert!(
        svg.contains("M400,90"),
        "饼图首扇区应从 (400, 90) 出发（圆心 (400,300)、半径 210 = 70% × min/2）"
    );
    assert!(
        svg.contains("L400,300"),
        "扇区路径应以画布中心 (400,300) 为顶点收口"
    );

    // 竖向图例：相邻两项文本基线差 = itemHeight(14) + itemGap(8) = 22
    let names = [
        "运维监控",
        "拒绝服务",
        "数据泄露",
        "可疑行为",
        "登录操作",
        "漏洞攻击",
        "web攻击",
        "网络行为",
    ];
    let mut ys: Vec<f64> = svg
        .split("<text")
        .skip(1)
        .filter_map(|seg| {
            let content = seg
                .split_once('>')
                .and_then(|(_, rest)| rest.split("</text>").next())
                .unwrap_or("");
            let y: f64 = seg
                .split("y=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .and_then(|s| s.parse().ok())?;
            names.contains(&content).then_some(y)
        })
        .collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(ys.len(), 8, "8 个图例项都应渲染，实际 {:?}", ys);
    for pair in ys.windows(2) {
        let pitch = pair[1] - pair[0];
        assert!(
            (pitch - 22.0).abs() < 0.5,
            "竖向图例相邻项间距应为 22px（itemHeight+itemGap），实际 {pitch}"
        );
    }
}
