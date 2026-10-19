//! 能力总览示例：liecharts 的图形生成能力，按能力点分组演示。
//!
//! 每个能力点尽量同时给出 Rust API 与 ECharts option JSON 两种构建方式
//! （配置一致）；暂无 JSON 对应的能力（Hug 自适应是 `FitMode` 布局策略，
//! 函数曲线的数据由 `DataFrame::from_function` 生成而非字面量）仅给 API 版，
//! 并在注释中说明原因。
//!
//! 能力点：
//! 1. 暗色主题（`theme("dark")` / `"theme": "dark"`）
//! 2. 值标签（`label`：位置 / 模板 / 颜色）
//! 3. 函数曲线 + LTTB 采样（万级数据点降采样到 50 个视觉点）
//! 4. Hug 自适应布局（`FitMode::Hug`：画布贴合内容）
//!
//! 运行：cargo run --example showcase

use liecharts::ChartBuilder;
use liecharts::api::*;
use liecharts::pipeline::types::ValueLabelPos;

#[path = "../common/mod.rs"]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dark_theme_via_api()?;
    dark_theme_via_json()?;
    value_label_via_api()?;
    value_label_via_json()?;
    function_curve()?;
    hug_layout()?;
    Ok(())
}

// ── 能力 1：暗色主题 ──

/// Rust API 方式：暗色主题分组柱状图。
fn dark_theme_via_api() -> Result<(), Box<dyn std::error::Error>> {
    let df = dataframe!(
        "day" => ["周一", "周二", "周三", "周四", "周五"],
        "A" => [320.0, 332.0, 301.0, 334.0, 390.0],
        "B" => [220.0, 182.0, 191.0, 234.0, 290.0],
        "C" => [150.0, 232.0, 201.0, 154.0, 190.0],
    );

    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .data(df)
        .title(Title::new("深色主题示例").subtext("API 构建"))
        .legend(Legend::new().data(["产品A", "产品B", "产品C"]))
        .x_axis(Axis::category().data(["周一", "周二", "周三", "周四", "周五"]))
        .y_axis(Axis::value().name("销量"))
        .theme("dark")
        .add_bar(Bar::new().x("day").y("A").name("产品A"))
        .add_bar(Bar::new().x("day").y("B").name("产品B"))
        .add_bar(Bar::new().x("day").y("C").name("产品C"));
    common::save(&chart, "showcase_theme_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `dark_theme_via_api` 相同的暗色主题配置。
fn dark_theme_via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "深色主题示例", "subtext": "JSON 构建" },
  "theme": "dark",
  "legend": { "data": ["产品A", "产品B", "产品C"] },
  "xAxis": {
    "type": "category",
    "data": ["周一", "周二", "周三", "周四", "周五"]
  },
  "yAxis": { "type": "value", "name": "销量" },
  "series": [
    { "type": "bar", "name": "产品A", "data": [320, 332, 301, 334, 390] },
    { "type": "bar", "name": "产品B", "data": [220, 182, 191, 234, 290] },
    { "type": "bar", "name": "产品C", "data": [150, 232, 201, 154, 190] }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "showcase_theme_json")?;
    Ok(())
}

// ── 能力 2：值标签 ──

/// Rust API 方式：柱顶外侧值标签 + 模板单位。
fn value_label_via_api() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("柱状图值标签").subtext("API 构建"))
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
                .label_position(ValueLabelPos::Top)
                .label_formatter("{c} 万"),
        );
    common::save(&chart, "showcase_label_api")?;
    Ok(())
}

/// ECharts option JSON 方式：与 `value_label_via_api` 相同的值标签配置。
fn value_label_via_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
{
  "title": { "text": "柱状图值标签", "subtext": "JSON 构建" },
  "legend": { "data": ["销售额"] },
  "xAxis": {
    "type": "category",
    "data": ["1月", "2月", "3月", "4月", "5月", "6月"]
  },
  "yAxis": { "type": "value" },
  "series": [
    {
      "type": "bar",
      "name": "销售额",
      "label": { "show": true, "position": "top", "formatter": "{c} 万" },
      "data": [120, 200, 150, 80, 70, 110]
    }
  ]
}"#;
    let chart =
        ChartBuilder::from_option_json(json)?.build(common::DEFAULT_W, common::DEFAULT_H)?;
    common::save(&chart, "showcase_label_json")?;
    Ok(())
}

// ── 能力 3：函数曲线 + LTTB 采样 ──

/// Rust API 方式：`y = x²` 函数曲线，万级数据点经 LTTB 降采样到 50 个视觉点。
///
/// 无 JSON 对应版本：曲线数据由 `DataFrame::from_function` 程序化生成，
/// 而 JSON option 只能携带显式数据点（一万个点的字面量不具可读性）。
fn function_curve() -> Result<(), Box<dyn std::error::Error>> {
    let df = DataFrame::from_function("x", "y", -1.0..=1.0, 10_000, |x| x * x);

    let chart = Chart::new(800, 480)
        .data(df)
        .title(Title::new("y = x²").subtext("万级数据点 + LTTB 采样 50 点"))
        .x_axis(Axis::value().min(-1.0).max(1.0))
        .y_axis(Axis::value().min(-0.1).max(1.0))
        .add_line(
            Line::new()
                .x("x")
                .y("y")
                .name("y = x²")
                .smooth(true)
                .sampling(Sampling::Lttb(50)),
        );
    common::save(&chart, "showcase_function")?;
    Ok(())
}

// ── 能力 4：Hug 自适应布局 ──

/// Rust API 方式：`FitMode::Hug` —— 画布贴合内容，长 Y 轴刻度自动旋转也不越界。
///
/// 无 JSON 对应版本：Hug 是渲染期的布局策略（`FitMode`），不属于 ECharts option
/// 的描述范围。
fn hug_layout() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(320, 240)
        .title(Title::new("Hug: 长Y轴刻度").subtext("Fixed vs Hug"))
        .y_axis(Axis::value().name("数值(元)"))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "cat" => ["A", "B", "C", "D"],
                    "val" => [1.0, 2.0, 3.0, 4.0],
                ))
                .x("cat")
                .y("val")
                .name("值"),
        );
    common::save(&chart, "showcase_hug_fixed")?;
    common::save(&chart.fit(FitMode::Hug), "showcase_hug")?;
    Ok(())
}
