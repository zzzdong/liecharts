//! 坐标轴渲染模块
//!
//! 按坐标系拆分为三个独立渲染器：
//! - `CartesianAxisRenderer`：X/Y 笛卡尔坐标轴
//! - `RadarAxisRenderer`：雷达图坐标轴（同心多边形 + 径向线）
//! - `PolarAxisRenderer`：极坐标轴（同心圆 + 射线）
//!
//! 调度函数 `render_axes` 根据图表类型自动选择渲染器。

use crate::{
    SceneNode,
    pipeline::types::{
        AxisSpec, ChartType, ColorContext, ResolvedAxisRanges, SeriesSpec, SubplotSpec,
        TextMeasurer,
    },
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

/// 计算坐标轴刻度，返回 `(归一化位置 t∈[0,1], 标签文本)`。
///
/// 根据轴类型分派：
/// - `Value`：线性「美观」刻度
/// - `Log`：在 log 空间按整数幂生成刻度，标签为实际值（`10^v`）
/// - `Time`：在 log/线性空间生成刻度，标签为日期字符串
pub fn axis_ticks(
    axis_type: crate::pipeline::types::AxisType,
    min: f64,
    max: f64,
) -> (Vec<f64>, Vec<String>) {
    match axis_type {
        crate::pipeline::types::AxisType::Log => log_ticks(min, max),
        crate::pipeline::types::AxisType::Time => {
            // 时间戳（秒/毫秒）仍按线性刻度，标签格式化为日期
            let ticks = compute_nice_ticks(min, max, 5);
            let positions = normalize_ticks(&ticks, min, max);
            let labels: Vec<String> = ticks.iter().map(|&v| format_time_label(v)).collect();
            (positions, labels)
        }
        _ => {
            let ticks = compute_nice_ticks(min, max, 5);
            let positions = normalize_ticks(&ticks, min, max);
            let labels: Vec<String> = format_value_ticks(&ticks);
            (positions, labels)
        }
    }
}

/// 格式化数值刻度标签：按刻度步长决定小数位数。
///
/// 刻度步长小于 1 时必须保留小数，否则 `0.5` 会被格式化为 `"0"`、
/// `-0.5` 变成 `"-0"`（浮点数 `{:.0}` 四舍五入到偶数）。
fn format_value_ticks(ticks: &[f64]) -> Vec<String> {
    let step = if ticks.len() > 1 {
        (ticks[1] - ticks[0]).abs()
    } else {
        1.0
    };
    let decimals = if step >= 1.0 {
        0
    } else {
        ((-step.log10()).ceil() as i32).clamp(0, 6) as usize
    };
    ticks
        .iter()
        .map(|&v| {
            // 规避 "-0"：浮点四舍五入可能产生负零
            let v = if v.abs() < 1e-12 { 0.0 } else { v };
            format!("{v:.decimals$}")
        })
        .collect()
}

/// 将原始刻度值归一化为 `[0,1]` 区间内的位置，供坐标轴网格线/标签定位使用。
///
/// `(v - min) / (max - min)`。当 `max == min`（区间退化）时返回 0，避免除零。
fn normalize_ticks(ticks: &[f64], min: f64, max: f64) -> Vec<f64> {
    let range = max - min;
    if range.abs() < f64::EPSILON {
        return vec![0.0; ticks.len()];
    }
    ticks.iter().map(|&v| (v - min) / range).collect()
}

/// 生成 Log 轴刻度。min/max 为 log10 空间，返回归一化位置与对应标签。
fn log_ticks(log_min: f64, log_max: f64) -> (Vec<f64>, Vec<String>) {
    let range = log_max - log_min;
    if range <= 0.0 {
        return (vec![0.0], vec!["1".to_string()]);
    }
    let lo = log_min.ceil();
    let hi = log_max.floor();
    let mut positions = Vec::new();
    let mut labels = Vec::new();
    let mut v = lo;
    while v <= hi {
        let t = (v - log_min) / range;
        positions.push(t);
        let actual = 10.0_f64.powf(v);
        labels.push(format_log_label(actual));
        v += 1.0;
    }
    // 至少一个刻度
    if positions.is_empty() {
        positions.push(0.0);
        labels.push("1".to_string());
    }
    (positions, labels)
}

/// 格式化 Log 轴刻度标签：整数显示整数，否则保留 1 位小数
fn format_log_label(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{:.0}", v)
    } else {
        format!("{:.1}", v)
    }
}

/// 将时间戳（秒或毫秒）格式化为日期字符串。
fn format_time_label(ts: f64) -> String {
    // 尝试解析为日期：毫秒级时间戳通常 > 10^11，秒级 > 10^9
    let secs = if ts >= 1e11 { ts / 1000.0 } else { ts } as i64;
    if secs <= 0 {
        return format!("{:.0}", ts);
    }
    // 简易本地时区无关的 UTC 格式化（YYYY-MM-DD）
    let days = secs / 86_400;
    let (y, m, d) = days_to_ymd(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// 将天数（自 epoch）转换为 (年, 月, 日)
fn days_to_ymd(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468; // 修正儒略日
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
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
) -> Vec<SceneNode> {
    let bounds = subplot.bounds;
    if bounds.width() <= 0.0 || bounds.height() <= 0.0 {
        return Vec::new();
    }

    // 检查当前 subplot 包含的图表类型
    let has_radar = series.iter().any(|s| s.chart_type() == ChartType::Radar);
    let has_polar = series.iter().any(|s| {
        matches!(
            s.chart_type(),
            ChartType::PolarBar | ChartType::PolarScatter
        )
    });
    let has_normal_chart = series.iter().any(|s| {
        !matches!(
            s.chart_type(),
            ChartType::Pie
                | ChartType::Radar
                | ChartType::Gauge
                | ChartType::PolarBar
                | ChartType::PolarScatter
        )
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
