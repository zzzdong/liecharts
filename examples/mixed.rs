use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("混合图表示例").subtext("柱状图和折线图组合"))
        .legend(Legend::new().data(["销量", "增长率"]))
        .y_axis(Axis::value().name("销量").position(AxisPosition::Left))
        .y_axis(
            Axis::value()
                .name("增长率(%)")
                .position(AxisPosition::Right),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "day" => ["周一", "周二", "周三", "周四", "周五"],
                    "value" => [120.0, 200.0, 150.0, 80.0, 70.0],
                ))
                .x("day")
                .y("value")
                .name("销量")
                .y_axis_index(0),
        )
        .add_line(
            Line::new()
                .data(dataframe!(
                    "day" => ["周一", "周二", "周三", "周四", "周五"],
                    "value" => [10.0, 20.0, 15.0, 8.0, 7.0],
                ))
                .x("day")
                .y("value")
                .name("增长率")
                .y_axis_index(1),
        )
        .render_to_svg("mixed.svg")?;
    println!("混合图表已保存到 mixed.svg");
    Ok(())
}