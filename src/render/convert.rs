//! 渲染入口：把 pipeline 产出的 `Vec<SceneNode>` 组装成 `lievisual::Scene`，
//! 交给 lievisual 的 `SvgRenderer` / `VelloPixmapRenderer` 渲染。

use lievisual::scene::{Scene, SceneNode};

/// 由 pipeline 产出的节点列表构建一个 lievisual::Scene。
pub fn to_scene(nodes: &[SceneNode], width: u32, height: u32) -> Scene {
    let mut scene = Scene::new(f64::from(width), f64::from(height));
    for node in nodes {
        scene.push_node(node.clone());
    }
    scene
}
