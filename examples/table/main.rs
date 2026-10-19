//! 表格图表示例（Rust API 构建）。
//!
//! 配置：手机产品销售数据表（4 行 × 5 列）。
//!
//! 说明：本示例仅提供 API 版本 —— ECharts option JSON 的 table 路径在
//! 兼容层（`pipeline::compat`）尚未实现（series 数据被置为空 DataFrame，
//! 渲染结果只有标题、没有表体），待兼容层补齐后可参照其它示例补上 `via_json`。
//!
//! 运行：cargo run --example table

use liecharts::api::*;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("产品销售数据表").subtext("Table Chart"))
        .add_table(
            Table::new()
                .data(dataframe!(
                    "产品" => ["iPhone 15", "Galaxy S24", "Pixel 8", "小米 14"],
                    "销量(万)" => [1200, 980, 450, 680],
                    "单价(元)" => [7999, 6999, 5999, 4299],
                    "总营收(亿)" => [959.9, 685.9, 269.9, 292.3],
                    "评分" => [4.8, 4.6, 4.5, 4.7]
                ))
                .name("销售数据"),
        );
    common::save(&chart, "table")?;
    Ok(())
}
