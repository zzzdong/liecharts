use crate::error::Result;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::SeriesOption;

/// DataProcessor trait
///
/// 每个系列类型实现此 trait，完成从数据到视觉元素的完整转换。
pub trait DataProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData>;
}

/// 根据系列类型和索引创建对应的 DataProcessor
pub fn create_processor(
    series: &SeriesOption,
    series_index: usize,
) -> Box<dyn DataProcessor> {
    match series {
        SeriesOption::Pie(_) => Box::new(super::processor::pie::PieProcessor::new(series_index)),
        SeriesOption::Bar(_) => Box::new(super::processor::bar::BarProcessor::new(series_index)),
        SeriesOption::Line(_) => Box::new(super::processor::line::LineProcessor::new(series_index)),
        SeriesOption::Scatter(_) => Box::new(super::processor::scatter::ScatterProcessor::new(series_index)),
        // Phase 4: 添加其他系列类型的 Processor
        _ => todo!("Processor for {:?} not yet implemented", series),
    }
}