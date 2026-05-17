use liecharts::{
    AxisType, AxisPosition, DataPoint, LieChart, LieChartOption,
    SeriesOption,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("混合图表示例".to_string()),
            subtext: Some("柱状图和折线图组合".to_string()),
            ..Default::default()
        }),
        legend: Some(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["销量".to_string(), "增长率".to_string()]),
            ..Default::default()
        }),
        x_axis: vec![
            liecharts::AxisOption {
                axis_type: Some(AxisType::Category),
                data: Some(vec![
                    "周一".to_string(),
                    "周二".to_string(),
                    "周三".to_string(),
                    "周四".to_string(),
                    "周五".to_string(),
                ]),
                ..Default::default()
            },
        ],
        y_axis: vec![
            liecharts::AxisOption {
                axis_type: Some(AxisType::Value),
                name: Some("销量".to_string()),
                position: Some(AxisPosition::Left),
                ..Default::default()
            },
            liecharts::AxisOption {
                axis_type: Some(AxisType::Value),
                name: Some("增长率(%)".to_string()),
                position: Some(AxisPosition::Right),
                ..Default::default()
            },
        ],
        series: vec![
            SeriesOption::Bar(liecharts::BarSeriesOption {
                name: Some("销量".to_string()),
                data: vec![
                    DataPoint::Number(120.0),
                    DataPoint::Number(200.0),
                    DataPoint::Number(150.0),
                    DataPoint::Number(80.0),
                    DataPoint::Number(70.0),
                ],
                y_axis_index: Some(0),
                ..Default::default()
            }),
            SeriesOption::Line(liecharts::LineSeriesOption {
                name: Some("增长率".to_string()),
                data: vec![
                    DataPoint::Number(10.0),
                    DataPoint::Number(20.0),
                    DataPoint::Number(15.0),
                    DataPoint::Number(8.0),
                    DataPoint::Number(7.0),
                ],
                y_axis_index: Some(1),
                ..Default::default()
            }),
        ],
        ..Default::default()
    };

    let chart = LieChart::new(800, 600);
    chart.render_to_image(option, "mixed.png")?;
    println!("混合图表已保存到 mixed.png");
    Ok(())
}