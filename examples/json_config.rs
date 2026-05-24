use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_config = r#"
    {
        "title": {
            "text": "JSON配置示例",
            "subtext": "通过JSON配置图表"
        },
        "legend": {
            "show": true,
            "data": ["邮件营销", "联盟广告", "视频广告"]
        },
        "xAxis": [
            {
                "type": "category",
                "data": ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]
            }
        ],
        "yAxis": [
            {
                "type": "value",
                "name": "访问量"
            }
        ],
        "series": [
            {
                "type": "line",
                "name": "邮件营销",
                "data": [120, 132, 101, 134, 90, 230, 210]
            },
            {
                "type": "line",
                "name": "联盟广告",
                "data": [220, 182, 191, 234, 290, 330, 310]
            },
            {
                "type": "line",
                "name": "视频广告",
                "data": [150, 232, 201, 154, 190, 330, 410]
            }
        ]
    }
    "#;

    let chart = ChartBuilder::from_option_json(json_config)?.build(800, 600)?;
    chart.render_to_svg("json_config.svg")?;
    println!("JSON配置图表已保存到 json_config.svg");

    Ok(())
}
