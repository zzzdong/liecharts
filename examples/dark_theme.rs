use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dark_theme = Theme::dark();

    ChartBuilder::new()
        .with_theme(dark_theme)
        .with_title(TitleOption::new("深色主题示例").subtext("Dark Theme Demo"))
        .with_legend(
            LegendOption::default()
                .data(["产品A", "产品B", "产品C"])
                .show(true),
        )
        .with_x_axis(AxisOption::category().data(["周一", "周二", "周三", "周四", "周五"]))
        .with_y_axis(AxisOption::value().name("销量"))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption::new(
            "产品A",
            vec![320.0, 332.0, 301.0, 334.0, 390.0],
        )))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption::new(
            "产品B",
            vec![220.0, 182.0, 191.0, 234.0, 290.0],
        )))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption::new(
            "产品C",
            vec![150.0, 232.0, 201.0, 154.0, 190.0],
        )))
        .build(800, 600)?
        .render_to_svg("dark_theme.svg")?;
    println!("深色主题图表已保存到 dark_theme.svg");

    Ok(())
}
