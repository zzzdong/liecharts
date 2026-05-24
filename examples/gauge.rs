use liecharts::{GaugeDataPoint, prelude::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(liecharts::TitleOption {
            text: Some("任务完成率".to_string()),
            subtext: Some("Gauge Chart".to_string()),
            ..Default::default()
        })
        .with_series(SeriesOption::Gauge(liecharts::GaugeSeriesOption {
            name: Some("完成率".to_string()),
            data: vec![GaugeDataPoint {
                value: 75.5,
                name: Some("完成率".to_string()),
            }],
            min: Some(0.0),
            max: Some(100.0),
            radius: Some("75%".to_string()),
            center: Some(vec!["50%".to_string(), "55%".to_string()]),
            ..Default::default()
        }))
        .build(800, 600)?
        .render_to_svg("gauge.svg")?;
    println!("仪表盘已保存到 gauge.svg");

    Ok(())
}
