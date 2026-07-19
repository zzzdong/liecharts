use liecharts::option::ChartOption;

fn main() {
    let files = vec![
        "line_with_tooltip_and_mark",
        "bar_with_visual_map",
        "pie_rose",
        "stacked_bar",
        "area_smooth",
        "radar_multi",
        "scatter_datazoom",
        "gauge_detailed",
    ];

    for name in files {
        let path = format!("site/examples/{}.json", name);
        let json = std::fs::read_to_string(&path).unwrap();
        match serde_json::from_str::<ChartOption>(&json) {
            Ok(_) => println!("{}: OK", name),
            Err(e) => println!("{}: FAIL - {}", name, e),
        }
    }
}
