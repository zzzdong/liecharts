use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("y = x²"))
        .with_x_axis(AxisOption::value().min(-1.0).max(1.0))
        .with_y_axis(AxisOption::value().min(-0.1).max(1.0))
        .with_series(SeriesOption::Line(
            LineSeriesOption::new("y = x²", function_data(-1.0..=1.0, 10_000, |x| x * x))
                .smooth(true)
                .sampling(SamplingOption::lttb(50)),
        ))
        .render_to_svg(800, 480, "function.svg")?;

    println!("generated function.svg (10_000 points sampled to 50 via LTTB)");

    Ok(())
}
