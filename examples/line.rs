use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(liecharts::TitleOption {
            text: Some("月度趋势图".to_string()),
            subtext: Some("2024年销售额趋势".to_string()),
            ..Default::default()
        })
        .with_legend(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["销售额".to_string(), "目标".to_string()]),
            ..Default::default()
        })
        .with_x_axis(liecharts::AxisOption {
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
        })
        .with_y_axis(liecharts::AxisOption {
            axis_type: Some(AxisType::Value),
            name: Some("金额(万元)".to_string()),
            ..Default::default()
        })
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            name: Some("销售额".to_string()),
            data: vec![
                DataPoint::Number(120.0),
                DataPoint::Number(200.0),
                DataPoint::Number(150.0),
                DataPoint::Number(80.0),
                DataPoint::Number(70.0),
                DataPoint::Number(110.0),
            ],
            smooth: Some(true),
            ..Default::default()
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            name: Some("目标".to_string()),
            data: vec![
                DataPoint::Number(100.0),
                DataPoint::Number(100.0),
                DataPoint::Number(100.0),
                DataPoint::Number(100.0),
                DataPoint::Number(100.0),
                DataPoint::Number(100.0),
            ],
            ..Default::default()
        }))
        .build(800, 600)?
        .render_to_image("line.png")?;
    println!("折线图已保存到 line.png");

    Ok(())
}
