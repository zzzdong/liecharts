use liecharts::{AxisType, ChartBuilder, DataPoint, LieChart, SeriesOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = ChartBuilder::new()
        .with_title(liecharts::TitleOption {
            text: Some("堆叠面积图".to_string()),
            subtext: Some("Stacked Area Chart".to_string()),
            ..Default::default()
        })
        .with_legend(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec![
                "产品A".to_string(),
                "产品B".to_string(),
                "产品C".to_string(),
            ]),
            ..Default::default()
        })
        .with_x_axis(liecharts::AxisOption {
            axis_type: Some(AxisType::Category),
            data: Some(vec![
                "周一".to_string(),
                "周二".to_string(),
                "周三".to_string(),
                "周四".to_string(),
                "周五".to_string(),
            ]),
            ..Default::default()
        })
        .with_y_axis(liecharts::AxisOption {
            axis_type: Some(AxisType::Value),
            name: Some("访问量".to_string()),
            ..Default::default()
        })
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            name: Some("产品A".to_string()),
            stack: Some("总量".to_string()),
            data: vec![
                DataPoint::Number(120.0),
                DataPoint::Number(200.0),
                DataPoint::Number(150.0),
                DataPoint::Number(80.0),
                DataPoint::Number(70.0),
            ],
            area_style: Some(liecharts::AreaStyleOption {
                color: None,
                opacity: None,
            }),
            ..Default::default()
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            name: Some("产品B".to_string()),
            stack: Some("总量".to_string()),
            data: vec![
                DataPoint::Number(100.0),
                DataPoint::Number(80.0),
                DataPoint::Number(120.0),
                DataPoint::Number(200.0),
                DataPoint::Number(150.0),
            ],
            area_style: Some(liecharts::AreaStyleOption {
                color: None,
                opacity: None,
            }),
            ..Default::default()
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            name: Some("产品C".to_string()),
            stack: Some("总量".to_string()),
            data: vec![
                DataPoint::Number(80.0),
                DataPoint::Number(120.0),
                DataPoint::Number(180.0),
                DataPoint::Number(60.0),
                DataPoint::Number(100.0),
            ],
            area_style: Some(liecharts::AreaStyleOption {
                color: None,
                opacity: None,
            }),
            ..Default::default()
        }))
        .build()?;

    let chart = LieChart::new(800, 600);
    chart.render_to_image(&model, "stacked_area.png")?;
    println!("堆叠面积图已保存到 stacked_area.png");

    Ok(())
}
