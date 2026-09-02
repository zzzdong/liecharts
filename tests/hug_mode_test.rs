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
        !has_rotated_text(&svg),
        "Hug 下长日期标签应保持水平（不旋转/不抽稀）"
    );
    // 对照组：Fixed 下同一份数据确实会旋转（证明上面断言不是空转）
    let fixed = Chart::new(300, 200)
        .fit(FitMode::Fixed)
        .data(liecharts::dataframe!(
            "cat" => [
                "2024-01-01", "2024-01-02", "2024-01-03", "2024-01-04",
                "2024-01-05", "2024-01-06", "2024-01-07", "2024-01-08",
                "2024-01-09", "2024-01-10", "2024-01-11", "2024-01-12",
            ],
            "val" => [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
        ))
        .add_bar(Bar::new().name("B").x("cat").y("val"))
        .render_svg()
        .unwrap();
    assert!(
        has_rotated_text(&fixed),
        "Fixed 下长日期标签应旋转（对照组，用于证明 Hug 断言有效）"
    );
}

/// 画布上是否出现旋转文本（`transform="rotate(...)"`）。
///
/// 只匹配 `transform` 属性中的 rotate，避免误伤文本内容里恰好出现的 "rotate"。
fn has_rotated_text(svg: &str) -> bool {
    svg.split('<')
        .filter(|tag| tag.contains("transform="))
        .any(|tag| tag.contains("rotate("))
}

/// 提取所有 `<text>` 的 (x, y)
fn text_positions(svg: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for seg in svg.split("<text").skip(1) {
        let x = seg
            .split("x=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok());
        let y = seg
            .split("y=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok());
        if let (Some(x), Some(y)) = (x, y) {
            out.push((x, y));
        }
    }
    out
}

#[test]
fn hug_grows_canvas_for_multi_row_table() {
    // P1：8 行表格在 260px 画布下 Fixed 会压扁行高（<28px），
    // Hug 应把画布加高到 (rows+1) * TABLE_MIN_ROW_H 以上
    let df = liecharts::dataframe!(
        "名称" => ["产品A", "产品B", "产品C", "产品D", "产品E", "产品F", "产品G", "产品H"],
        "销量" => [10.0, 20.0, 15.0, 25.0, 30.0, 18.0, 22.0, 12.0],
    );
    let mk = |fit: FitMode| {
        Chart::new(320, 260)
            .fit(fit)
            .title(Title::new("Hug: 多行表格"))
            .data(df.clone())
            .add_table(Table::new().name("库存表"))
            .render_svg()
            .unwrap()
    };

    let (_, fixed_h) = svg_size(&mk(FitMode::Fixed));
    let (hug_w, hug_h) = svg_size(&mk(FitMode::Hug));

    assert_eq!(fixed_h, 260.0, "Fixed 画布高度必须保持 260");
    assert_eq!(hug_w, 320.0, "本例横向无缺口，宽度不变");
    assert!(
        hug_h > fixed_h,
        "Hug 应加高画布容纳 9 行（含表头）× 最小行高，实际 {hug_h}"
    );
}

#[test]
fn legend_rows_stay_inside_canvas() {
    // 图例项总宽超出画布时换行（Fixed 与 Hug 都生效），且不得出现
    // 负坐标 / 越出右边界（历史 bug：单行居中算出的 start_x 可为负）
    let df = liecharts::dataframe!(
        "day" => ["周一", "周二", "周三", "周四", "周五"],
        "v1" => [1.0, 2.0, 3.0, 4.0, 5.0],
    );
    let mut chart = Chart::new(320, 240)
        .title(Title::new("图例换行"))
        .data(df)
        .add_line(Line::new().name("营业收入").x("day").y("v1"));
    for name in ["营业成本", "毛利润", "净利润", "研发支出", "管理费用"] {
        chart = chart.add_line(Line::new().name(name).x("day").y("v1"));
    }

    for fit in [FitMode::Fixed, FitMode::Hug] {
        let svg = chart.clone().fit(fit).render_svg().unwrap();
        let (w, h) = svg_size(&svg);
        // 图例位于顶部：统计 y 明显小于画布中线且 x 分布在多行上的文本
        let legend_texts: Vec<(f64, f64)> = text_positions(&svg)
            .into_iter()
            .filter(|(_, y)| *y < h * 0.35)
            .collect();
        let rows = {
            let mut ys: Vec<f64> = legend_texts.iter().map(|(_, y)| *y).collect();
            ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
            ys.dedup_by(|a, b| (*a - *b).abs() < 2.0);
            ys.len()
        };
        assert!(
            rows >= 2,
            "{fit:?} 下 6 个图例项应换行成多行，实际 {rows} 行"
        );
        for (x, y) in legend_texts {
            assert!(x >= 0.0, "{fit:?} 下图例文本 x={x} 越出画布左缘");
            assert!(x <= w, "{fit:?} 下图例文本 x={x} 越出画布右缘");
            assert!(y >= 0.0, "{fit:?} 下图例文本 y={y} 越出画布上缘");
        }
    }
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
