//! 箱线图示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器（`Chart` + `Boxplot`）
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 配置：五分类箱线图（min/Q1/median/Q3/max），
//! 输出 `boxplot_api` / `boxplot_json` 两份对比。
//!
//! 运行：cargo run --example boxplot

use liecharts::ChartBuilder;
use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    via_api()?;
    via_json()?;
    Ok(())
}

/// Rust API 方式：箱线图。
fn via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("箱线图示例").subtext("API 构建"))
        .legend(Legend::new().data(["实验组 A"]))
        .add_boxplot(
            Boxplot::new()
                .data(dataframe!(
                    "category" => ["A", "B", "C", "D", "E"],
                    "min" => [10.0, 15.0, 8.0, 12.0, 18.0],
                    "q1" => [25.0, 30.0, 22.0, 28.0, 32.0],
                    "median" => [40.0, 45.0, 38.0, 42.0, 48.0],
                    "q3" => [55.0, 60.0, 52.0, 58.0, 62.0],
                    "max" => [70.0, 75.0, 68.0, 72.0, 78.0],
                ))
                .category("category")
                .min("min")
                .q1("q1")
                .median("median")
                .q3("q3")
                .max("max")
                .name("实验组 A"),
        );
    common::save(&chart, "boxplot_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `via_api` 相同的箱线图配置。
fn via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "箱线图示例", "subtext": "JSON 构建" },
  "legend": { "data": ["实验组 A"] },
  "xAxis": {
    "type": "category",
    "data": ["A", "B", "C", "D", "E"]
  },
  "yAxis": { "type": "value" },
  "series": [
    {
      "type": "boxplot",
      "name": "实验组 A",
      "data": [
        { "name": "A", "value": [10, 25, 40, 55, 70] },
        { "name": "B", "value": [15, 30, 45, 60, 75] },
        { "name": "C", "value": [8, 22, 38, 52, 68] },
        { "name": "D", "value": [12, 28, 42, 58, 72] },
        { "name": "E", "value": [18, 32, 48, 62, 78] }
      ]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "boxplot_json")?;
    Ok(())
}
