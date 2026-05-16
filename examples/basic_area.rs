use liecharts::{
    AxisType, DataPoint, LieChart, LieChartOption, SeriesOption,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chart = LieChart::new(800, 600);

    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("基础面积图".to_string()),
            subtext: Some("Basic Area Chart".to_string()),
            ..Default::default()
        }),
        legend: Some(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["访问量".to_string()]),
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
            ]),
            ..Default::default()
        }],
        y_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Value),
            name: Some("访问量".to_string()),
            ..Default::default()
        }],
        series: vec![SeriesOption::Line(liecharts::LineSeriesOption {
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
        })],
        ..Default::default()
    };

    chart.set_option(option, None)?;
    chart.render_to_image("basic_area.png")?;
    chart.render_to_svg("basic_area.svg")?;
    println!("基础面积图已保存到 basic_area.png 和 basic_area.svg");

    Ok(())
}