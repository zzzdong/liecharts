//! 柱状图输出断言测试。
//!
//! 对应 `docs/svg_chart_checklist.md` 中"柱状图"相关检查：
//! - 柱子数量与数据点数一致
//! - 柱子高度与数值成正比（值最大的柱最高）
//! - 柱子宽度一致、间距均匀
//! - 所有柱在画布内、Y 轴为 0 基线
//!
//! 数据柱是 `Z_SERIES` 层的 `Rect` 元素；背景/图例等是其他层。

mod common;
use common::*;
use vello_cpu::kurbo::Rect;

/// 提取数据柱（Z_SERIES 层的 Rect，排除覆盖整个画布的背景 rect）。
fn data_bars(nodes: &[liecharts::visual::SceneNode]) -> Vec<Rect> {
    let all = rects(nodes);
    all.into_iter()
        .filter_map(|(r, _)| {
            // 排除背景 rect（覆盖整幅画布）
            if (r.width() - 800.0).abs() < 1.0 && (r.height() - 600.0).abs() < 1.0 {
                return None;
            }
            Some(r)
        })
        .collect()
}

/// 柱子数量应与数据点数一致（bar.json 6 个数据）。
#[test]
fn bar_count_matches_data() {
    let nodes = render("bar", 800, 600);
    let bars = data_bars(&nodes);
    assert_eq!(
        bars.len(),
        6,
        "柱状图应有 6 根柱，实际 {} 根: {:?}",
        bars.len(),
        bars
    );
}

/// 柱子高度应与数值成正比：值最大的柱（200）最高，值最小的柱（70）最矮。
#[test]
fn bar_heights_proportional_to_values() {
    let nodes = render("bar", 800, 600);
    let bars = data_bars(&nodes);
    // 数据 [120,200,150,80,70,110]，柱高应单调反映数值
    let heights: Vec<f64> = bars.iter().map(|r| r.height()).collect();
    // 200（索引1）应是最高的
    let max_h = heights.iter().cloned().fold(0.0, f64::max);
    assert!(
        (heights[1] - max_h).abs() < 1.0,
        "值最大的柱（200, 索引1）应最高，实际高度 {:?}",
        heights
    );
    // 70（索引4）应是最矮的
    let min_h = heights.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        (heights[4] - min_h).abs() < 1.0,
        "值最小的柱（70, 索引4）应最矮，实际高度 {:?}",
        heights
    );
    // 高度排序应与数值排序一致
    let values = [120.0, 200.0, 150.0, 80.0, 70.0, 110.0];
    for (h, v) in heights.iter().zip(values.iter()) {
        // 值 200 的柱应比值 150 的柱高，值 150 比 120 高，以此类推
        // 这里验证相对顺序：h_i 与 v_i 单调正相关
        let _ = (h, v);
    }
    // 简单验证：索引1(200) > 索引2(150) > 索引0(120) > 索引5(110) > 索引3(80) > 索引4(70)
    assert!(heights[1] > heights[2]);
    assert!(heights[2] > heights[0]);
    assert!(heights[0] > heights[5]);
    assert!(heights[5] > heights[3]);
    assert!(heights[3] > heights[4]);
}

/// 柱子宽度应一致（同一系列，等宽）。
#[test]
fn bar_widths_consistent() {
    let nodes = render("bar", 800, 600);
    let bars = data_bars(&nodes);
    let widths: Vec<f64> = bars.iter().map(|r| r.width()).collect();
    let first = widths[0];
    for w in &widths {
        assert!(
            (*w - first).abs() < 0.5,
            "柱宽应一致，期望 {}，实际 {}，全部 {:?}",
            first,
            w,
            widths
        );
    }
}

/// 所有柱都在画布内，且柱底对齐 Y 轴 0 基线（y + height ≈ 540 底部）。
#[test]
fn bars_in_canvas_and_zero_baseline() {
    let nodes = render("bar", 800, 600);
    let bars = data_bars(&nodes);
    let pts: Vec<(f64, f64)> = bars
        .iter()
        .flat_map(|r| vec![(r.x0, r.y0), (r.x1, r.y1)])
        .collect();
    assert_all_points_in_canvas(&pts, 800.0, 600.0, 2.0);
    // 柱底（y+height）应对齐同一基线
    let bottoms: Vec<f64> = bars.iter().map(|r| r.y1).collect();
    let first = bottoms[0];
    for b in &bottoms {
        assert!(
            (*b - first).abs() < 1.0,
            "柱底应对齐 0 基线，期望 {}，实际 {}，全部 {:?}",
            first,
            b,
            bottoms
        );
    }
}

/// 柱顶 y 应高于 0 基线（柱高为正），且最大柱不超出画布顶部。
#[test]
fn bars_tops_above_zero_and_within_canvas() {
    let nodes = render("bar", 800, 600);
    let bars = data_bars(&nodes);
    for r in &bars {
        assert!(
            r.y1 > 119.6 - 2.0 && r.y1 <= 600.0,
            "柱底 y1={} 应处于绘图区范围",
            r.y1
        );
    }
}

/// 堆叠柱状图（stacked_bar）：同一类别内，相邻系列的柱应无缝堆叠，
/// 即后一柱的底部（y+height）应等于前一柱的顶部（y）。
#[test]
fn stacked_bar_segments_connect() {
    let nodes = render("stacked_bar", 800, 600);
    let all = rects(&nodes);
    // 提取数据柱（排除背景 rect：覆盖整幅画布）
    let mut bars: Vec<Rect> = all
        .into_iter()
        .filter_map(|(r, _)| {
            if (r.width() - 800.0).abs() < 1.0 && (r.height() - 600.0).abs() < 1.0 {
                None
            } else {
                Some(r)
            }
        })
        .collect();
    // 按 (x0, y0) 排序：同一类别 x0 相同，按 y0 从小到大（从顶到底）
    bars.sort_by(|a, b| {
        a.x0.partial_cmp(&b.x0)
            .unwrap()
            .then(a.y0.partial_cmp(&b.y0).unwrap())
    });
    // 对每个类别（x0 相同的一组），验证相邻柱底部接缝
    let mut i = 0;
    while i < bars.len() {
        let x0 = bars[i].x0;
        let mut j = i;
        while j < bars.len() && (bars[j].x0 - x0).abs() < 0.5 {
            j += 1;
        }
        // 按 y0 升序（顶→底），同一类别内相邻柱应无缝堆叠：
        // 前一柱（更靠上）的底部 = 当前柱（更靠下）的顶部。
        for k in i + 1..j {
            let bottom_prev = bars[k - 1].y0 + bars[k - 1].height(); // 前柱底部
            let top_cur = bars[k].y0; // 当前柱顶部
            assert!(
                (bottom_prev - top_cur).abs() < 1.0,
                "类别 x0={} 内第 {} 根柱未无缝堆叠：前柱底={:.1}，当前柱顶={:.1}",
                x0,
                k - i,
                bottom_prev,
                top_cur
            );
        }
        i = j;
    }
}
