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
//! // Dual-axis chart: line (left) + bar (right)
//! let svg = Chart::new(800, 600)
//!     .title("Monthly Sales")
//!     .add_line(
//!         Line::new()
//!             .data(sales.clone())
//!             .x("month").y("revenue")
//!             .name("Revenue"),
//!     )
//!     .add_bar(
//!         Bar::new()
//!             .data(sales)
//!             .x("month").y("cost")
//!             .name("Cost")
//!             .right_axis(),
//!     )
//!     .render_svg()
//!     .unwrap();
//! ```
//!
//! # Multi-grid layout with sub_grid (no manual grid_index)
//!
//! ```no_run
//! use liecharts::api::*;
//!
//! let _svg = Chart::new(1000, 900)
//!     .sub_grid(
//!         Grid::new().left(Position::pct(3.0)).top(Position::pct(3.0))
//!                     .right(Position::pct(50.0)).bottom(Position::pct(50.0)),
//!         |g| g
//!             .x_axis(Axis::category().data(["A", "B", "C"]))
//!             .y_axis(Axis::value())
//!             .add_layer(Bar::new().data(dataframe!("x"=>["A","B","C"],"y"=>[1.0,2.0,3.0])).x("x").y("y")),
//!     )
//!     .sub_grid(
//!         Grid::new().left(Position::pct(50.0)).top(Position::pct(3.0))
//!                     .right(Position::pct(3.0)).bottom(Position::pct(50.0)),
//!         |g| g
//!             .x_axis(Axis::category().data(["D", "E", "F"]))
//!             .y_axis(Axis::value())
//!             .add_layer(Line::new().data(dataframe!("x"=>["D","E","F"],"y"=>[4.0,5.0,6.0])).x("x").y("y")),
//!     )
//!     .add_pie(Pie::new().data(dataframe!("name"=>["X","Y"],"val"=>[10.0,20.0])).category("name").value("val"))
//!     .render_svg()
//!     .unwrap();
//! ```

mod chart;
mod layer;

pub use chart::{
    Axis, AxisPosition, AxisType, Chart, Grid, GridBuilder, Legend, Orient, Position, Size, Title,
};
pub use layer::*;

pub use crate::pipeline::dataframe::DataFrame;

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
