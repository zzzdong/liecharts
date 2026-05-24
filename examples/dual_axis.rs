use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("温度与降水量").subtext("双 Y 轴示例"))
        .with_legend(LegendOption::default().data(["温度", "降水量"]).show(true))
        .with_x_axis(AxisOption::category().data(["1月", "2月", "3月", "4月", "5月", "6月"]))
        .with_y_axis(AxisOption::value().name("温度 (°C)"))
        .with_y_axis(AxisOption::value().name("降水量 (mm)"))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            y_axis_index: Some(0),
            ..liecharts::LineSeriesOption::new("温度", vec![5.0, 8.0, 12.0, 18.0, 24.0, 30.0])
        }))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            y_axis_index: Some(1),
            ..liecharts::BarSeriesOption::new("降水量", vec![50.0, 60.0, 80.0, 120.0, 150.0, 200.0])
        }))
        .build(800, 600)?
        .render_to_svg("dual_axis.svg")?;
    println!("双轴图已保存到 dual_axis.svg");

    Ok(())
}
