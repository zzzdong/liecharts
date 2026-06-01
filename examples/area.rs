use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("访问量趋势面积图").subtext("Area Chart"))
        .legend(Legend::new().data(["访问量"]))
        .add_line(
            Line::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
                ))
                .x("month")
                .y("value")
                .name("访问量")
                .area(true),
        )
        .render_to_svg("area.svg")?;
    println!("面积图已保存到 area.svg");

    Ok(())
}
