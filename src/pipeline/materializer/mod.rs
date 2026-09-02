//! Materializer 阶段：将 SeriesSpec 转换为 TypedSeries
//!
//! 职责：
//! - 从 DataFrame 提取数据
//! - 解析颜色（从 ColorContext）
//! - 将数据点分配到轴槽位 → 像素坐标
//! - Bar 系列先收集，最后做分组分析

use lievisual::Color;
use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        typed_series::{GroupedBarRow, LineSeries, TypedSeries},
        types::{ChartSpec, ChartType, ColorContext, ResolvedAxisRanges, SeriesSpec},
    },
};

pub mod bar;
pub mod boxplot;
pub mod bubble;
pub mod candlestick;
pub mod gauge;
pub mod heatmap;
pub mod line;
pub mod pie;
pub mod polar_bar;
pub mod polar_scatter;
pub mod radar;
pub mod scatter;
pub mod table;

pub use bar::BarMaterializer;
pub use boxplot::BoxplotMaterializer;
pub use bubble::BubbleMaterializer;
pub use candlestick::CandlestickMaterializer;
pub use gauge::GaugeMaterializer;
pub use heatmap::HeatmapMaterializer;
pub use line::LineMaterializer;
pub use pie::PieMaterializer;
pub use polar_bar::PolarBarMaterializer;
pub use polar_scatter::PolarScatterMaterializer;
pub use radar::RadarMaterializer;
pub use scatter::ScatterMaterializer;
pub use table::TableMaterializer;

/// 每种图表类型实现此 trait，将 SeriesSpec 转换为对应的 TypedSeries
pub trait SeriesMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,                     // 子图像素边界
        axis_ranges: &ResolvedAxisRanges, // 轴范围（用于槽位分配）
        color: Color,
        colors: &ColorContext,
    ) -> Result<TypedSeries>;
}

/// Materializer 函数类型（用于动态分发）
pub type MaterializerFn =
    fn(&SeriesSpec, Rect, &ResolvedAxisRanges, Color, &ColorContext) -> Result<TypedSeries>;

/// 创建对应类型的 Materializer
pub fn create_materializer(chart_type: ChartType) -> MaterializerFn {
    match chart_type {
        ChartType::Line => line_materializer_fn,
        ChartType::Bar => bar_materializer_fn,
        ChartType::Scatter => scatter_materializer_fn,
        ChartType::Pie => pie_materializer_fn,
        ChartType::Bubble => bubble_materializer_fn,
        ChartType::Candlestick => candlestick_materializer_fn,
        ChartType::Boxplot => boxplot_materializer_fn,
        ChartType::Heatmap => heatmap_materializer_fn,
        ChartType::Radar => radar_materializer_fn,
        ChartType::PolarBar => polar_bar_materializer_fn,
        ChartType::PolarScatter => polar_scatter_materializer_fn,
        ChartType::Gauge => gauge_materializer_fn,
        ChartType::Table => table_materializer_fn,
    }
}

fn line_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    LineMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn bar_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    BarMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn scatter_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    ScatterMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn pie_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    PieMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn bubble_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    BubbleMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn candlestick_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    CandlestickMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn boxplot_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    BoxplotMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn heatmap_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    HeatmapMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn radar_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    RadarMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn polar_bar_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    PolarBarMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn polar_scatter_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    PolarScatterMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn gauge_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    GaugeMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn table_materializer_fn(
    spec: &SeriesSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    color: Color,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    TableMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

/// 辅助函数：将数据值映射到像素 X 坐标。
///
/// 支持 `inverse`（像素方向翻转，所有轴类型一致）：
/// - X 轴：`inverse` 时 min 在右端、max 在左端 → `x = x1 − t·W`；
/// - Category 轴：数据坐标仍按类目升序传入（见 `ResolvedAxisRange::category_value`），
///   顺序反转同样在这里完成，保证折线/散点与柱状图族行为一致。
pub fn map_x_to_pixel(
    x: f64,
    x_range: &crate::pipeline::types::ResolvedAxisRange,
    bounds: Rect,
) -> f64 {
    let range = x_range.max - x_range.min;
    if range <= 0.0 {
        // 单点退化的轴：inverse 时锚到末端，否则锚到起始端
        return if x_range.inverse {
            bounds.x1
        } else {
            bounds.x0
        };
    }
    // Log 轴：对数据值做 log10 后再线性映射
    let t = if x_range.axis_type == crate::pipeline::types::AxisType::Log {
        (x.max(f64::MIN_POSITIVE).log10() - x_range.min) / range
    } else {
        (x - x_range.min) / range
    };
    if x_range.inverse {
        bounds.x1 - t * bounds.width()
    } else {
        bounds.x0 + t * bounds.width()
    }
}

/// 辅助函数：将数据值映射到像素 Y 坐标。
///
/// 支持 `inverse`（像素方向翻转，所有轴类型一致）：
/// - Y 轴：`inverse` 时 min 在顶部、max 在底部 → `y = y0 + t·H`；
/// - Category 轴：数据坐标仍按类目升序传入（见 `ResolvedAxisRange::category_value`）。
pub fn map_y_to_pixel(
    y: f64,
    y_range: &crate::pipeline::types::ResolvedAxisRange,
    bounds: Rect,
) -> f64 {
    let range = y_range.max - y_range.min;
    if range <= 0.0 {
        return if y_range.inverse {
            bounds.y0
        } else {
            bounds.y1
        };
    }
    // Log 轴：对数据值做 log10 后再线性映射（轴范围本身已是 log 空间）
    let t = if y_range.axis_type == crate::pipeline::types::AxisType::Log {
        (y.max(f64::MIN_POSITIVE).log10() - y_range.min) / range
    } else {
        (y - y_range.min) / range
    };
    if y_range.inverse {
        bounds.y0 + t * bounds.height()
    } else {
        bounds.y1 - t * bounds.height()
    }
}

/// 根据数据值计算标注线（average/min/max）的像素位置。
///
/// 返回每个标注线的 Y 像素坐标与标签文本。
pub fn compute_mark_lines(
    mark_line_specs: &[crate::pipeline::types::MarkLineSpec],
    values: &[f64],
    y_range: &crate::pipeline::types::ResolvedAxisRange,
    bounds: Rect,
) -> Vec<crate::pipeline::typed_series::MarkLineRender> {
    use crate::pipeline::{typed_series::MarkLineRender, types::MarkLineType};

    if mark_line_specs.is_empty() || values.is_empty() {
        return Vec::new();
    }

    // 过滤非有限值（NaN/Inf），避免统计与像素坐标被污染
    let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        return Vec::new();
    }

    // 计算统计数据
    let sum: f64 = finite.iter().sum();
    let avg = sum / finite.len() as f64;
    let min = finite.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = finite.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut result = Vec::new();
    for spec in mark_line_specs {
        let value = match spec.data_type {
            MarkLineType::Average => avg,
            MarkLineType::Min => min,
            MarkLineType::Max => max,
        };
        let text = format_value(value);
        let name = spec.name.clone().unwrap_or_else(|| match spec.data_type {
            MarkLineType::Average => "平均值".to_string(),
            MarkLineType::Min => "最小值".to_string(),
            MarkLineType::Max => "最大值".to_string(),
        });
        result.push(MarkLineRender {
            y: map_y_to_pixel(value, y_range, bounds),
            label: format!("{}: {}", name, text),
            color: Color::rgb(220, 60, 60),
        });
    }
    result
}

/// 格式化数值（整数值不带小数，否则保留 1 位）
fn format_value(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{:.0}", v)
    } else {
        format!("{:.1}", v)
    }
}

/// Materialize 所有系列
///
/// 对 Bar 系列做两遍处理：
/// 1. 第一遍：收集所有 Bar 类型的 SeriesSpec，暂不生成 TypedSeries
/// 2. 分组分析：识别 SideBySide / Stacked 组
/// 3. 第二遍：
///    - Single Bar → TypedSeries::Bar
///    - SideBySide / Stacked 组 → TypedSeries::GroupedBar
///
/// 对堆叠 Line 系列（stacked area）：
/// 1. 拦截所有 stack.is_some() && Line 的 series
/// 2. 按 stack 名称分组
/// 3. 计算累积 Y 值，生成堆叠的 LineSeries（含 baseline_points）
pub fn materialize_all(
    series_indices: &[usize], // 当前 subplot 内的 series 索引
    spec: &ChartSpec,
    bounds: Rect,                     // 子图像素边界
    axis_ranges: &ResolvedAxisRanges, // 轴范围
    colors: &ColorContext,
) -> Result<Vec<TypedSeries>> {
    let mut result: Vec<(usize, TypedSeries)> = Vec::new();
    let mut bar_specs: Vec<(usize, &SeriesSpec)> = Vec::new();
    let mut stacked_line_specs: Vec<(usize, &SeriesSpec)> = Vec::new();

    for &global_idx in series_indices {
        let s = &spec.series[global_idx];
        match s.chart_type() {
            ChartType::Bar => {
                bar_specs.push((global_idx, s));
            }
            // 堆叠 Line 系列（面积图）单独处理
            ChartType::Line if s.stack.is_some() => {
                stacked_line_specs.push((global_idx, s));
            }
            _ => {
                let color = colors.get_series_color(global_idx);
                let materializer = create_materializer(s.chart_type());
                let typed = materializer(s, bounds, axis_ranges, color, colors)?;
                result.push((global_idx, typed));
            }
        }
    }

    // 处理堆叠 Line 系列（栈面积图）
    if !stacked_line_specs.is_empty() {
        let typed_series = materialize_stacked_line_groups(
            &stacked_line_specs,
            spec,
            bounds,
            axis_ranges,
            colors,
        )?;
        for (idx, typed) in typed_series {
            result.push((idx, typed));
        }
    }

    // 对 Bar 系列做分组分析
    if !bar_specs.is_empty() {
        let bar_plans = analyze_bar_groups(&bar_specs, &spec.series);
        for plan in bar_plans {
            let typed = materialize_bar_group(&plan, spec, axis_ranges, bounds, colors)?;
            result.push((plan.first_index, typed));
        }
    }

    // 按原始索引排序，保持声明顺序
    result.sort_by_key(|(idx, _)| *idx);
    Ok(result.into_iter().map(|(_, s)| s).collect())
}

/// Bar 分组计划
pub struct BarGroupPlan {
    pub first_index: usize,
    pub series_indices: Vec<usize>,
    pub group_type: BarGroupType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarGroupType {
    Single,
    SideBySide,
    Stacked,
}

/// 分析 Bar 系列的分组情况
fn analyze_bar_groups(
    bar_specs: &[(usize, &SeriesSpec)],
    _all_series: &[SeriesSpec],
) -> Vec<BarGroupPlan> {
    if bar_specs.is_empty() {
        return Vec::new();
    }

    // 如果只有一个 Bar 系列，直接返回 Single
    if bar_specs.len() == 1 {
        return vec![BarGroupPlan {
            first_index: bar_specs[0].0,
            series_indices: vec![bar_specs[0].0],
            group_type: BarGroupType::Single,
        }];
    }

    // 检查是否有 stack 字段
    let has_stack = bar_specs.iter().any(|(_, s)| s.stack.is_some());

    if has_stack {
        // 按 stack 名称分组
        use std::collections::HashMap;
        let mut stack_groups: HashMap<Option<String>, Vec<usize>> = HashMap::new();

        for (idx, s) in bar_specs {
            stack_groups.entry(s.stack.clone()).or_default().push(*idx);
        }

        stack_groups
            .into_values()
            .map(|indices| {
                let first = *indices.first().unwrap();
                let group_type = if indices.len() == 1 {
                    BarGroupType::Single
                } else {
                    BarGroupType::Stacked
                };
                BarGroupPlan {
                    first_index: first,
                    series_indices: indices,
                    group_type,
                }
            })
            .collect()
    } else {
        // 没有 stack，检查是否在同一 grid 且共享 X 轴
        // 如果是，则为 SideBySide
        let first_grid = bar_specs[0].1.grid_index;
        let first_x_axis = bar_specs[0].1.x_axis_index;

        let same_grid_and_x = bar_specs
            .iter()
            .all(|(_, s)| s.grid_index == first_grid && s.x_axis_index == first_x_axis);

        if same_grid_and_x {
            // 并排的 Bar
            vec![BarGroupPlan {
                first_index: bar_specs[0].0,
                series_indices: bar_specs.iter().map(|(idx, _)| *idx).collect(),
                group_type: BarGroupType::SideBySide,
            }]
        } else {
            // 不同 grid 或不同 X 轴，各自独立
            bar_specs
                .iter()
                .map(|(idx, _)| BarGroupPlan {
                    first_index: *idx,
                    series_indices: vec![*idx],
                    group_type: BarGroupType::Single,
                })
                .collect()
        }
    }
}

/// Materialize Bar 分组
fn materialize_bar_group(
    plan: &BarGroupPlan,
    spec: &ChartSpec,
    axis_ranges: &ResolvedAxisRanges,
    bounds: Rect,
    colors: &ColorContext,
) -> Result<TypedSeries> {
    use crate::pipeline::typed_series::{
        BarGroupType as TypedBarGroupType, BarSubSeries, GroupedBarSeries,
    };

    match plan.group_type {
        BarGroupType::Single => {
            let series_idx = plan.series_indices[0];
            let series_spec = &spec.series[series_idx];
            let color = colors.get_series_color(series_idx);
            BarMaterializer::materialize(series_spec, bounds, axis_ranges, color, colors)
        }
        BarGroupType::SideBySide => {
            // 并排柱状图
            let sub_series: Vec<BarSubSeries> = plan
                .series_indices
                .iter()
                .map(|&idx| {
                    let s = &spec.series[idx];
                    BarSubSeries {
                        name: s.name.clone(),
                        color: colors.get_series_color(idx),
                    }
                })
                .collect();

            let rows = materialize_side_by_side_bars(
                &plan.series_indices,
                spec,
                axis_ranges,
                bounds,
                colors,
            )?;

            let label = first_bar_label_config(spec, &plan.series_indices);

            Ok(TypedSeries::GroupedBar(GroupedBarSeries {
                sub_series,
                group_type: TypedBarGroupType::SideBySide,
                rows,
                label,
            }))
        }
        BarGroupType::Stacked => {
            // 堆叠柱状图
            let sub_series: Vec<BarSubSeries> = plan
                .series_indices
                .iter()
                .map(|&idx| {
                    let s = &spec.series[idx];
                    BarSubSeries {
                        name: s.name.clone(),
                        color: colors.get_series_color(idx),
                    }
                })
                .collect();

            let rows =
                materialize_stacked_bars(&plan.series_indices, spec, axis_ranges, bounds, colors)?;

            let label = first_bar_label_config(spec, &plan.series_indices);

            Ok(TypedSeries::GroupedBar(GroupedBarSeries {
                sub_series,
                group_type: TypedBarGroupType::Stacked,
                rows,
                label,
            }))
        }
    }
}

/// 将 spec 层的 `ValueLabelPos` 映射为 typed 层的 `SeriesLabelPosition`。
///
/// `None` 时取默认值 `Top`（值端外侧），与 ECharts `label.position` 默认一致。
pub(crate) fn map_label_position(
    pos: Option<crate::pipeline::types::ValueLabelPos>,
) -> crate::pipeline::typed_series::SeriesLabelPosition {
    use crate::pipeline::typed_series::SeriesLabelPosition as Out;
    use crate::pipeline::types::ValueLabelPos as In;

    match pos.unwrap_or_default() {
        In::Top => Out::Top,
        In::Bottom => Out::Bottom,
        In::Inside => Out::Inside,
    }
}

/// 构造折线系列的值标签配置。未开启 label 时返回 None，Builder 不渲染标签。
pub(crate) fn line_label_config(
    cfg: &crate::pipeline::types::LineConfig,
) -> Option<crate::pipeline::typed_series::SeriesLabelConfig> {
    if !cfg.label_show {
        return None;
    }
    Some(crate::pipeline::typed_series::SeriesLabelConfig {
        show: true,
        position: map_label_position(cfg.label_position),
        color: cfg.label_color,
        font_size: cfg.label_font_size,
        formatter: cfg.label_formatter.clone(),
    })
}

/// 构造柱状系列的值标签配置。未开启 label 时返回 None，Builder 不渲染标签。
pub(crate) fn bar_label_config(
    cfg: &crate::pipeline::types::BarConfig,
) -> Option<crate::pipeline::typed_series::SeriesLabelConfig> {
    if !cfg.label_show {
        return None;
    }
    Some(crate::pipeline::typed_series::SeriesLabelConfig {
        show: true,
        position: map_label_position(cfg.label_position),
        color: cfg.label_color,
        font_size: cfg.label_font_size,
        formatter: cfg.label_formatter.clone(),
    })
}

/// 从分组柱状图中提取第一个 Bar 系列的标签配置。
///
/// 分组柱状图的每个子系列共享同一个标签配置，取第一个系列即可。
/// 未开启 label 时返回 None，Builder 不会渲染数据标签。
fn first_bar_label_config(
    spec: &ChartSpec,
    series_indices: &[usize],
) -> Option<crate::pipeline::typed_series::SeriesLabelConfig> {
    use crate::pipeline::types::SeriesConfig;

    series_indices
        .first()
        .and_then(|&idx| match spec.series.get(idx).map(|s| &s.config) {
            Some(SeriesConfig::Bar(c)) => Some(c),
            _ => None,
        })
        .and_then(bar_label_config)
}

/// Materialize 并排柱状图
fn materialize_side_by_side_bars(
    series_indices: &[usize],
    spec: &ChartSpec,
    axis_ranges: &ResolvedAxisRanges,
    bounds: Rect,
    colors: &ColorContext,
) -> Result<Vec<GroupedBarRow>> {
    use vello_cpu::kurbo::Rect as KurboRect;

    use crate::pipeline::{typed_series::GroupedBarRow, types::AxisType};

    let first_series = &spec.series[series_indices[0]];
    let x_range = axis_ranges
        .get_x_range(first_series.x_axis_index)
        .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("X axis not found".into()))?;
    let y_range = axis_ranges
        .get_y_range(first_series.y_axis_index)
        .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("Y axis not found".into()))?;

    let is_horizontal = matches!(y_range.axis_type, AxisType::Category);
    let series_count = series_indices.len();
    let bar_width_ratio = 0.6; // 默认柱宽比例

    // 类目总数与留白风格直接取自解析结果（与坐标轴刻度同源），避免各系列按自身
    // 行数反推 boundary_gap 而算出不同的 n 导致系列间错位。
    let cat_range = if is_horizontal { y_range } else { x_range };
    let cat_count = cat_range.category_count().max(1);
    let (group_dim, bar_dim): (f64, f64) = if is_horizontal {
        let group_dim = bounds.height() / cat_count as f64 * bar_width_ratio;
        (group_dim, group_dim / series_count as f64)
    } else {
        let group_dim = bounds.width() / cat_count as f64 * bar_width_ratio;
        (group_dim, group_dim / series_count as f64)
    };

    let mut rows = Vec::new();

    // 计算基线位置：安全处理轴范围不包含 0 的情况
    let (baseline_x, baseline_y) = if is_horizontal {
        let bx = if x_range.min <= 0.0 && x_range.max >= 0.0 {
            map_x_to_pixel(0.0, x_range, bounds)
        } else if x_range.min > 0.0 {
            bounds.x0
        } else {
            bounds.x1
        };
        (bx, 0.0)
    } else {
        let by = if y_range.min <= 0.0 && y_range.max >= 0.0 {
            map_y_to_pixel(0.0, y_range, bounds)
        } else if y_range.min > 0.0 {
            bounds.y1
        } else {
            bounds.y0
        };
        (0.0, by)
    };

    for (sub_idx, &series_idx) in series_indices.iter().enumerate() {
        let series = &spec.series[series_idx];
        let color = colors.get_series_color(series_idx);

        let y_col = series.config.y_col_name();
        let x_col = series.config.x_col_name();

        let (value_col, category_col) = if is_horizontal {
            (x_col, y_col)
        } else {
            (y_col, x_col)
        };

        if let (Some(value_series), Some(cat_series)) = (
            series.data.get_column(value_col),
            series.data.get_column(category_col),
        ) {
            for i in 0..series.data.row_count() {
                let value = value_series.as_f64(i).unwrap_or(0.0);
                let category = cat_series.as_string(i).unwrap_or_default();
                let cat_idx = if is_horizontal {
                    cat_count.saturating_sub(1 + i)
                } else {
                    i.min(cat_count - 1)
                };

                if is_horizontal {
                    // 横向柱状图：Y=Category, X=Value
                    let py = map_y_to_pixel(y_range.category_value(cat_idx), y_range, bounds);
                    // 组内偏移
                    let bar_y = py - group_dim / 2.0 + sub_idx as f64 * bar_dim;
                    let px = map_x_to_pixel(value, x_range, bounds);

                    let rect = KurboRect::new(
                        px.min(baseline_x),
                        bar_y,
                        px.max(baseline_x),
                        bar_y + bar_dim,
                    );

                    rows.push(GroupedBarRow {
                        bar_rect: rect,
                        sub_series_idx: sub_idx,
                        color,
                        category,
                        value,
                    });
                } else {
                    // 纵向柱状图：X=Category, Y=Value
                    let group_x = map_x_to_pixel(x_range.category_value(cat_idx), x_range, bounds);
                    let bar_x = group_x - group_dim / 2.0 + sub_idx as f64 * bar_dim;

                    let py = map_y_to_pixel(value, y_range, bounds);

                    let rect = KurboRect::new(
                        bar_x,
                        py.min(baseline_y),
                        bar_x + bar_dim,
                        py.max(baseline_y),
                    );

                    rows.push(GroupedBarRow {
                        bar_rect: rect,
                        sub_series_idx: sub_idx,
                        color,
                        category,
                        value,
                    });
                }
            }
        }
    }

    Ok(rows)
}

/// Materialize 堆叠柱状图
fn materialize_stacked_bars(
    series_indices: &[usize],
    spec: &ChartSpec,
    axis_ranges: &ResolvedAxisRanges,
    bounds: Rect,
    colors: &ColorContext,
) -> Result<Vec<GroupedBarRow>> {
    use vello_cpu::kurbo::Rect as KurboRect;

    use crate::pipeline::{typed_series::GroupedBarRow, types::AxisType};

    let first_series = &spec.series[series_indices[0]];
    let x_range = axis_ranges
        .get_x_range(first_series.x_axis_index)
        .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("X axis not found".into()))?;
    let y_range = axis_ranges
        .get_y_range(first_series.y_axis_index)
        .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("Y axis not found".into()))?;

    let is_horizontal = matches!(y_range.axis_type, AxisType::Category);
    let bar_width_ratio = 0.6;

    // 类目总数与留白风格直接取自解析结果（与坐标轴刻度同源）
    let cat_range = if is_horizontal { y_range } else { x_range };
    let cat_count = cat_range.category_count().max(1);
    let bar_dim = if is_horizontal {
        bounds.height() / cat_count as f64 * bar_width_ratio
    } else {
        bounds.width() / cat_count as f64 * bar_width_ratio
    };

    // 收集每个类别的堆叠值
    let mut category_stacks: Vec<Vec<(usize, f64, f64)>> = vec![Vec::new(); cat_count];
    // (sub_series_idx, value, accumulated_base)

    for (sub_idx, &series_idx) in series_indices.iter().enumerate() {
        let series = &spec.series[series_idx];
        let y_col = series.config.y_col_name();
        let x_col = series.config.x_col_name();
        let value_col = if is_horizontal { x_col } else { y_col };

        if let Some(value_series) = series.data.get_column(value_col) {
            for i in 0..series.data.row_count() {
                let cat_idx = if is_horizontal {
                    cat_count.saturating_sub(1 + i)
                } else {
                    i.min(cat_count - 1)
                };
                let value = value_series.as_f64(i).unwrap_or(0.0);
                category_stacks[cat_idx].push((sub_idx, value, 0.0));
            }
        }
    }

    // 计算累积值。正负值分别从基线向两个方向堆叠（ECharts 语义）：
    // 正值 base 从 0 向上累计；负值 base 从 0 向下累计。避免负段被误叠在正段上方。
    for stack in &mut category_stacks {
        let mut pos_acc = 0.0;
        let mut neg_acc = 0.0;
        for item in stack.iter_mut() {
            if item.1 >= 0.0 {
                item.2 = pos_acc;
                pos_acc += item.1;
            } else {
                item.2 = neg_acc;
                neg_acc += item.1;
            }
        }
    }

    let mut rows = Vec::new();

    for (cat_idx, stack) in category_stacks.iter().enumerate() {
        if is_horizontal {
            // 横向堆叠：Y=Category, X=Value
            let py = map_y_to_pixel(y_range.category_value(cat_idx), y_range, bounds);
            let bar_y = py - bar_dim / 2.0;

            for &(sub_idx, value, base) in stack {
                let series_idx = series_indices[sub_idx];
                let color = colors.get_series_color(series_idx);
                let y_col = spec.series[series_idx].config.y_col_name();
                let x_col = spec.series[series_idx].config.x_col_name();
                let category_col = if is_horizontal { y_col } else { x_col };

                let data_row_idx = if is_horizontal {
                    cat_count - 1 - cat_idx
                } else {
                    cat_idx
                };
                let category = spec.series[series_idx]
                    .data
                    .get_column(category_col)
                    .and_then(|s| s.as_string(data_row_idx))
                    .unwrap_or_default();

                // 值轴两端分别映射后取矩形，并裁剪到绘图区内（越界的堆叠总值不外溢）
                let p_a = map_x_to_pixel(base, x_range, bounds);
                let p_b = map_x_to_pixel(base + value, x_range, bounds);
                let px_left = p_a.min(p_b).max(bounds.x0);
                let px_right = p_a.max(p_b).min(bounds.x1);
                if px_right <= px_left {
                    continue;
                }
                let rect = KurboRect::new(px_left, bar_y, px_right, bar_y + bar_dim);

                rows.push(GroupedBarRow {
                    bar_rect: rect,
                    sub_series_idx: sub_idx,
                    color,
                    category,
                    value,
                });
            }
        } else {
            // 纵向堆叠：X=Category, Y=Value
            let group_x = map_x_to_pixel(x_range.category_value(cat_idx), x_range, bounds);
            let bar_x = group_x - bar_dim / 2.0;

            for &(sub_idx, value, base) in stack {
                let series_idx = series_indices[sub_idx];
                let color = colors.get_series_color(series_idx);
                let y_col = spec.series[series_idx].config.y_col_name();
                let x_col = spec.series[series_idx].config.x_col_name();
                let category_col = if is_horizontal { y_col } else { x_col };

                let category = spec.series[series_idx]
                    .data
                    .get_column(category_col)
                    .and_then(|s| s.as_string(cat_idx))
                    .unwrap_or_default();

                // 值轴两端分别映射后取矩形，并裁剪到绘图区内（越界的堆叠总值不外溢）
                let p_a = map_y_to_pixel(base, y_range, bounds);
                let p_b = map_y_to_pixel(base + value, y_range, bounds);
                let py_top = p_a.min(p_b).max(bounds.y0);
                let py_bot = p_a.max(p_b).min(bounds.y1);
                if py_bot <= py_top {
                    continue;
                }
                let rect = KurboRect::new(bar_x, py_top, bar_x + bar_dim, py_bot);

                rows.push(GroupedBarRow {
                    bar_rect: rect,
                    sub_series_idx: sub_idx,
                    color,
                    category,
                    value,
                });
            }
        }
    }

    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════
// 堆叠 Line 系列（堆叠面积图）处理
// ═══════════════════════════════════════════════════════════════════

/// 物化堆叠 Line 组，生成累积 Y 值的堆叠 LineSeries
fn materialize_stacked_line_groups(
    stacked_line_specs: &[(usize, &SeriesSpec)],
    spec: &ChartSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    colors: &ColorContext,
) -> Result<Vec<(usize, TypedSeries)>> {
    use std::collections::HashMap;

    // 按 stack 名称分组
    let mut groups: HashMap<Option<String>, Vec<(usize, &SeriesSpec)>> = HashMap::new();
    for &(idx, s) in stacked_line_specs {
        groups.entry(s.stack.clone()).or_default().push((idx, s));
    }

    let mut all_results: Vec<(usize, TypedSeries)> = Vec::new();

    for (_, group) in groups {
        let sub_results =
            materialize_one_stacked_line_group(&group, spec, bounds, axis_ranges, colors)?;
        all_results.extend(sub_results);
    }

    Ok(all_results)
}

/// 物化一个堆叠 Line 组（同一 stack 名称）
fn materialize_one_stacked_line_group(
    group: &[(usize, &SeriesSpec)],
    _spec: &ChartSpec,
    bounds: Rect,
    axis_ranges: &ResolvedAxisRanges,
    colors: &ColorContext,
) -> Result<Vec<(usize, TypedSeries)>> {
    if group.is_empty() {
        return Ok(Vec::new());
    }

    let first_s = group[0].1;
    let x_range = axis_ranges
        .get_x_range(first_s.x_axis_index)
        .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("X axis not found".into()))?;
    let y_range = axis_ranges
        .get_y_range(first_s.y_axis_index)
        .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("Y axis not found".into()))?;

    let row_count = group
        .iter()
        .map(|(_, s)| s.data.row_count())
        .max()
        .unwrap_or(0);
    if row_count == 0 {
        return Ok(Vec::new());
    }

    // 累积值：每行一个浮点数，初始为 0
    let mut running_total: Vec<f64> = vec![0.0; row_count];
    let mut prev_points: Option<Vec<Point>> = None;

    let mut results: Vec<(usize, TypedSeries)> = Vec::with_capacity(group.len());

    for &(series_idx, s) in group {
        let color = colors.get_series_color(series_idx);
        let cfg = match &s.config {
            crate::pipeline::types::SeriesConfig::Line(c) => c,
            _ => continue,
        };

        // 获取 X 值
        let x_vals = s
            .data
            .get_column(&cfg.x_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.x_col.clone()))?;
        let is_numeric_x = x_vals.as_f64(0).is_some();
        let is_category_with_gap =
            !is_numeric_x && (x_range.max - x_range.min) > row_count as f64 - 1.0;

        // 将当前系列的 Y 值累加到 running_total，并生成累积点
        let mut points = Vec::new();
        let mut values = Vec::new();
        let y_col = s.config.y_col_name();
        for (i, cumulative) in running_total.iter_mut().enumerate().take(row_count) {
            // X 坐标
            let x = if is_numeric_x {
                x_vals.as_f64(i)
            } else {
                let idx = i as f64;
                Some(if is_category_with_gap { idx + 0.5 } else { idx })
            };

            // 累加当前系列的 Y 值
            if let Some(col) = s.data.get_column(y_col)
                && let Some(v) = col.as_f64(i)
            {
                *cumulative += v;
            }
            let cumulative_y = *cumulative;

            if let Some(x) = x {
                let px = map_x_to_pixel(x, x_range, bounds);
                let py = map_y_to_pixel(cumulative_y, y_range, bounds);
                points.push(Point::new(px, py));
                values.push(cumulative_y);
            }
        }

        // 基线
        let baseline_y = if y_range.min > 0.0 {
            bounds.y1
        } else {
            map_y_to_pixel(0.0, y_range, bounds)
        };

        // 面积颜色：使用系列颜色
        let area_color = Some(color);

        let line_series = LineSeries {
            name: s.name.clone(),
            color,
            line_width: cfg.line_width,
            smooth: cfg.smooth,
            step: cfg.step.map(|s| match s {
                crate::pipeline::types::StepType::Start => {
                    crate::pipeline::typed_series::StepType::Start
                }
                crate::pipeline::types::StepType::Middle => {
                    crate::pipeline::typed_series::StepType::Middle
                }
                crate::pipeline::types::StepType::End => {
                    crate::pipeline::typed_series::StepType::End
                }
            }),
            area_color,
            area_opacity: cfg.area_opacity,
            symbol_type: line::map_symbol_type(cfg.symbol_type),
            symbol_size: cfg.symbol_size,
            points: points.clone(),
            values: values.clone(),
            baseline_y,
            baseline_points: prev_points.clone(),
            label: line_label_config(cfg),
            mark_lines: compute_mark_lines(&cfg.mark_line, &values, y_range, bounds),
        };

        prev_points = Some(points);
        results.push((series_idx, TypedSeries::Line(line_series)));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use vello_cpu::kurbo::Rect;

    use super::*;
    use crate::pipeline::{
        dataframe::{DataFrame, Series as DfSeries},
        types::{
            AxisPosition, AxisType, BarConfig, ResolvedAxisRange, ResolvedAxisRanges, SeriesConfig,
            SeriesSpec,
        },
    };

    fn bar_spec(name: &str, category: &str, value: f64, stack: &str) -> SeriesSpec {
        let mut df = DataFrame::new();
        df.add_column(DfSeries::new("x", vec![category.into()]));
        df.add_column(DfSeries::new("y", vec![value.into()]));
        SeriesSpec {
            name: name.into(),
            data: df,
            stack: Some(stack.into()),
            config: SeriesConfig::Bar(BarConfig {
                x_col: "x".into(),
                y_col: "y".into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn y_range(min: f64, max: f64) -> ResolvedAxisRange {
        ResolvedAxisRange {
            axis_index: 0,
            position: AxisPosition::Left,
            axis_type: AxisType::Value,
            min,
            max,
            is_user_defined: false,
            tick_count_hint: None,
            categories: vec![],
            inverse: false,
            boundary_gap: true,
        }
    }

    #[test]
    fn stacked_bars_split_positive_and_negative_baselines() {
        // H4 回归：正负值堆叠时应分别从基线向上/向下累计，负段不得叠在正段上方。
        let spec = crate::pipeline::types::ChartSpec {
            width: 200,
            height: 100,
            grids: vec![],
            x_axes: vec![],
            y_axes: vec![],
            series: vec![
                bar_spec("a", "c0", 10.0, "s"),
                bar_spec("b", "c0", -5.0, "s"),
                bar_spec("c", "c0", -3.0, "s"),
            ],
            title: None,
            legend: None,
            background: Color::rgb(255, 255, 255),
            palette: vec![],
            theme_name: None,
            fit_mode: crate::pipeline::types::FitMode::Fixed,
        };
        let ranges = ResolvedAxisRanges {
            ranges: vec![
                ResolvedAxisRange {
                    axis_index: 0,
                    position: AxisPosition::Bottom,
                    axis_type: AxisType::Category,
                    min: 0.0,
                    max: 1.0,
                    is_user_defined: false,
                    tick_count_hint: None,
                    categories: vec![],
                    inverse: false,
                    boundary_gap: true,
                },
                y_range(-10.0, 10.0),
            ],
        };
        let bounds = Rect::new(0.0, 0.0, 200.0, 100.0);
        let colors = ColorContext::default();
        let rows = materialize_stacked_bars(&[0, 1, 2], &spec, &ranges, bounds, &colors).unwrap();

        assert_eq!(rows.len(), 3);
        // y 像素：-10→100(底)  0→50  5→25  10→0(顶)  -5→75  -8→90
        let (r0, r1, r2) = (&rows[0], &rows[1], &rows[2]);
        assert_eq!(r0.value, 10.0);
        // 正段：数值 [0, 10] → 像素 [0, 50]
        assert!((r0.bar_rect.y0 - 0.0).abs() < 1e-6, "正段顶部应到 y=0");
        assert!(
            (r0.bar_rect.y1 - 50.0).abs() < 1e-6,
            "正段底部应到基线 y=50"
        );
        // 负段1：数值 [-5, 0] → 像素 [50, 75]（在基线下方，不与正段重叠）
        assert_eq!(r1.value, -5.0);
        assert!((r1.bar_rect.y0 - 50.0).abs() < 1e-6, "负段1顶部应接基线");
        assert!((r1.bar_rect.y1 - 75.0).abs() < 1e-6, "负段1应延伸到 y=75");
        // 负段2：数值 [-8, -5] → 像素 [75, 90]
        assert_eq!(r2.value, -3.0);
        assert!((r2.bar_rect.y0 - 75.0).abs() < 1e-6, "负段2应接负段1底端");
        assert!((r2.bar_rect.y1 - 90.0).abs() < 1e-6, "负段2应延伸到 y=90");
    }

    #[test]
    fn single_bar_no_gap_aligns_to_data_points() {
        // H2 回归：boundary_gap=false（范围 [0, n-1]）时柱体中心应落在数据点
        // 位置（首类在绘图区左缘、末类在右缘），而非 band 中心。
        use crate::pipeline::dataframe::DataValue;
        let mut df = DataFrame::new();
        df.add_column(DfSeries::new(
            "x",
            vec![
                DataValue::Float(0.0),
                DataValue::Float(1.0),
                DataValue::Float(2.0),
            ],
        ));
        df.add_column(DfSeries::new(
            "y",
            vec![
                DataValue::Float(5.0),
                DataValue::Float(6.0),
                DataValue::Float(7.0),
            ],
        ));
        let spec = SeriesSpec {
            name: "s".into(),
            data: df,
            config: SeriesConfig::Bar(BarConfig {
                x_col: "x".into(),
                y_col: "y".into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let ranges = ResolvedAxisRanges {
            ranges: vec![
                ResolvedAxisRange {
                    axis_index: 0,
                    position: AxisPosition::Bottom,
                    axis_type: AxisType::Category,
                    min: 0.0,
                    max: 2.0, // n=3，无留白 → [0, n-1]
                    is_user_defined: false,
                    tick_count_hint: None,
                    categories: vec!["a".into(), "b".into(), "c".into()],
                    inverse: false,
                    boundary_gap: false,
                },
                y_range(0.0, 10.0),
            ],
        };
        let bounds = Rect::new(0.0, 0.0, 120.0, 100.0);
        let colors = ColorContext::default();
        let typed = crate::pipeline::materializer::bar::BarMaterializer::materialize(
            &spec,
            bounds,
            &ranges,
            Color::rgb(0, 0, 0),
            &colors,
        )
        .unwrap();
        let crate::pipeline::typed_series::TypedSeries::Bar(bar) = typed else {
            panic!("应为 Bar 系列");
        };
        assert_eq!(bar.bars.len(), 3);
        for (i, b) in bar.bars.iter().enumerate() {
            let center_x = (b.rect.x0 + b.rect.x1) / 2.0;
            let expect = bounds.x0 + i as f64 / 2.0 * bounds.width(); // i/(n-1)
            assert!(
                (center_x - expect).abs() < 1e-6,
                "柱 {i} 中心应落在数据点 x={expect}，实际 {center_x}"
            );
        }
    }

    #[test]
    fn uneven_row_counts_share_band_layout() {
        // H2 回归：boundary_gap=false 且各系列行数不等时，类目总数必须统一取解析
        // 范围（此前各系列按自身行数反推 boundary_gap，行数少的系列被误判为带中心，
        // 与行数多的系列错位半槽）。
        use crate::pipeline::dataframe::DataValue;
        let mk_spec = |name: &str, rows: usize| {
            let mut df = DataFrame::new();
            df.add_column(DfSeries::new(
                "x",
                (0..rows)
                    .map(|i| DataValue::from(format!("c{i}")))
                    .collect(),
            ));
            df.add_column(DfSeries::new(
                "y",
                (0..rows)
                    .map(|i| DataValue::Float((i + 1) as f64))
                    .collect(),
            ));
            SeriesSpec {
                name: name.into(),
                data: df,
                stack: None,
                config: SeriesConfig::Bar(BarConfig {
                    x_col: "x".into(),
                    y_col: "y".into(),
                    ..Default::default()
                }),
                ..Default::default()
            }
        };
        let spec = crate::pipeline::types::ChartSpec {
            width: 200,
            height: 100,
            grids: vec![],
            x_axes: vec![],
            y_axes: vec![],
            series: vec![mk_spec("A", 3), mk_spec("B", 5)],
            title: None,
            legend: None,
            background: Color::rgb(255, 255, 255),
            palette: vec![],
            theme_name: None,
            fit_mode: crate::pipeline::types::FitMode::Fixed,
        };
        let ranges = ResolvedAxisRanges {
            ranges: vec![
                ResolvedAxisRange {
                    axis_index: 0,
                    position: AxisPosition::Bottom,
                    axis_type: AxisType::Category,
                    min: 0.0,
                    max: 4.0, // n=5（各系列最大行数），无留白 → [0, n-1]
                    is_user_defined: false,
                    tick_count_hint: None,
                    categories: vec![],
                    inverse: false,
                    boundary_gap: false,
                },
                y_range(0.0, 10.0),
            ],
        };
        let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let colors = ColorContext::default();
        let rows = materialize_side_by_side_bars(&[0, 1], &spec, &ranges, bounds, &colors).unwrap();
        assert_eq!(rows.len(), 8, "3 + 5 根柱");
        // 行 i 的 A/B 两根柱关于类目数据点 i/(n-1) 对称 → 两系列共享同一套带布局
        for i in 0..3 {
            let a = &rows[i];
            let b = &rows[3 + i];
            let group_center =
                (a.bar_rect.x0 + a.bar_rect.x1 + b.bar_rect.x0 + b.bar_rect.x1) / 4.0;
            let expect = bounds.x0 + i as f64 / 4.0 * bounds.width();
            assert!(
                (group_center - expect).abs() < 1e-6,
                "第 {i} 组中心应落在数据点 x={expect}，实际 {group_center}"
            );
        }
    }

    #[test]
    fn category_inverse_mirrors_band_positions() {
        // Category 轴 inverse：柱体位置与刻度标签同步镜像，cat i 应落在普通轴
        // cat (n-1-i) 的位置，且与刻度归一化位置（category_norm）一致。
        let mk_range = |inverse: bool| ResolvedAxisRange {
            axis_index: 0,
            position: AxisPosition::Bottom,
            axis_type: AxisType::Category,
            min: 0.0,
            max: 3.0,
            is_user_defined: false,
            tick_count_hint: None,
            categories: vec!["a".into(), "b".into(), "c".into()],
            inverse,
            boundary_gap: true,
        };
        let bounds = Rect::new(0.0, 0.0, 120.0, 100.0);
        let normal = mk_range(false);
        let flipped = mk_range(true);
        for i in 0..3 {
            let px_f = map_x_to_pixel(flipped.category_value(i), &flipped, bounds);
            let expect = map_x_to_pixel(normal.category_value(2 - i), &normal, bounds);
            assert!(
                (px_f - expect).abs() < 1e-6,
                "inverse 后 cat {i} 应落在普通轴 cat {} 的位置",
                2 - i
            );
            // 数据侧（map_x_to_pixel）与刻度侧（category_norm）必须一致
            let expect_tick = bounds.x0
                + crate::pipeline::types::category_norm(i, 3, true, true) * bounds.width();
            assert!(
                (px_f - expect_tick).abs() < 1e-6,
                "cat {i} 数据侧与刻度侧应一致"
            );
        }
    }
}
