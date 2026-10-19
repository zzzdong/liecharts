//! 仪表盘示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器（`Chart` + `Gauge`）
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 配置：任务完成率 75.5%（量程 0–100），输出 `gauge_api` / `gauge_json` 两份对比。
//!
//! 运行：cargo run --example gauge

use liecharts::ChartBuilder;
use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    via_api()?;
    via_json()?;
    Ok(())
}

/// Rust API 方式：仪表盘。
fn via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("任务完成率").subtext("API 构建"))
        .add_gauge(
            Gauge::new()
                .data(dataframe!(
                    "name" => ["完成率"],
                    "value" => [75.5],
                ))
                .name("完成率")
                .value("value")
                .range(0.0, 100.0)
                .center(Size::pct(50.0), Size::pct(55.0))
                .radius(Size::pct(75.0)),
        );
    common::save(&chart, "gauge_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `via_api` 相同的仪表盘配置。
fn via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "任务完成率", "subtext": "JSON 构建" },
  "series": [
    {
      "type": "gauge",
      "name": "完成率",
      "min": 0,
      "max": 100,
      "center": ["50%", "55%"],
      "radius": "75%",
      "data": [{ "name": "完成率", "value": 75.5 }]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "gauge_json")?;
    Ok(())
}
