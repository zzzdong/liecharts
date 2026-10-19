//! 柱状图示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器（`Chart` + `Bar`）
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 两个函数配置完全一致（标题 / 图例 / 三组数据相同），输出分别落
//! `svg_output/bar_api.*` 与 `svg_output/bar_json.*`（SVG + PNG 各一份，
//! 见 `common::save`），可直接对比两种构建方式的结果。
//!
//! 运行：cargo run --example bar

use liecharts::ChartBuilder;
use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    via_api()?;
    via_json()?;
    Ok(())
}

/// Rust API 方式：分组柱状图（三个系列并列）。
fn via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("分组柱状图").subtext("API 构建"))
        .legend(Legend::new().data(["产品A", "产品B", "产品C"]))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "quarter" => ["Q1", "Q2", "Q3", "Q4"],
                    "value" => [120.0, 200.0, 150.0, 80.0],
                ))
                .x("quarter")
                .y("value")
                .name("产品A"),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "quarter" => ["Q1", "Q2", "Q3", "Q4"],
                    "value" => [80.0, 160.0, 120.0, 70.0],
                ))
                .x("quarter")
                .y("value")
                .name("产品B"),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "quarter" => ["Q1", "Q2", "Q3", "Q4"],
                    "value" => [60.0, 120.0, 90.0, 60.0],
                ))
                .x("quarter")
                .y("value")
                .name("产品C"),
        );
    common::save(&chart, "bar_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `via_api` 相同的分组柱状图配置。
fn via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "分组柱状图", "subtext": "JSON 构建" },
  "legend": { "data": ["产品A", "产品B", "产品C"] },
  "xAxis": {
    "type": "category",
    "data": ["Q1", "Q2", "Q3", "Q4"]
  },
  "yAxis": { "type": "value" },
  "series": [
    { "type": "bar", "name": "产品A", "data": [120, 200, 150, 80] },
    { "type": "bar", "name": "产品B", "data": [80, 160, 120, 70] },
    { "type": "bar", "name": "产品C", "data": [60, 120, 90, 60] }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "bar_json")?;
    Ok(())
}
