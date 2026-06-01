use crate::pipeline::{
    data_processor::DataProcessorInput, dataframe::DataFrame, mapper::CoordinateMapper,
};

pub struct NoopMapper;

impl CoordinateMapper for NoopMapper {
    fn map_coordinates(
        &self,
        _df: &mut DataFrame,
        _input: &DataProcessorInput,
        _x_axis_idx: usize,
        _y_axis_idx: usize,
    ) {
    }
}
