use liecharts::{BubbleDataPoint, prelude::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(liecharts::TitleOption {
            text: Some("编程语言生态分析".to_string()),
            subtext: Some("气泡大小 = 气泡大小表示就业岗位相对数量".to_string()),
            ..Default::default()
        })
        .with_x_axis(liecharts::AxisOption {
            axis_type: Some(AxisType::Value),
            name: Some("诞生年份".to_string()),
            ..Default::default()
        })
        .with_y_axis(liecharts::AxisOption {
            axis_type: Some(AxisType::Value),
            name: Some("TIOBE 流行度指数".to_string()),
            ..Default::default()
        })
        .with_series(SeriesOption::Bubble(liecharts::BubbleSeriesOption {
            name: Some("编程语言".to_string()),
            data: vec![
                BubbleDataPoint {
                    x: 1991.0,
                    y: 14.0,
                    size: Some(400.0),
                    name: Some("Python".to_string()),
                },
                BubbleDataPoint {
                    x: 1995.0,
                    y: 12.0,
                    size: Some(320.0),
                    name: Some("Java".to_string()),
                },
                BubbleDataPoint {
                    x: 1972.0,
                    y: 11.0,
                    size: Some(80.0),
                    name: Some("C".to_string()),
                },
                BubbleDataPoint {
                    x: 1985.0,
                    y: 10.0,
                    size: Some(60.0),
                    name: Some("C++".to_string()),
                },
                BubbleDataPoint {
                    x: 1995.0,
                    y: 3.0,
                    size: Some(250.0),
                    name: Some("JavaScript".to_string()),
                },
                BubbleDataPoint {
                    x: 2009.0,
                    y: 2.0,
                    size: Some(35.0),
                    name: Some("Go".to_string()),
                },
                BubbleDataPoint {
                    x: 2010.0,
                    y: 1.0,
                    size: Some(10.0),
                    name: Some("Rust".to_string()),
                },
                BubbleDataPoint {
                    x: 2014.0,
                    y: 1.0,
                    size: Some(20.0),
                    name: Some("Swift".to_string()),
                },
                BubbleDataPoint {
                    x: 2011.0,
                    y: 1.0,
                    size: Some(15.0),
                    name: Some("Kotlin".to_string()),
                },
                BubbleDataPoint {
                    x: 2012.0,
                    y: 2.0,
                    size: Some(100.0),
                    name: Some("TypeScript".to_string()),
                },
            ],
            ..Default::default()
        }))
        .build(800, 600)?
        .render_to_image("scatter.png")?;
    println!("散点图已保存到 scatter.png");

    Ok(())
}
