use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("堆叠面积图").subtext("Stacked Area Chart"))
        .with_legend(
            LegendOption::default()
                .data(["产品A", "产品B", "产品C"])
                .show(true),
        )
        .with_x_axis(AxisOption::category().data(["周一", "周二", "周三", "周四", "周五"]))
        .with_y_axis(AxisOption::value().name("访问量"))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            stack: Some("总量".to_string()),
            area_style: Some(liecharts::AreaStyleOption {
                color: None,
                opacity: None,
            }),
            ..liecharts::LineSeriesOption::new("产品A", vec![120.0, 200.0, 150.0, 80.0, 70.0])
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            stack: Some("总量".to_string()),
            area_style: Some(liecharts::AreaStyleOption {
                color: None,
                opacity: None,
            }),
            ..liecharts::LineSeriesOption::new("产品B", vec![100.0, 80.0, 120.0, 200.0, 150.0])
        }))
        .with_series(SeriesOption::Line(liecharts::LineSeriesOption {
            stack: Some("总量".to_string()),
            area_style: Some(liecharts::AreaStyleOption {
                color: None,
                opacity: None,
            }),
            ..liecharts::LineSeriesOption::new("产品C", vec![80.0, 120.0, 180.0, 60.0, 100.0])
        }))
        .build(800, 600)?
        .render_to_svg("stacked_area.svg")?;
    println!("堆叠面积图已保存到 stacked_area.svg");

    Ok(())
}
