use liecharts::{ChartBuilder, DataPoint, LabelPosition, LieChart, SeriesOption};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data1 = HashMap::new();
    data1.insert(
        "name".to_string(),
        serde_json::Value::String("直接访问".to_string()),
    );
    data1.insert("value".to_string(), serde_json::json!(335.0));

    let mut data2 = HashMap::new();
    data2.insert(
        "name".to_string(),
        serde_json::Value::String("邮件营销".to_string()),
    );
    data2.insert("value".to_string(), serde_json::json!(310.0));

    let mut data3 = HashMap::new();
    data3.insert(
        "name".to_string(),
        serde_json::Value::String("联盟广告".to_string()),
    );
    data3.insert("value".to_string(), serde_json::json!(234.0));

    let mut data4 = HashMap::new();
    data4.insert(
        "name".to_string(),
        serde_json::Value::String("视频广告".to_string()),
    );
    data4.insert("value".to_string(), serde_json::json!(135.0));

    let mut data5 = HashMap::new();
    data5.insert(
        "name".to_string(),
        serde_json::Value::String("搜索引擎".to_string()),
    );
    data5.insert("value".to_string(), serde_json::json!(1548.0));

    let model = ChartBuilder::new()
        .with_title(liecharts::TitleOption {
            text: Some("访问来源分布".to_string()),
            subtext: Some("Pie Chart".to_string()),
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
        .with_series(SeriesOption::Pie(liecharts::PieSeriesOption {
            name: Some("访问来源".to_string()),
            data: vec![
                DataPoint::Object(data1),
                DataPoint::Object(data2),
                DataPoint::Object(data3),
                DataPoint::Object(data4),
                DataPoint::Object(data5),
            ],
            radius: Some(vec!["0%".to_string(), "75%".to_string()]),
            center: Some(vec!["50%".to_string(), "50%".to_string()]),
            label: Some(liecharts::LabelOption {
                show: Some(true),
                position: Some(LabelPosition::Outside),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .build()?;

    let chart = LieChart::new(800, 600);
    chart.render_to_image(&model, "pie.png")?;
    println!("饼图已保存到 pie.png");

    Ok(())
}
