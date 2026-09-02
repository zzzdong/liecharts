//! 极坐标类半径边界回归：半径以「画布 min/2」为基准折算（P2a），
//! 渲染前必须 clamp 到绘图区内接半径，否则小 grid + 大画布会越出画布
//! （docs/布局自适应改造计划.md P5）

use liecharts::api::*;

/// 提取 `<path d="...">` 中的坐标点。
///
/// `d` 形如 `M204.1,142.9 C261.7,142.9 ... L204.1,322.8 Z`：把命令字母与空格
/// 统一视为分隔符，剩下的数字流两两成 (x, y)。
///
/// 只取 path 是刻意的：背景 `<rect>` 会引入画布四角，污染边界断言。
/// 注意饼图扇形用三次贝塞尔近似圆弧，控制点比真实曲线外扩约 14%
/// （`r * sqrt(1 + kappa²)`，kappa=0.5523），故断言按控制点留 15% + 2px 容差。
fn path_points(svg: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for seg in svg.split("<path").skip(1) {
        let Some(d) = seg.split("d=\"").nth(1).and_then(|s| s.split('"').next()) else {
            continue;
        };
        let nums: Vec<f64> = d
            .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .filter(|t| !t.is_empty() && *t != "-")
            .filter_map(|t| t.parse().ok())
            .collect();
        for &[x, y] in nums.as_chunks::<2>().0 {
            out.push((x, y));
        }
    }
    out
}

/// 小 grid + 大画布：画布基准半径（500×75%=375）远大于绘图区内接半径（75）
///
/// grid: left/right/top/bottom = 5%/80%/5%/80% → 绘图区 (50,50)-(200,200)，
/// 即 150×150，内接半径 75。
fn tight_grid_pie_svg() -> String {
    let df = liecharts::dataframe!(
        "category" => ["A", "B", "C"],
        "value" => [30.0, 50.0, 20.0],
    );
    Chart::new(1000, 1000)
        .grid(
            Grid::new()
                .left(Position::pct(5.0))
                .right(Position::pct(80.0))
                .top(Position::pct(5.0))
                .bottom(Position::pct(80.0)),
        )
        .add_pie(Pie::new().data(df))
        .render_svg()
        .unwrap()
}

#[test]
fn tight_grid_pie_stays_inside_canvas() {
    let svg = tight_grid_pie_svg();
    let pts = path_points(&svg);
    assert!(!pts.is_empty(), "应解析出饼图扇形的坐标点");

    for (x, y) in pts {
        assert!(
            x >= 0.0,
            "饼图坐标 x={x} 越出画布左缘（半径未按绘图区 clamp）"
        );
        assert!(x <= 1000.0, "饼图坐标 x={x} 越出画布右缘");
        assert!(y >= 0.0, "饼图坐标 y={y} 越出画布上缘");
        assert!(y <= 1000.0, "饼图坐标 y={y} 越出画布下缘");
    }
}

/// x 方向跨度，作为「实际半径」的代理指标。
///
/// 不直接量半径是因为：① `center_y` 有 0.55 偏移，y 方向不对称；
/// ② 扇形用三次贝塞尔近似圆弧，控制点比真实曲线外扩，且外扩系数随弧角度
/// 变化（非定值）。跨度对这两者都不敏感，且 clamp 前后差异达数倍，区分度足够。
fn x_span(svg: &str) -> f64 {
    let xs: Vec<f64> = path_points(svg).iter().map(|(x, _)| *x).collect();
    xs.iter().copied().fold(f64::MIN, f64::max) - xs.iter().copied().fold(f64::MAX, f64::min)
}

#[test]
fn tight_grid_pie_is_clamped_to_plot_area() {
    // 绘图区 150×150 → 内接半径 75，跨度应 ≈150~200（含控制点外扩）。
    // 若未 clamp（画布基准 375）则跨度 ≈750~825。
    let span = x_span(&tight_grid_pie_svg());
    assert!(
        span < 300.0,
        "小 grid 下半径应被 clamp 到内接半径 75，实测跨度 {span}（未 clamp 时约 800）"
    );
    assert!(span > 100.0, "饼图不应被过度压缩，实测跨度 {span}");
}

/// 反向对照：证明 clamp 只在必要时介入，而非无条件缩小。
///
/// grid 几乎占满画布时内接半径（480）大于画布基准半径（375），
/// 此时 clamp 不应生效，跨度应仍 ≈750~825。
#[test]
fn large_grid_pie_keeps_canvas_benchmark_radius() {
    let df = liecharts::dataframe!(
        "category" => ["A", "B", "C"],
        "value" => [30.0, 50.0, 20.0],
    );
    let svg = Chart::new(1000, 1000)
        .grid(
            Grid::new()
                .left(Position::pct(2.0))
                .right(Position::pct(2.0))
                .top(Position::pct(2.0))
                .bottom(Position::pct(2.0)),
        )
        .add_pie(Pie::new().data(df))
        .render_svg()
        .unwrap();

    let span = x_span(&svg);
    assert!(
        span > 700.0,
        "大 grid 下半径应保持画布基准 375（clamp 不生效），实测跨度 {span}"
    );
    assert!(span < 1000.0, "饼图不应超出画布，实测跨度 {span}");
}

/// path 点到圆心的最小距离（环形图内径的代理指标）。
///
/// 圆心与 builder 一致：`bounds.x0 + w/2`、`bounds.y0 + h*0.55`。
fn min_radius_from_center(svg: &str, cx: f64, cy: f64) -> f64 {
    path_points(svg)
        .iter()
        .map(|(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
        .fold(f64::MAX, f64::min)
}

/// 环形图（inner > 0）在紧边距下：外径被 clamp 后内径必须同比例缩放。
///
/// `radius=["40%","75%"]`（1000×1000）折算为 (200, 375)px；绘图区内接半径 75
/// → clamp 比例 0.2 → 内径应为 40。若内外径被独立 clamp 成相等的 75，
/// 圆环退化为实心圆（孔洞消失）。
#[test]
fn tight_grid_donut_inner_scales_with_clamped_outer() {
    let df = liecharts::dataframe!(
        "category" => ["A", "B", "C"],
        "value" => [30.0, 50.0, 20.0],
    );
    let svg = Chart::new(1000, 1000)
        .grid(
            Grid::new()
                .left(Position::pct(5.0))
                .right(Position::pct(80.0))
                .top(Position::pct(5.0))
                .bottom(Position::pct(80.0)),
        )
        .add_pie(Pie::new().data(df).radius(Size::pct(40.0), Size::pct(75.0)))
        .render_svg()
        .unwrap();

    // 外径 clamp 生效：跨度 ≈150~200（未 clamp 时约 800）
    let span = x_span(&svg);
    assert!(span < 300.0, "环形图外径应被 clamp，实测跨度 {span}");
    assert!(span > 100.0, "环形图不应被过度压缩，实测跨度 {span}");

    // 孔洞存在性：圆心到 path 点最小距离应 ≈ 内径 40。
    // 退化为实心时该值 ≈ 75（与外径相等）。
    let center = (125.0, 132.5); // 绘图区 (50,50)-(200,200)，y 偏移 0.55
    let min_r = min_radius_from_center(&svg, center.0, center.1);
    assert!(
        min_r < 60.0,
        "环形图孔洞应随外径 clamp 同比例缩小（内径≈40），实测最小距离 {min_r}（退化为实心时≈75）"
    );
}
