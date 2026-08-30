use liecharts::{api::*, pipeline::dataframe::DataValue};
use lievisual::Color;

#[path = "common/mod.rs"]
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 7 天 × 8 小时的模拟打卡数据（计算网格，故用迭代构造而非 `dataframe!` 字面量宏）
    let xs: Vec<DataValue> = (0..7)
        .flat_map(|d| (0..8).map(move |_| DataValue::Float(d as f64)))
        .collect();
    let ys: Vec<DataValue> = (0..7)
        .flat_map(|_| (0..8).map(|h| DataValue::Float(h as f64)))
        .collect();
    let vals: Vec<DataValue> = (0..7)
        .flat_map(|d| (0..8).map(move |h| DataValue::Float(((d + h * 2) % 10) as f64)))
        .collect();
    let mut df = liecharts::api::DataFrame::new();
    df.add_column(liecharts::pipeline::dataframe::Series::new("x", xs));
    df.add_column(liecharts::pipeline::dataframe::Series::new("y", ys));
    df.add_column(liecharts::pipeline::dataframe::Series::new("value", vals));

    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("热力图示例").subtext("一周打卡分布"))
        .x_axis(Axis::category().data(["周一", "周二", "周三", "周四", "周五", "周六", "周日"]))
        .y_axis(Axis::category().data(["0点", "1点", "2点", "3点", "4点", "5点", "6点", "7点"]))
        .add_heatmap(
            Heatmap::new()
                .data(df)
                .x("x")
                .y("y")
                .value("value")
                .min(0.0)
                .max(10.0)
                .colors([
                    Color::rgb(80, 163, 186),
                    Color::rgb(234, 199, 54),
                    Color::rgb(217, 78, 93),
                ])
                .border_color(Color::rgb(255, 255, 255))
                .border_width(1.0)
                .name("打卡次数"),
        );
    common::save(&chart, "heatmap.svg")?;

    Ok(())
}
