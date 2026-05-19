//! Data sampling algorithms for reducing large datasets before rendering.
//!
//! Use [`SamplingOption`] to configure sampling on supported series (line, bar, scatter, etc.).
//! When the number of data points exceeds `threshold`, the selected algorithm downsamples
//! the data to at most `threshold` points while preserving visual characteristics.

use serde::{Deserialize, Serialize};

// ── Public types ───────────────────────────────────────────

/// Sampling algorithm selector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SamplingType {
    /// Largest-Triangle-Three-Buckets — preserves visual shape best.
    /// Recommended for line and scatter series.
    Lttb,
    /// Bucket average — smooths out noise, suitable for overviews.
    Average,
    /// Bucket maximum — retains peaks, useful for monitoring.
    Max,
    /// Bucket minimum — retains valleys.
    Min,
}

/// Sampling configuration attached to a series option.
///
/// When `threshold` is greater than or equal to the data length, no sampling is applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingOption {
    /// Sampling algorithm to use.
    #[serde(rename = "type")]
    pub ty: SamplingType,
    /// Target maximum number of data points after sampling.
    pub threshold: usize,
}

impl SamplingOption {
    /// Creates a new sampling config with the given algorithm and threshold.
    pub fn new(ty: SamplingType, threshold: usize) -> Self {
        Self { ty, threshold }
    }

    /// Shortcut for LTTB sampling.
    pub fn lttb(threshold: usize) -> Self {
        Self {
            ty: SamplingType::Lttb,
            threshold,
        }
    }
}

// ── Internal sampling functions ────────────────────────────

use crate::model::{CandlestickDataItem, DataItem, ScatterDataItem};

/// Samples `data` according to `config`. Returns a new `Vec` that is at most `threshold` long.
pub(crate) fn sample(data: &[DataItem], config: &SamplingOption) -> Vec<DataItem> {
    if data.len() <= config.threshold || config.threshold < 2 {
        return data.to_vec();
    }
    match config.ty {
        SamplingType::Lttb => lttb_indexed(data, config.threshold),
        SamplingType::Average => bucket_reduce_indexed(data, config.threshold, |bucket| {
            let sum: f64 = bucket.iter().map(|d| d.value).sum();
            sum / bucket.len() as f64
        }),
        SamplingType::Max => bucket_reduce_indexed(data, config.threshold, |bucket| {
            bucket
                .iter()
                .map(|d| d.value)
                .fold(f64::NEG_INFINITY, f64::max)
        }),
        SamplingType::Min => bucket_reduce_indexed(data, config.threshold, |bucket| {
            bucket.iter().map(|d| d.value).fold(f64::INFINITY, f64::min)
        }),
    }
}

/// Samples scatter data (x, y pairs) — uses LTTB for Lttb type, bucket otherwise.
pub(crate) fn sample_scatter(
    data: &[ScatterDataItem],
    config: &SamplingOption,
) -> Vec<ScatterDataItem> {
    if data.len() <= config.threshold || config.threshold < 2 {
        return data.to_vec();
    }
    match config.ty {
        SamplingType::Lttb => lttb_scatter(data, config.threshold),
        SamplingType::Average => bucket_reduce_scatter(data, config.threshold, |bucket| {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let n = bucket.len() as f64;
            for d in bucket {
                sum_x += d.x;
                sum_y += d.y;
            }
            ScatterDataItem {
                x: sum_x / n,
                y: sum_y / n,
                name: bucket[0].name.clone(),
            }
        }),
        SamplingType::Max => bucket_reduce_scatter(data, config.threshold, |bucket| {
            let mut best = &bucket[0];
            for d in bucket {
                if d.y > best.y {
                    best = d;
                }
            }
            best.clone()
        }),
        SamplingType::Min => bucket_reduce_scatter(data, config.threshold, |bucket| {
            let mut best = &bucket[0];
            for d in bucket {
                if d.y < best.y {
                    best = d;
                }
            }
            best.clone()
        }),
    }
}

/// Samples candlestick data — always uses OHLC bucket averaging.
pub(crate) fn sample_candlestick(
    data: &[CandlestickDataItem],
    config: &SamplingOption,
) -> Vec<CandlestickDataItem> {
    if data.len() <= config.threshold || config.threshold < 2 {
        return data.to_vec();
    }
    bucket_reduce_candlestick(data, config.threshold)
}

// ── LTTB (Largest Triangle Three Buckets) ──────────────────
// Algorithm description:
//   1. Always keep the first and last data points.
//   2. Divide the remaining points into (threshold - 2) buckets.
//   3. For each bucket, pick the point that forms the largest triangle
//      with the previously selected point and the average of the next bucket.

fn lttb_indexed(data: &[DataItem], threshold: usize) -> Vec<DataItem> {
    let n = data.len();
    debug_assert!(n > threshold && threshold >= 2);

    let mut result = Vec::with_capacity(threshold);
    result.push(data[0].clone());

    let bucket_size = (n - 2) as f64 / (threshold - 2) as f64;
    let mut prev_idx = 0usize;

    for i in 1..(threshold - 1) {
        let range_start = ((i - 1) as f64 * bucket_size).ceil() as usize + 1;
        let range_end = (i as f64 * bucket_size).ceil() as usize + 1;
        let range_end = range_end.min(n - 1);

        if range_start >= range_end {
            let idx = range_start.min(n - 1);
            result.push(data[idx].clone());
            prev_idx = idx;
            continue;
        }

        // Average of the next bucket
        let next_start = range_end;
        let next_end = ((i + 1) as f64 * bucket_size).ceil() as usize + 1;
        let next_end = next_end.min(n);

        let (avg_x, avg_y) = if next_end > next_start {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let count = (next_end - next_start) as f64;
            for j in next_start..next_end {
                sum_x += data[j].x_value.unwrap_or(j as f64);
                sum_y += data[j].value;
            }
            (sum_x / count, sum_y / count)
        } else {
            let idx = next_start - 1;
            (data[idx].x_value.unwrap_or(idx as f64), data[idx].value)
        };

        // Previous selected point
        let prev_x = data[prev_idx].x_value.unwrap_or(prev_idx as f64);
        let prev_y = data[prev_idx].value;

        // Find triangle with largest area in current bucket
        let mut best_area = -1.0f64;
        let mut best_idx = range_start;

        for j in range_start..range_end {
            let x = data[j].x_value.unwrap_or(j as f64);
            let area = triangle_area(prev_x, prev_y, x, data[j].value, avg_x, avg_y);
            if area > best_area {
                best_area = area;
                best_idx = j;
            }
        }

        result.push(data[best_idx].clone());
        prev_idx = best_idx;
    }

    result.push(data[n - 1].clone());
    result
}

fn lttb_scatter(data: &[ScatterDataItem], threshold: usize) -> Vec<ScatterDataItem> {
    let n = data.len();
    debug_assert!(n > threshold && threshold >= 2);

    let mut result = Vec::with_capacity(threshold);
    result.push(data[0].clone());

    let bucket_size = (n - 2) as f64 / (threshold - 2) as f64;
    let mut prev_idx = 0usize;

    for i in 1..(threshold - 1) {
        let range_start = ((i - 1) as f64 * bucket_size).ceil() as usize + 1;
        let range_end = (i as f64 * bucket_size).ceil() as usize + 1;
        let range_end = range_end.min(n - 1);

        if range_start >= range_end {
            let idx = range_start.min(n - 1);
            result.push(data[idx].clone());
            prev_idx = idx;
            continue;
        }

        let next_start = range_end;
        let next_end = ((i + 1) as f64 * bucket_size).ceil() as usize + 1;
        let next_end = next_end.min(n);

        let (avg_x, avg_y) = if next_end > next_start {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let count = (next_end - next_start) as f64;
            for j in next_start..next_end {
                sum_x += data[j].x;
                sum_y += data[j].y;
            }
            (sum_x / count, sum_y / count)
        } else {
            (data[next_start - 1].x, data[next_start - 1].y)
        };

        let prev_x = data[prev_idx].x;
        let prev_y = data[prev_idx].y;

        let mut best_area = -1.0f64;
        let mut best_idx = range_start;

        for j in range_start..range_end {
            let area = triangle_area(prev_x, prev_y, data[j].x, data[j].y, avg_x, avg_y);
            if area > best_area {
                best_area = area;
                best_idx = j;
            }
        }

        result.push(data[best_idx].clone());
        prev_idx = best_idx;
    }

    result.push(data[n - 1].clone());
    result
}

fn triangle_area(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> f64 {
    ((x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2)) / 2.0).abs()
}

// ── Bucket helpers ─────────────────────────────────────────

fn bucket_indices(n: usize, buckets: usize) -> Vec<(usize, usize)> {
    let bucket_size = n as f64 / buckets as f64;
    let mut ranges = Vec::with_capacity(buckets);
    for i in 0..buckets {
        let start = (i as f64 * bucket_size).round() as usize;
        let end = ((i + 1) as f64 * bucket_size).round() as usize;
        ranges.push((start, end.min(n)));
    }
    ranges
}

fn bucket_reduce_indexed(
    data: &[DataItem],
    threshold: usize,
    pick: fn(&[DataItem]) -> f64,
) -> Vec<DataItem> {
    let ranges = bucket_indices(data.len(), threshold);
    let mut result = Vec::with_capacity(threshold);
    for (start, end) in ranges {
        if start >= end {
            result.push(data[start.min(data.len() - 1)].clone());
        } else {
            let value = pick(&data[start..end]);
            result.push(DataItem {
                name: data[start].name.clone(),
                value,
                x_value: data[start].x_value,
            });
        }
    }
    result
}

fn bucket_reduce_scatter(
    data: &[ScatterDataItem],
    threshold: usize,
    pick: fn(&[ScatterDataItem]) -> ScatterDataItem,
) -> Vec<ScatterDataItem> {
    let ranges = bucket_indices(data.len(), threshold);
    let mut result = Vec::with_capacity(threshold);
    for (start, end) in ranges {
        if start >= end {
            result.push(data[start.min(data.len() - 1)].clone());
        } else {
            result.push(pick(&data[start..end]));
        }
    }
    result
}

fn bucket_reduce_candlestick(
    data: &[CandlestickDataItem],
    threshold: usize,
) -> Vec<CandlestickDataItem> {
    let ranges = bucket_indices(data.len(), threshold);
    let mut result = Vec::with_capacity(threshold);
    for (start, end) in ranges {
        if start >= end {
            result.push(data[start.min(data.len() - 1)].clone());
            continue;
        }
        let mut sum_open = 0.0;
        let mut sum_close = 0.0;
        let mut high = f64::NEG_INFINITY;
        let mut low = f64::INFINITY;
        let count = (end - start) as f64;
        for d in &data[start..end] {
            sum_open += d.open;
            sum_close += d.close;
            if d.high > high {
                high = d.high;
            }
            if d.low < low {
                low = d.low;
            }
        }
        result.push(CandlestickDataItem {
            open: sum_open / count,
            close: sum_close / count,
            high,
            low,
            name: data[start].name.clone(),
        });
    }
    result
}

// ── Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data(n: usize) -> Vec<DataItem> {
        (0..n)
            .map(|i| DataItem {
                name: None,
                value: (i as f64 * 0.1).sin(),
                x_value: None,
            })
            .collect()
    }

    #[test]
    fn test_noop_when_under_threshold() {
        let data = make_data(10);
        let cfg = SamplingOption::lttb(100);
        assert_eq!(sample(&data, &cfg).len(), 10);
    }

    #[test]
    fn test_lttb_reduces() {
        let data = make_data(1000);
        let cfg = SamplingOption::lttb(50);
        let sampled = sample(&data, &cfg);
        assert!(sampled.len() <= 50);
        assert_eq!(sampled[0].value, data[0].value);
        assert_eq!(sampled.last().unwrap().value, data[999].value);
    }

    #[test]
    fn test_average_reduces() {
        let data = make_data(1000);
        let cfg = SamplingOption::new(SamplingType::Average, 50);
        let sampled = sample(&data, &cfg);
        assert!(sampled.len() <= 50);
    }

    #[test]
    fn test_max_reduces() {
        let data = make_data(1000);
        let cfg = SamplingOption::new(SamplingType::Max, 50);
        let sampled = sample(&data, &cfg);
        assert!(sampled.len() <= 50);
    }

    #[test]
    fn test_min_reduces() {
        let data = make_data(1000);
        let cfg = SamplingOption::new(SamplingType::Min, 50);
        let sampled = sample(&data, &cfg);
        assert!(sampled.len() <= 50);
    }

    #[test]
    fn test_threshold_of_one() {
        let data = make_data(100);
        let cfg = SamplingOption::lttb(1);
        // threshold < 2 returns original
        assert_eq!(sample(&data, &cfg).len(), 100);
    }

    #[test]
    fn test_scatter_lttb() {
        let data: Vec<ScatterDataItem> = (0..500)
            .map(|i| ScatterDataItem {
                x: i as f64,
                y: (i as f64 * 0.05).sin() * 10.0,
                name: None,
            })
            .collect();
        let cfg = SamplingOption::lttb(30);
        let sampled = sample_scatter(&data, &cfg);
        assert!(sampled.len() <= 30);
        assert_eq!(sampled[0].x, data[0].x);
        assert_eq!(sampled.last().unwrap().x, data[499].x);
    }
}
