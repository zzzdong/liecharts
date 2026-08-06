//! DataProcessor 阶段：在 ColorAssigner 之后、Materializer 之前对数据进行转换
//!
//! 职责：
//! - 采样（LTTB / Average / Max / Min）
//! - 是未来数据转换（过滤、排序、计算列等）的扩展点
//!
//! DataProcessor 是管线中的纯数据阶段，不涉及任何渲染逻辑。

use crate::pipeline::types::SeriesSpec;

/// 处理所有 SeriesSpec 的数据变换
///
/// 对每个 SeriesSpec：
/// 1. 如果配置了采样，则对 DataFrame 进行降采样
/// 2. 后续可扩展其他数据转换
pub fn process_series(series: &mut [SeriesSpec]) {
    for s in series.iter_mut() {
        // 采样处理
        if let Some((sampling_type, threshold)) = s.sampling {
            s.data = crate::sampling::SamplingProcessor::sample(&s.data, threshold, sampling_type);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pipeline::{
            dataframe::{DataFrame, DataValue, Series},
            types::{LineConfig, SeriesConfig},
        },
        sampling::SamplingType,
    };

    #[test]
    fn test_sampling_processor_reduces_rows() {
        let mut df = DataFrame::new();
        let x: Vec<DataValue> = (0..100).map(|i| DataValue::Float(i as f64)).collect();
        let y: Vec<DataValue> = (0..100)
            .map(|i| DataValue::Float((i as f64).sin()))
            .collect();
        df.add_column(Series::new("x", x));
        df.add_column(Series::new("y", y));

        let spec = SeriesSpec {
            name: "test".into(),
            data: df,
            sampling: Some((SamplingType::Lttb, 10)),
            config: SeriesConfig::Line(LineConfig::default()),
            ..Default::default()
        };

        let mut series = vec![spec];
        process_series(&mut series);

        assert!(
            series[0].data.row_count() <= 10,
            "Sampled data should have at most 10 rows, got {}",
            series[0].data.row_count()
        );
        assert!(
            series[0].data.row_count() >= 2,
            "Sampled data should have at least 2 rows (first + last), got {}",
            series[0].data.row_count()
        );
    }

    #[test]
    fn test_no_sampling_keeps_all_rows() {
        let mut df = DataFrame::new();
        let x: Vec<DataValue> = (0..50).map(|i| DataValue::Float(i as f64)).collect();
        let y: Vec<DataValue> = (0..50)
            .map(|i| DataValue::Float((i as f64).cos()))
            .collect();
        df.add_column(Series::new("x", x));
        df.add_column(Series::new("y", y));

        let spec = SeriesSpec {
            name: "test".into(),
            data: df.clone(),
            sampling: None,
            config: SeriesConfig::Line(LineConfig::default()),
            ..Default::default()
        };

        assert_eq!(spec.data.row_count(), 50);
        let mut series = vec![spec];
        process_series(&mut series);
        assert_eq!(
            series[0].data.row_count(),
            50,
            "Without sampling, all rows should be preserved"
        );
    }
}
