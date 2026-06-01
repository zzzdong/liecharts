use crate::pipeline::{
    data_processor::DataProcessorInput,
    dataframe::{DataFrame, DataValue, Series},
    mapper::CoordinateMapper,
};

pub struct PolarMapper {
    pub radius_ratio: f64,
}

impl PolarMapper {
    pub fn new(radius_ratio: f64) -> Self {
        Self { radius_ratio }
    }
}

impl CoordinateMapper for PolarMapper {
    fn map_coordinates(
        &self,
        df: &mut DataFrame,
        input: &DataProcessorInput,
        _x_axis_idx: usize,
        _y_axis_idx: usize,
    ) {
        let bounds = input.bounds;
        let cx = bounds.x0 + bounds.width() / 2.0;
        let cy = bounds.y0 + bounds.height() / 2.0;
        let max_radius = bounds.width().min(bounds.height()) / 2.0 * self.radius_ratio;

        let row_count = df.row_count();
        df.add_column(Series::new_constant(
            "center_x",
            DataValue::Float(cx),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "center_y",
            DataValue::Float(cy),
            row_count,
        ));
        df.add_column(Series::new_constant(
            "max_radius",
            DataValue::Float(max_radius),
            row_count,
        ));
    }
}
