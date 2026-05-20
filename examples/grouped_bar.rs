use std::env;

use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = ChartBuilder::new()
        .with_title(TitleOption::new("分组柱状图示例"))
        .with_x_axis(AxisOption::category().data(["Q1", "Q2", "Q3", "Q4"]))
        .with_y_axis(AxisOption::value().name("销售额"))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "产品A",
            vec![120.0, 200.0, 150.0, 80.0],
        )))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "产品B",
            vec![80.0, 160.0, 120.0, 70.0],
        )))
        .with_series(SeriesOption::Bar(BarSeriesOption::new(
            "产品C",
            vec![60.0, 120.0, 90.0, 60.0],
        )))
        .build(800, 600)?;

    let output_path = "grouped_bar.png";
    chart.render_to_image(output_path)?;

    let full_path = env::current_dir()?.join(output_path);
    println!("分组柱状图已保存到: {}", full_path.display());
    Ok(())
}
