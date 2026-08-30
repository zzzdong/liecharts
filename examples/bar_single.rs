use liecharts::api::*;

#[path = "common/mod.rs"]
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("单系列柱状图（纵向）").subtext("2024年"))
        .legend(Legend::new().data(["销售额"]))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
                ))
                .x("month")
                .y("value")
                .name("销售额"),
        );
    common::save(&chart, "bar_single_v.svg")?;

    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("单系列柱状图（横向）"))
        .legend(Legend::new().data(["销售额"]))
        .x_axis(Axis::value().name("销售额（万元）"))
        .y_axis(Axis::category().data(["产品A", "产品B", "产品C", "产品D"]))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "product" => ["产品A", "产品B", "产品C", "产品D"],
                    "value" => [120.0, 200.0, 150.0, 80.0],
                ))
                .x("product")
                .y("value")
                .name("销售额"),
        );
    common::save(&chart, "bar_single_h.svg")?;

    Ok(())
}
