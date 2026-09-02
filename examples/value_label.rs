//! line/bar 值标签（`label`）示例：位置、颜色、模板与负值处理。
//!
//! 值标签的位置语义以**值端**为基准（对齐 ECharts）：
//! - `Top`    —— 值端外侧：正值柱在柱顶上方、负值柱在柱底下方；折线在点上方
//! - `Inside` —— 值端内侧（柱内）；柱高不足容纳文字时自动回退柱外，避免溢出
//! - `Bottom` —— 折线点下方（柱状图回退为 `Top`）
//!
//! 颜色缺省时跟随语义默认：柱外/折线取系列色、柱内取白字。

use liecharts::Color;
use liecharts::api::*;
use liecharts::pipeline::types::ValueLabelPos;

#[path = "common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) 柱状图 · 柱顶外侧（默认位置），标签跟随系列色
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("柱状图值标签 · 柱顶外侧").subtext("label_position = Top（默认）"))
        .legend(Legend::new().data(["销售额"]))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
                ))
                .x("month")
                .y("value")
                .name("销售额")
                .label_show(true)
                .label_position(ValueLabelPos::Top),
        );
    common::save(&chart, "value_label_bar_top.svg")?;

    // 2) 柱状图 · 柱内（白字），并用模板追加单位
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(
            Title::new("柱状图值标签 · 柱内白字").subtext("label_position = Inside + 模板 {c} 万"),
        )
        .legend(Legend::new().data(["销售额"]))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [120.0, 200.0, 150.0, 80.0, 70.0, 110.0],
                ))
                .x("month")
                .y("value")
                .name("销售额")
                .label_show(true)
                .label_position(ValueLabelPos::Inside)
                .label_formatter("{c} 万"),
        );
    common::save(&chart, "value_label_bar_inside.svg")?;

    // 3) 柱状图 · 含负值：正值标签在柱顶上方、负值标签在柱底下方
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("含负值的值标签").subtext("标签恒贴“值端”外侧"))
        .legend(Legend::new().data(["同比增减"]))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [120.0, -60.0, 200.0, -35.0, 90.0, -110.0],
                ))
                .x("month")
                .y("value")
                .name("同比增减")
                .label_show(true),
        );
    common::save(&chart, "value_label_bar_negative.svg")?;

    // 4) 折线图 · 上/下两种位置，并指定标签颜色
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("折线图值标签 · 上方与下方").subtext("Top / Bottom + label_color"))
        .legend(Legend::new().data(["访问量", "下单量"]))
        .add_line(
            Line::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0],
                ))
                .x("month")
                .y("value")
                .name("访问量")
                .label_show(true)
                .label_position(ValueLabelPos::Top),
        )
        .add_line(
            Line::new()
                .data(dataframe!(
                    "month" => ["1月", "2月", "3月", "4月", "5月", "6月"],
                    "value" => [220.0, 282.0, 271.0, 334.0, 390.0, 430.0],
                ))
                .x("month")
                .y("value")
                .name("下单量")
                .label_show(true)
                .label_position(ValueLabelPos::Bottom)
                .label_color(Color::rgb(120, 120, 128)),
        );
    common::save(&chart, "value_label_line.svg")?;

    Ok(())
}
