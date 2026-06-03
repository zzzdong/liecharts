use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(1000, 800)
        .title(Title::new("多子图展示").subtext("Multi Grid Example"))
        .grid(
            Grid::new()
                .left(Position::pct(3.0))
                .top(Position::pct(15.0))
                .right(Position::pct(52.0))
                .bottom(Position::pct(52.0)),
        )
        .grid(
            Grid::new()
                .left(Position::pct(52.0))
                .top(Position::pct(15.0))
                .right(Position::pct(3.0))
                .bottom(Position::pct(52.0)),
        )
        .grid(
            Grid::new()
                .left(Position::pct(3.0))
                .top(Position::pct(52.0))
                .right(Position::pct(3.0))
                .bottom(Position::pct(3.0)),
        )
        .x_axis(
            Axis::category()
                .data(["1月", "2月", "3月", "4月", "5月", "6月"])
                .grid_index(0),
        )
        .x_axis(
            Axis::category()
                .data(["1月", "2月", "3月", "4月", "5月", "6月"])
                .grid_index(1),
        )
        .x_axis(
            Axis::category()
                .data(["1月", "2月", "3月", "4月", "5月", "6月"])
                .grid_index(2),
        )
        .y_axis(Axis::value().grid_index(0))
        .y_axis(Axis::value().grid_index(1))
        .y_axis(Axis::value().grid_index(2))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "cat" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "val" => [120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
                ))
                .x("cat")
                .y("val")
                .name("子图1-柱状图")
                .grid_index(0),
        )
        .add_line(
            Line::new()
                .data(dataframe!(
                    "cat" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "val" => [30.0, 50.0, 80.0, 120.0, 90.0, 60.0],
                ))
                .x("cat")
                .y("val")
                .name("子图2-折线图")
                .grid_index(1),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "cat" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "val" => [200.0, 300.0, 250.0, 180.0, 220.0, 280.0],
                ))
                .x("cat")
                .y("val")
                .name("子图3")
                .grid_index(2),
        )
        .render_to_svg("multi_grid.svg")?;
    println!("多子图已保存到 multi_grid.svg");

    Ok(())
}
