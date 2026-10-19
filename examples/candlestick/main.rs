//! K 线图示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器（`Chart` + `Candlestick`）
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 配置：9 个交易日 OHLC，输出 `candlestick_api` / `candlestick_json` 两份对比。
//!
//! 运行：cargo run --example candlestick

use liecharts::ChartBuilder;
use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    via_api()?;
    via_json()?;
    Ok(())
}

/// Rust API 方式：K 线图。
fn via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("K线图示例").subtext("API 构建"))
        .legend(Legend::new().data(["日K"]))
        .add_candlestick(
            Candlestick::new()
                .data(dataframe!(
                    "date" => ["2024-01-02", "2024-01-03", "2024-01-04", "2024-01-05", "2024-01-08", "2024-01-09", "2024-01-10", "2024-01-11", "2024-01-12"],
                    "open" => [100.0, 105.0, 102.0, 108.0, 115.0, 112.0, 118.0, 125.0, 120.0],
                    "close" => [105.0, 102.0, 108.0, 115.0, 112.0, 118.0, 125.0, 120.0, 122.0],
                    "low" => [98.0, 100.0, 101.0, 106.0, 110.0, 111.0, 116.0, 118.0, 115.0],
                    "high" => [108.0, 110.0, 112.0, 118.0, 120.0, 125.0, 128.0, 130.0, 125.0],
                ))
                .category("date")
                .open("open")
                .close("close")
                .low("low")
                .high("high")
                .name("日K"),
        );
    common::save(&chart, "candlestick_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `via_api` 相同的 K 线配置。
fn via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "K线图示例", "subtext": "JSON 构建" },
  "legend": { "data": ["日K"] },
  "xAxis": {
    "type": "category",
    "data": ["2024-01-02", "2024-01-03", "2024-01-04", "2024-01-05", "2024-01-08", "2024-01-09", "2024-01-10", "2024-01-11", "2024-01-12"]
  },
  "yAxis": { "type": "value" },
  "series": [
    {
      "type": "candlestick",
      "name": "日K",
      "data": [
        { "name": "2024-01-02", "open": 100, "close": 105, "low": 98,  "high": 108 },
        { "name": "2024-01-03", "open": 105, "close": 102, "low": 100, "high": 110 },
        { "name": "2024-01-04", "open": 102, "close": 108, "low": 101, "high": 112 },
        { "name": "2024-01-05", "open": 108, "close": 115, "low": 106, "high": 118 },
        { "name": "2024-01-08", "open": 115, "close": 112, "low": 110, "high": 120 },
        { "name": "2024-01-09", "open": 112, "close": 118, "low": 111, "high": 125 },
        { "name": "2024-01-10", "open": 118, "close": 125, "low": 116, "high": 128 },
        { "name": "2024-01-11", "open": 125, "close": 120, "low": 118, "high": 130 },
        { "name": "2024-01-12", "open": 120, "close": 122, "low": 115, "high": 125 }
      ]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "candlestick_json")?;
    Ok(())
}
