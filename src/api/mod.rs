//! DataFrame-centric chart API.
//!
//! This module provides a clean, Rust-idiomatic API for creating charts
//! using DataFrames as the primary data input, in contrast to the
//! ECharts-JSON-compatible [`crate::option`] module.
//!
//! # Quick Start
//!
//! ```no_run
//! use liecharts::api::*;
//!
//! // Create data as a DataFrame
//! let sales = dataframe!(
//!     "month" => ["Jan", "Feb", "Mar", "Apr"],
//!     "revenue" => [120.0, 200.0, 150.0, 80.0],
//!     "cost" => [90.0, 140.0, 110.0, 60.0],
//! );
//!
//! // Build a chart with layers
//! let svg = Chart::new(800, 600)
//!     .title("Monthly Sales")
//!     .add_line(
//!         LineLayer::new(sales.clone())
//!             .x("month")
//!             .y("revenue")
//!             .name("Revenue"),
//!     )
//!     .add_bar(
//!         BarLayer::new(sales)
//!             .x("month")
//!             .y("cost")
//!             .name("Cost"),
//!     )
//!     .render_svg()
//!     .unwrap();
//! ```

mod chart;
mod config;
mod layer;

pub use chart::Chart;
pub use config::*;
pub use layer::*;

/// Macro to create a [`DataFrame`](crate::pipeline::dataframe::DataFrame) with a concise syntax.
///
/// Each column is specified as `"name" => [val1, val2, ...]`.
/// Values are automatically converted via `DataValue::from`.
///
/// # Examples
///
/// ```
/// use liecharts::api::dataframe;
///
/// let df = dataframe!(
///     "category" => ["A", "B", "C"],
///     "value" => [10.0, 20.0, 30.0],
/// );
/// assert_eq!(df.row_count(), 3);
/// assert_eq!(df.column_count(), 2);
/// ```
#[macro_export]
macro_rules! dataframe {
    ($($col_name:expr => [$($value:expr),* $(,)?]),* $(,)?) => {{
        let mut __columns = Vec::new();
        $(
            let __data: Vec<$crate::pipeline::dataframe::DataValue> = vec![
                $($crate::pipeline::dataframe::DataValue::from($value)),*
            ];
            __columns.push($crate::pipeline::dataframe::Series::new($col_name, __data));
        )*
        $crate::pipeline::dataframe::DataFrame::from_columns(__columns)
    }};
}

pub use crate::dataframe;