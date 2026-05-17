use liecharts::{LieChart, LieChartOption, SeriesOption};
use liecharts::option::GaugeDataPoint;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("任务完成率".to_string()),
            subtext: Some("Gauge Chart".to_string()),
            ..Default::default()
        }),
        series: vec![
            SeriesOption::Gauge(liecharts::GaugeSeriesOption {
                name: Some("完成率".to_string()),
                data: vec![
                    GaugeDataPoint { value: 75.5, name: Some("完成率".to_string()) },
                ],
                min: Some(0.0),
                max: Some(100.0),
                radius: Some("75%".to_string()),
                center: Some(vec!["50%".to_string(), "55%".to_string()]),
                ..Default::default()
            }),
        ],
        ..Default::default()
    };

    let chart = LieChart::new(800, 600);
    chart.render_to_image(option, "gauge.png")?;
    println!("仪表盘已保存到 gauge.png");

    Ok(())
}