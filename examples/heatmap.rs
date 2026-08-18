use liecharts::{api::*, pipeline::dataframe::DataValue};
use lievisual::Color;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 7 天 × 8 小时的模拟打卡数据
    let mut xs: Vec<DataValue> = Vec::new();
    let mut ys: Vec<DataValue> = Vec::new();
    let mut vals: Vec<DataValue> = Vec::new();
    for d in 0..7 {
        for h in 0..8 {
            xs.push(DataValue::Float(d as f64));
            ys.push(DataValue::Float(h as f64));
            vals.push(DataValue::Float(((d + h * 2) % 10) as f64));
        }
    }
    let mut df = liecharts::api::DataFrame::new();
    df.add_column(liecharts::pipeline::dataframe::Series::new("x", xs));
    df.add_column(liecharts::pipeline::dataframe::Series::new("y", ys));
    df.add_column(liecharts::pipeline::dataframe::Series::new("value", vals));

    Chart::new(800, 600)
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
        )
        .render_to_svg("heatmap.svg")?;
    println!("热力图已保存到 heatmap.svg");

    Ok(())
}
