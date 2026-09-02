//! 渲染入口：把 pipeline 产出的 `Vec<SceneNode>` 组装成 `lievisual::Scene`，
//! 交给 lievisual 的 `SvgRenderer` / `VelloPixmapRenderer` 渲染。

use lievisual::{
    Color,
    fit::{FitOptions, fit_scene},
    scene::{Element, Fill, Scene, SceneNode},
};

use crate::Z_BACKGROUND;

/// 由 pipeline 产出的节点列表构建一个 lievisual::Scene。
pub fn to_scene(nodes: &[SceneNode], width: u32, height: u32) -> Scene {
    let mut scene = Scene::new(f64::from(width), f64::from(height));
    for node in nodes {
        scene.push_node(node.clone());
    }
    scene
}

/// HugMax：把内容整体等比缩放回 `fit_max`（上限）内的 Scene。
///
/// 步骤：
/// 1. 提取并移除整画布背景矩形（`Z_BACKGROUND` 的全幅 rect），背景色
///    转入 `Scene::background` —— 否则 bbox ≡ 画布，`fit_scene` 退化为平移；
/// 2. `fit_scene` 按内容包围盒等比缩放（`upscale=false`，内容超出上限时
///    缩小、装得下时不放大），并把节点包进 `Group(translate∘scale)`；
/// 3. 输出尺寸取缩放后的 `scene.width / height`（贴合内容，留 8px 边距）。
///
/// 语义对齐 liemermaid 的 `fit_options`（见 `liemermaid/src/builder/mod.rs`）。
pub fn to_fit_scene(elements: &[SceneNode], width: u32, height: u32, fit_max: (f64, f64)) -> Scene {
    const FIT_MARGIN: f64 = 8.0;

    let mut background = Color::WHITE;
    let nodes: Vec<SceneNode> = elements
        .iter()
        .filter(|n| {
            // 跳过整画布背景矩形，提取其颜色
            if n.z_index == Z_BACKGROUND
                && let Element::Rect { rect, style } = &n.element
                && rect.width() >= width as f64
                && rect.height() >= height as f64
            {
                if let Some(Fill::Solid(c)) = style.fill {
                    background = c;
                }
                return false;
            }
            true
        })
        .cloned()
        .collect();

    let mut scene = Scene::new(f64::from(width), f64::from(height));
    scene.background = background;
    for node in nodes {
        scene.push_node(node);
    }
    fit_scene(
        &mut scene,
        FitOptions::new()
            .with_margin(FIT_MARGIN)
            .with_max_width(fit_max.0)
            .with_max_height(fit_max.1),
    );
    scene
}
