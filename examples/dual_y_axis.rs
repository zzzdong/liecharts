use liecharts::{AxisType, DataPoint, LieChart, LieChartOption, SeriesOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("双Y轴示例".to_string()),
            subtext: Some("温度与降水量".to_string()),
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
                "1月".to_string(),
                "2月".to_string(),
                "3月".to_string(),
                "4月".to_string(),
                "5月".to_string(),
                "6月".to_string(),
                "7月".to_string(),
                "8月".to_string(),
                "9月".to_string(),
                "10月".to_string(),
                "11月".to_string(),
                "12月".to_string(),
            ]),
            name: Some("月份".to_string()),
            ..Default::default()
        }],
        y_axis: vec![
            liecharts::AxisOption {
                axis_type: Some(AxisType::Value),
                name: Some("温度 (°C)".to_string()),
                min: Some(-10.0),
                max: Some(40.0),
                ..Default::default()
            },
            liecharts::AxisOption {
                axis_type: Some(AxisType::Value),
                name: Some("降水量 (mm)".to_string()),
                min: Some(0.0),
                max: Some(250.0),
                ..Default::default()
            },
        ],
        series: vec![
            SeriesOption::Line(liecharts::LineSeriesOption {
                name: Some("温度".to_string()),
                data: vec![
                    DataPoint::Number(-5.0),
                    DataPoint::Number(-2.0),
                    DataPoint::Number(5.0),
                    DataPoint::Number(12.0),
                    DataPoint::Number(18.0),
                    DataPoint::Number(25.0),
                    DataPoint::Number(32.0),
                    DataPoint::Number(30.0),
                    DataPoint::Number(24.0),
                    DataPoint::Number(16.0),
                    DataPoint::Number(8.0),
                    DataPoint::Number(-1.0),
                ],
                y_axis_index: Some(0),
                ..Default::default()
            }),
            SeriesOption::Bar(liecharts::BarSeriesOption {
                name: Some("降水量".to_string()),
                data: vec![
                    DataPoint::Number(15.0),
                    DataPoint::Number(20.0),
                    DataPoint::Number(35.0),
                    DataPoint::Number(55.0),
                    DataPoint::Number(85.0),
                    DataPoint::Number(120.0),
                    DataPoint::Number(180.0),
                    DataPoint::Number(160.0),
                    DataPoint::Number(95.0),
                    DataPoint::Number(45.0),
                    DataPoint::Number(25.0),
                    DataPoint::Number(10.0),
                ],
                y_axis_index: Some(1),
                ..Default::default()
            }),
        ],
        ..Default::default()
    };

    let mut chart = LieChart::new(800, 600);
    chart.set_option(option, None)?;

    chart.render_to_image("dual_y_axis.png")?;
    chart.render_to_svg("dual_y_axis.svg")?;
    println!("双Y轴图表已保存到 dual_y_axis.png 和 dual_y_axis.svg");

    Ok(())
}