use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_config = r#"
        {"legend":{"data":[],"top":"8%"},"series":[{"data":[7067,7265,13006,7533,5514,5881,6284],"label":{"show":false},"name":"每日请求量趋势","type":"line"}],"title":{"left":"center","text":"每日请求量趋势"},"tooltip":{"trigger":"axis"},"xAxis":[{"data":["2026-07-27 00:00:00 +0800 CST","2026-07-28 00:00:00 +0800 CST","2026-07-29 00:00:00 +0800 CST","2026-07-30 00:00:00 +0800 CST","2026-07-31 00:00:00 +0800 CST","2026-08-01 00:00:00 +0800 CST","2026-08-02 00:00:00 +0800 CST"],"type":"category"}],"yAxis":[{"type":"value"}]}
    "#;

    let chart = ChartBuilder::from_option_json(json_config)?.build(800, 600)?;
    chart.render_to_svg("json_config.svg")?;
    println!("JSON配置图表已保存到 json_config.svg");

    Ok(())
}
