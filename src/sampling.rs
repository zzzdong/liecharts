use serde::{Deserialize, Serialize};

use crate::pipeline::dataframe::{DataFrame, Series};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SamplingType {
    Lttb,
    Average,
    Max,
    Min,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingOption {
    #[serde(rename = "type")]
    pub ty: SamplingType,
    pub threshold: usize,
}

impl SamplingOption {
    pub fn new(ty: SamplingType, threshold: usize) -> Self {
        Self { ty, threshold }
    }

    pub fn lttb(threshold: usize) -> Self {
        Self {
            ty: SamplingType::Lttb,
            threshold,
        }
    }
}

/// Data sampling processor for reducing data points in large datasets.
pub struct SamplingProcessor;

impl SamplingProcessor {
    /// Apply sampling to a DataFrame, preserving all columns.
    /// Uses the first numeric column found for sampling calculations.
    pub fn sample(df: &DataFrame, threshold: usize, sampling_type: SamplingType) -> DataFrame {
        if df.row_count() <= threshold {
            return df.clone();
        }

        match sampling_type {
            SamplingType::Lttb => Self::sample_lttb(df, threshold),
            SamplingType::Average => Self::sample_bucket(df, threshold, BucketOp::Average),
            SamplingType::Max => Self::sample_bucket(df, threshold, BucketOp::Max),
            SamplingType::Min => Self::sample_bucket(df, threshold, BucketOp::Min),
        }
    }

    fn sample_lttb(df: &DataFrame, threshold: usize) -> DataFrame {
        let row_count = df.row_count();
        let bucket_size = (row_count - 2) as f64 / (threshold - 2) as f64;

        // Find first numeric column for value reference
        let value_col = df
            .column_names()
            .iter()
            .find(|name| df.get_column(name).and_then(|c| c.as_f64(0)).is_some());

        let value_col = match value_col {
            Some(c) => c.clone(),
            None => return df.clone(),
        };

        let values: Vec<f64> = (0..row_count)
            .filter_map(|i| df.get_column(&value_col).and_then(|c| c.as_f64(i)))
            .collect();

        if values.len() < 3 || threshold < 3 {
            return df.clone();
        }

        let mut selected = vec![0usize]; // Always keep first point
        let mut prev = 0usize;

        for i in 1..threshold - 1 {
            let bucket_start = ((i as f64) * bucket_size).ceil() as usize;
            let bucket_end = (((i as f64) + 1.0) * bucket_size).floor() as usize;
            let bucket_start = bucket_start.max(prev + 1).min(row_count - 1);
            let bucket_end = bucket_end.max(bucket_start + 1).min(row_count - 1);

            let mut best_idx = bucket_start;
            let mut best_area = -1.0f64;

            let x_prev = prev as f64;
            let y_prev = values[prev];

            let x_next = (row_count - 1) as f64;
            let y_next = values[row_count - 1];

            for (j, &y_j) in values.iter().enumerate().take(bucket_end).skip(bucket_start) {
                let x_j = j as f64;
                let area =
                    ((x_prev - x_j) * (y_next - y_j) - (x_prev - x_next) * (y_prev - y_j)).abs();
                if area > best_area {
                    best_area = area;
                    best_idx = j;
                }
            }

            selected.push(best_idx);
            prev = best_idx;
        }

        selected.push(row_count - 1); // Always keep last point
        selected.sort();
        selected.dedup();

        Self::select_rows(df, &selected)
    }

    fn sample_bucket(df: &DataFrame, threshold: usize, op: BucketOp) -> DataFrame {
        let row_count = df.row_count();
        let bucket_size = (row_count as f64 / threshold as f64).ceil() as usize;
        let mut result = DataFrame::new();
        let cols = df.column_names().to_vec();

        for col_name in &cols {
            let col = df.get_column(col_name).expect("Column exists");
            let mut new_data: Vec<crate::pipeline::dataframe::DataValue> = Vec::new();

            for bucket in 0..threshold {
                let start = bucket * bucket_size;
                let end = ((bucket + 1) * bucket_size).min(row_count);
                if start >= row_count {
                    break;
                }

                let bucket_val = match op {
                    BucketOp::Average => {
                        let mut sum = 0.0f64;
                        let mut count = 0usize;
                        for i in start..end {
                            if let Some(v) = col.as_f64(i) {
                                sum += v;
                                count += 1;
                            }
                        }
                        if count > 0 {
                            Some(crate::pipeline::dataframe::DataValue::Float(
                                sum / count as f64,
                            ))
                        } else {
                            Some(col.data[start].clone())
                        }
                    }
                    BucketOp::Max => {
                        let mut max_val = f64::NEG_INFINITY;
                        let mut has_val = false;
                        for i in start..end {
                            if let Some(v) = col.as_f64(i) {
                                max_val = max_val.max(v);
                                has_val = true;
                            }
                        }
                        if has_val {
                            Some(crate::pipeline::dataframe::DataValue::Float(max_val))
                        } else {
                            Some(col.data[start].clone())
                        }
                    }
                    BucketOp::Min => {
                        let mut min_val = f64::INFINITY;
                        let mut has_val = false;
                        for i in start..end {
                            if let Some(v) = col.as_f64(i) {
                                min_val = min_val.min(v);
                                has_val = true;
                            }
                        }
                        if has_val {
                            Some(crate::pipeline::dataframe::DataValue::Float(min_val))
                        } else {
                            Some(col.data[start].clone())
                        }
                    }
                };

                new_data.push(bucket_val.expect("bucket_val is always Some"));
            }

            result.add_column(Series::new(col_name.clone(), new_data));
        }

        result
    }

    fn select_rows(df: &DataFrame, indices: &[usize]) -> DataFrame {
        let mut result = DataFrame::new();
        for col_name in df.column_names() {
            let col = df.get_column(col_name).expect("Column exists");
            let selected: Vec<crate::pipeline::dataframe::DataValue> = indices
                .iter()
                .filter_map(|&i| col.data.get(i).cloned())
                .collect();
            result.add_column(Series::new(col_name.clone(), selected));
        }
        result
    }
}

enum BucketOp {
    Average,
    Max,
    Min,
}
