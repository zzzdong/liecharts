//! 散点图/气泡图输出断言测试。
//!
//! 对应 `docs/svg_chart_checklist.md` 中"散点/气泡图"相关检查：
//! - 数据点（circle）数量 = 总数据点数
//! - 每个系列的点颜色一致、不同系列颜色不同
//! - 所有点半径一致（普通散点）且在画布内
//! - X/Y 数值轴刻度在画布内

mod common;
use common::*;
use liecharts::visual::Fill;

const W: f64 = 800.0;
const H: f64 = 600.0;

/// 散点数量 = 各系列数据点之和（scatter.json: 10×2=20）。
#[test]
fn scatter_point_count_matches_data() {
    let nodes = render("scatter", 800, 600);
    let pts = circles(&nodes);
    assert_eq!(
        pts.len(),
        20,
        "散点图应有 20 个数据点（2 系列×10 点），实际 {} 个",
        pts.len()
    );
}

/// 两个系列颜色不同，各系列内部颜色一致。
#[test]
fn scatter_series_colors() {
    let nodes = render("scatter", 800, 600);
    let pts = circles(&nodes);
    let mut colors: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for (_, _, s) in &pts {
        if let Some(Fill::Solid(c)) = &s.fill {
            *colors.entry(solid_color(c)).or_insert(0) += 1;
        }
    }
    assert_eq!(
        colors.len(),
        2,
        "散点图应有 2 种系列颜色，实际 {}: {:?}",
        colors.len(),
        colors
    );
    for (color, count) in &colors {
        assert_eq!(*count, 10, "颜色 {} 应有 10 个点，实际 {}", color, count);
    }
}

/// 普通散点的半径应一致。
#[test]
fn scatter_radius_uniform() {
    let nodes = render("scatter", 800, 600);
    let pts = circles(&nodes);
    let r0 = pts[0].1;
    for (_, r, _) in &pts {
        assert!(
            (*r - r0).abs() < 0.01,
            "普通散点半径应一致，期望 {}，实际 {}",
            r0,
            r
        );
    }
}

/// 所有散点都在画布内。
#[test]
fn scatter_points_in_canvas() {
    let nodes = render("scatter", 800, 600);
    let pts: Vec<(f64, f64)> = circles(&nodes)
        .iter()
        .map(|(c, _, _)| (c.x, c.y))
        .collect();
    assert_all_points_in_canvas(&pts, W, H, 2.0);
}

/// 散点的 X 坐标应在数值轴范围内递增（不越界）。
#[test]
fn scatter_points_span_value_axis() {
    let nodes = render("scatter", 800, 600);
    let pts = circles(&nodes);
    // X 值范围应在绘图区内（约 60~720）
    for (c, _, _) in &pts {
        assert!(
            c.x > 50.0 && c.x < 740.0,
            "散点 X={} 应在数值轴范围内",
            c.x
        );
    }
}
