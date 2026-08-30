use liecharts::prelude::*;

#[path = "common/mod.rs"]
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_config = r#"
{
  "title": {"text": "安全事件类型分布"},
  "tooltip": {"trigger": "item"},
  "legend": {"orient": "vertical", "left": "left"},
  "series": [{
    "name": "安全事件类型分布",
    "type": "pie",
    "radius": ["0%", "70%"],
    "label": {
      "show": true,
      "formatter": "{b}: {d}%"
    },
    "data": [
      {"name": "运维监控", "value": 67403},
      {"name": "拒绝服务", "value": 5764},
      {"name": "数据泄露", "value": 4155},
      {"name": "可疑行为", "value": 2458},
      {"name": "登录操作", "value": 1536},
      {"name": "漏洞攻击", "value": 141},
      {"name": "web攻击", "value": 103},
      {"name": "网络行为", "value": 27}
    ]
  }]
}
"#;

    let chart = ChartBuilder::from_option_json(json_config)?.build(800, 600)?;
    common::save(&chart, "json_config.svg")?;

    Ok(())
}
