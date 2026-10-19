//! 组合图表示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 两个场景（各含 api/json 一对，配置一致）：
//! 1. 双轴组合 —— 柱状图（销量，左轴）+ 折线图（增长率，右轴）
//! 2. 多子图 —— 3 个 grid 分别放柱状 / 折线 / 柱状
//!
//! 运行：cargo run --example combo

use liecharts::ChartBuilder;
use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dual_axis_via_api()?;
    dual_axis_via_json()?;
    multi_grid_via_api()?;
    multi_grid_via_json()?;
    Ok(())
}

// ── 场景 1：双轴组合图 ──

/// Rust API 方式：双轴组合图。
fn dual_axis_via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("温度与降水量").subtext("API 构建"))
        .legend(Legend::new().data(["温度", "降水量"]))
        .y_axis(Axis::value().name("温度 (°C)"))
        .y_axis(Axis::value().name("降水量 (mm)"))
        .add_line(
            Line::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [5.0, 8.0, 12.0, 18.0, 24.0, 30.0],
                ))
                .x("month")
                .y("value")
                .name("温度"),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [50.0, 60.0, 80.0, 120.0, 150.0, 200.0],
                ))
                .x("month")
                .y("value")
                .name("降水量")
                .right_axis(),
        );
    common::save(&chart, "combo_dual_axis_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `dual_axis_via_api` 相同的配置。
fn dual_axis_via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "温度与降水量", "subtext": "JSON 构建" },
  "legend": { "data": ["温度", "降水量"] },
  "xAxis": {
    "type": "category",
    "data": ["1月", "2月", "3月", "4月", "5月", "6月"]
  },
  "yAxis": [
    { "type": "value", "name": "温度 (°C)" },
    { "type": "value", "name": "降水量 (mm)" }
  ],
  "series": [
    {
      "type": "line",
      "name": "温度",
      "data": [5, 8, 12, 18, 24, 30]
    },
    {
      "type": "bar",
      "name": "降水量",
      "yAxisIndex": 1,
      "data": [50, 60, 80, 120, 150, 200]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "combo_dual_axis_json")?;
    Ok(())
}

// ── 场景 2：多子图网格 ──

/// Rust API 方式：3 个子图（柱状 / 折线 / 柱状）。
fn multi_grid_via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(1000, 800)
        .title(Title::new("多子图展示").subtext("API 构建"))
        .grid(
            Grid::new()
                .left(Position::pct(3.0))
                .top(Position::pct(15.0))
                .right(Position::pct(52.0))
                .bottom(Position::pct(52.0)),
        )
        .grid(
            Grid::new()
                .left(Position::pct(52.0))
                .top(Position::pct(15.0))
                .right(Position::pct(3.0))
                .bottom(Position::pct(52.0)),
        )
        .grid(
            Grid::new()
                .left(Position::pct(3.0))
                .top(Position::pct(52.0))
                .right(Position::pct(3.0))
                .bottom(Position::pct(3.0)),
        )
        .x_axis(
            Axis::category()
                .data(["1月", "2月", "3月", "4月", "5月", "6月"])
                .grid_index(0),
        )
        .x_axis(
            Axis::category()
                .data(["1月", "2月", "3月", "4月", "5月", "6月"])
                .grid_index(1),
        )
        .x_axis(
            Axis::category()
                .data(["1月", "2月", "3月", "4月", "5月", "6月"])
                .grid_index(2),
        )
        .y_axis(Axis::value().grid_index(0))
        .y_axis(Axis::value().grid_index(1))
        .y_axis(Axis::value().grid_index(2))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "cat" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "val" => [120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
                ))
                .x("cat")
                .y("val")
                .name("子图1-柱状图")
                .grid_index(0),
        )
        .add_line(
            Line::new()
                .data(dataframe!(
                    "cat" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "val" => [30.0, 50.0, 80.0, 120.0, 90.0, 60.0],
                ))
                .x("cat")
                .y("val")
                .name("子图2-折线图")
                .grid_index(1),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "cat" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "val" => [200.0, 300.0, 250.0, 180.0, 220.0, 280.0],
                ))
                .x("cat")
                .y("val")
                .name("子图3")
                .grid_index(2),
        );
    common::save(&chart, "combo_multi_grid_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `multi_grid_via_api` 相同的配置。
fn multi_grid_via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "多子图展示", "subtext": "JSON 构建" },
  "grid": [
    { "left": "3%",  "top": "15%", "right": "52%", "bottom": "52%" },
    { "left": "52%", "top": "15%", "right": "3%",  "bottom": "52%" },
    { "left": "3%",  "top": "52%", "right": "3%",  "bottom": "3%" }
  ],
  "xAxis": [
    { "type": "category", "gridIndex": 0, "data": ["1月", "2月", "3月", "4月", "5月", "6月"] },
    { "type": "category", "gridIndex": 1, "data": ["1月", "2月", "3月", "4月", "5月", "6月"] },
    { "type": "category", "gridIndex": 2, "data": ["1月", "2月", "3月", "4月", "5月", "6月"] }
  ],
  "yAxis": [
    { "type": "value", "gridIndex": 0 },
    { "type": "value", "gridIndex": 1 },
    { "type": "value", "gridIndex": 2 }
  ],
  "series": [
    {
      "type": "bar", "name": "子图1-柱状图", "gridIndex": 0,
      "data": [120, 200, 150, 80, 70, 110]
    },
    {
      "type": "line", "name": "子图2-折线图", "gridIndex": 1,
      "data": [30, 50, 80, 120, 90, 60]
    },
    {
      "type": "bar", "name": "子图3", "gridIndex": 2,
      "data": [200, 300, 250, 180, 220, 280]
    }
  ]
}"#;
    let chart = ChartBuilder::from_option_json(json)?.build(1000, 800)?;
    common::save(&chart, "combo_multi_grid_json")?;
    Ok(())
}
