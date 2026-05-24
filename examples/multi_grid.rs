use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = ChartBuilder::new()
        .with_title(TitleOption::new("多子图展示").subtext("Multi Grid Example"))
        .with_grid(
            GridOption::default()
                .left(PositionOption::percent(3.0))
                .top(PositionOption::percent(15.0))
                .right(PositionOption::percent(52.0))
                .bottom(PositionOption::percent(52.0)),
        )
        .with_grid(
            GridOption::default()
                .left(PositionOption::percent(52.0))
                .top(PositionOption::percent(15.0))
                .right(PositionOption::percent(3.0))
                .bottom(PositionOption::percent(52.0)),
        )
        .with_grid(
            GridOption::default()
                .left(PositionOption::percent(3.0))
                .top(PositionOption::percent(52.0))
                .right(PositionOption::percent(3.0))
                .bottom(PositionOption::percent(3.0)),
        )
        .with_x_axis(
            AxisOption::category()
                .grid_index(0)
                .data(["1月", "2月", "3月", "4月", "5月", "6月"]),
        )
        .with_x_axis(
            AxisOption::category()
                .grid_index(1)
                .data(["1月", "2月", "3月", "4月", "5月", "6月"]),
        )
        .with_x_axis(
            AxisOption::category()
                .grid_index(2)
                .data(["1月", "2月", "3月", "4月", "5月", "6月"]),
        )
        .with_y_axis(AxisOption::value().grid_index(0))
        .with_y_axis(AxisOption::value().grid_index(1))
        .with_y_axis(AxisOption::value().grid_index(2))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            grid_index: Some(0),
            ..liecharts::BarSeriesOption::new(
                "子图1-柱状图",
                vec![120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
            )
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            grid_index: Some(1),
            ..liecharts::LineSeriesOption::new(
                "子图2-折线图",
                vec![30.0, 50.0, 80.0, 120.0, 90.0, 60.0],
            )
        }))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            grid_index: Some(2),
            ..liecharts::BarSeriesOption::new(
                "子图3",
                vec![200.0, 300.0, 250.0, 180.0, 220.0, 280.0],
            )
        }))
        .build(1000, 800)?;
    chart.render_to_svg("multi_grid.svg")?;
    println!("多子图已保存到 multi_grid.svg");

    Ok(())
}
