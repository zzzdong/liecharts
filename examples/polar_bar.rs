use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("极坐标柱状图").subtext("Polar Bar Chart"))
        .legend(Legend::new().data(["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"]))
        .add_polar_bar(
            PolarBar::new()
                .data(dataframe!(
                    "label" => ["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"],
                    "angle" => [0.0, 72.0, 144.0, 216.0, 288.0],
                    "radius" => [335.0, 310.0, 234.0, 135.0, 1548.0],
                ))
                .name("访问来源")
                .angle("angle")
                .radius("radius"),
        )
        .render_to_svg("polar_bar.svg")?;
    println!("极坐标柱状图已保存到 polar_bar.svg");

    Ok(())
}
