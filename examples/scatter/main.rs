//! 散点图示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器（`Chart` + `Scatter`）
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 配置：XY 二维散点（两个系列），输出 `scatter_api` / `scatter_json` 两份对比。
//!
//! 运行：cargo run --example scatter

use liecharts::ChartBuilder;
use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    via_api()?;
    via_json()?;
    Ok(())
}

/// Rust API 方式：XY 散点图。
fn via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("身高体重分布").subtext("API 构建"))
        .legend(Legend::new().data(["男生", "女生"]))
        .x_axis(Axis::value().name("身高(cm)"))
        .y_axis(Axis::value().name("体重(kg)"))
        .add_scatter(
            Scatter::new()
                .data(dataframe!(
                    "height" => [170.0, 175.0, 168.0, 180.0, 172.0, 178.0],
                    "weight" => [65.0, 70.0, 62.0, 78.0, 68.0, 74.0],
                ))
                .x("height")
                .y("weight")
                .name("男生")
                .symbol_size(12.0),
        )
        .add_scatter(
            Scatter::new()
                .data(dataframe!(
                    "height" => [160.0, 165.0, 158.0, 170.0, 162.0, 168.0],
                    "weight" => [50.0, 55.0, 48.0, 60.0, 52.0, 58.0],
                ))
                .x("height")
                .y("weight")
                .name("女生")
                .symbol_size(12.0),
        );
    common::save(&chart, "scatter_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `via_api` 相同的散点配置。
fn via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "身高体重分布", "subtext": "JSON 构建" },
  "legend": { "data": ["男生", "女生"] },
  "xAxis": { "type": "value", "name": "身高(cm)" },
  "yAxis": { "type": "value", "name": "体重(kg)" },
  "series": [
    {
      "type": "scatter",
      "name": "男生",
      "symbolSize": 12,
      "data": [[170, 65], [175, 70], [168, 62], [180, 78], [172, 68], [178, 74]]
    },
    {
      "type": "scatter",
      "name": "女生",
      "symbolSize": 12,
      "data": [[160, 50], [165, 55], [158, 48], [170, 60], [162, 52], [168, 58]]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "scatter_json")?;
    Ok(())
}
