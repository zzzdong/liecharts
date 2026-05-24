use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("混合图表示例").subtext("柱状图和折线图组合"))
        .with_legend(LegendOption::default().data(["销量", "增长率"]).show(true))
        .with_x_axis(AxisOption::category().data(["周一", "周二", "周三", "周四", "周五"]))
        .with_y_axis(
            AxisOption::value()
                .name("销量")
                .position(AxisPosition::Left),
        )
        .with_y_axis(
            AxisOption::value()
                .name("增长率(%)")
                .position(AxisPosition::Right),
        )
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            y_axis_index: Some(0),
            ..liecharts::BarSeriesOption::new("销量", vec![120.0, 200.0, 150.0, 80.0, 70.0])
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            y_axis_index: Some(1),
            ..liecharts::LineSeriesOption::new("增长率", vec![10.0, 20.0, 15.0, 8.0, 7.0])
        }))
        .build(800, 600)?
        .render_to_svg("mixed.svg")?;
    println!("混合图表已保存到 mixed.svg");
    Ok(())
}
