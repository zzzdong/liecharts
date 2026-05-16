use liecharts::{
    DataPoint, LieChart, LieChartOption, LegendOption, PolarBarSeriesOption, SeriesOption,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chart = LieChart::new(800, 600);

    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("极坐标柱状图".to_string()),
            subtext: Some("月度销售数据".to_string()),
            ..Default::default()
        }),
        legend: Some(LegendOption {
            show: Some(true),
            data: Some(vec!["销售额".to_string()]),
            ..Default::default()
        }),
        series: vec![SeriesOption::PolarBar(PolarBarSeriesOption {
            name: Some("销售额".to_string()),
            data: vec![
                DataPoint::Number(120.0),
                DataPoint::Number(200.0),
                DataPoint::Number(150.0),
                DataPoint::Number(80.0),
                DataPoint::Number(70.0),
                DataPoint::Number(110.0),
                DataPoint::Number(180.0),
                DataPoint::Number(95.0),
                DataPoint::Number(140.0),
                DataPoint::Number(160.0),
                DataPoint::Number(130.0),
                DataPoint::Number(85.0),
            ],
            ..Default::default()
        })],
        ..Default::default()
    };

    chart.set_option(option, None)?;
    chart.render_to_image("polar_bar_chart.png")?;
    chart.render_to_svg("polar_bar_chart.svg")?;
    println!("极坐标柱状图已保存到 polar_bar_chart.png 和 polar_bar_chart.svg");

    Ok(())
}