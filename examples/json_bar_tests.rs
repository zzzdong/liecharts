use liecharts::prelude::*;

#[path = "common/mod.rs"]
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 测试 1: 基础柱状图
    let json_basic_bar = r##"
{
  "title": {
    "text": "基础柱状图"
  },
  "xAxis": {
    "type": "category",
    "data": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
  },
  "yAxis": {
    "type": "value"
  },
  "series": [
    {
      "type": "bar",
      "data": [120, 200, 150, 80, 70, 110, 130]
    }
  ]
}"##;

    let chart = ChartBuilder::from_option_json(json_basic_bar)?.build(800, 600)?;
    common::save(&chart, "json_bar_basic.svg")?;

    // 测试 2: 分组柱状图
    let json_grouped_bar = r##"
{
  "title": {
    "text": "分组柱状图"
  },
  "legend": {
    "data": ["直接访问", "邮件营销"]
  },
  "xAxis": {
    "type": "category",
    "data": ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]
  },
  "yAxis": {
    "type": "value"
  },
  "series": [
    {
      "name": "直接访问",
      "type": "bar",
      "data": [320, 302, 301, 334, 390, 330, 320]
    },
    {
      "name": "邮件营销",
      "type": "bar",
      "data": [120, 132, 101, 134, 90, 230, 210]
    }
  ]
}"##;

    let chart = ChartBuilder::from_option_json(json_grouped_bar)?.build(800, 600)?;
    common::save(&chart, "json_bar_grouped.svg")?;

    // 测试 3: 堆叠柱状图
    let json_stacked_bar = r##"
{
  "title": {
    "text": "堆叠柱状图"
  },
  "legend": {
    "data": ["邮件营销", "联盟广告", "视频广告", "直接访问"]
  },
  "xAxis": {
    "type": "category",
    "data": ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]
  },
  "yAxis": {
    "type": "value"
  },
  "series": [
    {
      "name": "邮件营销",
      "type": "bar",
      "stack": "总量",
      "data": [120, 132, 101, 134, 90, 230, 210]
    },
    {
      "name": "联盟广告",
      "type": "bar",
      "stack": "总量",
      "data": [220, 182, 191, 234, 290, 330, 310]
    },
    {
      "name": "视频广告",
      "type": "bar",
      "stack": "总量",
      "data": [150, 232, 201, 154, 190, 330, 410]
    },
    {
      "name": "直接访问",
      "type": "bar",
      "stack": "总量",
      "data": [320, 332, 301, 334, 390, 330, 320]
    }
  ]
}"##;

    let chart = ChartBuilder::from_option_json(json_stacked_bar)?.build(800, 600)?;
    common::save(&chart, "json_bar_stacked.svg")?;

    // 测试 4: 水平柱状图
    let json_horizontal_bar = r##"
{
  "title": {
    "text": "水平柱状图"
  },
  "xAxis": {
    "type": "value"
  },
  "yAxis": {
    "type": "category",
    "data": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
  },
  "series": [
    {
      "type": "bar",
      "data": [120, 200, 150, 80, 70, 110, 130]
    }
  ]
}"##;

    let chart = ChartBuilder::from_option_json(json_horizontal_bar)?.build(800, 600)?;
    common::save(&chart, "json_bar_horizontal.svg")?;

    // 测试 5: 带背景色的柱状图
    let json_bar_with_bg = r##"
{
  "title": {
    "text": "带背景色的柱状图"
  },
  "xAxis": {
    "type": "category",
    "data": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
  },
  "yAxis": {
    "type": "value"
  },
  "series": [
    {
      "type": "bar",
      "showBackground": true,
      "backgroundStyle": {
        "color": "rgba(180, 180, 180, 0.2)"
      },
      "data": [120, 200, 150, 80, 70, 110, 130]
    }
  ]
}"##;

    let chart = ChartBuilder::from_option_json(json_bar_with_bg)?.build(800, 600)?;
    common::save(&chart, "json_bar_with_bg.svg")?;

    println!("\n所有 Bar 图表测试完成！");
    Ok(())
}
