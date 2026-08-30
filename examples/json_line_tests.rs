use liecharts::prelude::*;

#[path = "common/mod.rs"]
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 测试 1: 基础折线图
    let json_basic_line = r##"
{
  "title": {
    "text": "基础折线图"
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
      "type": "line",
      "data": [150, 230, 224, 218, 135, 147, 260]
    }
  ]
}"##;

    let chart = ChartBuilder::from_option_json(json_basic_line)?.build(800, 600)?;
    common::save(&chart, "json_line_basic.svg")?;

    // 测试 2: 平滑曲线 + 面积图
    let json_smooth_area = r##"
{
  "title": {
    "text": "平滑面积图"
  },
  "xAxis": {
    "type": "category",
    "boundaryGap": false,
    "data": ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]
  },
  "yAxis": {
    "type": "value"
  },
  "series": [
    {
      "type": "line",
      "smooth": true,
      "areaStyle": {},
      "data": [820, 932, 901, 934, 1290, 1330, 1320]
    }
  ]
}"##;

    let chart = ChartBuilder::from_option_json(json_smooth_area)?.build(800, 600)?;
    common::save(&chart, "json_line_smooth_area.svg")?;

    // 测试 3: 多条折线
    let json_multi_line = r##"
{
  "title": {
    "text": "多条折线对比"
  },
  "legend": {
    "data": ["邮件营销", "联盟广告", "视频广告"]
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
      "type": "line",
      "data": [120, 132, 101, 134, 90, 230, 210]
    },
    {
      "name": "联盟广告",
      "type": "line",
      "data": [220, 182, 191, 234, 290, 330, 310]
    },
    {
      "name": "视频广告",
      "type": "line",
      "data": [150, 232, 201, 154, 190, 330, 410]
    }
  ]
}"##;

    let chart = ChartBuilder::from_option_json(json_multi_line)?.build(800, 600)?;
    common::save(&chart, "json_line_multi.svg")?;

    // 测试 4: 数值轴折线图（散点数据）
    let json_value_axis = r##"
{
  "title": {
    "text": "数值轴折线图"
  },
  "xAxis": {
    "type": "value"
  },
  "yAxis": {
    "type": "value"
  },
  "series": [
    {
      "type": "line",
      "data": [[0, 10], [10, 30], [20, 25], [30, 40], [40, 35], [50, 50]]
    }
  ]
}"##;

    let chart = ChartBuilder::from_option_json(json_value_axis)?.build(800, 600)?;
    common::save(&chart, "json_line_value_axis.svg")?;

    // 测试 5: 带标记点和标记线
    let json_markers = r##"
{
  "title": {
    "text": "带标记的折线图"
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
      "type": "line",
      "smooth": true,
      "data": [120, 200, 150, 80, 70, 110, 130]
    }
  ]
}"##;

    let chart = ChartBuilder::from_option_json(json_markers)?.build(800, 600)?;
    common::save(&chart, "json_line_markers.svg")?;

    println!("\n所有 Line 图表测试完成！");
    Ok(())
}
