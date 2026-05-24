use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("折线图示例").subtext("Line Chart"))
        .with_legend(LegendOption::default().data(["销售额"]).show(true))
        .with_x_axis(
            AxisOption::category().data(["周一", "周二", "周三", "周四", "周五", "周六", "周日"]),
        )
        .with_y_axis(AxisOption::value().name("销售额(元)"))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption::new(
            "销售额",
            vec![120.0, 200.0, 150.0, 80.0, 70.0, 110.0, 130.0],
        ).smooth(true)))
        .build(800, 600)?
        .render_to_svg("line.svg")?;
    println!("折线图已保存到 line.svg");

    Ok(())
}
