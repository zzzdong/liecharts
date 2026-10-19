//! 热力图示例：同一配置分别用 Rust API 与 ECharts option JSON 构建。
//!
//! - `via_api`  —— `liecharts::api` 链式构建器（`Chart` + `Heatmap`）
//! - `via_json` —— `ChartBuilder::from_option_json`（ECharts option 兼容层）
//!
//! 配置：7 天 × 8 小时打卡分布（value = (day + hour*2) % 10），
//! 输出 `heatmap_api` / `heatmap_json` 两份对比。
//!
//! 运行：cargo run --example heatmap

use liecharts::ChartBuilder;
use liecharts::api::*;
use liecharts::pipeline::dataframe::DataValue;
use lievisual::Color;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    via_api()?;
    via_json()?;
    Ok(())
}

/// 7 天 × 8 小时的模拟打卡数据（计算网格，故用迭代构造而非 `dataframe!` 字面量宏）。
fn grid_dataframe() -> liecharts::pipeline::dataframe::DataFrame {
    let xs: Vec<DataValue> = (0..7)
        .flat_map(|d| (0..8).map(move |_| DataValue::Float(d as f64)))
        .collect();
    let ys: Vec<DataValue> = (0..7)
        .flat_map(|_| (0..8).map(|h| DataValue::Float(h as f64)))
        .collect();
    let vals: Vec<DataValue> = (0..7)
        .flat_map(|d| (0..8).map(move |h| DataValue::Float(((d + h * 2) % 10) as f64)))
        .collect();
    let mut df = liecharts::pipeline::dataframe::DataFrame::new();
    df.add_column(liecharts::pipeline::dataframe::Series::new("x", xs));
    df.add_column(liecharts::pipeline::dataframe::Series::new("y", ys));
    df.add_column(liecharts::pipeline::dataframe::Series::new("value", vals));
    df
}

/// 与 `grid_dataframe` 等价的 JSON 数据点（`[x, y, value]`）。
fn grid_json_points() -> String {
    let mut s = String::new();
    for d in 0..7 {
        for h in 0..8 {
            if !s.is_empty() {
                s.push_str(",\n        ");
            }
            s.push_str(&format!("[{d}, {h}, {}]", (d + h * 2) % 10));
        }
    }
    s
}

/// Rust API 方式：热力图。
fn via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("热力图示例").subtext("API 构建"))
        .x_axis(Axis::category().data(["周一", "周二", "周三", "周四", "周五", "周六", "周日"]))
        .y_axis(Axis::category().data(["0点", "1点", "2点", "3点", "4点", "5点", "6点", "7点"]))
        .add_heatmap(
            Heatmap::new()
                .data(grid_dataframe())
                .x("x")
                .y("y")
                .value("value")
                .min(0.0)
                .max(10.0)
                .colors([
                    Color::rgb(80, 163, 186),
                    Color::rgb(234, 199, 54),
                    Color::rgb(217, 78, 93),
                ])
                .border_color(Color::rgb(255, 255, 255))
                .border_width(1.0)
                .name("打卡次数"),
        );
    common::save(&chart, "heatmap_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `via_api` 相同的热力图配置。
fn via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = format!(
        r#"
{{
  "title": {{ "text": "热力图示例", "subtext": "JSON 构建" }},
  "xAxis": {{
    "type": "category",
    "data": ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]
  }},
  "yAxis": {{
    "type": "category",
    "data": ["0点", "1点", "2点", "3点", "4点", "5点", "6点", "7点"]
  }},
  "series": [
    {{
      "type": "heatmap",
      "name": "打卡次数",
      "data": [
        {}
      ]
    }}
  ]
}}"#,
        grid_json_points()
    );
    let chart =
        ChartBuilder::from_option_json(&json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "heatmap_json")?;
    Ok(())
}
