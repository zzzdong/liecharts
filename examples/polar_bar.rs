use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("极坐标柱状图").subtext("Polar Bar Chart"))
        .with_legend(
            LegendOption::default()
                .data(["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"])
                .show(true),
        )
        .with_series(SeriesOption::PolarBar(
            liecharts::PolarBarSeriesOption::new(
                "访问来源",
                vec![335.0, 310.0, 234.0, 135.0, 1548.0],
            ),
        ))
        .build(800, 600)?
        .render_to_svg("polar_bar.svg")?;
    println!("极坐标柱状图已保存到 polar_bar.svg");

    Ok(())
}
