use liecharts::api::*;

#[path = "common/mod.rs"]
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate y = x² curve data using DataFrame::from_function
    let df = DataFrame::from_function("x", "y", -1.0..=1.0, 10_000, |x| x * x);

    let chart = Chart::new(800, 480)
        .data(df)
        .title(Title::new("y = x²"))
        .x_axis(Axis::value().min(-1.0).max(1.0))
        .y_axis(Axis::value().min(-0.1).max(1.0))
        .add_line(
            Line::new()
                .x("x")
                .y("y")
                .name("y = x²")
                .smooth(true)
                .sampling(Sampling::Lttb(50)),
        );
    common::save(&chart, "function.svg")?;

    Ok(())
}
