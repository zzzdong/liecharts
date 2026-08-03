use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_config = r#"
    {"series":[{"areaStyle":{},"data":[70840845],"smooth":true,"type":"line"}],"title":{"text":"请求趋势"},"tooltip":{"trigger":"axis"},"xAxis":[{"data":["2026-08-03T00:00:00+08:00"],"type":"category"}],"yAxis":[{"type":"value"}]}
    "#;

    let chart = ChartBuilder::from_option_json(json_config)?.build(800, 600)?;
    chart.render_to_svg("json_config.svg")?;
    println!("JSON配置图表已保存到 json_config.svg");

    Ok(())
}
