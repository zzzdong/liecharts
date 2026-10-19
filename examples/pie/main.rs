//! 饼图示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器（`Chart` + `Pie`）
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 配置：五分类饼图（radius 0%–75%、显示标签），输出 `pie_api` / `pie_json`
//! 两份对比。
//!
//! 运行：cargo run --example pie

use liecharts::ChartBuilder;
use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    via_api()?;
    via_json()?;
    Ok(())
}

/// Rust API 方式：五分类饼图。
fn via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("某站点用户访问来源").subtext("API 构建"))
        .legend(Legend::new().data(["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"]))
        .add_pie(
            Pie::new()
                .data(dataframe!(
                    "source" => ["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"],
                    "value" => [335.0, 310.0, 234.0, 135.0, 1548.0],
                ))
                .name("访问来源")
                .category("source")
                .value("value")
                .radius(Size::pct(0.0), Size::pct(75.0))
                .label(true),
        );
    common::save(&chart, "pie_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `via_api` 相同的饼图配置。
fn via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "某站点用户访问来源", "subtext": "JSON 构建" },
  "legend": { "data": ["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"] },
  "series": [
    {
      "type": "pie",
      "name": "访问来源",
      "radius": ["0%", "75%"],
      "label": { "show": true },
      "data": [
        { "name": "直接访问", "value": 335 },
        { "name": "邮件营销", "value": 310 },
        { "name": "联盟广告", "value": 234 },
        { "name": "视频广告", "value": 135 },
        { "name": "搜索引擎", "value": 1548 }
      ]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "pie_json")?;
    Ok(())
}
