use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("访问量趋势面积图").subtext("Area Chart"))
        .with_legend(LegendOption::default().data(["访问量"]).show(true))
        .with_x_axis(AxisOption::category().data(["1月", "2月", "3月", "4月", "5月", "6月"]))
        .with_y_axis(AxisOption::value().name("访问量"))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            area_style: Some(liecharts::AreaStyleOption {
                color: None,
                opacity: None,
            }),
            ..liecharts::LineSeriesOption::new(
                "访问量",
                vec![120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
            )
        }))
        .build(800, 600)?
        .render_to_image("area.png")?;
    println!("面积图已保存到 area.png");

    Ok(())
}
