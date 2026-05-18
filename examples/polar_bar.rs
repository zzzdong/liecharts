use liecharts::{ChartBuilder, DataPoint, LieChart, SeriesOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = ChartBuilder::new()
        .with_title(liecharts::TitleOption {
            text: Some("极坐标柱状图".to_string()),
            subtext: Some("Polar Bar Chart".to_string()),
            ..Default::default()
        })
        .with_legend(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec![
                "直接访问".to_string(),
                "邮件营销".to_string(),
                "联盟广告".to_string(),
                "视频广告".to_string(),
                "搜索引擎".to_string(),
            ]),
            ..Default::default()
        })
        .with_series(SeriesOption::PolarBar(liecharts::PolarBarSeriesOption {
            name: Some("访问来源".to_string()),
            data: vec![
                DataPoint::Number(335.0),
                DataPoint::Number(310.0),
                DataPoint::Number(234.0),
                DataPoint::Number(135.0),
                DataPoint::Number(1548.0),
            ],
            ..Default::default()
        }))
        .build()?;

    let chart = LieChart::new(800, 600);
    chart.render_to_image(&model, "polar_bar.png")?;
    println!("极坐标柱状图已保存到 polar_bar.png");

    Ok(())
}
