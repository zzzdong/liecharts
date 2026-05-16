use liecharts::{LieChart, LieChartOption, SeriesOption};
use liecharts::option::GaugeDataPoint;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("仪表盘示例".to_string()),
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

    let mut chart = LieChart::new(800, 600);
    chart.set_option(option, None)?;

    chart.render_to_image("gauge_chart.png")?;
    chart.render_to_svg("gauge_chart.svg")?;
    println!("仪表盘已保存到 gauge_chart.png 和 gauge_chart.svg");

    Ok(())
}