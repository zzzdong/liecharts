# LieCharts

[![Crates.io](https://img.shields.io/crates/v/liecharts)](https://crates.io/crates/liecharts)
[![Documentation](https://docs.rs/liecharts/badge.svg)](https://docs.rs/liecharts)
[![Crates.io](https://img.shields.io/crates/l/liecharts)](LICENSE)

A Rust charting library, inspired by ECharts.

## Requirements

Minimum supported Rust version: **1.92**.

## Features

- **Chart Types**: Line, Bar, Pie, Area, Scatter, Radar, Gauge, Candlestick, Boxplot, Heatmap, Polar Bar, Polar Scatter, Table.
- **Renderers**: PNG/JPEG and SVG.
- **Themes**: customizable themes.
- **Config**: JSON configuration, compatible with ECharts-style options.
- **Layouts**: mixed charts, multiple Y axes, and more.

## Limitations

LieCharts is a **static renderer**: it outputs fixed-size PNG/JPEG or SVG images, not interactive web charts.

- **No interactive tooltip**: `tooltip` options are parsed (ECharts-compatible, no errors) but not rendered, since static images have no hover events. Use `series.label` or the legend instead for data details.
- **SVG is a snapshot**: the SVG output has no DOM event bindings, so no hover/click interaction in the browser either.
- **Data format**: `gauge` supports standard ECharts JSON (e.g. `data: [{ value: 60, name: "CPU" }]`) and renders fully. Some other types (heatmap, polar bar) require specific data column shapes — see `examples/` for samples.

## Usage

### DataFrame API (Recommended)

```rust
use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let df = dataframe!(
        "month"   => ["Jan", "Feb", "Mar"],
        "revenue" => [120.0, 200.0, 150.0],
    );

    Chart::new(800, 600)
        .title("Monthly Trend")
        .add_bar(
            Bar::new()
                .data(df)
                .x("month")
                .y("revenue")
                .name("Revenue"),
        )
        .render_to_svg("chart.svg")?;
    Ok(())
}
```

### JSON Configuration

```rust
use liecharts::ChartBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{
        "title": { "text": "Monthly Trend" },
        "xAxis": [{ "type": "category", "data": ["Jan", "Feb", "Mar"] }],
        "yAxis": [{ "type": "value" }],
        "series": [{ "type": "bar", "name": "Revenue", "data": [120, 200, 150] }]
    }"#;

    ChartBuilder::from_option_json(json)?
        .build(800, 600)?
        .render_to_image("chart.png")?;
    Ok(())
}
```

### Canvas Fit Modes

The `width`/`height` you pass to `Chart::new` can mean three different things — pick one with `.fit(FitMode::...)`:

- **`FitMode::Fixed`** (default): the canvas is rigid. When content does not fit, the plot area shrinks, labels rotate/thin, and rows compress — historical behavior, byte-compatible.
- **`FitMode::Hug`**: `width`/`height` are *desired* sizes. Layout components report their space needs (axis labels, legend wrapping, table minimum row height) and the canvas **grows on demand** — zero information loss, constant font sizes, no global scaling.
- **`FitMode::HugMax`**: same as `Hug`, but the size is also a *limit*: if the content outgrows it, the whole chart is scaled down to fit.

```rust
use liecharts::api::*;

// Long date labels: Hug widens the canvas to keep them horizontal
// instead of rotating/thinning them
Chart::new(300, 200)
    .fit(FitMode::Hug)
    .data(df)
    .add_line(Line::new().x("date").y("value"))
    .render_to_svg("hug.svg")?;

// Dashboard slot: let the chart grow, then clamp back to the slot size
Chart::new(320, 240)
    .fit(FitMode::HugMax)
    .data(df)
    .add_bar(Bar::new().x("cat").y("val"))
    .render_to_png("slot.png")?;
```

See `examples/hug_demo.rs` for a side-by-side comparison, and
`docs/布局自适应改造计划.md` for the full layout redesign notes.

## Examples

Check the [`examples`](./examples) directory. Run with: `cargo run --example <name>`

Online Demo: [https://zzzdong.github.io/liecharts](https://zzzdong.github.io/liecharts)

## License

Apache-2.0
