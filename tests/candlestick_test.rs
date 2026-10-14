//! K 线图输出断言：实体填充与涨跌色对齐 ECharts 默认语义。
//!
//! 回归：实体曾按 A 股惯例把阳线画成**空心**（仅描边），与官方输出不一致；
//! 涨跌色也曾用 #ea5543 / #50aa5e 而非 ECharts 的 #ec0000 / #00da3c。

use liecharts::api::*;

/// 一涨一跌两根蜡烛：实体都应**实心填充**，颜色分别为默认涨色/跌色。
#[test]
fn candlestick_bodies_are_filled_with_echarts_default_colors() {
    let svg = Chart::new(800, 600)
        .add_candlestick(
            Candlestick::new()
                .data(dataframe!(
                    "date" => ["D1", "D2"],
                    "open" => [100.0, 110.0],
                    "close" => [110.0, 105.0],
                    "low" => [98.0, 104.0],
                    "high" => [112.0, 112.0],
                ))
                .category("date")
                .open("open")
                .close("close")
                .low("low")
                .high("high")
                .name("日K"),
        )
        .render_svg()
        .unwrap();

    // 涨（D1: 100 -> 110）实体 = ECharts 默认阳线色 #ec0000
    assert_eq!(
        svg.matches("fill=\"#ec0000\"").count(),
        1,
        "阳线实体应为 1 个、填充 #ec0000（ECharts itemStyle.color 默认值）"
    );
    // 跌（D2: 110 -> 105）实体 = ECharts 默认阴线色 #00da3c
    assert_eq!(
        svg.matches("fill=\"#00da3c\"").count(),
        1,
        "阴线实体应为 1 个、填充 #00da3c（ECharts itemStyle.color0 默认值）"
    );
    // 两根实体都不得是空心（描边-only 的 rect 带 fill="none"）。
    // 注意：只检查 `<rect ...>` **标签自身**的属性区——`<line>` 也带
    // fill="none"，按 "split('<rect')" 切分会把后续所有元素算进最后一段。
    let hollow_bodies = svg
        .match_indices("<rect")
        .filter(|(i, _)| {
            let rest = &svg[*i..];
            let end = rest.find('>').unwrap_or(rest.len());
            rest[..end].contains("fill=\"none\"")
        })
        .count();
    assert_eq!(hollow_bodies, 0, "K 线实体不应有空心（fill=none）矩形");
}
