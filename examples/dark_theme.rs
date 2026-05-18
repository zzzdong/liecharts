use liecharts::{AxisType, ChartBuilder, DataPoint, LieChart, SeriesOption, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dark_theme = Theme::dark();

    let model = ChartBuilder::new()
        .with_theme(dark_theme)
        .with_title(liecharts::TitleOption {
            text: Some("深色主题示例".to_string()),
            subtext: Some("Dark Theme Demo".to_string()),
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
            name: Some("销量".to_string()),
            ..Default::default()
        })
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            name: Some("产品A".to_string()),
            data: vec![
                DataPoint::Number(320.0),
                DataPoint::Number(332.0),
                DataPoint::Number(301.0),
                DataPoint::Number(334.0),
                DataPoint::Number(390.0),
            ],
            ..Default::default()
        }))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            name: Some("产品B".to_string()),
            data: vec![
                DataPoint::Number(220.0),
                DataPoint::Number(182.0),
                DataPoint::Number(191.0),
                DataPoint::Number(234.0),
                DataPoint::Number(290.0),
            ],
            ..Default::default()
        }))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            name: Some("产品C".to_string()),
            data: vec![
                DataPoint::Number(150.0),
                DataPoint::Number(232.0),
                DataPoint::Number(201.0),
                DataPoint::Number(154.0),
                DataPoint::Number(190.0),
            ],
            ..Default::default()
        }))
        .build()?;

    let chart = LieChart::new(800, 600);
    chart.render_to_image(&model, "dark_theme.png")?;
    println!("深色主题图表已保存到 dark_theme.png");

    Ok(())
}
