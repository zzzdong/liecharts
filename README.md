LieCharts
=============

[![Crates.io](https://img.shields.io/crates/v/liecharts)](https://crates.io/crates/liecharts)
[![Documentation](https://docs.rs/liecharts/badge.svg)](https://docs.rs/liecharts)
[![Crates.io](https://img.shields.io/crates/l/liecharts)](LICENSE)

## Overview

A Rust library for creating charts, inspired by ECharts.

## Features

- **Multiple Chart Types**: Line, Bar, Pie, Area, Scatter, Radar, Gauge, Candlestick, Boxplot, Heatmap, Polar Bar, Polar Scatter, Table.
- **Double Rendering Engines**: PNG/JPEG and SVG.
- **Theme System**: customizable themes.
- **JSON Configuration**: configurable JSON configuration.
- **Complex Layouts**: Mixed charts, multiple Y axes, and more.

## Limitations

LieCharts 是**静态渲染**库：输出固定尺寸的 PNG/JPEG 或 SVG 图像，而非网页端可交互的图表。

- **不支持交互式 tooltip**：`tooltip` 配置会被正确解析（兼容 ECharts 风格 JSON，不会报错），但渲染阶段不会产生任何提示框。由于输出是静态图像、没有"鼠标悬停"事件，交互式 tooltip 在静态渲染下无法实现。如需数据点详情，请使用 `series.label` 数据标签或图例。
- **SVG 为矢量快照**：SVG 输出是渲染结果的一帧快照，不含 DOM 事件绑定，因此浏览器打开时同样无 hover/click 交互。
- **图表类型数据格式**：`gauge` 已支持标准 ECharts JSON 写法（如 `data: [{ value: 60, name: "CPU" }]`），可完整渲染仪表盘（渐变进度条/刻度/指针/中心值）。部分其他类型（如热力图、极坐标柱状图）对数据列的维度/结构有要求，请参照 `examples/` 中的样例写法。

## Usage

### DataFrame API (Recommended)

```rust
use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let df = dataframe!(
        "month"   => ["1月", "2月", "3月"],
        "revenue" => [120.0, 200.0, 150.0],
    );

    Chart::new(800, 600)
        .title("月度趋势")
        .add_bar(
            Bar::new()
                .data(df)
                .x("month")
                .y("revenue")
                .name("销售额"),
        )
        .render_to_svg("chart.svg")?;
    Ok(())
}
```

### JSON Configuration

```rust
use liecharts::ChartBuilder;

let json = r#"{
    "title": { "text": "月度趋势" },
    "xAxis": [{ "type": "category", "data": ["1月", "2月", "3月"] }],
    "yAxis": [{ "type": "value" }],
    "series": [{ "type": "bar", "name": "销售额", "data": [120, 200, 150] }]
}"#;

ChartBuilder::from_option_json(json)?
    .build(800, 600)?
    .render_to_image("chart.png")?;
```

## Examples

Check the [`examples`](./examples) directory. Run with: `cargo run --example <name>`

Online Demo: [https://zzzdong.github.io/liecharts](https://zzzdong.github.io/liecharts)

## License

Apache-2.0
