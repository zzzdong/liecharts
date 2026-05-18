use liecharts::{
    ChartBuilder, LieChart, PolarScatterDataPoint, PolarScatterSeriesOption, SeriesOption,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wind_data: Vec<PolarScatterDataPoint> = vec![
        PolarScatterDataPoint {
            angle: 0.0,
            radius: 5.0,
            symbol_size: Some(8.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 15.0,
            radius: 8.0,
            symbol_size: Some(10.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 30.0,
            radius: 12.0,
            symbol_size: Some(15.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 45.0,
            radius: 15.0,
            symbol_size: Some(18.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 60.0,
            radius: 10.0,
            symbol_size: Some(12.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 75.0,
            radius: 7.0,
            symbol_size: Some(9.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 90.0,
            radius: 20.0,
            symbol_size: Some(25.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 105.0,
            radius: 18.0,
            symbol_size: Some(22.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 120.0,
            radius: 14.0,
            symbol_size: Some(16.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 135.0,
            radius: 9.0,
            symbol_size: Some(11.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 150.0,
            radius: 6.0,
            symbol_size: Some(7.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 165.0,
            radius: 4.0,
            symbol_size: Some(5.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 180.0,
            radius: 11.0,
            symbol_size: Some(13.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 195.0,
            radius: 16.0,
            symbol_size: Some(19.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 210.0,
            radius: 22.0,
            symbol_size: Some(28.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 225.0,
            radius: 19.0,
            symbol_size: Some(24.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 240.0,
            radius: 13.0,
            symbol_size: Some(16.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 255.0,
            radius: 8.0,
            symbol_size: Some(10.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 270.0,
            radius: 17.0,
            symbol_size: Some(21.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 285.0,
            radius: 21.0,
            symbol_size: Some(26.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 300.0,
            radius: 25.0,
            symbol_size: Some(30.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 315.0,
            radius: 23.0,
            symbol_size: Some(27.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 330.0,
            radius: 15.0,
            symbol_size: Some(18.0),
            name: None,
        },
        PolarScatterDataPoint {
            angle: 345.0,
            radius: 10.0,
            symbol_size: Some(12.0),
            name: None,
        },
    ];

    let model = ChartBuilder::new()
        .with_title(liecharts::TitleOption {
            text: Some("极坐标散点图".to_string()),
            subtext: Some("风向风速分布".to_string()),
            ..Default::default()
        })
        .with_legend(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["风速".to_string()]),
            ..Default::default()
        })
        .with_series(SeriesOption::PolarScatter(PolarScatterSeriesOption {
            name: Some("风速".to_string()),
            data: wind_data,
            ..Default::default()
        }))
        .build()?;

    let chart = LieChart::new(800, 600);
    chart.render_to_image(&model, "polar_scatter.png")?;
    println!("极坐标散点图已保存到 polar_scatter.png");

    Ok(())
}
