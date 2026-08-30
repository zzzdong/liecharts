use liecharts::api::*;

#[path = "common/mod.rs"]
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("折线图示例").subtext("Line Chart"))
        .legend(Legend::new().data(["销售额"]))
        .y_axis(Axis::value().name("销售额(元)"))
        .add_line(
            Line::new()
                .data(dataframe!(
                    "day" => ["周一", "周二", "周三", "周四", "周五", "周六", "周日"],
                    "revenue" => [120.0, 200.0, 150.0, 80.0, 70.0, 110.0, 130.0],
                ))
                .x("day")
                .y("revenue")
                .name("销售额")
                .smooth(true),
        );
    common::save(&chart, "line.svg")?;

    Ok(())
}
