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
    /// `fit_max` 为 [`FitMode::HugMax`](crate::pipeline::types::FitMode::HugMax)
    /// 的整体缩放回缩目标（画布超出上限时贴合内容等比缩放）；其它模式传 `None`。
    pub fn render(self, elements: &[SceneNode], fit_max: Option<(f64, f64)>) -> Result<Pixmap> {
        let scene = match fit_max {
            Some(max) => crate::render::to_fit_scene(elements, self.width, self.height, max),
            None => to_scene(elements, self.width, self.height),
        };
        let w = (scene.width.round() as i64).max(1) as u32;
        let h = (scene.height.round() as i64).max(1) as u32;
        let mut renderer = lievisual::render::VelloPixmapRenderer::new(w, h);
        Ok(renderer.render_scene_to_pixmap(&scene))
    }
}
