//! Multiple rendering backends (bitmap, SVG, etc.).
//!
//! Both backends delegate the actual drawing to the `lievisual` crate: they
//! convert the legacy liecharts IR (`Vec<SceneNode>`) into a
//! `lievisual::Scene` and hand it off to lievisual's `SvgRenderer` /
//! `VelloPixmapRenderer`.

mod convert;
mod pixmap;
mod svg;

pub use convert::to_fit_scene;
pub use convert::to_scene;
pub use pixmap::PixmapRenderer;
pub use svg::SvgRenderer;
