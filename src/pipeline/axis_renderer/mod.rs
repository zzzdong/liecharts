//! 坐标轴渲染模块
//!
//! 按坐标系拆分为三个独立渲染器：
//! - `CartesianAxisRenderer`：X/Y 笛卡尔坐标轴
//! - `RadarAxisRenderer`：雷达图坐标轴（同心多边形 + 径向线）
//! - `PolarAxisRenderer`：极坐标轴（同心圆 + 射线）
//!
//! 调度函数 `render_axes` 根据图表类型自动选择渲染器。

use crate::{
    option::{ChartOption, SeriesOption},
    pipeline::types::{ColorContext, ResolvedAxisRanges, SubplotSpec, TextMeasurer},
    visual::VisualElement,
};

mod cartesian;
mod polar;
mod radar;

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
    option: &ChartOption,
    axis_ranges: &ResolvedAxisRanges,
    colors: &ColorContext,
    text_measurer: &mut TextMeasurer,
) -> Vec<VisualElement> {
    let bounds = subplot.bounds;
    if bounds.width() <= 0.0 || bounds.height() <= 0.0 {
        return Vec::new();
    }

    // 检查当前 subplot 包含的图表类型
    let has_radar = option.series.iter().any(|s| matches!(s, SeriesOption::Radar(_)));
    let has_polar = option.series.iter().any(|s| {
        matches!(s, SeriesOption::PolarBar(_) | SeriesOption::PolarScatter(_))
    });
    let has_normal_chart = option.series.iter().any(|s| {
        !matches!(
            s,
            SeriesOption::Pie(_)
                | SeriesOption::Radar(_)
                | SeriesOption::Gauge(_)
                | SeriesOption::PolarBar(_)
                | SeriesOption::PolarScatter(_)
        )
    });
    let has_pie = option.series.iter().any(|s| matches!(s, SeriesOption::Pie(_)));
    let has_gauge = option.series.iter().any(|s| matches!(s, SeriesOption::Gauge(_)));

    // 纯饼图/仪表盘不需要坐标轴
    if (has_pie || has_gauge) && !has_radar && !has_polar && !has_normal_chart {
        return Vec::new();
    }

    let mut elements = Vec::new();

    // 雷达图坐标轴
    if has_radar {
        if let Some(ref radar_option) = option.radar {
            if let Some(ref indicators) = radar_option.indicator {
                elements.extend(RadarAxisRenderer::render(subplot, indicators, colors));
            }
        }
    }

    // 极坐标轴
    if has_polar {
        elements.extend(PolarAxisRenderer::render(subplot, colors, text_measurer));
    }

    // 笛卡尔坐标轴
    if has_normal_chart {
        elements.extend(CartesianAxisRenderer::render(
            subplot,
            option,
            axis_ranges,
            colors,
            text_measurer,
        ));
    }

    elements
}

/// 计算"美观"的刻度值序列
fn compute_nice_ticks(min: f64, max: f64, max_ticks: usize) -> Vec<f64> {
    if max <= min || max_ticks == 0 {
        return vec![min];
    }

    let range = max - min;
    let rough_step = range / max_ticks as f64;

    // 取整到"美观"的步长
    let magnitude = 10_f64.powf(rough_step.log10().floor());
    let residual = rough_step / magnitude;

    let nice_step = if residual < 1.5 {
        magnitude
    } else if residual < 3.5 {
        2.0 * magnitude
    } else if residual < 7.5 {
        5.0 * magnitude
    } else {
        10.0 * magnitude
    };

    // 生成刻度
    let start = (min / nice_step).floor() * nice_step;
    let end = (max / nice_step).ceil() * nice_step;
    let count = ((end - start) / nice_step).round() as usize;

    let mut ticks = Vec::with_capacity(count + 1);
    let mut v = start;
    for _ in 0..=count {
        if v >= min - nice_step * 1e-10 && v <= max + nice_step * 1e-10 {
            ticks.push(v);
        }
        v += nice_step;
    }

    if ticks.is_empty() {
        vec![min, max]
    } else {
        ticks
    }
}
