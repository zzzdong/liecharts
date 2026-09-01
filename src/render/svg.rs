//! SVG renderer — delegates vector output to lievisual's `SvgRenderer`.

use lievisual::render::Renderer as _;

use crate::{SceneNode, error::Result, render::to_scene};

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
    ///
    /// `fit_max` 为 [`FitMode::HugMax`](crate::pipeline::types::FitMode::HugMax)
    /// 的整体缩放回缩目标（画布超出上限时贴合内容等比缩放，见 `to_fit_scene`）；
    /// 其它模式传 `None`。
    pub fn render(
        self,
        elements: &[SceneNode],
        width: u32,
        height: u32,
        fit_max: Option<(f64, f64)>,
    ) -> Result<String> {
        let scene = match fit_max {
            Some(max) => crate::render::to_fit_scene(elements, width, height, max),
            None => to_scene(elements, width, height),
        };
        let out_w = scene.width;
        let out_h = scene.height;
        let mut renderer = lievisual::render::SvgRenderer::new(out_w, out_h);
        renderer.render_scene(&scene);
        Ok(renderer.into_string())
    }
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self::new()
    }
}
