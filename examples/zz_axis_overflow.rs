//! 复现 Y 轴刻度标签左侧超界问题：读取临时高值 JSON 渲染
use liecharts::chart::Chart;
use liecharts::option::ChartOption;
use liecharts::theme::Theme;

fn main() {
    let json = std::fs::read_to_string("site/examples/_high_value.json").unwrap();
    let option: ChartOption = serde_json::from_str(&json).unwrap();
    let chart = Chart::new(option, Theme::echarts(), 800, 600);
    let elements = chart.collect_visual_elements().unwrap();
    // 打印绘图区边界（z=Z_BACKGROUND..Z_GRID 之间的 rect 或轴线 line）
    use liecharts::visual::SceneNode;
    fn walk(nodes: &[SceneNode], depth: usize) {
        for n in nodes {
            if let lievisual::scene::Element::Rect { rect, .. } = &n.element {
                // 只打印接近绘图区大小的 rect
                if rect.width() > 100.0 && rect.height() > 100.0 {
                    println!(
                        "{}rect ({:.0},{:.0})-({:.0},{:.0}) z={}",
                        "  ".repeat(depth),
                        rect.x0,
                        rect.y0,
                        rect.x1,
                        rect.y1,
                        n.z_index
                    );
                }
            }
            if let lievisual::scene::Element::Group { children } = &n.element {
                walk(children, depth + 1);
            }
        }
    }
    walk(&elements, 0);
    let svg = chart.render_svg().unwrap();
    std::fs::write("axis_overflow.svg", svg).unwrap();
    println!("saved");
}
