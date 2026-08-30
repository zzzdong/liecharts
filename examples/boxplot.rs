use liecharts::api::*;

#[path = "common/mod.rs"]
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("箱线图示例").subtext("统计分布"))
        .legend(Legend::new().data(["实验组 A"]))
        .add_boxplot(
            Boxplot::new()
                .data(dataframe!(
                    "category" => ["A", "B", "C", "D", "E"],
                    "min" => [10.0, 15.0, 8.0, 12.0, 18.0],
                    "q1" => [25.0, 30.0, 22.0, 28.0, 32.0],
                    "median" => [40.0, 45.0, 38.0, 42.0, 48.0],
                    "q3" => [55.0, 60.0, 52.0, 58.0, 62.0],
                    "max" => [70.0, 75.0, 68.0, 72.0, 78.0],
                ))
                .category("category")
                .min("min")
                .q1("q1")
                .median("median")
                .q3("q3")
                .max("max")
                .name("实验组 A"),
        );
    common::save(&chart, "boxplot.svg")?;

    Ok(())
}
