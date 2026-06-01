use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(1000, 900)
        .title(Title::new("混合布局图表").subtext("Mixed Grid Layout"))
        .grid(
            Grid::new()
                .left(Position::pct(3.0))
                .top(Position::pct(12.0))
                .right(Position::pct(52.0))
                .bottom(Position::pct(52.0)),
        )
        .grid(
            Grid::new()
                .left(Position::pct(52.0))
                .top(Position::pct(12.0))
                .right(Position::pct(3.0))
                .bottom(Position::pct(52.0)),
        )
        .grid(
            Grid::new()
                .left(Position::pct(3.0))
                .top(Position::pct(52.0))
                .right(Position::pct(52.0))
                .bottom(Position::pct(3.0)),
        )
        .grid(
            Grid::new()
                .left(Position::pct(52.0))
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
                .data(["周一", "周二", "周三", "周四", "周五", "周六"])
                .grid_index(1),
        )
        .x_axis(
            Axis::category()
                .data(["产品A", "产品B", "产品C", "产品D", "产品E"])
                .grid_index(2),
        )
        .x_axis(
            Axis::category()
                .data(["Q1", "Q2", "Q3", "Q4"])
                .grid_index(3),
        )
        .y_axis(Axis::value().grid_index(0))
        .y_axis(Axis::value().grid_index(1))
        .y_axis(Axis::value().grid_index(2))
        .y_axis(Axis::value().grid_index(3))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "cat" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "val" => [120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
                ))
                .x("cat")
                .y("val")
                .name("柱状图-子图1")
                .grid_index(0),
        )
        .add_line(
            Line::new()
                .data(dataframe!(
                    "cat" => ["周一", "周二", "周三", "周四", "周五", "周六"],
                    "val" => [50.0, 80.0, 120.0, 90.0, 60.0, 100.0],
                ))
                .x("cat")
                .y("val")
                .name("折线图-子图2")
                .grid_index(1),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "cat" => ["产品A", "产品B", "产品C", "产品D", "产品E"],
                    "val" => [300.0, 250.0, 180.0, 280.0, 220.0],
                ))
                .x("cat")
                .y("val")
                .name("柱状图-子图3")
                .grid_index(2),
        )
        .add_line(
            Line::new()
                .data(dataframe!(
                    "cat" => ["Q1", "Q2", "Q3", "Q4"],
                    "val" => [100.0, 150.0, 120.0, 180.0],
                ))
                .x("cat")
                .y("val")
                .name("折线图-子图4")
                .grid_index(3),
        )
        .render_to_svg("mixed_grid.svg")?;
    println!("混合布局图表已保存到 mixed_grid.svg");
    Ok(())
}