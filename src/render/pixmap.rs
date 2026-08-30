//! Pixmap renderer — delegates rasterization to lievisual's
//! `VelloPixmapRenderer` (backed by vello_cpu).

use vello_cpu::Pixmap;

use crate::{SceneNode, error::Result, render::to_scene};

/// Bitmap renderer backed by lievisual / vello_cpu.
///
/// Produces a `vello_cpu::Pixmap` that callers can encode to PNG/JPEG via the
/// `image` crate.
#[derive(Debug)]
pub struct PixmapRenderer {
    width: u32,
    height: u32,
}

impl PixmapRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// 渲染视觉元素序列并输出 Pixmap。
    ///
    /// Converts the legacy element list into a `lievisual::Scene` and hands it
    /// to lievisual's `VelloPixmapRenderer`.
    pub fn render(self, elements: &[SceneNode]) -> Result<Pixmap> {
        let scene = to_scene(elements, self.width, self.height);
        let mut renderer = lievisual::render::VelloPixmapRenderer::new(self.width, self.height);
        Ok(renderer.render_scene_to_pixmap(&scene))
    }
}
