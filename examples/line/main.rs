//! 折线图示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器（`Chart` + `Line`）
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 配置：两个系列（「访问量」平滑曲线 + 「下单量」普通折线），与 `bar` 示例
//! 同样输出 `*_api` / `*_json` 两份，便于横向对比。
//!
//! 运行：cargo run --example line

use liecharts::ChartBuilder;
use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    via_api()?;
    via_json()?;
    Ok(())
}

/// Rust API 方式：平滑折线 + 普通折线。
fn via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("访问量与下单量").subtext("API 构建"))
        .legend(Legend::new().data(["访问量", "下单量"]))
        .add_line(
            Line::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0],
                ))
                .x("month")
                .y("value")
                .name("访问量")
                .smooth(true),
        )
        .add_line(
            Line::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [220.0, 282.0, 271.0, 334.0, 390.0, 430.0],
                ))
                .x("month")
                .y("value")
                .name("下单量"),
        );
    common::save(&chart, "line_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `via_api` 相同的折线配置。
fn via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "访问量与下单量", "subtext": "JSON 构建" },
  "legend": { "data": ["访问量", "下单量"] },
  "xAxis": {
    "type": "category",
    "data": ["1月", "2月", "3月", "4月", "5月", "6月"]
  },
  "yAxis": { "type": "value" },
  "series": [
    {
      "type": "line",
      "name": "访问量",
      "smooth": true,
      "data": [820, 932, 901, 934, 1290, 1330]
    },
    {
      "type": "line",
      "name": "下单量",
      "data": [220, 282, 271, 334, 390, 430]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "line_json")?;
    Ok(())
}
