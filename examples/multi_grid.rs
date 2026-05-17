use liecharts::{
    AxisType, DataPoint, GridOption, LieChart, LieChartOption, SeriesOption,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = LieChart::new(1000, 800);

    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("多子图展示".to_string()),
            subtext: Some("Multi Grid Example".to_string()),
            ..Default::default()
        }),
        grid: vec![
            GridOption {
                left: Some(liecharts::Position::percent(3.0)),
                top: Some(liecharts::Position::percent(15.0)),
                right: Some(liecharts::Position::percent(52.0)),
                bottom: Some(liecharts::Position::percent(52.0)),
                ..Default::default()
            },
            GridOption {
                left: Some(liecharts::Position::percent(52.0)),
                top: Some(liecharts::Position::percent(15.0)),
                right: Some(liecharts::Position::percent(3.0)),
                bottom: Some(liecharts::Position::percent(52.0)),
                ..Default::default()
            },
            GridOption {
                left: Some(liecharts::Position::percent(3.0)),
                top: Some(liecharts::Position::percent(52.0)),
                right: Some(liecharts::Position::percent(3.0)),
                bottom: Some(liecharts::Position::percent(3.0)),
                ..Default::default()
            },
        ],
        x_axis: vec![
            liecharts::AxisOption {
                grid_index: Some(0),
                axis_type: Some(AxisType::Category),
                data: Some(vec![
                    "1月".to_string(), "2月".to_string(), "3月".to_string(),
                    "4月".to_string(), "5月".to_string(), "6月".to_string(),
                ]),
                ..Default::default()
            },
            liecharts::AxisOption {
                grid_index: Some(1),
                axis_type: Some(AxisType::Category),
                data: Some(vec![
                    "1月".to_string(), "2月".to_string(), "3月".to_string(),
                    "4月".to_string(), "5月".to_string(), "6月".to_string(),
                ]),
                ..Default::default()
            },
            liecharts::AxisOption {
                grid_index: Some(2),
                axis_type: Some(AxisType::Category),
                data: Some(vec![
                    "1月".to_string(), "2月".to_string(), "3月".to_string(),
                    "4月".to_string(), "5月".to_string(), "6月".to_string(),
                ]),
                ..Default::default()
            },
        ],
        y_axis: vec![
            liecharts::AxisOption {
                grid_index: Some(0),
                ..Default::default()
            },
            liecharts::AxisOption {
                grid_index: Some(1),
                ..Default::default()
            },
            liecharts::AxisOption {
                grid_index: Some(2),
                ..Default::default()
            },
        ],
        series: vec![
            SeriesOption::Bar(liecharts::BarSeriesOption {
                name: Some("子图1-柱状图".to_string()),
                data: vec![
                    DataPoint::Number(120.0),
                    DataPoint::Number(200.0),
                    DataPoint::Number(150.0),
                    DataPoint::Number(80.0),
                    DataPoint::Number(70.0),
                    DataPoint::Number(110.0),
                ],
                grid_index: Some(0),
                ..Default::default()
            }),
            SeriesOption::Line(liecharts::LineSeriesOption {
                name: Some("子图2-折线图".to_string()),
                data: vec![
                    DataPoint::Number(30.0),
                    DataPoint::Number(50.0),
                    DataPoint::Number(80.0),
                    DataPoint::Number(120.0),
                    DataPoint::Number(90.0),
                    DataPoint::Number(60.0),
                ],
                grid_index: Some(1),
                ..Default::default()
            }),
            SeriesOption::Bar(liecharts::BarSeriesOption {
                name: Some("子图3".to_string()),
                data: vec![
                    DataPoint::Number(200.0),
                    DataPoint::Number(300.0),
                    DataPoint::Number(250.0),
                    DataPoint::Number(180.0),
                    DataPoint::Number(220.0),
                    DataPoint::Number(280.0),
                ],
                grid_index: Some(2),
                ..Default::default()
            }),
        ],
        ..Default::default()
    };

    chart.render_to_image(option, "multi_grid.png")?;
    println!("多子图已保存到 multi_grid.png");

    Ok(())
}