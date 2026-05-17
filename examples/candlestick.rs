use liecharts::{AxisType, LieChart, LieChartOption, SeriesOption, CandlestickDataPoint};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = LieChart::new(800, 600);

    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("K线图示例".to_string()),
            subtext: Some("股票价格走势".to_string()),
            ..Default::default()
        }),
        legend: Some(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["日K".to_string()]),
            ..Default::default()
        }),
        x_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Category),
            data: Some(vec![
                "2024-01-02".to_string(),
                "2024-01-03".to_string(),
                "2024-01-04".to_string(),
                "2024-01-05".to_string(),
                "2024-01-08".to_string(),
                "2024-01-09".to_string(),
                "2024-01-10".to_string(),
                "2024-01-11".to_string(),
                "2024-01-12".to_string(),
            ]),
            ..Default::default()
        }],
        y_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Value),
            ..Default::default()
        }],
        series: vec![SeriesOption::Candlestick(liecharts::CandlestickSeriesOption {
            name: Some("日K".to_string()),
            data: vec![
                CandlestickDataPoint { open: 100.0, close: 105.0, low: 98.0, high: 108.0, name: None },
                CandlestickDataPoint { open: 105.0, close: 102.0, low: 100.0, high: 110.0, name: None },
                CandlestickDataPoint { open: 102.0, close: 108.0, low: 101.0, high: 112.0, name: None },
                CandlestickDataPoint { open: 108.0, close: 115.0, low: 106.0, high: 118.0, name: None },
                CandlestickDataPoint { open: 115.0, close: 112.0, low: 110.0, high: 120.0, name: None },
                CandlestickDataPoint { open: 112.0, close: 118.0, low: 111.0, high: 125.0, name: None },
                CandlestickDataPoint { open: 118.0, close: 125.0, low: 116.0, high: 128.0, name: None },
                CandlestickDataPoint { open: 125.0, close: 120.0, low: 118.0, high: 130.0, name: None },
                CandlestickDataPoint { open: 120.0, close: 122.0, low: 115.0, high: 125.0, name: None },
            ],
            ..Default::default()
        })],
        ..Default::default()
    };

    chart.render_to_image(option, "candlestick.png")?;
    println!("K线图已保存到 candlestick.png");

    Ok(())
}