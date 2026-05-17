use liecharts::{
    AxisType, DataPoint, LieChart, LieChartOption, SeriesOption,
    Theme,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dark_theme = Theme::dark();

    let chart = LieChart::new(800, 600).with_theme(dark_theme);

    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("深色主题示例".to_string()),
            subtext: Some("Dark Theme Demo".to_string()),
            ..Default::default()
        }),
        legend: Some(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["产品A".to_string(), "产品B".to_string(), "产品C".to_string()]),
            ..Default::default()
        }),
        x_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Category),
            data: Some(vec![
                "周一".to_string(),
                "周二".to_string(),
                "周三".to_string(),
                "周四".to_string(),
                "周五".to_string(),
            ]),
            ..Default::default()
        }],
        y_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Value),
            name: Some("销量".to_string()),
            ..Default::default()
        }],
        series: vec![
            SeriesOption::Bar(liecharts::BarSeriesOption {
                name: Some("产品A".to_string()),
                data: vec![
                    DataPoint::Number(320.0),
                    DataPoint::Number(332.0),
                    DataPoint::Number(301.0),
                    DataPoint::Number(334.0),
                    DataPoint::Number(390.0),
                ],
                ..Default::default()
            }),
            SeriesOption::Bar(liecharts::BarSeriesOption {
                name: Some("产品B".to_string()),
                data: vec![
                    DataPoint::Number(220.0),
                    DataPoint::Number(182.0),
                    DataPoint::Number(191.0),
                    DataPoint::Number(234.0),
                    DataPoint::Number(290.0),
                ],
                ..Default::default()
            }),
            SeriesOption::Bar(liecharts::BarSeriesOption {
                name: Some("产品C".to_string()),
                data: vec![
                    DataPoint::Number(150.0),
                    DataPoint::Number(232.0),
                    DataPoint::Number(201.0),
                    DataPoint::Number(154.0),
                    DataPoint::Number(190.0),
                ],
                ..Default::default()
            }),
        ],
        theme: Some("dark".to_string()),
        ..Default::default()
    };

    chart.render_to_image(option, "dark_theme.png")?;
    println!("深色主题图表已保存到 dark_theme.png");

    Ok(())
}