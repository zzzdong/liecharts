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
