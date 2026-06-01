use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("极坐标散点图").subtext("风向风速分布"))
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
        )
        .render_to_svg("polar_scatter.svg")?;
    println!("极坐标散点图已保存到 polar_scatter.svg");

    Ok(())
}
