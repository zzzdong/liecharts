//! Commonly used types for ergonomic imports.
//!
//! Use `use liecharts::prelude::*;` to bring in the most frequently used
//! chart-building types with a single import statement.
//!
//! # Examples
//!
//! ```no_run
//! use liecharts::prelude::*;
//!
//! ChartBuilder::new()
//!     .with_title(TitleOption::new("Sales"))
//!     .with_x_axis(AxisOption::category().data(["Q1", "Q2"]))
//!     .with_y_axis(AxisOption::value())
//!     .with_series(SeriesOption::Bar(
//!         BarSeriesOption::new("Revenue", vec![100.0, 200.0]),
//!     ))
//!     .build(800, 600)
//!     .unwrap()
//!     .render_to_image("chart.png")
//!     .unwrap();
//! ```

pub use crate::builder::function_data;
pub use crate::{
    // Common option types
    AxisOption,
    AxisPosition,
    AxisType,
    BarSeriesOption,
    // Builder — entry point
    ChartBuilder,
    ChartError,
    DataPoint,
    GridOption,
    LabelPosition,
    LegendOption,
    LineSeriesOption,
    PieSeriesOption,
    PositionOption,
    // Sampling
    SamplingOption,
    SamplingType,
    ScatterSeriesOption,
    SeriesOption,
    Theme,
    TitleOption,
};
