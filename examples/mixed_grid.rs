use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("混合布局图表").subtext("Mixed Grid Layout"))
        .with_grid(
            GridOption::default()
                .left(PositionOption::percent(3.0))
                .top(PositionOption::percent(12.0))
                .right(PositionOption::percent(52.0))
                .bottom(PositionOption::percent(52.0)),
        )
        .with_grid(
            GridOption::default()
                .left(PositionOption::percent(52.0))
                .top(PositionOption::percent(12.0))
                .right(PositionOption::percent(3.0))
                .bottom(PositionOption::percent(52.0)),
        )
        .with_grid(
            GridOption::default()
                .left(PositionOption::percent(3.0))
                .top(PositionOption::percent(52.0))
                .right(PositionOption::percent(52.0))
                .bottom(PositionOption::percent(3.0)),
        )
        .with_grid(
            GridOption::default()
                .left(PositionOption::percent(52.0))
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
                .data(["周一", "周二", "周三", "周四", "周五", "周六"]),
        )
        .with_x_axis(
            AxisOption::category()
                .grid_index(2)
                .data(["产品A", "产品B", "产品C", "产品D", "产品E"]),
        )
        .with_x_axis(
            AxisOption::category()
                .grid_index(3)
                .data(["Q1", "Q2", "Q3", "Q4"]),
        )
        .with_y_axis(AxisOption::category().grid_index(0))
        .with_y_axis(AxisOption::category().grid_index(1))
        .with_y_axis(AxisOption::category().grid_index(2))
        .with_y_axis(AxisOption::category().grid_index(3))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            grid_index: Some(0),
            ..liecharts::BarSeriesOption::new(
                "柱状图-子图1",
                vec![120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
            )
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            grid_index: Some(1),
            ..liecharts::LineSeriesOption::new(
                "折线图-子图2",
                vec![50.0, 80.0, 120.0, 90.0, 60.0, 100.0],
            )
        }))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            grid_index: Some(2),
            ..liecharts::BarSeriesOption::new(
                "柱状图-子图3",
                vec![300.0, 250.0, 180.0, 280.0, 220.0],
            )
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            grid_index: Some(3),
            ..liecharts::LineSeriesOption::new("折线图-子图4", vec![100.0, 150.0, 120.0, 180.0])
        }))
        .build(1000, 900)?
        .render_to_image("mixed_grid.png")?;
    println!("混合布局图表已保存到 mixed_grid.png");
    Ok(())
}
