use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    vertical();
    horizontal()?;
    Ok(())
}

fn vertical() {
    ChartBuilder::new()
        .with_title(TitleOption::new("单系列柱状图（纵向）").subtext("2024年"))
        .with_legend(LegendOption::default().data(["销售额"]).show(true))
        .with_x_axis(AxisOption::category().data(["1月", "2月", "3月", "4月", "5月", "6月"]))
        .with_y_axis(AxisOption::value().name("销售额（万元）"))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "销售额",
            vec![120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
        )))
        .build(800, 600)
        .unwrap()
        .render_to_svg("bar_single_v.svg")
        .unwrap();
    println!("单系列纵向柱状图 → bar_single_v.svg");
}

fn horizontal() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("单系列柱状图（横向）"))
        .with_legend(LegendOption::default().data(["销售额"]).show(true))
        .with_x_axis(AxisOption::value().name("销售额（万元）"))
        .with_y_axis(AxisOption::category().data(["产品A", "产品B", "产品C", "产品D"]))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "销售额",
            vec![120.0, 200.0, 150.0, 80.0],
        )))
        .build(800, 600)?
        .render_to_svg("bar_single_h.svg")?;
    println!("单系列横向柱状图 → bar_single_h.svg");
    Ok(())
}
