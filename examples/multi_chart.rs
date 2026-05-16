use liecharts::{
    AxisOption, AxisType, DataPoint, LieChart, LieChartOption,
    Position, SeriesOption, BarSeriesOption, LineSeriesOption,
    GridOption, LegendOption, TitleOption,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let option = LieChartOption {
        title: Some(TitleOption {
            text: Some("多图表示例".to_string()),
            subtext: Some("一个画布多个子图".to_string()),
            ..Default::default()
        }),
        legend: Some(LegendOption {
            show: Some(true),
            data: Some(vec!["销量".to_string(), "趋势".to_string()]),
            ..Default::default()
        }),
        grid: vec![
            GridOption {
                left: Some(Position::percent(5.0)),
                top: Some(Position::px(100.0)),
                right: Some(Position::percent(55.0)),
                bottom: Some(Position::percent(15.0)),
                ..Default::default()
            },
            GridOption {
                left: Some(Position::percent(55.0)),
                top: Some(Position::px(100.0)),
                right: Some(Position::percent(5.0)),
                bottom: Some(Position::percent(15.0)),
                ..Default::default()
            },
        ],
        x_axis: vec![
            AxisOption {
                axis_type: Some(AxisType::Category),
                data: Some(vec![
                    "周一".to_string(),
                    "周二".to_string(),
                    "周三".to_string(),
                    "周四".to_string(),
                    "周五".to_string(),
                ]),
                grid_index: Some(0),
                ..Default::default()
            },
            AxisOption {
                axis_type: Some(AxisType::Category),
                data: Some(vec![
                    "1月".to_string(),
                    "2月".to_string(),
                    "3月".to_string(),
                    "4月".to_string(),
                    "5月".to_string(),
                ]),
                grid_index: Some(1),
                ..Default::default()
            },
        ],
        y_axis: vec![
            AxisOption {
                axis_type: Some(AxisType::Value),
                name: Some("销量".to_string()),
                grid_index: Some(0),
                ..Default::default()
            },
            AxisOption {
                axis_type: Some(AxisType::Value),
                name: Some("数值".to_string()),
                grid_index: Some(1),
                ..Default::default()
            },
        ],
        series: vec![
            SeriesOption::Bar(BarSeriesOption {
                name: Some("销量".to_string()),
                data: vec![
                    DataPoint::Number(120.0),
                    DataPoint::Number(200.0),
                    DataPoint::Number(150.0),
                    DataPoint::Number(80.0),
                    DataPoint::Number(70.0),
                ],
                grid_index: Some(0),
                ..Default::default()
            }),
            SeriesOption::Line(LineSeriesOption {
                name: Some("趋势".to_string()),
                data: vec![
                    DataPoint::Number(820.0),
                    DataPoint::Number(932.0),
                    DataPoint::Number(901.0),
                    DataPoint::Number(934.0),
                    DataPoint::Number(1290.0),
                ],
                grid_index: Some(1),
                ..Default::default()
            }),
        ],
        ..Default::default()
    };

    let mut chart = LieChart::new(1000, 600);

    chart.set_option(option, None)?;
    chart.render_to_image("multi_chart.png")?;
    chart.render_to_svg("multi_chart.svg")?;
    println!("多图表已保存到 multi_chart.png 和 multi_chart.svg");
    Ok(())
}