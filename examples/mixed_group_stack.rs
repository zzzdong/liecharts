use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("混合分组与堆叠示例"))
        .with_legend(
            LegendOption::default()
                .data(["产品A-1", "产品A-2", "产品B-1", "产品B-2"])
                .show(true),
        )
        .with_x_axis(AxisOption::category().data(["Q1", "Q2", "Q3", "Q4"]))
        .with_y_axis(AxisOption::value().name("销售额"))
        // 组1: 无 stack，两个系列并排显示
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "产品A-1",
            vec![100.0, 100.0, 100.0, 100.0],
        )))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "产品A-2",
            vec![80.0, 80.0, 80.0, 80.0],
        )))
        // 组2: 相同 stack，两个系列堆叠显示
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("产品B-1", vec![50.0, 50.0, 50.0, 50.0]).stack("B组"),
        ))
        .with_series(SeriesOption::Bar(
            BarSeriesOption::new("产品B-2", vec![30.0, 30.0, 30.0, 30.0]).stack("B组"),
        ))
        .build(800, 600)?
        .render_to_svg("mixed_group_stack.svg")?;

    println!("混合图表已保存到 mixed_group_stack.svg");
    Ok(())
}
