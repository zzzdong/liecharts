use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("横向分组+堆叠柱状图"))
        .with_legend(
            LegendOption::default()
                .data(["直接-线上", "直接-代理", "国外-线上", "国外-代理"])
                .show(true),
        )
        .with_x_axis(AxisOption::value().name("销售额（万元）"))
        .with_y_axis(
            AxisOption::category()
                .data(["华北", "华东", "华南", "华西"]),
        )
        // 组1: 直接渠道，两个堆叠系列
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("直接-线上", vec![40.0, 60.0, 50.0, 30.0])
                .stack("直接")
        ))
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("直接-代理", vec![20.0, 40.0, 30.0, 20.0])
                .stack("直接")
        ))
        // 组2: 国外渠道，两个堆叠系列
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("国外-线上", vec![30.0, 50.0, 40.0, 25.0])
                .stack("国外")
        ))
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("国外-代理", vec![15.0, 30.0, 25.0, 15.0])
                .stack("国外")
        ))
        .build(800, 600)?
        .render_to_svg("horizontal_grouped_stacked.svg")?;

    println!("横向分组+堆叠柱状图已保存到 horizontal_grouped_stacked.svg");
    Ok(())
}
