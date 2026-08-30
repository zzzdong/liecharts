use liecharts::api::*;

#[path = "common/mod.rs"]
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
    common::save(&chart, "table.svg")?;

    Ok(())
}
