use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(liecharts::TitleOption {
            text: Some("产品销售数据表".to_string()),
            subtext: Some("Table Chart".to_string()),
            ..Default::default()
        })
        .with_series(SeriesOption::Table(liecharts::TableSeriesOption {
            name: Some("销售数据".to_string()),
            columns: Some(vec![
                "产品".to_string(),
                "销量(万)".to_string(),
                "单价(元)".to_string(),
                "总营收(亿)".to_string(),
                "评分".to_string(),
            ]),
            data: Some(vec![
                vec![
                    serde_json::json!("iPhone 15"),
                    serde_json::json!(1200),
                    serde_json::json!(7999),
                    serde_json::json!(959.9),
                    serde_json::json!(4.8),
                ],
                vec![
                    serde_json::json!("Galaxy S24"),
                    serde_json::json!(980),
                    serde_json::json!(6999),
                    serde_json::json!(685.9),
                    serde_json::json!(4.6),
                ],
                vec![
                    serde_json::json!("Pixel 8"),
                    serde_json::json!(450),
                    serde_json::json!(5999),
                    serde_json::json!(269.9),
                    serde_json::json!(4.5),
                ],
                vec![
                    serde_json::json!("小米 14"),
                    serde_json::json!(680),
                    serde_json::json!(4299),
                    serde_json::json!(292.3),
                    serde_json::json!(4.7),
                ],
            ]),
            ..Default::default()
        }))
        .build(800, 600)?
        .render_to_svg("table.svg")?;
    println!("数据表已保存到 table.svg");

    Ok(())
}
