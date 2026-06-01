use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("堆叠面积图").subtext("Stacked Area Chart"))
        .legend(Legend::new().data(["产品A", "产品B", "产品C"]))
        .add_line(
            Line::new()
                .data(dataframe!(
                    "day" => ["周一", "周二", "周三", "周四", "周五"],
                    "value" => [120.0, 200.0, 150.0, 80.0, 70.0],
                ))
                .x("day")
                .y("value")
                .name("产品A")
                .stack("总量")
                .area(true),
        )
        .add_line(
            Line::new()
                .data(dataframe!(
                    "day" => ["周一", "周二", "周三", "周四", "周五"],
                    "value" => [100.0, 80.0, 120.0, 200.0, 150.0],
                ))
                .x("day")
                .y("value")
                .name("产品B")
                .stack("总量")
                .area(true),
        )
        .add_line(
            Line::new()
                .data(dataframe!(
                    "day" => ["周一", "周二", "周三", "周四", "周五"],
                    "value" => [80.0, 120.0, 180.0, 60.0, 100.0],
                ))
                .x("day")
                .y("value")
                .name("产品C")
                .stack("总量")
                .area(true),
        )
        .render_to_svg("stacked_area.svg")?;
    println!("堆叠面积图已保存到 stacked_area.svg");

    Ok(())
}