//! 示例共享辅助模块。
//!
//! 提供一个 `save` 函数，统一把渲染结果输出到仓库根下的 `svg_output/`
//! 目录，并消除 24 个示例里重复的「`render_to_svg` + `println!`」样板。
//!
//! 通过 `RenderToSvg` trait 同时覆盖两套图表类型：
//! - `liecharts::api::Chart`（DataFrame 链式构建，示例主用）
//! - `liecharts::chart::Chart`（由 `ChartBuilder::build` 返回）

use std::path::PathBuf;

use liecharts::{api::Chart as ApiChart, chart::Chart as OptionChart, error::Result};

/// 示例默认画布尺寸。
#[allow(dead_code)]
pub const DEFAULT_W: u32 = 800;
#[allow(dead_code)]
pub const DEFAULT_H: u32 = 600;

/// 渲染到 SVG 文件所需的最小接口，由两套 `Chart` 类型分别实现。
pub trait RenderToSvg {
    fn render_to_svg(&self, path: &str) -> Result<()>;
}

impl RenderToSvg for ApiChart {
    fn render_to_svg(&self, path: &str) -> Result<()> {
        ApiChart::render_to_svg(self, path)
    }
}

impl RenderToSvg for OptionChart {
    fn render_to_svg(&self, path: &str) -> Result<()> {
        OptionChart::render_to_svg(self, path)
    }
}

/// 把 `chart` 渲染到 `svg_output/<name>`，并打印确认信息。
pub fn save<T: RenderToSvg>(chart: &T, name: &str) -> Result<()> {
    std::fs::create_dir_all("svg_output")?;
    let path = PathBuf::from("svg_output").join(name);
    chart.render_to_svg(path.to_str().unwrap())?;
    println!("✓ 已保存到 {}", path.display());
    Ok(())
}
