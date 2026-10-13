//! 底部图例与绘图区 / 轴标签的重叠回归。
//!
//! 历史缺陷（mixed_grid.svg / hug_demo_3 场景）：
//! 1. 自动图例（未显式配置 `legend` 的多系列图表）不参与底部空间预留，
//!    图例压在底部子图的绘图区与 X 轴标签上；
//! 2. 用户显式 `grid.bottom` 小于图例占用时（多子图各留 3% 边距），
//!    绘图区与轴标签侵入底部图例带，无任何让位。
//!
//! 修复后：图例带成为 planner 的硬约束——绘图区与底部轴标签都以带顶为界。

use liecharts::api::*;

/// 提取所有 `<text>` 的 (内容, x, y)（与 hug_mode_test 同口径）。
fn text_entries(svg: &str) -> Vec<(String, f64, f64)> {
    let mut out = Vec::new();
    for seg in svg.split("<text").skip(1) {
        let attr = |name: &str| -> Option<f64> {
            let needle = format!("{name}=\"");
            seg.split(needle.as_str())
                .nth(1)
                .and_then(|s| s.split('"').next())
                .and_then(|s| s.parse().ok())
        };
        let content = seg
            .split_once('>')
            .and_then(|(_, rest)| rest.split("</text>").next())
            .unwrap_or("")
            .to_string();
        if let (Some(x), Some(y)) = (attr("x"), attr("y")) {
            out.push((content, x, y));
        }
    }
    out
}

/// 4 个 sub_grid 的底边距都压到 3%：贴底自动图例必须整体落到
/// 全部 X 轴类别标签之下（而不是压在底部两个子图上）。
#[test]
fn legend_stays_below_axis_labels_in_multi_grid() {
    let legend_names = [
        "柱状图-子图1",
        "折线图-子图2",
        "柱状图-子图3",
        "折线图-子图4",
    ];
    let cat_labels = [
        "1月", "2月", "3月", "4月", "5月", "6月", "周一", "周二", "周三", "周四", "周五", "周六",
        "产品A", "产品B", "产品C", "产品D", "产品E", "Q1", "Q2", "Q3", "Q4",
    ];
    let chart = mixed_grid_chart();
    let svg = chart.render_svg().unwrap();
    let texts = text_entries(&svg);

    let legend_ys: Vec<f64> = texts
        .iter()
        .filter(|(t, _, _)| legend_names.contains(&t.as_str()))
        .map(|(_, _, y)| *y)
        .collect();
    let label_ys: Vec<f64> = texts
        .iter()
        .filter(|(t, _, _)| cat_labels.contains(&t.as_str()))
        .map(|(_, _, y)| *y)
        .collect();

    assert_eq!(legend_ys.len(), 4, "4 个图例项都应渲染");
    assert!(!label_ys.is_empty(), "应存在 X 轴类别标签");

    let lowest_label = label_ys.iter().cloned().fold(f64::MIN, f64::max);
    let highest_legend = legend_ys.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        highest_legend > lowest_label + 15.0,
        "图例（最高基线 y={highest_legend}）应明显低于全部 X 轴标签（最低基线 y={lowest_label}），\
         不得压入底部子图"
    );
}

/// 6 系列 × 320px 窄画布：自动图例换行成多行时，底部空间必须按
/// 换行后的实际高度预留，X 轴标签不得压入图例带（hug_demo_3 场景）。
#[test]
fn multi_row_auto_legend_reserves_footer_space() {
    let names = [
        "营业收入",
        "营业成本",
        "毛利润",
        "净利润",
        "研发支出",
        "管理费用",
    ];
    let df = liecharts::dataframe!(
        "day" => ["周一", "周二", "周三", "周四", "周五"],
        "v1" => [1.0, 2.0, 3.0, 4.0, 5.0],
    );
    let mut chart = Chart::new(320, 240)
        .title(Title::new("Hug: 图例换行"))
        .y_axis(Axis::value())
        .data(df.clone());
    chart = chart.add_line(Line::new().name(names[0]).data(df.clone()).x("day").y("v1"));
    for name in &names[1..] {
        chart = chart.add_line(Line::new().name(*name).data(df.clone()).x("day").y("v1"));
    }
    let svg = chart.render_svg().unwrap();
    let texts = text_entries(&svg);

    let legend_ys: Vec<f64> = texts
        .iter()
        .filter(|(t, _, _)| names.contains(&t.as_str()))
        .map(|(_, _, y)| *y)
        .collect();
    let label_ys: Vec<f64> = texts
        .iter()
        .filter(|(t, _, _)| ["周一", "周二", "周三", "周四", "周五"].contains(&t.as_str()))
        .map(|(_, _, y)| *y)
        .collect();

    assert_eq!(legend_ys.len(), 6, "6 个图例项都应渲染");
    assert_eq!(label_ys.len(), 5, "5 个 X 轴标签都应渲染");

    let lowest_label = label_ys.iter().cloned().fold(f64::MIN, f64::max);
    let highest_legend = legend_ys.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        highest_legend > lowest_label,
        "多行图例（最高基线 y={highest_legend}）应整体位于 X 轴标签（最低基线 y={lowest_label}）之下"
    );
}

/// mixed_grid 示例同款：4 个 sub_grid + 各 1 系列（自动图例）。
fn mixed_grid_chart() -> Chart {
    Chart::new(1000, 900)
        .title(Title::new("混合布局图表").subtext("sub_grid 闭包风格"))
        .sub_grid(
            Grid::new()
                .left(Position::pct(3.0))
                .top(Position::pct(12.0))
                .right(Position::pct(52.0))
                .bottom(Position::pct(52.0)),
            |g| {
                g.x_axis(Axis::category().data(["1月", "2月", "3月", "4月", "5月", "6月"]))
                    .y_axis(Axis::value())
                    .add_layer(
                        Bar::new()
                            .data(dataframe!(
                                "cat" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                                "val" => [120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
                            ))
                            .x("cat")
                            .y("val")
                            .name("柱状图-子图1"),
                    )
            },
        )
        .sub_grid(
            Grid::new()
                .left(Position::pct(52.0))
                .top(Position::pct(12.0))
                .right(Position::pct(3.0))
                .bottom(Position::pct(52.0)),
            |g| {
                g.x_axis(Axis::category().data(["周一", "周二", "周三", "周四", "周五", "周六"]))
                    .y_axis(Axis::value())
                    .add_layer(
                        Line::new()
                            .data(dataframe!(
                                "cat" => ["周一", "周二", "周三", "周四", "周五", "周六"],
                                "val" => [50.0, 80.0, 120.0, 90.0, 60.0, 100.0],
                            ))
                            .x("cat")
                            .y("val")
                            .name("折线图-子图2"),
                    )
            },
        )
        .sub_grid(
            Grid::new()
                .left(Position::pct(3.0))
                .top(Position::pct(52.0))
                .right(Position::pct(52.0))
                .bottom(Position::pct(3.0)),
            |g| {
                g.x_axis(Axis::category().data(["产品A", "产品B", "产品C", "产品D", "产品E"]))
                    .y_axis(Axis::value())
                    .add_layer(
                        Bar::new()
                            .data(dataframe!(
                                "cat" => ["产品A", "产品B", "产品C", "产品D", "产品E"],
                                "val" => [300.0, 250.0, 180.0, 280.0, 220.0],
                            ))
                            .x("cat")
                            .y("val")
                            .name("柱状图-子图3"),
                    )
            },
        )
        .sub_grid(
            Grid::new()
                .left(Position::pct(52.0))
                .top(Position::pct(52.0))
                .right(Position::pct(3.0))
                .bottom(Position::pct(3.0)),
            |g| {
                g.x_axis(Axis::category().data(["Q1", "Q2", "Q3", "Q4"]))
                    .y_axis(Axis::value())
                    .add_layer(
                        Line::new()
                            .data(dataframe!(
                                "cat" => ["Q1", "Q2", "Q3", "Q4"],
                                "val" => [100.0, 150.0, 120.0, 180.0],
                            ))
                            .x("cat")
                            .y("val")
                            .name("折线图-子图4"),
                    )
            },
        )
}
