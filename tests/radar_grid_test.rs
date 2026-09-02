//! 雷达图网格形状回归：同心网格多边形的顶点数必须等于指示器（维度）数。
//!
//! 历史 bug：`render_axes` 调用 `RadarAxisRenderer::render` 时传空指示器
//! 数组，`len().max(3)` 兜底使**网格底纹退化为三角形**，而数据多边形
//! （builder 用真实 indicators）和指示器标签是正确的 N 边形——出现
//! "五维雷达图配三角形底纹"的错位。修复后网格维度数与数据多边形同源。

use liecharts::api::*;

/// 解析 path `d` 属性中的坐标点（雷达 path 均为 M/L/Z 多边形，无弧线）。
fn parse_points(d: &str) -> Vec<(f64, f64)> {
    let nums: Vec<f64> = d
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .filter(|t| !t.is_empty())
        .filter_map(|t| t.parse().ok())
        .collect();
    nums.as_chunks::<2>()
        .0
        .iter()
        .map(|c| (c[0], c[1]))
        .collect()
}

/// 提取 SVG 中所有 `<path d="...">` 的坐标点序列（在 d 属性闭合引号处截断）。
fn all_paths(svg: &str) -> Vec<Vec<(f64, f64)>> {
    svg.split("<path")
        .skip(1)
        .filter_map(|seg| seg.split("d=\"").nth(1))
        .filter_map(|rest| rest.split('"').next())
        .map(parse_points)
        .collect()
}

fn radar_svg() -> String {
    let indicators = ["销量", "品牌", "增长", "满意度", "市占"];
    let chart = Chart::new(800, 600)
        .title(Title::new("产品能力雷达图"))
        .legend(Legend::new().data(["产品A", "产品B"]))
        .add_radar(
            Radar::new(indicators.iter().map(|s| s.to_string()).collect())
                .data(dataframe!(
                    "name" => ["产品A"],
                    "value" => ["95,80,75,90,85"],
                ))
                .name("产品A")
                .values("value"),
        )
        .add_radar(
            Radar::new(indicators.iter().map(|s| s.to_string()).collect())
                .data(dataframe!(
                    "name" => ["产品B"],
                    "value" => ["70,95,90,75,60"],
                ))
                .name("产品B")
                .values("value"),
        );
    chart.render_svg().unwrap()
}

#[test]
fn five_dimension_radar_grid_is_pentagon() {
    let counts: Vec<usize> = all_paths(&radar_svg()).iter().map(Vec::len).collect();

    // 5 层同心网格 + 2 个系列 × (填充 + 描边) = 9 个多边形 path
    assert!(
        counts.len() >= 9,
        "应解析出网格与数据多边形 path，实际 {}",
        counts.len()
    );
    // 全部 path 顶点数 = 5（网格五边形 + 数据五边形），不得出现 3（三角形退化）
    for (i, c) in counts.iter().enumerate() {
        assert_eq!(
            *c, 5,
            "path[{i}] 顶点数应为 5（五边形），实际 {c}（3 = 网格维度丢失退化为三角形）"
        );
    }
}

#[test]
fn grid_and_data_share_geometry() {
    let paths = all_paths(&radar_svg());
    // 5 层网格在前，之后是 2 个系列 × (填充 + 描边)
    let (rings, data) = paths.split_at(5);
    assert_eq!(rings.len(), 5);
    assert!(!data.is_empty());

    // 所有网格/数据多边形起点都在 -90°（顶部）维度方向上：x = 圆心 x
    for (i, p) in paths.iter().enumerate() {
        assert_eq!(
            p.len(),
            5,
            "path[{i}] 应为五边形（网格与数据同维度），实际 {} 点",
            p.len()
        );
        assert!(
            (p[0].0 - 400.0).abs() < 1e-6,
            "path[{i}] 起点应位于顶部维度方向"
        );
    }

    // 数据顶点应落在最内圈与最外圈之间（与网格共用同一圆心/半径体系；
    // 历史上网格三角形与数据五边形几何错位时此断言不成立）
    let outer = rings[4][0].1; // 最外圈顶部 y（最小）
    let inner = rings[0][0].1; // 最内圈顶部 y（最大）
    for (i, p) in data.iter().enumerate() {
        assert!(
            p[0].1 >= outer - 1e-6 && p[0].1 <= inner + 1e-6,
            "数据 path[{i}] 顶部顶点 y={:.2} 应在网格环间 [{outer:.2}, {inner:.2}]",
            p[0].1
        );
    }
}
