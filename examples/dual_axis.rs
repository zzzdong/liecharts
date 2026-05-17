use liecharts::{
    AxisType, DataPoint, LieChart, LieChartOption, SeriesOption,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = LieChart::new(800, 600);

    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("温度与降水量".to_string()),
            subtext: Some("双 Y 轴示例".to_string()),
            ..Default::default()
        }),
        legend: Some(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["温度".to_string(), "降水量".to_string()]),
            ..Default::default()
        }),
        x_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Category),
            data: Some(vec![
                "1月".to_string(), "2月".to_string(), "3月".to_string(),
                "4月".to_string(), "5月".to_string(), "6月".to_string(),
            ]),
            ..Default::default()
        }],
        y_axis: vec![
            liecharts::AxisOption {
                axis_type: Some(AxisType::Value),
                name: Some("温度 (°C)".to_string()),
                ..Default::default()
            },
            liecharts::AxisOption {
                axis_type: Some(AxisType::Value),
                name: Some("降水量 (mm)".to_string()),
                ..Default::default()
            },
        ],
        series: vec![
            SeriesOption::Line(liecharts::LineSeriesOption {
                name: Some("温度".to_string()),
                data: vec![
                    DataPoint::Number(5.0),
                    DataPoint::Number(8.0),
                    DataPoint::Number(12.0),
                    DataPoint::Number(18.0),
                    DataPoint::Number(24.0),
                    DataPoint::Number(30.0),
                ],
                y_axis_index: Some(0),
                ..Default::default()
            }),
            SeriesOption::Bar(liecharts::BarSeriesOption {
                name: Some("降水量".to_string()),
                data: vec![
                    DataPoint::Number(50.0),
                    DataPoint::Number(60.0),
                    DataPoint::Number(80.0),
                    DataPoint::Number(120.0),
                    DataPoint::Number(150.0),
                    DataPoint::Number(200.0),
                ],
                y_axis_index: Some(1),
                ..Default::default()
            }),
        ],
        ..Default::default()
    };

    chart.render_to_image(option, "dual_axis.png")?;
    println!("双轴图已保存到 dual_axis.png");

    Ok(())
}