use liecharts::LieChart;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chart = LieChart::new(800, 400);

    let option = liecharts::LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("员工信息表".to_string()),
            ..Default::default()
        }),
        series: vec![
            liecharts::SeriesOption::Table(liecharts::TableSeriesOption {
                name: Some("员工数据".to_string()),
                columns: Some(vec![
                    "姓名".to_string(),
                    "年龄".to_string(),
                    "部门".to_string(),
                    "职位".to_string(),
                ]),
                data: Some(vec![
                    vec![
                        serde_json::Value::String("张三".to_string()),
                        serde_json::Value::Number(28.into()),
                        serde_json::Value::String("技术部".to_string()),
                        serde_json::Value::String("工程师".to_string()),
                    ],
                    vec![
                        serde_json::Value::String("李四".to_string()),
                        serde_json::Value::Number(32.into()),
                        serde_json::Value::String("产品部".to_string()),
                        serde_json::Value::String("产品经理".to_string()),
                    ],
                    vec![
                        serde_json::Value::String("王五".to_string()),
                        serde_json::Value::Number(25.into()),
                        serde_json::Value::String("设计部".to_string()),
                        serde_json::Value::String("UI 设计师".to_string()),
                    ],
                    vec![
                        serde_json::Value::String("赵六".to_string()),
                        serde_json::Value::Number(30.into()),
                        serde_json::Value::String("市场部".to_string()),
                        serde_json::Value::String("市场经理".to_string()),
                    ],
                    vec![
                        serde_json::Value::String("钱七".to_string()),
                        serde_json::Value::Number(27.into()),
                        serde_json::Value::String("人力资源部".to_string()),
                        serde_json::Value::String("HR 专员".to_string()),
                    ],
                ]),
                ..Default::default()
            }),
        ],
        ..Default::default()
    };

    chart.set_option(option, None)?;
    chart.render_to_image("table_chart.png")?;
    chart.render_to_svg("table_chart.svg")?;
    
    println!("Table chart saved to table_chart.png and table_chart.svg");
    
    Ok(())
}