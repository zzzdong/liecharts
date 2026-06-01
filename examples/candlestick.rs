use liecharts::api::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Chart::new(800, 600)
        .title(Title::new("K线图示例").subtext("股票价格走势"))
        .legend(Legend::new().data(["日K"]))
        .add_candlestick(
            Candlestick::new()
                .data(dataframe!(
                    "date" => ["2024-01-02", "2024-01-03", "2024-01-04", "2024-01-05", "2024-01-08", "2024-01-09", "2024-01-10", "2024-01-11", "2024-01-12"],
                    "open" => [100.0, 105.0, 102.0, 108.0, 115.0, 112.0, 118.0, 125.0, 120.0],
                    "close" => [105.0, 102.0, 108.0, 115.0, 112.0, 118.0, 125.0, 120.0, 122.0],
                    "low" => [98.0, 100.0, 101.0, 106.0, 110.0, 111.0, 116.0, 118.0, 115.0],
                    "high" => [108.0, 110.0, 112.0, 118.0, 120.0, 125.0, 128.0, 130.0, 125.0],
                ))
                .category("date")
                .open("open")
                .close("close")
                .low("low")
                .high("high")
                .name("日K"),
        )
        .render_to_svg("candlestick.svg")?;
    println!("K线图已保存到 candlestick.svg");

    Ok(())
}
