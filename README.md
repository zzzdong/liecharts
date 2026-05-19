LieCharts
=============

[![Crates.io](https://img.shields.io/crates/v/liecharts)](https://crates.io/crates/liecharts)
[![Documentation](https://docs.rs/liecharts/badge.svg)](https://docs.rs/liecharts)
[![Crates.io](https://img.shields.io/crates/l/liecharts)](LICENSE)

## Overview

A Rust library for creating charts, inspired by ECharts.

## Features

- **Multiple Chart Types**: Line, Bar, Pie, Area, Scatter, Radar, Gauge, Candlestick, Polar Bar, Polar Scatter, Table.
- **Double Rendering Engines**: PNG/JPEG and SVG.
- **Theme System**: customizable themes.
- **JSON Configuration**: configurable JSON configuration.
- **Complex Layouts**: Mixed charts, multiple Y axes, and more.

## Usage

### Builder API (Recommended)

```rust
use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("月度趋势").subtext("2024年"))
        .with_x_axis(AxisOption::category().data(["1月", "2月", "3月"]))
        .with_y_axis(AxisOption::value())
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "销售额",
            vec![120.0, 200.0, 150.0],
        )))
        .build(800, 600)?
        .render_to_image("chart.png")?;
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
