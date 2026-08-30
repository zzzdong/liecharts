use liecharts::api::*;

#[path = "common/mod.rs"]
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = Chart::new(common::DEFAULT_W, common::DEFAULT_H)
        .title(Title::new("产品能力雷达图").subtext("多维度对比分析"))
        .legend(Legend::new().data(["产品A", "产品B"]))
        .add_radar(
            Radar::new(vec![
                "销量".into(),
                "品牌".into(),
                "增长".into(),
                "满意度".into(),
                "市占".into(),
            ])
            .data(dataframe!(
                "name" => ["产品A"],
                "value" => ["95,80,75,90,85"],
            ))
            .name("产品A")
            .values("value"),
        )
        .add_radar(
            Radar::new(vec![
                "销量".into(),
                "品牌".into(),
                "增长".into(),
                "满意度".into(),
                "市占".into(),
            ])
            .data(dataframe!(
                "name" => ["产品B"],
                "value" => ["70,95,90,75,60"],
            ))
            .name("产品B")
            .values("value"),
        );
    common::save(&chart, "radar.svg")?;

    Ok(())
}
