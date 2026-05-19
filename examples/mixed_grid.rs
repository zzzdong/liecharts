use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(liecharts::TitleOption {
            text: Some("混合布局图表".to_string()),
            subtext: Some("Mixed Grid Layout".to_string()),
            ..Default::default()
        })
        .with_grid(GridOption {
            left: Some(liecharts::PositionOption::percent(3.0)),
            top: Some(liecharts::PositionOption::percent(12.0)),
            right: Some(liecharts::PositionOption::percent(52.0)),
            bottom: Some(liecharts::PositionOption::percent(52.0)),
            ..Default::default()
        })
        .with_grid(GridOption {
            left: Some(liecharts::PositionOption::percent(52.0)),
            top: Some(liecharts::PositionOption::percent(12.0)),
            right: Some(liecharts::PositionOption::percent(3.0)),
            bottom: Some(liecharts::PositionOption::percent(52.0)),
            ..Default::default()
        })
        .with_grid(GridOption {
            left: Some(liecharts::PositionOption::percent(3.0)),
            top: Some(liecharts::PositionOption::percent(52.0)),
            right: Some(liecharts::PositionOption::percent(52.0)),
            bottom: Some(liecharts::PositionOption::percent(3.0)),
            ..Default::default()
        })
        .with_grid(GridOption {
            left: Some(liecharts::PositionOption::percent(52.0)),
            top: Some(liecharts::PositionOption::percent(52.0)),
            right: Some(liecharts::PositionOption::percent(3.0)),
            bottom: Some(liecharts::PositionOption::percent(3.0)),
            ..Default::default()
        })
        .with_x_axis(liecharts::AxisOption {
            grid_index: Some(0),
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
        .with_x_axis(liecharts::AxisOption {
            grid_index: Some(1),
            axis_type: Some(AxisType::Category),
            data: Some(vec![
                "周一".to_string(),
                "周二".to_string(),
                "周三".to_string(),
                "周四".to_string(),
                "周五".to_string(),
                "周六".to_string(),
            ]),
            ..Default::default()
        })
        .with_x_axis(liecharts::AxisOption {
            grid_index: Some(2),
            axis_type: Some(AxisType::Category),
            data: Some(vec![
                "产品A".to_string(),
                "产品B".to_string(),
                "产品C".to_string(),
                "产品D".to_string(),
                "产品E".to_string(),
            ]),
            ..Default::default()
        })
        .with_x_axis(liecharts::AxisOption {
            grid_index: Some(3),
            axis_type: Some(AxisType::Category),
            data: Some(vec![
                "Q1".to_string(),
                "Q2".to_string(),
                "Q3".to_string(),
                "Q4".to_string(),
            ]),
            ..Default::default()
        })
        .with_y_axis(liecharts::AxisOption {
            grid_index: Some(0),
            ..Default::default()
        })
        .with_y_axis(liecharts::AxisOption {
            grid_index: Some(1),
            ..Default::default()
        })
        .with_y_axis(liecharts::AxisOption {
            grid_index: Some(2),
            ..Default::default()
        })
        .with_y_axis(liecharts::AxisOption {
            grid_index: Some(3),
            ..Default::default()
        })
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            name: Some("柱状图-子图1".to_string()),
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
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            name: Some("折线图-子图2".to_string()),
            data: vec![
                DataPoint::Number(50.0),
                DataPoint::Number(80.0),
                DataPoint::Number(120.0),
                DataPoint::Number(90.0),
                DataPoint::Number(60.0),
                DataPoint::Number(100.0),
            ],
            grid_index: Some(1),
            ..Default::default()
        }))
        .with_series(SeriesOption::Bar(liecharts::BarSeriesOption {
            name: Some("柱状图-子图3".to_string()),
            data: vec![
                DataPoint::Number(300.0),
                DataPoint::Number(250.0),
                DataPoint::Number(180.0),
                DataPoint::Number(280.0),
                DataPoint::Number(220.0),
            ],
            grid_index: Some(2),
            ..Default::default()
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            name: Some("折线图-子图4".to_string()),
            data: vec![
                DataPoint::Number(100.0),
                DataPoint::Number(150.0),
                DataPoint::Number(120.0),
                DataPoint::Number(180.0),
            ],
            grid_index: Some(3),
            ..Default::default()
        }))
        .build(1000, 900)?
        .render_to_image("mixed_grid.png")?;
    println!("混合布局图表已保存到 mixed_grid.png");

    Ok(())
}
