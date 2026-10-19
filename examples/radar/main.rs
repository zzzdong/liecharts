//! 雷达图示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器（`Chart` + `Radar`）
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 配置：五维度双产品对比，输出 `radar_api` / `radar_json` 两份对比。
//!
//! 运行：cargo run --example radar

use liecharts::ChartBuilder;
use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    via_api()?;
    via_json()?;
    Ok(())
}

/// Rust API 方式：雷达图。
fn via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("产品能力雷达图").subtext("API 构建"))
        .legend(Legend::new().data(["产品A", "产品B"]))
        .add_radar(
            Radar::new(vec![
                "销量".into(),
                "品牌".into(),
                "增长".into(),
                "满意度".into(),
                "市占".into(),
            ])
            .data(dataframe!(
                "name" => ["产品A"],
                "value" => ["95,80,75,90,85"],
            ))
            .name("产品A")
            .values("value"),
        )
        .add_radar(
            Radar::new(vec![
                "销量".into(),
                "品牌".into(),
                "增长".into(),
                "满意度".into(),
                "市占".into(),
            ])
            .data(dataframe!(
                "name" => ["产品B"],
                "value" => ["70,95,90,75,60"],
            ))
            .name("产品B")
            .values("value"),
        );
    common::save(&chart, "radar_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `via_api` 相同的雷达图配置。
fn via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "产品能力雷达图", "subtext": "JSON 构建" },
  "legend": { "data": ["产品A", "产品B"] },
  "radar": {
    "indicator": [
      { "name": "销量", "max": 100 },
      { "name": "品牌", "max": 100 },
      { "name": "增长", "max": 100 },
      { "name": "满意度", "max": 100 },
      { "name": "市占", "max": 100 }
    ]
  },
  "series": [
    {
      "type": "radar",
      "name": "产品A",
      "data": [{ "name": "产品A", "value": [95, 80, 75, 90, 85] }]
    },
    {
      "type": "radar",
      "name": "产品B",
      "data": [{ "name": "产品B", "value": [70, 95, 90, 75, 60] }]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "radar_json")?;
    Ok(())
}
