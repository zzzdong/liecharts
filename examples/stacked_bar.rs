use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("堆叠柱状图示例"))
        .with_legend(
            LegendOption::default()
                .data(["直接销售", "代理销售", "线上销售"])
                .show(true),
        )
        .with_x_axis(AxisOption::category().data(["Q1", "Q2", "Q3", "Q4"]))
        .with_y_axis(AxisOption::value().name("销售额（万元）"))
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("直接销售", vec![120.0, 200.0, 150.0, 80.0]).stack("总量"),
        ))
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("代理销售", vec![80.0, 160.0, 120.0, 70.0]).stack("总量"),
        ))
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("线上销售", vec![60.0, 120.0, 90.0, 60.0]).stack("总量"),
        ))
        .build(800, 600)?
        .render_to_svg("stacked_bar.svg")?;

    println!("堆叠柱状图已保存到 stacked_bar.svg");
    Ok(())
}
