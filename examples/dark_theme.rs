use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let df = dataframe!(
        "day" => ["周一", "周二", "周三", "周四", "周五"],
        "A" => [320.0, 332.0, 301.0, 334.0, 390.0],
        "B" => [220.0, 182.0, 191.0, 234.0, 290.0],
        "C" => [150.0, 232.0, 201.0, 154.0, 190.0],
    );

    Chart::new(800, 600)
        .data(df)
        .title(Title::new("深色主题示例").subtext("Dark Theme Demo"))
        .legend(Legend::new().data(["产品A", "产品B", "产品C"]))
        .x_axis(Axis::category().data(["周一", "周二", "周三", "周四", "周五"]))
        .y_axis(Axis::value().name("销量"))
        .theme("dark")
        .add_bar(Bar::new().x("day").y("A").name("产品A"))
        .add_bar(Bar::new().x("day").y("B").name("产品B"))
        .add_bar(Bar::new().x("day").y("C").name("产品C"))
        .render_to_svg("dark_theme.svg")?;
    println!("深色主题图表已保存到 dark_theme.svg");

    Ok(())
}
