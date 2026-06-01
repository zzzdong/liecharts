use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    vertical_grouped()?;
    vertical_stacked()?;
    horizontal_grouped()?;
    horizontal_stacked()?;
    Ok(())
}

fn vertical_grouped() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("分组柱状图（纵向并列）"))
        .with_legend(
            LegendOption::default()
                .data(["产品A", "产品B", "产品C"])
                .show(true),
        )
        .with_x_axis(AxisOption::category().data(["Q1", "Q2", "Q3", "Q4"]))
        .with_y_axis(AxisOption::value().name("销售额"))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "产品A",
            vec![120.0, 200.0, 150.0, 80.0],
        )))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "产品B",
            vec![80.0, 160.0, 120.0, 70.0],
        )))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "产品C",
            vec![60.0, 120.0, 90.0, 60.0],
        )))
        .build(800, 600)?
        .render_to_svg("bar_group_v_side.svg")?;
    println!("纵向并列分组 → bar_group_v_side.svg");
    Ok(())
}

fn vertical_stacked() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("分组柱状图（纵向堆叠）"))
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
        .render_to_svg("bar_group_v_stack.svg")?;
    println!("纵向堆叠分组 → bar_group_v_stack.svg");
    Ok(())
}

fn horizontal_grouped() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("分组柱状图（横向并列）"))
        .with_legend(
            LegendOption::default()
                .data(["直接渠道", "代理渠道"])
                .show(true),
        )
        .with_x_axis(AxisOption::value().name("销售额（万元）"))
        .with_y_axis(AxisOption::category().data(["华北", "华东", "华南", "华西"]))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "直接渠道",
            vec![40.0, 80.0, 60.0, 30.0],
        )))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "代理渠道",
            vec![30.0, 60.0, 50.0, 20.0],
        )))
        .build(800, 600)?
        .render_to_svg("bar_group_h_side.svg")?;
    println!("横向并列分组 → bar_group_h_side.svg");
    Ok(())
}

fn horizontal_stacked() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("分组柱状图（横向堆叠）"))
        .with_legend(LegendOption::default().data(["线上", "线下"]).show(true))
        .with_x_axis(AxisOption::value().name("销售额（万元）"))
        .with_y_axis(AxisOption::category().data(["华北", "华东", "华南", "华西"]))
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("线上", vec![20.0, 40.0, 30.0, 15.0]).stack("ch"),
        ))
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("线下", vec![30.0, 50.0, 40.0, 25.0]).stack("ch"),
        ))
        .build(800, 600)?
        .render_to_svg("bar_group_h_stack.svg")?;
    println!("横向堆叠分组 → bar_group_h_stack.svg");
    Ok(())
}
