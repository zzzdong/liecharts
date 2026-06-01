use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("温度与降水量").subtext("双 Y 轴示例"))
        .legend(Legend::new().data(["温度", "降水量"]))
        .y_axis(Axis::value().name("温度 (°C)"))
        .y_axis(Axis::value().name("降水量 (mm)"))
        .add_line(
            Line::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [5.0, 8.0, 12.0, 18.0, 24.0, 30.0],
                ))
                .x("month")
                .y("value")
                .name("温度")
                .y_axis_index(0),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [50.0, 60.0, 80.0, 120.0, 150.0, 200.0],
                ))
                .x("month")
                .y("value")
                .name("降水量")
                .y_axis_index(1),
        )
        .render_to_svg("dual_axis.svg")?;
    println!("双轴图已保存到 dual_axis.svg");

    Ok(())
}
