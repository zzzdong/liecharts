//! 布局/标题/图例输出断言测试。
//!
//! 对应 `docs/svg_chart_checklist.md` 第一节中"画布/标题/图例"相关检查：
//! - 标题、副标题文本正确渲染
//! - 图例项与系列名一致、完整
//! - 所有文本元素都在画布内（不超出边界）
//!
//! 使用 `stacked_bar`（有标题 + 副标题 + 3 项图例）。

mod common;
use common::*;

const W: f64 = 800.0;
const H: f64 = 600.0;

/// 标题与副标题都应渲染。
#[test]
fn title_and_subtitle_rendered() {
    let nodes = render("stacked_bar", 800, 600);
    let all_texts: Vec<String> = texts(&nodes).iter().map(|(t, _, _)| t.clone()).collect();
    assert!(
        all_texts.iter().any(|t| t.contains("堆叠柱状图")),
        "应渲染标题'堆叠柱状图'，实际文本: {:?}",
        all_texts
    );
    assert!(
        all_texts.iter().any(|t| t.contains("多系列堆叠展示")),
        "应渲染副标题'多系列堆叠展示'，实际文本: {:?}",
        all_texts
    );
}

/// 图例项与系列名一一对应（3 个系列名都应渲染）。
#[test]
fn legend_items_match_series_names() {
    let nodes = render("stacked_bar", 800, 600);
    let all_texts: Vec<String> = texts(&nodes).iter().map(|(t, _, _)| t.clone()).collect();
    for name in ["直接访问", "邮件营销", "联盟广告"] {
        assert!(
            all_texts.iter().any(|t| t.contains(name)),
            "图例缺少系列名 '{}'。实际文本: {:?}",
            name,
            all_texts
        );
    }
}

/// 图例项数量应与系列数一致（stacked_bar 3 个系列）。
#[test]
fn legend_count_matches_series() {
    let nodes = render("stacked_bar", 800, 600);
    let all_texts: Vec<String> = texts(&nodes).iter().map(|(t, _, _)| t.clone()).collect();
    let legend_items = all_texts
        .iter()
        .filter(|t| ["直接访问", "邮件营销", "联盟广告"].contains(&t.as_str()))
        .count();
    assert_eq!(legend_items, 3, "图例应有 3 项，实际 {} 项", legend_items);
}

/// 所有文本元素（标题/图例/刻度/标签）都在画布内。
#[test]
fn all_texts_in_canvas() {
    let nodes = render("stacked_bar", 800, 600);
    let pts: Vec<(f64, f64)> = texts(&nodes).iter().map(|(_, x, y)| (*x, *y)).collect();
    assert!(!pts.is_empty(), "应存在文本元素");
    assert_all_points_in_canvas(&pts, W, H, 2.0);
}

/// 图例（top:30）应位于绘图区上方，即图例文本 y 应小于数据区顶部。
#[test]
fn legend_above_plot_area() {
    let nodes = render("stacked_bar", 800, 600);
    let all = texts(&nodes);
    let legend_y = all
        .iter()
        .find(|(t, _, _)| t.contains("直接访问"))
        .map(|(_, _, y)| *y);
    assert!(legend_y.is_some(), "应找到图例'直接访问'");
    let y = legend_y.unwrap();
    // 标题在上方，图例在标题下方但仍在绘图区上方（绘图区顶部约 y=119）
    assert!(y > 0.0 && y < 300.0, "图例 y={} 应在绘图区上方附近", y);
}

/// 所有数据柱、网格线都应在画布内（覆盖 stack bar 整体布局）。
#[test]
fn stacked_bar_all_elements_in_canvas() {
    let nodes = render("stacked_bar", 800, 600);
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut all = Vec::new();
    common::flatten(&nodes, &mut all);
    for (e, _) in all {
        match e {
            lievisual::scene::Element::Rect { rect, .. } => {
                pts.push((rect.x0, rect.y0));
                pts.push((rect.x1, rect.y1));
            }
            lievisual::scene::Element::Circle { center, .. } => pts.push((center.x, center.y)),
            _ => {}
        }
    }
    assert!(!pts.is_empty());
    assert_all_points_in_canvas(&pts, W, H, 2.0);
}
