use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    vertical_grouped()?;
    vertical_stacked()?;
    horizontal_grouped()?;
    horizontal_stacked()?;
    Ok(())
}

fn vertical_grouped() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("分组柱状图（纵向并列）"))
        .legend(Legend::new().data(["产品A", "产品B", "产品C"]))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "quarter" => ["Q1", "Q2", "Q3", "Q4"],
                    "value" => [120.0, 200.0, 150.0, 80.0],
                ))
                .x("quarter")
                .y("value")
                .name("产品A"),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "quarter" => ["Q1", "Q2", "Q3", "Q4"],
                    "value" => [80.0, 160.0, 120.0, 70.0],
                ))
                .x("quarter")
                .y("value")
                .name("产品B"),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "quarter" => ["Q1", "Q2", "Q3", "Q4"],
                    "value" => [60.0, 120.0, 90.0, 60.0],
                ))
                .x("quarter")
                .y("value")
                .name("产品C"),
        )
        .render_to_svg("bar_group_v_side.svg")?;
    println!("纵向并列分组 → bar_group_v_side.svg");
    Ok(())
}

fn vertical_stacked() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("分组柱状图（纵向堆叠）"))
        .legend(Legend::new().data(["直接销售", "代理销售", "线上销售"]))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "quarter" => ["Q1", "Q2", "Q3", "Q4"],
                    "value" => [120.0, 200.0, 150.0, 80.0],
                ))
                .x("quarter")
                .y("value")
                .name("直接销售")
                .stack("总量"),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "quarter" => ["Q1", "Q2", "Q3", "Q4"],
                    "value" => [80.0, 160.0, 120.0, 70.0],
                ))
                .x("quarter")
                .y("value")
                .name("代理销售")
                .stack("总量"),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "quarter" => ["Q1", "Q2", "Q3", "Q4"],
                    "value" => [60.0, 120.0, 90.0, 60.0],
                ))
                .x("quarter")
                .y("value")
                .name("线上销售")
                .stack("总量"),
        )
        .render_to_svg("bar_group_v_stack.svg")?;
    println!("纵向堆叠分组 → bar_group_v_stack.svg");
    Ok(())
}

fn horizontal_grouped() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("分组柱状图（横向并列）"))
        .legend(Legend::new().data(["直接渠道", "代理渠道"]))
        .x_axis(Axis::value().name("销售额（万元）"))
        .y_axis(Axis::category().data(["华北", "华东", "华南", "华西"]))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "region" => ["华北", "华东", "华南", "华西"],
                    "value" => [40.0, 80.0, 60.0, 30.0],
                ))
                .x("region")
                .y("value")
                .name("直接渠道"),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "region" => ["华北", "华东", "华南", "华西"],
                    "value" => [30.0, 60.0, 50.0, 20.0],
                ))
                .x("region")
                .y("value")
                .name("代理渠道"),
        )
        .render_to_svg("bar_group_h_side.svg")?;
    println!("横向并列分组 → bar_group_h_side.svg");
    Ok(())
}

fn horizontal_stacked() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("分组柱状图（横向堆叠）"))
        .legend(Legend::new().data(["线上", "线下"]))
        .x_axis(Axis::value().name("销售额（万元）"))
        .y_axis(Axis::category().data(["华北", "华东", "华南", "华西"]))
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "region" => ["华北", "华东", "华南", "华西"],
                    "value" => [20.0, 40.0, 30.0, 15.0],
                ))
                .x("region")
                .y("value")
                .name("线上")
                .stack("ch"),
        )
        .add_bar(
            Bar::new()
                .data(dataframe!(
                    "region" => ["华北", "华东", "华南", "华西"],
                    "value" => [30.0, 50.0, 40.0, 25.0],
                ))
                .x("region")
                .y("value")
                .name("线下")
                .stack("ch"),
        )
        .render_to_svg("bar_group_h_stack.svg")?;
    println!("横向堆叠分组 → bar_group_h_stack.svg");
    Ok(())
}