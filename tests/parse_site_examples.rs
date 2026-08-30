use liecharts::option::ChartOption;

#[test]
fn parse_site_examples() {
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

    let mut failed = 0usize;
    for name in files {
        let path = format!("{}/site/examples/{}.json", env!("CARGO_MANIFEST_DIR"), name);
        let json = std::fs::read_to_string(&path).unwrap();
        match serde_json::from_str::<ChartOption>(&json) {
            Ok(_) => println!("{}: OK", name),
            Err(e) => {
                println!("{}: FAIL - {}", name, e);
                failed += 1;
            }
        }
    }
    assert_eq!(failed, 0, "{} 个站点示例 JSON 解析失败", failed);
}
