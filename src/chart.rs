pub use vello_cpu::Pixmap;

use crate::{
    error::Result,
    new_pipeline::pipeline::build_chart_with_theme,
    option::ChartOption,
    render::{PixmapRenderer, SvgRenderer},
    theme::Theme,
    visual::VisualElement,
};

pub struct Chart {
    option: ChartOption,
    theme: Theme,
    width: u32,
    height: u32,
}

impl Chart {
    pub fn new(option: ChartOption, theme: Theme, width: u32, height: u32) -> Self {
        Self {
            option,
            theme,
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn option(&self) -> &ChartOption {
        &self.option
    }

    pub fn render_to_image(&self, path: &str) -> Result<()> {
        let elements = self.collect_visual_elements()?;
        write_pixmap(&elements, self.width, self.height, path)
    }

    pub fn render_to_svg(&self, path: &str) -> Result<()> {
        let elements = self.collect_visual_elements()?;
        let svg = svg_string(&elements, self.width, self.height);
        std::fs::write(path, svg)?;
        Ok(())
    }

    pub fn render_png(&self) -> Result<Vec<u8>> {
        let elements = self.collect_visual_elements()?;
        png_bytes(&elements, self.width, self.height)
    }

    pub fn render_svg(&self) -> Result<String> {
        let elements = self.collect_visual_elements()?;
        Ok(svg_string(&elements, self.width, self.height))
    }

    pub fn collect_visual_elements(&self) -> Result<Vec<VisualElement>> {
        build_chart_with_theme(&self.option, self.width, self.height, &self.theme)
    }
}

fn write_pixmap(elements: &[VisualElement], width: u32, height: u32, path: &str) -> Result<()> {
    let renderer = PixmapRenderer::new(width, height);
    let pixmap = renderer.render(elements)?;
    let pw = pixmap.width() as u32;
    let ph = pixmap.height() as u32;
    let data: Vec<u8> = pixmap
        .data()
        .iter()
        .flat_map(|p| vec![p.r, p.g, p.b, p.a])
        .collect();
    let image = image::RgbaImage::from_raw(pw, ph, data)
        .ok_or_else(|| crate::error::ChartError::RenderError("Failed to create image".to_string()))?;
    image.save(path)?;
    Ok(())
}

fn svg_string(elements: &[VisualElement], width: u32, height: u32) -> String {
    let renderer = SvgRenderer::new();
    renderer.render(elements, width, height).unwrap_or_default()
}

fn png_bytes(elements: &[VisualElement], width: u32, height: u32) -> Result<Vec<u8>> {
    let renderer = PixmapRenderer::new(width, height);
    let pixmap = renderer.render(elements)?;
    let data: Vec<u8> = pixmap
        .data()
        .iter()
        .flat_map(|p| vec![p.r, p.g, p.b, p.a])
        .collect();
    let image = image::RgbaImage::from_raw(pixmap.width() as u32, pixmap.height() as u32, data)
        .ok_or_else(|| crate::error::ChartError::RenderError("Failed to create PNG image".to_string()))?;
    let mut buf = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
    Ok(buf)
}
