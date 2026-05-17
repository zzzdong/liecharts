//! 数据变换器 - 处理堆叠、归一化等数据变换

use crate::model::{DataItem, ResolvedSeries};
use std::collections::HashMap;

/// 变换后的数据项
#[derive(Debug, Clone)]
pub struct TransformedItem {
    /// 原始数据
    pub original: DataItem,
    /// 显示值（已应用堆叠、归一化等）
    pub display_value: f64,
    /// 基准值（用于面积/柱子底部）
    pub baseline: f64,
    /// 数据索引（用于排序后找回原始位置）
    pub data_index: usize,
}

/// 变换后的系列
#[derive(Debug, Clone)]
pub struct TransformedSeries {
    pub series_index: usize,
    pub items: Vec<TransformedItem>,
}

/// 数据变换器 trait
pub trait DataTransformer {
    fn transform(&self, all_series: &[ResolvedSeries]) -> Vec<TransformedSeries>;
}

/// 恒等变换 - 不做任何处理
pub struct IdentityTransformer;

impl DataTransformer for IdentityTransformer {
    fn transform(&self, all_series: &[ResolvedSeries]) -> Vec<TransformedSeries> {
        all_series
            .iter()
            .enumerate()
            .map(|(idx, series)| {
                let items = extract_data(series);
                TransformedSeries {
                    series_index: idx,
                    items: items
                        .iter()
                        .enumerate()
                        .map(|(i, item)| TransformedItem {
                            original: item.clone(),
                            display_value: item.value,
                            baseline: 0.0,
                            data_index: i,
                        })
                        .collect(),
                }
            })
            .collect()
    }
}

/// 堆叠变换器
pub struct StackedTransformer {
    /// 指定处理哪个 stack 组，None 表示处理所有带 stack 的系列
    pub stack_filter: Option<String>,
}

impl StackedTransformer {
    pub fn new(stack_filter: Option<String>) -> Self {
        Self { stack_filter }
    }
}

impl DataTransformer for StackedTransformer {
    fn transform(&self, all_series: &[ResolvedSeries]) -> Vec<TransformedSeries> {
        // 1. 按 stack 分组
        let mut stack_groups: HashMap<Option<String>, Vec<(usize, &ResolvedSeries)>> =
            HashMap::new();

        for (idx, series) in all_series.iter().enumerate() {
            let stack_name = extract_stack(series);

            // 如果指定了 filter，只处理匹配的组
            if let Some(ref filter) = self.stack_filter
                && stack_name.as_ref() != Some(filter) {
                    // 非目标组，使用恒等变换
                    continue;
                }

            stack_groups.entry(stack_name).or_default().push((idx, series));
        }

        // 2. 收集结果
        let mut result: Vec<TransformedSeries> = Vec::with_capacity(all_series.len());

        // 先处理非堆叠系列（恒等变换）
        for (idx, series) in all_series.iter().enumerate() {
            let stack_name = extract_stack(series);
            if stack_name.is_none() || !should_stack(series, &self.stack_filter) {
                result.push(transform_identity(idx, series));
            }
        }

        // 3. 对每个堆叠组进行累积计算
        for (_stack_name, group) in stack_groups {
            if group.is_empty() {
                continue;
            }

            // 获取数据长度
            let data_len = extract_data(group[0].1).len();
            let mut cumulative: Vec<f64> = vec![0.0; data_len];

            for (idx, series) in group {
                let items = extract_data(series);
                let transformed_items: Vec<TransformedItem> = items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let baseline = cumulative.get(i).copied().unwrap_or(0.0);
                        let value = item.value.max(0.0); // 堆叠只取正值
                        cumulative[i] = baseline + value;

                        TransformedItem {
                            original: item.clone(),
                            display_value: cumulative[i],
                            baseline,
                            data_index: i,
                        }
                    })
                    .collect();

                result.push(TransformedSeries {
                    series_index: idx,
                    items: transformed_items,
                });
            }
        }

        // 按 series_index 排序保持原始顺序
        result.sort_by_key(|s| s.series_index);
        result
    }
}

/// 百分比堆叠变换器
pub struct PercentStackedTransformer {
    pub stack_filter: Option<String>,
}

impl PercentStackedTransformer {
    pub fn new(stack_filter: Option<String>) -> Self {
        Self { stack_filter }
    }
}

impl DataTransformer for PercentStackedTransformer {
    fn transform(&self, all_series: &[ResolvedSeries]) -> Vec<TransformedSeries> {
        // 先进行普通堆叠
        let stacked = StackedTransformer {
            stack_filter: self.stack_filter.clone(),
        }
        .transform(all_series);

        // 计算每个数据点的总和
        let mut max_totals: Vec<f64> = Vec::new();
        for series in &stacked {
            for item in &series.items {
                if item.data_index >= max_totals.len() {
                    max_totals.resize(item.data_index + 1, 0.0);
                }
                max_totals[item.data_index] = max_totals[item.data_index].max(item.display_value);
            }
        }

        // 归一化到 0-100%
        stacked
            .into_iter()
            .map(|mut series| {
                for item in &mut series.items {
                    let total = max_totals.get(item.data_index).copied().unwrap_or(1.0);
                    if total > 0.0 {
                        item.display_value = (item.display_value / total) * 100.0;
                        item.baseline = (item.baseline / total) * 100.0;
                    }
                }
                series
            })
            .collect()
    }
}

// 辅助函数

fn extract_data(series: &ResolvedSeries) -> &[DataItem] {
    match series {
        ResolvedSeries::Bar(s) => &s.data,
        ResolvedSeries::Line(s) => &s.data,
        ResolvedSeries::Scatter(_) => &[], // 散点图使用特殊处理，不通过此函数
        ResolvedSeries::Pie(s) => &s.data,
        ResolvedSeries::Radar(_) => &[],
        ResolvedSeries::PolarBar(s) => &s.data,
        ResolvedSeries::PolarScatter(_) => &[],
        ResolvedSeries::Bubble(_) => &[],
        ResolvedSeries::Gauge(_) => &[],
        ResolvedSeries::Candlestick(_) => &[],
        ResolvedSeries::Table(_) => &[],
    }
}

fn extract_stack(series: &ResolvedSeries) -> Option<String> {
    match series {
        ResolvedSeries::Bar(s) => s.stack.clone(),
        ResolvedSeries::Line(s) => s.stack.clone(),
        _ => None,
    }
}

fn should_stack(series: &ResolvedSeries, filter: &Option<String>) -> bool {
    let stack = extract_stack(series);
    match filter {
        Some(f) => stack.as_ref() == Some(f),
        None => stack.is_some(),
    }
}

fn transform_identity(idx: usize, series: &ResolvedSeries) -> TransformedSeries {
    let items = extract_data(series);
    TransformedSeries {
        series_index: idx,
        items: items
            .iter()
            .enumerate()
            .map(|(i, item)| TransformedItem {
                original: item.clone(),
                display_value: item.value,
                baseline: 0.0,
                data_index: i,
            })
            .collect(),
    }
}
