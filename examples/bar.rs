use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("月度销售数据").subtext("2024年"))
        .with_legend(LegendOption::default().data(["销售额"]).show(true))
        .with_x_axis(AxisOption::category().data(["1月", "2月", "3月", "4月", "5月", "6月"]))
        .with_y_axis(AxisOption::value().name("销售额(万元)"))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption::new(
            "销售额",
            vec![120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
        )))
        .build(800, 600)?
        .render_to_svg("bar.svg")?;
    println!("柱状图已保存到 bar.svg");

    Ok(())
}
