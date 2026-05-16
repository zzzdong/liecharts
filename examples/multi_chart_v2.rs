use liecharts::{
    AxisOption, AxisType, DataPoint, GridOption, LieChart, LieChartOption,
    Position, SeriesOption, BarSeriesOption, PieSeriesOption,
    LegendOption, TitleOption,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let option = LieChartOption {
        title: Some(TitleOption {
            text: Some("多图表布局示例".to_string()),
            subtext: Some("左侧条形图 + 右侧饼图".to_string()),
            ..Default::default()
        }),
        legend: Some(LegendOption {
            show: Some(true),
            data: Some(vec!["产品A".to_string(), "产品B".to_string(), "产品C".to_string(), "产品D".to_string()]),
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
                    "产品A".to_string(),
                    "产品B".to_string(),
                    "产品C".to_string(),
                    "产品D".to_string(),
                ]),
                grid_index: Some(0),
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
        ],
        series: vec![
            SeriesOption::Bar(BarSeriesOption {
                name: Some("销量".to_string()),
                data: vec![
                    DataPoint::Number(120.0),
                    DataPoint::Number(200.0),
                    DataPoint::Number(150.0),
                    DataPoint::Number(80.0),
                ],
                grid_index: Some(0),
                ..Default::default()
            }),
            SeriesOption::Pie(PieSeriesOption {
                name: Some("占比".to_string()),
                data: vec![
                    DataPoint::Object({
                        let mut m = HashMap::new();
                        m.insert("name".to_string(), serde_json::json!("产品A"));
                        m.insert("value".to_string(), serde_json::json!(120.0));
                        m
                    }),
                    DataPoint::Object({
                        let mut m = HashMap::new();
                        m.insert("name".to_string(), serde_json::json!("产品B"));
                        m.insert("value".to_string(), serde_json::json!(200.0));
                        m
                    }),
                    DataPoint::Object({
                        let mut m = HashMap::new();
                        m.insert("name".to_string(), serde_json::json!("产品C"));
                        m.insert("value".to_string(), serde_json::json!(150.0));
                        m
                    }),
                    DataPoint::Object({
                        let mut m = HashMap::new();
                        m.insert("name".to_string(), serde_json::json!("产品D"));
                        m.insert("value".to_string(), serde_json::json!(80.0));
                        m
                    }),
                ],
                grid_index: Some(1),
                center: Some(vec!["50%".to_string(), "50%".to_string()]),
                radius: Some(vec!["0%".to_string(), "40%".to_string()]),
                ..Default::default()
            }),
        ],
        ..Default::default()
    };

    let mut chart = LieChart::new(1000, 600);

    chart.set_option(option, None)?;
    chart.render_to_image("multi_chart_v2.png")?;
    chart.render_to_svg("multi_chart_v2.svg")?;
    println!("多图表布局已保存到 multi_chart_v2.png 和 multi_chart_v2.svg");
    Ok(())
}