use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("编程语言生态分析").subtext("气泡大小表示就业岗位相对数量"))
        .x_axis(Axis::value().name("诞生年份"))
        .y_axis(Axis::value().name("TIOBE 流行度指数"))
        .add_bubble(
            Bubble::new()
                .data(dataframe!(
                    "x" => [1991.0, 1995.0, 1972.0, 1985.0, 1995.0, 2009.0, 2010.0, 2014.0, 2011.0, 2012.0],
                    "y" => [14.0, 12.0, 11.0, 10.0, 3.0, 2.0, 1.0, 1.0, 1.0, 2.0],
                    "size" => [400.0, 320.0, 80.0, 60.0, 250.0, 35.0, 10.0, 20.0, 15.0, 100.0],
                    "name" => ["Python", "Java", "C", "C++", "JavaScript", "Go", "Rust", "Swift", "Kotlin", "TypeScript"],
                ))
                .name("编程语言")
                .size_col("size")
                .name_col("name")
                .symbol_size_scale(0.3),
        )
        .render_to_svg("scatter.svg")?;
    println!("散点图已保存到 scatter.svg");

    Ok(())
}
