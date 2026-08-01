//! 坐标轴渲染模块
//!
//! 按坐标系拆分为三个独立渲染器：
//! - `CartesianAxisRenderer`：X/Y 笛卡尔坐标轴
//! - `RadarAxisRenderer`：雷达图坐标轴（同心多边形 + 径向线）
//! - `PolarAxisRenderer`：极坐标轴（同心圆 + 射线）
//!
//! 调度函数 `render_axes` 根据图表类型自动选择渲染器。

use crate::{
    pipeline::types::{AxisSpec, ChartType, ColorContext, ResolvedAxisRanges, SeriesSpec, SubplotSpec, TextMeasurer},
    visual::VisualElement,
};

mod cartesian;
mod polar;
mod radar;

/// 计算"美观"的刻度值，用于坐标轴网格线和标签
///
/// 在 `[min, max]` 范围内生成约 `count` 个刻度，每个刻度是"整洁"的数值
///（如 0, 10, 20 而非 0, 7, 14）。
///
/// 返回的刻度严格落在 `[min, max]` 区间内，避免超出画布。
fn compute_nice_ticks(min: f64, max: f64, count: usize) -> Vec<f64> {
    if (max - min).abs() < f64::EPSILON {
        return vec![min];
    }
    let range = nice_number(max - min, false);
    let tick_spacing = nice_number(range / count as f64, true);
    let mut ticks = Vec::new();
    // 从最接近 min 的 tick 开始，向下取整到 spacing 倍数
    let mut v = (min / tick_spacing).floor() * tick_spacing;
    while v <= max {
        if v >= min {
            ticks.push(v);
        }
        v += tick_spacing;
    }
    ticks
}

/// 计算"整洁"数值
fn nice_number(range: f64, round: bool) -> f64 {
    let exponent = range.abs().log10().floor();
    let fraction = range / 10.0_f64.powf(exponent);
    let nice_fraction = if round {
        match fraction {
            f if f <= 1.5 => 1.0,
            f if f <= 3.0 => 2.0,
            f if f <= 7.0 => 5.0,
            _ => 10.0,
        }
    } else {
        match fraction {
            f if f <= 1.0 => 1.0,
            f if f <= 2.0 => 2.0,
            f if f <= 5.0 => 5.0,
            _ => 10.0,
        }
    };
    nice_fraction * 10.0_f64.powf(exponent)
}

pub use cartesian::CartesianAxisRenderer;
pub use polar::PolarAxisRenderer;
pub use radar::RadarAxisRenderer;

/// 为指定 subplot 生成所有坐标轴和网格线视觉元素
///
/// 根据 subplot 中包含的图表类型，自动调度对应的坐标轴渲染器：
/// - 普通图表（折线、柱状、散点等）→ 笛卡尔坐标轴
/// - 雷达图 → 雷达坐标轴
/// - 极坐标图 → 极坐标轴
pub fn render_axes(
    subplot: &SubplotSpec,
    series: &[SeriesSpec],
    x_axes: &[AxisSpec],
    y_axes: &[AxisSpec],
    axis_ranges: &ResolvedAxisRanges,
    colors: &ColorContext,
    text_measurer: &mut TextMeasurer,
) -> Vec<VisualElement> {
    let bounds = subplot.bounds;
    if bounds.width() <= 0.0 || bounds.height() <= 0.0 {
        return Vec::new();
    }

    // 检查当前 subplot 包含的图表类型
    let has_radar = series.iter().any(|s| s.chart_type() == ChartType::Radar);
    let has_polar = series.iter().any(|s| matches!(s.chart_type(), ChartType::PolarBar | ChartType::PolarScatter));
    let has_normal_chart = series.iter().any(|s| {
        !matches!(s.chart_type(), ChartType::Pie | ChartType::Radar | ChartType::Gauge | ChartType::PolarBar | ChartType::PolarScatter)
    });
    let has_pie = series.iter().any(|s| s.chart_type() == ChartType::Pie);
    let has_gauge = series.iter().any(|s| s.chart_type() == ChartType::Gauge);

    // 纯饼图/仪表盘不需要坐标轴
    if (has_pie || has_gauge) && !has_radar && !has_polar && !has_normal_chart {
        return Vec::new();
    }

    let mut elements = Vec::new();

    // 雷达图坐标轴
    if has_radar {
        // 雷达指示器从雷达系列的 config 中获取
        // 框架内目前不渲染雷达图专用网格，但保留调度入口
        elements.extend(RadarAxisRenderer::render(subplot, &[], colors));
    }

    // 极坐标轴
    if has_polar {
        elements.extend(PolarAxisRenderer::render(subplot, colors, text_measurer));
    }

    // 标准笛卡尔坐标轴
    if has_normal_chart {
        elements.extend(CartesianAxisRenderer::render(
            subplot,
            x_axes,
            y_axes,
            axis_ranges,
            colors,
            text_measurer,
        ));
    }

    elements
}