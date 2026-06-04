//! Materializer 阶段：将 SeriesSpec 转换为 TypedSeries
//!
//! 职责：
//! - 从 DataFrame 提取数据
//! - 解析颜色（从 ColorContext）
//! - 将数据点分配到轴槽位 → 像素坐标
//! - Bar 系列先收集，最后做分组分析

use vello_cpu::kurbo::Rect;

use crate::{
    error::Result,
    pipeline::{
        types::{ChartSpec, ChartType, ColorContext, ResolvedAxisRanges, SeriesSpec},
        typed_series::{TypedSeries, GroupedBarRow},
    },
    visual::Color,
};

pub mod line;
pub mod bar;
pub mod scatter;
pub mod pie;
pub mod bubble;
pub mod candlestick;
pub mod radar;
pub mod polar_bar;
pub mod polar_scatter;
pub mod gauge;
pub mod table;

pub use line::LineMaterializer;
pub use bar::BarMaterializer;
pub use scatter::ScatterMaterializer;
pub use pie::PieMaterializer;
pub use bubble::BubbleMaterializer;
pub use candlestick::CandlestickMaterializer;
pub use radar::RadarMaterializer;
pub use polar_bar::PolarBarMaterializer;
pub use polar_scatter::PolarScatterMaterializer;
pub use gauge::GaugeMaterializer;
pub use table::TableMaterializer;

/// 每种图表类型实现此 trait，将 SeriesSpec 转换为对应的 TypedSeries
pub trait SeriesMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,                     // 子图像素边界
        axis_ranges: &ResolvedAxisRanges,  // 轴范围（用于槽位分配）
        color: Color,
        colors: &ColorContext,
    ) -> Result<TypedSeries>;
}

/// Materializer 函数类型（用于动态分发）
pub type MaterializerFn = fn(&SeriesSpec, Rect, &ResolvedAxisRanges, Color, &ColorContext) -> Result<TypedSeries>;

/// 创建对应类型的 Materializer
pub fn create_materializer(chart_type: ChartType) -> MaterializerFn {
    match chart_type {
        ChartType::Line => line_materializer_fn,
        ChartType::Bar => bar_materializer_fn,
        ChartType::Scatter => scatter_materializer_fn,
        ChartType::Pie => pie_materializer_fn,
        ChartType::Bubble => bubble_materializer_fn,
        ChartType::Candlestick => candlestick_materializer_fn,
        ChartType::Radar => radar_materializer_fn,
        ChartType::PolarBar => polar_bar_materializer_fn,
        ChartType::PolarScatter => polar_scatter_materializer_fn,
        ChartType::Gauge => gauge_materializer_fn,
        ChartType::Table => table_materializer_fn,
    }
}

fn line_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    LineMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn bar_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    BarMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn scatter_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    ScatterMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn pie_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    PieMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn bubble_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    BubbleMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn candlestick_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    CandlestickMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn radar_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    RadarMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn polar_bar_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    PolarBarMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn polar_scatter_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    PolarScatterMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn gauge_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    GaugeMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

fn table_materializer_fn(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color, colors: &ColorContext) -> Result<TypedSeries> {
    TableMaterializer::materialize(spec, bounds, axis_ranges, color, colors)
}

/// 辅助函数：将数据值映射到像素 X 坐标
pub fn map_x_to_pixel(x: f64, x_range: &crate::pipeline::types::ResolvedAxisRange, bounds: Rect) -> f64 {
    let range = x_range.max - x_range.min;
    if range <= 0.0 {
        bounds.x0
    } else {
        bounds.x0 + (x - x_range.min) / range * bounds.width()
    }
}

/// 辅助函数：将数据值映射到像素 Y 坐标
pub fn map_y_to_pixel(y: f64, y_range: &crate::pipeline::types::ResolvedAxisRange, bounds: Rect) -> f64 {
    let range = y_range.max - y_range.min;
    if range <= 0.0 {
        bounds.y1
    } else {
        bounds.y1 - (y - y_range.min) / range * bounds.height()
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
pub fn materialize_all(
    series_indices: &[usize],       // 当前 subplot 内的 series 索引
    spec: &ChartSpec,
    bounds: Rect,                    // 子图像素边界
    axis_ranges: &ResolvedAxisRanges,// 轴范围
    colors: &ColorContext,
) -> Result<Vec<TypedSeries>> {
    use crate::pipeline::group::GroupAnalyzer;

    let mut result: Vec<(usize, TypedSeries)> = Vec::new();
    let mut bar_specs: Vec<(usize, &SeriesSpec)> = Vec::new();

    for &global_idx in series_indices {
        let s = &spec.series[global_idx];
        match s.chart_type {
            ChartType::Bar => {
                bar_specs.push((global_idx, s));
            }
            _ => {
                let color = colors.get_series_color(global_idx);
                let materializer = create_materializer(s.chart_type);
                let typed = materializer(s, bounds, axis_ranges, color, colors)?;
                result.push((global_idx, typed));
            }
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
    all_series: &[SeriesSpec],
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
            .into_iter()
            .map(|(_, indices)| {
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

        let same_grid_and_x = bar_specs.iter().all(|(_, s)| {
            s.grid_index == first_grid && s.x_axis_index == first_x_axis
        });

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
    use crate::pipeline::typed_series::{BarSubSeries, GroupedBarRow, GroupedBarSeries, BarGroupType as TypedBarGroupType};

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

            Ok(TypedSeries::GroupedBar(GroupedBarSeries {
                sub_series,
                group_type: TypedBarGroupType::SideBySide,
                rows,
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

            let rows = materialize_stacked_bars(
                &plan.series_indices,
                spec,
                axis_ranges,
                bounds,
                colors,
            )?;

            Ok(TypedSeries::GroupedBar(GroupedBarSeries {
                sub_series,
                group_type: TypedBarGroupType::Stacked,
                rows,
            }))
        }
    }
}

/// Materialize 并排柱状图
fn materialize_side_by_side_bars(
    series_indices: &[usize],
    spec: &ChartSpec,
    axis_ranges: &ResolvedAxisRanges,
    bounds: Rect,
    colors: &ColorContext,
) -> Result<Vec<GroupedBarRow>> {
    use crate::pipeline::typed_series::GroupedBarRow;
    use crate::pipeline::types::AxisType;
    use vello_cpu::kurbo::Rect as KurboRect;

    let first_series = &spec.series[series_indices[0]];
    let x_range = axis_ranges.get_x_range(first_series.x_axis_index)
        .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("X axis not found".into()))?;
    let y_range = axis_ranges.get_y_range(first_series.y_axis_index)
        .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("Y axis not found".into()))?;

    let is_horizontal = matches!(y_range.axis_type, AxisType::Category);
    let series_count = series_indices.len();
    let bar_width_ratio = 0.6; // 默认柱宽比例

    let (cat_count, group_dim, bar_dim): (usize, f64, f64) = if is_horizontal {
        let cat_count = (y_range.max - y_range.min).max(1.0) as usize;
        let group_dim = bounds.height() / cat_count as f64 * bar_width_ratio;
        let bar_dim = group_dim / series_count as f64;
        (cat_count, group_dim, bar_dim)
    } else {
        let cat_count = (x_range.max - x_range.min).max(1.0) as usize;
        let group_dim = bounds.width() / cat_count as f64 * bar_width_ratio;
        let bar_dim = group_dim / series_count as f64;
        (cat_count, group_dim, bar_dim)
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

        if let (Some(y_series), Some(x_series)) = (series.data.get_column(y_col), series.data.get_column(x_col)) {
            for i in 0..series.data.row_count() {
                let value = y_series.as_f64(i).unwrap_or(0.0);
                let category = x_series.as_string(i).unwrap_or_default();
                let cat_idx = i % cat_count;

                if is_horizontal {
                    // 横向柱状图：Y=Category, X=Value
                    // 类别中心 Y 位置
                    let py = bounds.y1 - (cat_idx as f64 + 0.5) / cat_count as f64 * bounds.height();
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
                    let group_x = bounds.x0 + (cat_idx as f64 + 0.5) / cat_count as f64 * bounds.width();
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
    use crate::pipeline::typed_series::GroupedBarRow;
    use crate::pipeline::types::AxisType;
    use vello_cpu::kurbo::Rect as KurboRect;

    let first_series = &spec.series[series_indices[0]];
    let x_range = axis_ranges.get_x_range(first_series.x_axis_index)
        .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("X axis not found".into()))?;
    let y_range = axis_ranges.get_y_range(first_series.y_axis_index)
        .ok_or_else(|| crate::error::ChartError::InvalidAxisBinding("Y axis not found".into()))?;

    let is_horizontal = matches!(y_range.axis_type, AxisType::Category);
    let bar_width_ratio = 0.6;

    let (cat_count, bar_dim): (usize, f64) = if is_horizontal {
        let cat_count = (y_range.max - y_range.min).max(1.0) as usize;
        let bar_dim = bounds.height() / cat_count as f64 * bar_width_ratio;
        (cat_count, bar_dim)
    } else {
        let cat_count = (x_range.max - x_range.min).max(1.0) as usize;
        let bar_dim = bounds.width() / cat_count as f64 * bar_width_ratio;
        (cat_count, bar_dim)
    };

    // 收集每个类别的堆叠值
    let mut category_stacks: Vec<Vec<(usize, f64, f64)>> = vec![Vec::new(); cat_count];
    // (sub_series_idx, value, accumulated_base)

    for (sub_idx, &series_idx) in series_indices.iter().enumerate() {
        let series = &spec.series[series_idx];
        let y_col = series.config.y_col_name();

        if let Some(y_series) = series.data.get_column(y_col) {
            for i in 0..series.data.row_count() {
                let cat_idx = i % cat_count;
                let value = y_series.as_f64(i).unwrap_or(0.0);
                category_stacks[cat_idx].push((sub_idx, value, 0.0));
            }
        }
    }

    // 计算累积值
    for stack in &mut category_stacks {
        let mut acc = 0.0;
        for item in stack.iter_mut() {
            item.2 = acc;
            acc += item.1;
        }
    }

    let mut rows = Vec::new();

    // 计算基线位置
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

    for (cat_idx, stack) in category_stacks.iter().enumerate() {
        if is_horizontal {
            // 横向堆叠：Y=Category, X=Value
            let py = bounds.y1 - (cat_idx as f64 + 0.5) / cat_count as f64 * bounds.height();
            let bar_y = py - bar_dim / 2.0;

            let base_offset = baseline_x - map_x_to_pixel(0.0, x_range, bounds);

            for &(sub_idx, value, base) in stack {
                let series_idx = series_indices[sub_idx];
                let color = colors.get_series_color(series_idx);
                let x_col = spec.series[series_idx].config.x_col_name();

                let category = spec.series[series_idx].data.get_column(x_col)
                    .and_then(|s| s.as_string(cat_idx))
                    .unwrap_or_default();

                let px_base = map_x_to_pixel(base, x_range, bounds) + base_offset;
                let px_top = map_x_to_pixel(base + value, x_range, bounds) + base_offset;

                let rect = KurboRect::new(
                    px_top.min(px_base),
                    bar_y,
                    px_top.max(px_base),
                    bar_y + bar_dim,
                );

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
            let group_x = bounds.x0 + (cat_idx as f64 + 0.5) / cat_count as f64 * bounds.width();
            let bar_x = group_x - bar_dim / 2.0;

            let base_offset = baseline_y - map_y_to_pixel(0.0, y_range, bounds);

            for &(sub_idx, value, base) in stack {
                let series_idx = series_indices[sub_idx];
                let color = colors.get_series_color(series_idx);
                let x_col = spec.series[series_idx].config.x_col_name();

                let category = spec.series[series_idx].data.get_column(x_col)
                    .and_then(|s| s.as_string(cat_idx))
                    .unwrap_or_default();

                let py_base = map_y_to_pixel(base, y_range, bounds) + base_offset;
                let py_top = map_y_to_pixel(base + value, y_range, bounds) + base_offset;

                let rect = KurboRect::new(
                    bar_x,
                    py_top.min(py_base),
                    bar_x + bar_dim,
                    py_top.max(py_base),
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

    Ok(rows)
}
