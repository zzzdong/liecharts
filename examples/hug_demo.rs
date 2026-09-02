//! FitMode::Hug 演示：同一份窄画布数据分别以 Fixed / Hug 渲染，
//! 对比画布长大效果（docs/布局自适应改造计划.md P1）。
//!
//! 场景：
//! 1. 长 Y 轴数值刻度（千万级）→ 左侧边距不足，Hug 加宽画布
//! 2. 多行表格 → 行高被压扁，Hug 加高画布
//! 3. 多系列图例超出宽度 → 换行（Fixed 与 Hug 都生效）

use liecharts::api::*;

#[path = "common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ── 场景 1：长 Y 轴刻度（Bar，value 轴千万级）──
    let df_big = liecharts::dataframe!(
        "cat" => ["A", "B", "C"],
        "val" => [120_000_000.0, 240_000_000.0, 90_000_000.0],
    );
    let bar = Chart::new(320, 240)
        .title(Title::new("Hug: 长Y轴刻度"))
        .y_axis(Axis::value().name("数值(元)"))
        .add_bar(Bar::new().name("值").data(df_big).x("cat").y("val"));
    common::save(
        &bar.clone().fit(FitMode::Fixed),
        "hug_demo_1_long_y_fixed.svg",
    )?;
    common::save(&bar.fit(FitMode::Hug), "hug_demo_1_long_y_hug.svg")?;

    // ── 场景 2：多行表格（Hug 下画布加高，行高不被压扁）──
    let df_table = liecharts::dataframe!(
        "名称" => ["产品A", "产品B", "产品C", "产品D", "产品E", "产品F", "产品G", "产品H"],
        "销量" => [10.0, 20.0, 15.0, 25.0, 30.0, 18.0, 22.0, 12.0],
        "库存" => [100.0, 200.0, 150.0, 250.0, 300.0, 180.0, 220.0, 120.0],
    );
    let table = Chart::new(320, 260)
        .title(Title::new("Hug: 多行表格"))
        .add_table(Table::new().name("库存表").data(df_table));
    common::save(
        &table.clone().fit(FitMode::Fixed),
        "hug_demo_2_table_fixed.svg",
    )?;
    common::save(&table.fit(FitMode::Hug), "hug_demo_2_table_hug.svg")?;

    // ── 场景 3：多系列图例换行 ──
    let df_legend = liecharts::dataframe!(
        "day" => ["周一", "周二", "周三", "周四", "周五"],
        "v1" => [1.0, 2.0, 3.0, 4.0, 5.0],
    );
    let mut chart = Chart::new(320, 240)
        .title(Title::new("Hug: 图例换行"))
        .y_axis(Axis::value())
        .add_line(
            Line::new()
                .name("营业收入")
                .data(df_legend.clone())
                .x("day")
                .y("v1"),
        );
    for name in ["营业成本", "毛利润", "净利润", "研发支出", "管理费用"] {
        chart = chart.add_line(
            Line::new()
                .name(name)
                .data(df_legend.clone())
                .x("day")
                .y("v1"),
        );
    }
    common::save(
        &chart.clone().fit(FitMode::Fixed),
        "hug_demo_3_legend_fixed.svg",
    )?;
    common::save(&chart.fit(FitMode::Hug), "hug_demo_3_legend_hug.svg")?;

    println!("hug_demo 完成：对比 *_fixed.svg 与 *_hug.svg 的画布尺寸");
    Ok(())
}
