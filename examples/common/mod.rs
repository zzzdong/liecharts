//! 示例共享辅助模块。
//!
//! 提供 `save` 函数：一次调用同时输出 **SVG 与 PNG 两种格式**，分别落入仓库
//! 根下的 `svg_output/` 与 `png_output/` 目录，并消除 24 个示例里重复的
//! 「渲染 + 写文件 + `println!`」样板。
//!
//! `name` 传文件名主干即可（末尾的 `.svg` / `.png` 也会被剥去，兼容旧调用）：
//! `save(&chart, "line")` 产出 `svg_output/line.svg` + `png_output/line.png`。
//!
//! 通过 `RenderTo` trait 同时覆盖两套图表类型：
//! - `liecharts::api::Chart`（DataFrame 链式构建，示例主用）
//! - `liecharts::chart::Chart`（由 `ChartBuilder::build` 返回）

use std::path::{Path, PathBuf};

use liecharts::{api::Chart as ApiChart, chart::Chart as OptionChart, error::Result};

/// 示例默认画布尺寸。
#[allow(dead_code)]
pub const DEFAULT_W: u32 = 800;
#[allow(dead_code)]
pub const DEFAULT_H: u32 = 600;

/// 两种输出格式各自所需的最小接口，由两套 `Chart` 类型分别实现。
pub trait RenderTo {
    /// 渲染为 SVG 矢量图并写入 `path`。
    fn render_to_svg(&self, path: &str) -> Result<()>;

    /// 渲染为 PNG 位图并写入 `path`。
    fn render_to_png(&self, path: &str) -> Result<()>;
}

impl RenderTo for ApiChart {
    fn render_to_svg(&self, path: &str) -> Result<()> {
        ApiChart::render_to_svg(self, path)
    }

    fn render_to_png(&self, path: &str) -> Result<()> {
        ApiChart::render_to_image(self, path)
    }
}

impl RenderTo for OptionChart {
    fn render_to_svg(&self, path: &str) -> Result<()> {
        OptionChart::render_to_svg(self, path)
    }

    fn render_to_png(&self, path: &str) -> Result<()> {
        OptionChart::render_to_image(self, path)
    }
}

/// 把 `chart` 同时渲染为 SVG 与 PNG 两种格式，分别写入不同文件夹：
/// `svg_output/<name>.svg` 与 `png_output/<name>.png`。
///
/// `name` 取文件名主干（末尾的 `.svg` / `.png` 会被剥去）：
/// `save(&chart, "line")` → `svg_output/line.svg` + `png_output/line.png`。
pub fn save<T: RenderTo>(chart: &T, name: &str) -> Result<()> {
    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name);

    std::fs::create_dir_all("svg_output")?;
    let svg_path = PathBuf::from("svg_output").join(format!("{stem}.svg"));
    chart.render_to_svg(svg_path.to_str().unwrap())?;

    std::fs::create_dir_all("png_output")?;
    let png_path = PathBuf::from("png_output").join(format!("{stem}.png"));
    chart.render_to_png(png_path.to_str().unwrap())?;

    println!("✓ 已保存到 {} / {}", svg_path.display(), png_path.display());
    Ok(())
}
