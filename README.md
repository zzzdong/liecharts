# LieCharts

基于 Rust 的图表库，受 ECharts 启发，支持 PNG 和 SVG 渲染。

## 特性

- **图表类型**：折线图、柱状图、饼图、面积图、散点图、雷达图、仪表盘、K线图、极坐标图、表格
- **双渲染引擎**：PNG（vello_cpu）和 SVG
- **主题系统**：8 种内置主题（echarts、light、dark、vintage、macarons、infographic、shine、roma）
- **自定义主题**：支持自定义配色和样式
- **字体管理**：支持加载自定义字体
- **JSON 配置**：可通过 JSON 配置图表
- **复杂布局**：多 Grid、混合图表、双 Y 轴

## 快速开始

```toml
[dependencies]
liecharts = "0.1.0-beta"
```

```rust
use liecharts::{LieChart, LieChartOption, AxisType, DataPoint, SeriesOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = LieChart::new(800, 600);

    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("月度趋势".to_string()),
            subtext: Some("2024年".to_string()),
            ..Default::default()
        }),
        x_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Category),
            data: Some(vec!["1月".to_string(), "2月".to_string(), "3月".to_string()]),
            ..Default::default()
        }],
        y_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Value),
            ..Default::default()
        }],
        series: vec![SeriesOption::Bar(liecharts::BarSeriesOption {
            name: Some("销售额".to_string()),
            data: vec![
                DataPoint::Number(120.0),
                DataPoint::Number(200.0),
                DataPoint::Number(150.0),
            ],
            ..Default::default()
        })],
        ..Default::default()
    };

    chart.render_to_image(option, "chart.png")?;
    Ok(())
}
```

## 图表类型

| 类型 | 说明 |
|------|------|
| line | 折线图 |
| bar | 柱状图 |
| pie | 饼图 |
| area | 面积图 |
| scatter | 散点图 |
| radar | 雷达图 |
| gauge | 仪表盘 |
| candlestick | K线图 |
| polar_bar | 极坐标柱状图 |
| polar_scatter | 极坐标散点图 |
| table | 表格 |

## 主题

```rust
use liecharts::{LieChart, Theme};

let chart = LieChart::new(800, 600)
    .with_theme(Theme::dark());
```

内置主题：`echarts`、`light`、`dark`、`vintage`、`macarons`、`infographic`、`shine`、`roma`

## 字体

```rust
use liecharts::{FontSource, register_font};

// 从文件加载
register_font(FontSource::Path("font.ttf".into()), Some("MyFont"))?;

// 从内存加载
register_font(FontSource::Memory(bytes), Some("MyFont"))?;
```

## JSON 配置

```json
{
    "title": { "text": "月度趋势" },
    "xAxis": [{ "type": "category", "data": ["1月", "2月", "3月"] }],
    "yAxis": [{ "type": "value" }],
    "series": [{ "type": "bar", "data": [120, 200, 150] }]
}
```

## 示例

查看 `examples` 目录下的示例代码，运行方式：`cargo run --example <name>`

在线演示：[https://zzzdong.github.io/liecharts](https://zzzdong.github.io/liecharts)

## 许可证

Apache-2.0
