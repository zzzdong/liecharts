//! 面积图输出断言测试。
//!
//! 对应 `docs/svg_chart_checklist.md` 中"面积图"相关检查：
//! - 面积填充区域存在且为闭合路径
//! - 填充颜色与系列颜色一致（半透明）
//! - 数据点数量与数据一致
//! - 填充区域、边界线都在画布内

mod common;
use common::*;
use vello_cpu::kurbo::Shape;

const W: f64 = 800.0;
const H: f64 = 600.0;

/// 面积图应有 1 个填充区域路径（fill 非空）+ 1 个纯描边边界折线。
///
/// 注意：lievisual 的面积填充路径 `closed` 标志为 false，但通过非空 `fill`
/// 表现填充区域；上边界是 `fill=none`、`stroke` 有色的折线。
#[test]
fn area_has_fill_region_and_outline() {
    let nodes = render("area", 800, 600);
    let pl = paths(&nodes);
    let filled = pl
        .iter()
        .filter(|(_, style, _)| matches!(style.fill, Some(liecharts::visual::Fill::Solid(_))))
        .count();
    let stroked = pl
        .iter()
        .filter(|(_, style, _)| {
            style.stroke.is_some() && matches!(style.fill, None)
        })
        .count();
    assert!(
        filled >= 1,
        "面积图应有至少 1 个填充区域路径，实际 {} 个",
        filled
    );
    assert!(
        stroked >= 1,
        "面积图应有至少 1 个边界折线，实际 {} 个",
        stroked
    );
}

/// 面积填充路径颜色应与系列颜色一致（area.json 第一个系列色 #5070dd）。
#[test]
fn area_fill_color_matches_series() {
    let nodes = render("area", 800, 600);
    let mut fill_colors: Vec<String> = Vec::new();
    for (_, style, _) in paths(&nodes) {
        if let Some(liecharts::visual::Fill::Solid(c)) = &style.fill {
            fill_colors.push(solid_color(c));
        }
    }
    assert!(
        !fill_colors.is_empty(),
        "面积图应有带填充色的填充路径"
    );
    // 面积填充色应为 #5070dd（带 alpha 的填充路径归一化后得到纯色）
    assert!(
        fill_colors.contains(&"#5070dd".to_string()),
        "面积填充色应包含 #5070dd，实际 {:?}",
        fill_colors
    );
}

/// 面积图数据点数量 = 数据点数（area.json 6 点）。
#[test]
fn area_point_count_matches_data() {
    let nodes = render("area", 800, 600);
    let pts = circles(&nodes);
    assert_eq!(
        pts.len(),
        6,
        "面积图应有 6 个数据点，实际 {} 个",
        pts.len()
    );
}

/// 填充区域与数据点都应在画布内。
#[test]
fn area_elements_in_canvas() {
    let nodes = render("area", 800, 600);
    let mut pts: Vec<(f64, f64)> = circles(&nodes)
        .iter()
        .map(|(c, _, _)| (c.x, c.y))
        .collect();
    // 填充路径边界盒（fill 非空的 path）
    for (p, style, _) in paths(&nodes) {
        if matches!(style.fill, Some(liecharts::visual::Fill::Solid(_))) {
            let bb = p.bounding_box();
            pts.push((bb.x0, bb.y0));
            pts.push((bb.x1, bb.y1));
        }
    }
    assert_all_points_in_canvas(&pts, W, H, 2.0);
}

/// 面积填充区域的底部应延伸至 Y 轴 0 基线（填充到底部，不悬空）。
#[test]
fn area_fill_reaches_baseline() {
    let nodes = render("area", 800, 600);
    for (p, style, _) in paths(&nodes) {
        if matches!(style.fill, Some(liecharts::visual::Fill::Solid(_))) {
            let bb = p.bounding_box();
            // 填充区域底部应接近画布底部（Y 轴 0 位置，约 540）
            assert!(
                bb.y1 > 500.0,
                "面积填充底部 y1={} 应接近 0 基线（540），说明填充到底部",
                bb.y1
            );
        }
    }
}
