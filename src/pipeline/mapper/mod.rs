pub mod cartesian;
pub mod noop;
pub mod polar;

use crate::pipeline::{data_processor::DataProcessorInput, dataframe::DataFrame};

pub trait CoordinateMapper {
    fn map_coordinates(
        &self,
        df: &mut DataFrame,
        input: &DataProcessorInput,
        x_axis_idx: usize,
        y_axis_idx: usize,
    );
}

pub use cartesian::CartesianMapper;
pub use noop::NoopMapper;
pub use polar::PolarMapper;
