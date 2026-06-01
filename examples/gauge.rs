use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("任务完成率").subtext("Gauge Chart"))
        .add_gauge(
            Gauge::new()
                .data(dataframe!(
                    "name" => ["完成率"],
                    "value" => [75.5],
                ))
                .name("完成率")
                .value("value")
                .range(0.0, 100.0)
                .center(50.0, 55.0)
                .radius(75.0),
        )
        .render_to_svg("gauge.svg")?;
    println!("仪表盘已保存到 gauge.svg");

    Ok(())
}
