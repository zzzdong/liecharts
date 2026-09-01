//! FitMode::Hug 端到端验证：公开 API 渲染出的 SVG 画布应按内容长大
//! （docs/布局自适应改造计划.md P1）

use liecharts::api::*;

/// 长类别 X 轴标签（CJK 12 字 ≈ 132px）超出默认底部边距 60px：
/// Hug 下画布应加高，Fixed 保持 300×200 不变。
fn long_category_svg(fit: FitMode) -> String {
    let df = liecharts::dataframe!(
        "cat" => ["一个非常非常长的类别名称啊", "短", "短"],
        "val" => [1.0, 2.0, 3.0],
    );
    Chart::new(300, 200)
        .fit(fit)
        .data(df)
        .add_bar(Bar::new().name("B").x("cat").y("val"))
        .render_svg()
        .unwrap()
}

fn svg_size(svg: &str) -> (f64, f64) {
    // 根节点形如 width="W" height="H" viewBox="0 0 W H"
    let w = svg
        .split("width=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let h = svg
        .split("height=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    (w, h)
}

#[test]
fn fixed_keeps_canvas() {
    let svg = long_category_svg(FitMode::Fixed);
    let (w, h) = svg_size(&svg);
    assert_eq!((w, h), (300.0, 200.0), "Fixed 模式画布必须保持 300×200");
}

#[test]
fn hug_grows_canvas_and_keeps_ratio_consistent() {
    let svg = long_category_svg(FitMode::Hug);
    let (w, h) = svg_size(&svg);
    assert!(h > 200.0, "Hug 应加高画布容纳旋转后的 X 轴标签，实际 {h}");
    // 画布加高后 viewBox 必须与根节点尺寸一致（用户坐标系 == 输出坐标系）
    assert!(
        svg.contains(&format!("viewBox=\"0 0 {w:.2} {h:.2}\"")),
        "viewBox 应与根尺寸一致：w={w} h={h}"
    );
}

#[test]
fn hug_keeps_long_x_labels_horizontal() {
    // P3：12 个长日期标签在 300px 画布下 Fixed 会旋转 90°，
    // Hug 应加宽画布使标签保持水平（信息零损失）
    let df = liecharts::dataframe!(
        "cat" => [
            "2024-01-01", "2024-01-02", "2024-01-03", "2024-01-04",
            "2024-01-05", "2024-01-06", "2024-01-07", "2024-01-08",
            "2024-01-09", "2024-01-10", "2024-01-11", "2024-01-12",
        ],
        "val" => [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
    );
    let svg = Chart::new(300, 200)
        .fit(FitMode::Hug)
        .data(df)
        .add_bar(Bar::new().name("B").x("cat").y("val"))
        .render_svg()
        .unwrap();

    let (w, _) = svg_size(&svg);
    assert!(w > 600.0, "Hug 应加宽画布使日期标签水平放下，实际 {w}");
    assert!(
        !svg.contains("rotate"),
        "Hug 下长日期标签应保持水平（不旋转/不抽稀）"
    );
}

#[test]
fn hugmax_scales_down_to_limit() {
    // HugMax：12 个长日期标签会要求 >600px 宽，但上限 300×200，
    // 应整体等比缩放回上限内（贴合内容，含 8px 边距）
    let df = liecharts::dataframe!(
        "cat" => [
            "2024-01-01", "2024-01-02", "2024-01-03", "2024-01-04",
            "2024-01-05", "2024-01-06", "2024-01-07", "2024-01-08",
            "2024-01-09", "2024-01-10", "2024-01-11", "2024-01-12",
        ],
        "val" => [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
    );
    let svg = Chart::new(300, 200)
        .fit(FitMode::HugMax)
        .data(df)
        .add_bar(Bar::new().name("B").x("cat").y("val"))
        .render_svg()
        .unwrap();

    let (w, h) = svg_size(&svg);
    assert!(w <= 300.0, "HugMax 宽应缩回上限 300 内，实际 {w}");
    assert!(h <= 200.0, "HugMax 高应缩回上限 200 内，实际 {h}");
    // 缩放后输出应为贴合内容的 viewBox（根尺寸 == viewBox）
    assert!(
        svg.contains(&format!("viewBox=\"0 0 {w:.2} {h:.2}\"")),
        "viewBox 应与缩放后根尺寸一致"
    );
}

#[test]
fn hug_zero_growth_when_labels_fit() {
    // 短标签不需要长大：Hug 与 Fixed 画布一致
    let df = liecharts::dataframe!(
        "cat" => ["A", "B", "C"],
        "val" => [1.0, 2.0, 3.0],
    );
    let svg = Chart::new(300, 200)
        .fit(FitMode::Hug)
        .data(df)
        .add_bar(Bar::new().name("B").x("cat").y("val"))
        .render_svg()
        .unwrap();
    let (w, h) = svg_size(&svg);
    assert_eq!((w, h), (300.0, 200.0), "标签放得下时 Hug 不应扩大画布");
}
