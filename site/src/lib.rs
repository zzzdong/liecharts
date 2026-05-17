use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn render_chart(json: &str, width: u32, height: u32) -> Result<String, String> {
    let option = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let chart = liecharts::LieChart::new(width, height);
    chart.render_svg(option).map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn render_chart_png(json: &str, width: u32, height: u32) -> Result<Vec<u8>, String> {
    let option = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let chart = liecharts::LieChart::new(width, height);
    chart.render_png(option).map_err(|e| e.to_string())
}

/// 获取所有可用的主题名称列表（JSON 字符串数组）。
#[wasm_bindgen]
pub fn get_available_themes() -> String {
    let themes: Vec<String> = liecharts::theme::ThemeRegistry::new()
        .available_themes()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    serde_json::to_string(&themes).unwrap_or_else(|_| "[]".to_string())
}

/// 注册自定义字体（从 JS 传入字节数据）。
///
/// `name` 为字体族名称，供图表配置中的 `font_family` 使用。
/// `bytes` 为字体文件的原始二进制数据（TTF/OTF）。
#[wasm_bindgen]
pub fn register_font_bytes(name: &str, bytes: &[u8]) -> Result<(), String> {
    liecharts::register_font(
        liecharts::FontSource::Memory(bytes.to_vec()),
        Some(name),
    )
    .map_err(|e| e.to_string())
}