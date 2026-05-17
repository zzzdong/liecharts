use liecharts::{
    LieChart, LieChartOption, SeriesOption,
    RadarOption, RadarIndicatorOption, RadarSeriesOption, RadarDataOption,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = LieChart::new(800, 600);

    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("产品能力雷达图".to_string()),
            subtext: Some("多维度对比分析".to_string()),
            ..Default::default()
        }),
        legend: Some(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["产品A".to_string(), "产品B".to_string()]),
            ..Default::default()
        }),
        radar: Some(RadarOption {
            indicator: Some(vec![
                RadarIndicatorOption { name: Some("销量".to_string()), max: Some(100.0) },
                RadarIndicatorOption { name: Some("品牌".to_string()), max: Some(100.0) },
                RadarIndicatorOption { name: Some("增长".to_string()), max: Some(100.0) },
                RadarIndicatorOption { name: Some("满意度".to_string()), max: Some(100.0) },
                RadarIndicatorOption { name: Some("市占".to_string()), max: Some(100.0) },
            ]),
            center: Some(vec!["50%".to_string(), "55%".to_string()]),
            radius: Some(vec!["0%".to_string(), "65%".to_string()]),
            split_number: Some(5),
            ..Default::default()
        }),
        series: vec![
            SeriesOption::Radar(RadarSeriesOption {
                name: Some("产品A".to_string()),
                data: vec![
                    RadarDataOption {
                        value: vec![95.0, 80.0, 75.0, 90.0, 85.0],
                        name: Some("产品A".to_string()),
                    },
                ],
                ..Default::default()
            }),
            SeriesOption::Radar(RadarSeriesOption {
                name: Some("产品B".to_string()),
                data: vec![
                    RadarDataOption {
                        value: vec![70.0, 95.0, 90.0, 75.0, 60.0],
                        name: Some("产品B".to_string()),
                    },
                ],
                ..Default::default()
            }),
        ],
        ..Default::default()
    };

    chart.render_to_image(option, "radar.png")?;
    println!("雷达图已保存到 radar.png");

    Ok(())
}