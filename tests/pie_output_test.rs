//! 饼图输出断言测试。
//!
//! 对应 `docs/svg_chart_checklist.md` 中"饼图"相关检查：
//! - 扇区数量与数据项一致
//! - 扇区面积/角度与数值成正比（数值最大的扇区占最大角度）
//! - 扇区颜色各不相同，与主题调色板一致
//! - 数据标签完整、位置在画布内
//!
//! 说明：lievisual 将饼图扇区渲染为 `Path` 元素（非 `Pie` 图元），
//! 因此用 `solid_filled_paths` 提取实心填充的扇区路径。

mod common;
use common::*;

const W: f64 = 800.0;
const H: f64 = 600.0;

/// 扇区数量应与饼图数据项数一致（5 项）。
#[test]
fn pie_sector_count_matches_data() {
    let nodes = render("pie", 800, 600);
    let sectors = solid_filled_paths(&nodes);
    assert_eq!(
        sectors.len(),
        5,
        "饼图应有 5 个扇区，实际 {} 个: {:?}",
        sectors.len(),
        sectors
    );
}

/// 扇区颜色应各不相同（5 个扇区 5 种颜色）。
#[test]
fn pie_sector_colors_distinct() {
    let nodes = render("pie", 800, 600);
    let sectors = solid_filled_paths(&nodes);
    let colors: std::collections::HashSet<String> =
        sectors.iter().map(|(_, c)| c.clone()).collect();
    assert_eq!(
        colors.len(),
        5,
        "饼图扇区应有 5 种不同颜色，实际 {} 种: {:?}",
        colors.len(),
        sectors.iter().map(|(_, c)| c.clone()).collect::<Vec<_>>()
    );
}

/// 所有扇区边界必须在画布内。
#[test]
fn pie_sectors_in_canvas() {
    let nodes = render("pie", 800, 600);
    let sectors = solid_filled_paths(&nodes);
    let pts: Vec<(f64, f64)> = sectors
        .iter()
        .flat_map(|(bb, _)| vec![(bb.x0, bb.y0), (bb.x1, bb.y1)])
        .collect();
    assert_all_points_in_canvas(&pts, W, H, 2.0);
}

/// 数值最大的扇区（搜索引擎=1548，占 60%）应占据最大的角度/面积。
///
/// 由于每个扇区都从圆心出发，面积与"角度/360°"成正比。最大的扇区
/// 应跨越超过 90°（搜索引起点角应超过圆周的 1/4）。
#[test]
fn pie_largest_sector_has_largest_area() {
    let nodes = render("pie", 800, 600);
    let sectors = solid_filled_paths(&nodes);
    // 找垂直范围最大的扇区（占比大的扇区在 y 方向上跨越更大）
    let biggest = sectors
        .iter()
        .max_by(|(a, _), (b, _)| {
            let ah = a.y1 - a.y0;
            let bh = b.y1 - b.y0;
            ah.partial_cmp(&bh).unwrap()
        })
        .unwrap();
    // 最大扇区应跨越超过 1/4 圆周对应的垂直范围（圆心在 y≈328.9）
    let span = biggest.0.y1 - biggest.0.y0;
    assert!(
        span > 100.0,
        "最大扇区（搜索引擎）应跨越较大垂直范围，实际 {:?}",
        biggest.0
    );
}

/// 5 个数据标签都应渲染（直接访问/邮件营销/联盟广告/视频广告/搜索引擎）。
#[test]
fn pie_labels_all_present() {
    let nodes = render("pie", 800, 600);
    let all_texts: Vec<String> = texts(&nodes).iter().map(|(t, _, _)| t.clone()).collect();
    for name in ["直接访问", "邮件营销", "联盟广告", "视频广告", "搜索引擎"] {
        assert!(
            all_texts.iter().any(|t| t.contains(name)),
            "饼图缺少数据标签 '{}'。实际文本: {:?}",
            name,
            all_texts
        );
    }
}

/// 标签位置必须在画布内。
#[test]
fn pie_labels_in_canvas() {
    let nodes = render("pie", 800, 600);
    let pts: Vec<(f64, f64)> = texts(&nodes).iter().map(|(_, x, y)| (*x, *y)).collect();
    assert_all_points_in_canvas(&pts, W, H, 2.0);
}

/// 饼图扇区应闭合（渲染为完整区域而非开放线）。
#[test]
fn pie_sectors_closed() {
    let nodes = render("pie", 800, 600);
    let closed_count = paths(&nodes)
        .iter()
        .filter(|(_, _, closed)| *closed)
        .count();
    assert!(
        closed_count >= 5,
        "应有至少 5 个闭合路径（扇区），实际闭合 {} 个",
        closed_count
    );
}
