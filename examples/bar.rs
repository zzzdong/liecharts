use liecharts::{AxisType, DataPoint, LieChart, LieChartOption, SeriesOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = LieChart::new(800, 600);

    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("月度销售数据".to_string()),
            subtext: Some("2024年".to_string()),
            ..Default::default()
        }),
        legend: Some(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["销售额".to_string()]),
            ..Default::default()
        }),
        x_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Category),
            data: Some(vec![
                "1月".to_string(),
                "2月".to_string(),
                "3月".to_string(),
                "4月".to_string(),
                "5月".to_string(),
                "6月".to_string(),
            ]),
            ..Default::default()
        }],
        y_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Value),
            name: Some("销售额(万元)".to_string()),
            ..Default::default()
        }],
        series: vec![SeriesOption::Bar(liecharts::BarSeriesOption {
            name: Some("销售额".to_string()),
            data: vec![
                DataPoint::Number(120.0),
                DataPoint::Number(200.0),
                DataPoint::Number(150.0),
                DataPoint::Number(80.0),
                DataPoint::Number(70.0),
                DataPoint::Number(110.0),
            ],
            ..Default::default()
        })],
        ..Default::default()
    };

    chart.render_to_image(option, "bar.png")?;
    println!("柱状图已保存到 bar.png");

    Ok(())
}
