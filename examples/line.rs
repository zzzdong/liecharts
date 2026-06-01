use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
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
        )
        .render_to_svg("line.svg")?;
    println!("折线图已保存到 line.svg");

    Ok(())
}
