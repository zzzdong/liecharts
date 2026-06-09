//! Commonly used types for ergonomic imports.
//!
//! Use `use liecharts::prelude::*;` to bring in the most frequently used
//! chart-building types with a single import statement.
//!
//! # Examples
//!
//! ```no_run
//! use liecharts::prelude::*;
//! ```

pub use crate::{
    AxisOption, AxisPosition, AxisType, Chart, ChartBuilder, ChartError, ChartOption, DataPoint,
    GridOption, LabelPosition, LegendOption, SamplingOption, SamplingType, SeriesOption, Theme,
    TitleOption,
};
