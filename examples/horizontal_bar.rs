use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("横向柱状图示例"))
        .with_x_axis(AxisOption::value().name("销售额（万元）"))
        .with_y_axis(AxisOption::category().data(["产品A", "产品B", "产品C", "产品D"]))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "销售额",
            vec![120.0, 200.0, 150.0, 80.0],
        )))
        .build(800, 600)?
        .render_to_svg("horizontal_bar.svg")?;

    println!("横向柱状图已保存到 horizontal_bar.svg");
    Ok(())
}
