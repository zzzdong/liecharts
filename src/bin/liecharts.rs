use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use liecharts::prelude::*;

#[derive(ValueEnum, Clone, Debug)]
enum Format {
    Png,
    Svg,
}

/// Infer output format from file extension
fn infer_format_from_ext(path: &Path) -> Option<Format> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("png") => Some(Format::Png),
        Some("svg") => Some(Format::Svg),
        _ => None,
    }
}

/// Render ECharts JSON configuration files to PNG or SVG images
#[derive(Parser, Debug)]
#[command(name = "liecharts")]
#[command(about = "Render ECharts JSON config to PNG/SVG images")]
struct Args {
    /// Input ECharts JSON configuration file path (.json)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output image file path (format inferred from extension: .png, .svg)
    #[arg(short, long, value_name = "FILE")]
    output: PathBuf,

    /// Output format (overrides extension-based inference)
    #[arg(short, long, value_enum)]
    format: Option<Format>,

    /// Chart width in pixels
    #[arg(short = 'W', long, default_value_t = 800)]
    width: u32,

    /// Chart height in pixels
    #[arg(short = 'H', long, default_value_t = 600)]
    height: u32,

    /// Theme name (e.g. "dark")
    #[arg(short = 't', long, value_name = "NAME")]
    theme: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Read JSON config from file
    let json_content = std::fs::read_to_string(&args.input)
        .map_err(|e| anyhow::anyhow!("Failed to read input file {}: {}", args.input.display(), e))?;

    // Build chart from JSON
    let mut builder = ChartBuilder::from_option_json(&json_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse ECharts JSON config: {}", e))?;

    // Apply theme if specified
    if let Some(theme_name) = &args.theme {
        builder = match theme_name.to_ascii_lowercase().as_str() {
            "dark" => builder.with_theme(Theme::dark()),
            "echarts" | "default" | "light" => builder.with_theme(Theme::echarts()),
            other => anyhow::bail!("Unknown theme: {}. Supported: light (default), dark", other),
        };
    }

    // Infer output format
    let format = args
        .format
        .clone()
        .or_else(|| infer_format_from_ext(&args.output))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Cannot determine output format from extension '{}'. Use -f/--format or a supported extension (.png, .svg).",
                args.output.extension().and_then(|e| e.to_str()).unwrap_or("(none)")
            )
        })?;

    // Build and render chart
    let chart = builder.build(args.width, args.height)
        .map_err(|e| anyhow::anyhow!("Failed to build chart: {}", e))?;

    let output_str = args.output.to_string_lossy().to_string();

    match format {
        Format::Png => {
            chart
                .render_to_image(&output_str)
                .map_err(|e| anyhow::anyhow!("Failed to render PNG: {}", e))?;
            println!("PNG saved to: {}", args.output.display());
        }
        Format::Svg => {
            chart
                .render_to_svg(&output_str)
                .map_err(|e| anyhow::anyhow!("Failed to render SVG: {}", e))?;
            println!("SVG saved to: {}", args.output.display());
        }
    }

    Ok(())
}
