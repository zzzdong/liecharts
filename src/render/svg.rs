//! SVG renderer — delegates vector output to lievisual's `SvgRenderer`.

use lievisual::render::Renderer as _;

use crate::{
    error::Result,
    render::to_scene,
    visual::VisualElement,
};

/// SVG renderer that produces XML markup for vector output.
///
/// Converts the legacy element list into a `lievisual::Scene` and hands it to
/// lievisual's `SvgRenderer`.
#[derive(Debug)]
pub struct SvgRenderer;

impl SvgRenderer {
    pub fn new() -> Self {
        Self
    }

    /// 渲染视觉元素序列并输出 SVG 字符串。
    pub fn render(self, elements: &[VisualElement], width: u32, height: u32) -> Result<String> {
        let scene = to_scene(elements, width, height);
        let mut renderer = lievisual::render::SvgRenderer::new(width as f64, height as f64);
        renderer.render_scene(&scene);
        Ok(renderer.into_string())
    }
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self::new()
    }
}
