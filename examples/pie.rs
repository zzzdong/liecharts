use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("某站点用户访问来源").subtext("Pie Chart"))
        .legend(Legend::new().data(["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"]))
        .add_pie(
            Pie::new()
                .data(dataframe!(
                    "source" => ["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"],
                    "value" => [335.0, 310.0, 234.0, 135.0, 1548.0],
                ))
                .name("访问来源")
                .category("source")
                .value("value")
                .radius(0.0, 75.0),
        )
        .render_to_svg("pie.svg")?;
    println!("饼图已保存到 pie.svg");

    Ok(())
}
