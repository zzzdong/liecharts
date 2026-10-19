//! 极坐标图表示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器（`PolarBar` / `PolarScatter`）
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 两个场景（各含 api/json 一对，配置一致）：
//! 1. 极坐标柱状图 —— 五分类访问来源（角度 + 半径）
//! 2. 极坐标散点图 —— 24 个风向/风速采样点
//!
//! 运行：cargo run --example polar

use liecharts::ChartBuilder;
use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    polar_bar_via_api()?;
    polar_bar_via_json()?;
    polar_scatter_via_api()?;
    polar_scatter_via_json()?;
    Ok(())
}

// ── 场景 1：极坐标柱状图 ──

/// Rust API 方式：极坐标柱状图。
fn polar_bar_via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("极坐标柱状图").subtext("API 构建"))
        .legend(Legend::new().data(["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"]))
        .add_polar_bar(
            PolarBar::new()
                .data(dataframe!(
                    "label" => ["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"],
                    "angle" => [0.0, 72.0, 144.0, 216.0, 288.0],
                    "radius" => [335.0, 310.0, 234.0, 135.0, 1548.0],
                ))
                .name("访问来源")
                .angle("angle")
                .radius("radius"),
        );
    common::save(&chart, "polar_bar_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `polar_bar_via_api` 相同的配置。
///
/// ECharts 标准极坐标写法：`type:"bar"` + `coordinateSystem:"polar"`，
/// 类目名声明在 `angleAxis.data`。
fn polar_bar_via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "极坐标柱状图", "subtext": "JSON 构建" },
  "legend": { "data": ["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"] },
  "angleAxis": {
    "type": "category",
    "data": ["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"]
  },
  "radiusAxis": { "type": "value" },
  "series": [
    {
      "type": "bar",
      "coordinateSystem": "polar",
      "name": "访问来源",
      "data": [
        [0, 335],
        [72, 310],
        [144, 234],
        [216, 135],
        [288, 1548]
      ]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "polar_bar_json")?;
    Ok(())
}

// ── 场景 2：极坐标散点图 ──

/// Rust API 方式：极坐标散点图。
fn polar_scatter_via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("极坐标散点图").subtext("API 构建"))
        .legend(Legend::new().data(["风速"]))
        .add_polar_scatter(
            PolarScatter::new()
                .data(dataframe!(
                    "angle" => [0.0, 15.0, 30.0, 45.0, 60.0, 75.0, 90.0, 105.0, 120.0, 135.0, 150.0, 165.0, 180.0, 195.0, 210.0, 225.0, 240.0, 255.0, 270.0, 285.0, 300.0, 315.0, 330.0, 345.0],
                    "radius" => [5.0, 8.0, 12.0, 15.0, 10.0, 7.0, 20.0, 18.0, 14.0, 9.0, 6.0, 4.0, 11.0, 16.0, 22.0, 19.0, 13.0, 8.0, 17.0, 21.0, 25.0, 23.0, 15.0, 10.0],
                ))
                .name("风速")
                .angle("angle")
                .radius("radius")
                .symbol_size(5.0),
        );
    common::save(&chart, "polar_scatter_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `polar_scatter_via_api` 相同的配置。
fn polar_scatter_via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "极坐标散点图", "subtext": "JSON 构建" },
  "legend": { "data": ["风速"] },
  "angleAxis": { "type": "value" },
  "radiusAxis": { "type": "value" },
  "series": [
    {
      "type": "polarScatter",
      "name": "风速",
      "symbolSize": 5,
      "data": [
        { "angle": 0,   "radius": 5 },
        { "angle": 15,  "radius": 8 },
        { "angle": 30,  "radius": 12 },
        { "angle": 45,  "radius": 15 },
        { "angle": 60,  "radius": 10 },
        { "angle": 75,  "radius": 7 },
        { "angle": 90,  "radius": 20 },
        { "angle": 105, "radius": 18 },
        { "angle": 120, "radius": 14 },
        { "angle": 135, "radius": 9 },
        { "angle": 150, "radius": 6 },
        { "angle": 165, "radius": 4 },
        { "angle": 180, "radius": 11 },
        { "angle": 195, "radius": 16 },
        { "angle": 210, "radius": 22 },
        { "angle": 225, "radius": 19 },
        { "angle": 240, "radius": 13 },
        { "angle": 255, "radius": 8 },
        { "angle": 270, "radius": 17 },
        { "angle": 285, "radius": 21 },
        { "angle": 300, "radius": 25 },
        { "angle": 315, "radius": 23 },
        { "angle": 330, "radius": 15 },
        { "angle": 345, "radius": 10 }
      ]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "polar_scatter_json")?;
    Ok(())
}
