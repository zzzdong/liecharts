use crate::pipeline::dataframe::{DataFrame, DataValue, Series};

/// 采样处理器 - 用于减少数据点数量
pub struct SamplingProcessor;

impl SamplingProcessor {
    /// 对 DataFrame 进行采样
    /// 假设 DataFrame 包含 'x' 和 'y' 列
    pub fn sample(
        df: &DataFrame,
        threshold: usize,
        ty: crate::sampling::SamplingType,
    ) -> DataFrame {
        if df.row_count() <= threshold {
            return df.clone();
        }

        let sampled_indices = match ty {
            crate::sampling::SamplingType::Lttb => Self::lttb_indices(df, threshold),
            crate::sampling::SamplingType::Average => Self::average_indices(df, threshold),
            crate::sampling::SamplingType::Max => Self::max_indices(df, threshold),
            crate::sampling::SamplingType::Min => Self::min_indices(df, threshold),
        };

        Self::extract_rows(df, &sampled_indices)
    }

    /// LTTB (Largest Triangle Three Buckets) 算法
    /// 保留视觉特征的同时减少数据点
    fn lttb_indices(df: &DataFrame, threshold: usize) -> Vec<usize> {
        let data_length = df.row_count();
        if threshold >= data_length {
            return (0..data_length).collect();
        }

        let mut sampled = Vec::with_capacity(threshold);
        let bucket_size = (data_length - 2) as f64 / (threshold - 2) as f64;

        // 始终保留第一个点
        sampled.push(0);

        // 获取所有点的坐标
        let points: Vec<(f64, f64)> = (0..data_length)
            .map(|i| {
                let x = df
                    .get_column("x")
                    .and_then(|c| c.as_f64(i))
                    .unwrap_or(i as f64);
                let y = df.get_column("y").and_then(|c| c.as_f64(i)).unwrap_or(0.0);
                (x, y)
            })
            .collect();

        // 处理中间的桶
        for i in 0..(threshold - 2) {
            let bucket_start = ((i as f64 * bucket_size) as usize + 1).min(data_length - 1);
            let bucket_end = (((i + 1) as f64 * bucket_size) as usize + 1).min(data_length - 1);

            if bucket_start >= bucket_end {
                continue;
            }

            // 计算当前桶的平均点
            let avg_x = (bucket_start..bucket_end).map(|j| points[j].0).sum::<f64>()
                / (bucket_end - bucket_start) as f64;
            let avg_y = (bucket_start..bucket_end).map(|j| points[j].1).sum::<f64>()
                / (bucket_end - bucket_start) as f64;

            // 上一个选中的点
            let last_idx = *sampled.last().unwrap();
            let last_x = points[last_idx].0;
            let last_y = points[last_idx].1;

            // 在当前桶中找到形成最大三角形的点
            let mut max_area = -1.0;
            let mut max_idx = bucket_start;

            for j in bucket_start..bucket_end {
                let x = points[j].0;
                let y = points[j].1;

                // 计算三角形面积 (last, avg, current)
                let area =
                    ((last_x - avg_x) * (y - last_y) - (last_x - x) * (avg_y - last_y)).abs();

                if area > max_area {
                    max_area = area;
                    max_idx = j;
                }
            }

            sampled.push(max_idx);
        }

        // 始终保留最后一个点
        if data_length > 1 {
            sampled.push(data_length - 1);
        }

        sampled
    }

    /// Average 采样 - 将数据分成多个桶，取每个桶的平均值
    fn average_indices(df: &DataFrame, threshold: usize) -> Vec<usize> {
        let data_length = df.row_count();
        if threshold >= data_length {
            return (0..data_length).collect();
        }

        let mut indices = Vec::with_capacity(threshold);
        let bucket_size = data_length as f64 / threshold as f64;

        for i in 0..threshold {
            let start = (i as f64 * bucket_size) as usize;
            let end = ((i + 1) as f64 * bucket_size) as usize;
            let mid = (start + end) / 2;
            indices.push(mid.min(data_length - 1));
        }

        indices
    }

    /// Max 采样 - 将数据分成多个桶，取每个桶的最大值
    fn max_indices(df: &DataFrame, threshold: usize) -> Vec<usize> {
        let data_length = df.row_count();
        if threshold >= data_length {
            return (0..data_length).collect();
        }

        let y_col = df.get_column("y");
        let mut indices = Vec::with_capacity(threshold);
        let bucket_size = data_length as f64 / threshold as f64;

        for i in 0..threshold {
            let start = (i as f64 * bucket_size) as usize;
            let end = ((i + 1) as f64 * bucket_size).min(data_length as f64) as usize;

            let mut max_val = f64::NEG_INFINITY;
            let mut max_idx = start;

            for j in start..end {
                if let Some(y) = y_col.and_then(|c| c.as_f64(j)) {
                    if y > max_val {
                        max_val = y;
                        max_idx = j;
                    }
                }
            }

            indices.push(max_idx);
        }

        indices
    }

    /// Min 采样 - 将数据分成多个桶，取每个桶的最小值
    fn min_indices(df: &DataFrame, threshold: usize) -> Vec<usize> {
        let data_length = df.row_count();
        if threshold >= data_length {
            return (0..data_length).collect();
        }

        let y_col = df.get_column("y");
        let mut indices = Vec::with_capacity(threshold);
        let bucket_size = data_length as f64 / threshold as f64;

        for i in 0..threshold {
            let start = (i as f64 * bucket_size) as usize;
            let end = ((i + 1) as f64 * bucket_size).min(data_length as f64) as usize;

            let mut min_val = f64::INFINITY;
            let mut min_idx = start;

            for j in start..end {
                if let Some(y) = y_col.and_then(|c| c.as_f64(j)) {
                    if y < min_val {
                        min_val = y;
                        min_idx = j;
                    }
                }
            }

            indices.push(min_idx);
        }

        indices
    }

    /// 根据索引从 DataFrame 中提取行
    fn extract_rows(df: &DataFrame, indices: &[usize]) -> DataFrame {
        let mut result = DataFrame::new();

        for col_name in df.column_names() {
            let col = df.get_column(col_name).unwrap();
            let new_data: Vec<DataValue> = indices
                .iter()
                .map(|&i| col.data.get(i).cloned().unwrap_or(DataValue::Null))
                .collect();
            result.add_column(Series::new(col_name, new_data));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::dataframe::{DataFrame, DataValue, Series};

    fn create_test_dataframe() -> DataFrame {
        let mut df = DataFrame::new();
        df.add_column(Series::new(
            "x",
            vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]
                .into_iter()
                .map(DataValue::Float)
                .collect(),
        ));
        df.add_column(Series::new(
            "y",
            vec![0.0, 1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 81.0]
                .into_iter()
                .map(DataValue::Float)
                .collect(),
        ));
        df
    }

    #[test]
    fn test_lttb_sampling() {
        let df = create_test_dataframe();
        let sampled = SamplingProcessor::sample(&df, 5, crate::sampling::SamplingType::Lttb);
        assert_eq!(sampled.row_count(), 5);
        // 第一个和最后一个点应该被保留
        assert_eq!(sampled.get_column("x").unwrap().as_f64(0), Some(0.0));
        assert_eq!(sampled.get_column("x").unwrap().as_f64(4), Some(9.0));
    }

    #[test]
    fn test_average_sampling() {
        let df = create_test_dataframe();
        let sampled = SamplingProcessor::sample(&df, 5, crate::sampling::SamplingType::Average);
        assert_eq!(sampled.row_count(), 5);
    }

    #[test]
    fn test_max_sampling() {
        let df = create_test_dataframe();
        let sampled = SamplingProcessor::sample(&df, 5, crate::sampling::SamplingType::Max);
        assert_eq!(sampled.row_count(), 5);
    }

    #[test]
    fn test_min_sampling() {
        let df = create_test_dataframe();
        let sampled = SamplingProcessor::sample(&df, 5, crate::sampling::SamplingType::Min);
        assert_eq!(sampled.row_count(), 5);
    }

    #[test]
    fn test_no_sampling_when_under_threshold() {
        let df = create_test_dataframe();
        let sampled = SamplingProcessor::sample(&df, 20, crate::sampling::SamplingType::Lttb);
        assert_eq!(sampled.row_count(), 10);
    }
}
