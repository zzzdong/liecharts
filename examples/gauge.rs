use liecharts::api::*;

#[path = "common/mod.rs"]
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
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
                .center(Size::pct(50.0), Size::pct(55.0))
                .radius(Size::pct(75.0)),
        );
    common::save(&chart, "gauge.svg")?;

    Ok(())
}
