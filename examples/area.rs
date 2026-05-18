use liecharts::{AxisType, ChartBuilder, DataPoint, LieChart, SeriesOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = ChartBuilder::new()
        .with_title(liecharts::TitleOption {
            text: Some("访问量趋势面积图".to_string()),
            subtext: Some("Area Chart".to_string()),
            ..Default::default()
        })
        .with_legend(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["访问量".to_string()]),
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
            name: Some("访问量".to_string()),
            ..Default::default()
        })
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            name: Some("访问量".to_string()),
            data: vec![
                DataPoint::Number(120.0),
                DataPoint::Number(200.0),
                DataPoint::Number(150.0),
                DataPoint::Number(80.0),
                DataPoint::Number(70.0),
                DataPoint::Number(110.0),
            ],
            area_style: Some(liecharts::AreaStyleOption {
                color: None,
                opacity: None,
            }),
            ..Default::default()
        }))
        .build()?;

    let chart = LieChart::new(800, 600);
    chart.render_to_image(&model, "area.png")?;
    println!("面积图已保存到 area.png");

    Ok(())
}
