use liecharts::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ChartBuilder::new()
        .with_title(TitleOption::new("某站点用户访问来源").subtext("Pie Chart"))
        .with_legend(
            LegendOption::default()
                .data(["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"])
                .show(true),
        )
        .with_series(SeriesOption::Pie(liecharts::PieSeriesOption {
            radius: Some(vec!["0%".to_string(), "75%".to_string()]),
            center: Some(vec!["50%".to_string(), "50%".to_string()]),
            label: Some(liecharts::LabelOption {
                show: Some(true),
                position: Some(LabelPosition::Outside),
                ..Default::default()
            }),
            ..liecharts::PieSeriesOption::new(
                "访问来源",
                vec![
                    ("直接访问", 335.0),
                    ("邮件营销", 310.0),
                    ("联盟广告", 234.0),
                    ("视频广告", 135.0),
                    ("搜索引擎", 1548.0),
                ],
            )
        }))
        .build(800, 600)?
        .render_to_image("pie.png")?;
    println!("饼图已保存到 pie.png");

    Ok(())
}
