//! 折线图/面积图输出断言测试。
//!
//! 对应 `docs/svg_chart_checklist.md` 中"折线图/面积图"相关检查：
//! - 数据点（circle）数量 = 总数据点数
//! - 每个系列的点的颜色一致，不同系列颜色不同
//! - 点的 Y 坐标随数值单调变化（值越大 y 越小，即越高）
//! - 折线 path 数量 = 系列数
//! - 所有元素在画布内

mod common;
use common::*;

/// 折线数据点数量 = 各系列数据点之和（line.json: 6×2=12）。
#[test]
fn line_point_count_matches_data() {
    let nodes = render("line", 800, 600);
    let points = circles(&nodes);
    assert_eq!(
        points.len(),
        12,
        "折线图应有 12 个数据点（2 系列×6 点），实际 {} 个",
        points.len()
    );
}

/// 两个系列的圆点颜色应不同，且各系列内部颜色一致。
#[test]
fn line_series_colors_distinct_and_consistent() {
    let nodes = render("line", 800, 600);
    let points = circles(&nodes);
    let mut colors: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for (_, _, style) in &points {
        if let Some(s) = &style.stroke {
            *colors.entry(solid_color(&s.color)).or_insert(0) += 1;
        }
    }
    // 两个系列 → 2 种颜色，每种颜色 6 个点
    assert_eq!(
        colors.len(),
        2,
        "折线图应有 2 种系列颜色，实际 {}: {:?}",
        colors.len(),
        colors
    );
    for (color, count) in &colors {
        assert_eq!(
            *count, 6,
            "颜色 {} 应有 6 个点，实际 {} 个",
            color, count
        );
    }
}

/// 系列"销售额"的 Y 坐标应反映数值：值 200（索引1）的点最高（y 最小），
/// 值 70（索引4）的点最低（y 最大）。
#[test]
fn line_series_y_reflects_values() {
    let nodes = render("line", 800, 600);
    let points = circles(&nodes);
    // 取第一个系列（描边 #5070dd，销售额）的点，按索引 0..6
    let mut sales: Vec<(f64, f64)> = points
        .iter()
        .filter(|(_, _, s)| {
            s.stroke
                .as_ref()
                .map(|st| solid_color(&st.color) == "#5070dd")
                .unwrap_or(false)
        })
        .map(|(c, _, _)| (c.x, c.y))
        .collect();
    // 按 x 排序（因为折线按类别从左到右）
    sales.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    assert_eq!(sales.len(), 6, "销售额系列应有 6 个点");
    let ys: Vec<f64> = sales.iter().map(|(_, y)| *y).collect();
    // 值 [120,200,150,80,70,110]：y 应越小越高
    // 200（idx1）最高 → ys[1] 最小
    let min_y = ys.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        (ys[1] - min_y).abs() < 1.0,
        "值 200 的点应最高（y 最小），实际 ys={:?}",
        ys
    );
    // 70（idx4）最低 → ys[4] 最大
    let max_y = ys.iter().cloned().fold(f64::MIN, f64::max);
    assert!(
        (ys[4] - max_y).abs() < 1.0,
        "值 70 的点应最低（y 最大），实际 ys={:?}",
        ys
    );
}

/// 折线 path 数量 = 系列数（2 条折线）。
#[test]
fn line_polyline_count_matches_series() {
    let nodes = render("line", 800, 600);
    let pl = paths(&nodes);
    // 折线是闭合=false 的 path；面积填充才是闭合=true
    let polylines = pl.iter().filter(|(_, _, closed)| !*closed).count();
    assert_eq!(
        polylines, 2,
        "折线图应有 2 条折线 path，实际 {} 条",
        polylines
    );
}

/// 所有数据点和折线都在画布内。
#[test]
fn line_elements_in_canvas() {
    let nodes = render("line", 800, 600);
    let pts: Vec<(f64, f64)> = circles(&nodes)
        .iter()
        .map(|(c, _, _)| (c.x, c.y))
        .collect();
    assert_all_points_in_canvas(&pts, 800.0, 600.0, 2.0);
}
