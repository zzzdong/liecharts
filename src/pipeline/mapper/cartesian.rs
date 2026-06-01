use crate::{
    option::AxisType,
    pipeline::{
        data_processor::DataProcessorInput,
        dataframe::{DataFrame, DataValue},
        mapper::CoordinateMapper,
    },
};

pub struct CartesianMapper;

impl CoordinateMapper for CartesianMapper {
    fn map_coordinates(
        &self,
        df: &mut DataFrame,
        input: &DataProcessorInput,
        x_axis_idx: usize,
        y_axis_idx: usize,
    ) {
        let bounds = input.bounds;

        let x_axis_config = input.option.x_axis.get(x_axis_idx);
        let y_axis_config = input.option.y_axis.get(y_axis_idx);

        let is_val_x = x_axis_config
            .and_then(|a| a.axis_type)
            .map(|t| t == AxisType::Value)
            .unwrap_or(false);

        let is_val_y = y_axis_config
            .and_then(|a| a.axis_type)
            .map(|t| t == AxisType::Value)
            .unwrap_or(true);

        let has_stack_base = df.get_column("stack_base").is_some();

        let x_range = input.axis_ranges.get_x_range(x_axis_idx);
        let y_range = input.axis_ranges.get_y_range(y_axis_idx);

        let (x_min, x_max) = x_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 1.0));
        let (y_min, y_max) = y_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 100.0));

        let eff_x_max = if is_val_x {
            (0..df.row_count())
                .filter_map(|i| df.get_column("x").and_then(|c| c.as_f64(i)))
                .fold(x_max, |a, b| a.max(b))
        } else {
            x_max
        };

        let eff_y_max = if is_val_y {
            (0..df.row_count())
                .filter_map(|i| df.get_column("y").and_then(|c| c.as_f64(i)))
                .fold(y_max, |a, b| a.max(b))
        } else {
            y_max
        };

        let x_scale = bounds.width() / (eff_x_max - x_min).max(0.001);
        let y_scale = bounds.height() / (eff_y_max - y_min).max(0.001);

        df.compute_column("px", |i, df| {
            if is_val_x {
                if let Some(v) = df.get_column("x").and_then(|c| c.as_f64(i)) {
                    DataValue::Float(bounds.x0 + (v - x_min) * x_scale)
                } else {
                    DataValue::Null
                }
            } else {
                let cat_idx = df
                    .get_column("cat_idx")
                    .and_then(|c| c.as_f64(i))
                    .unwrap_or(i as f64);
                let cat_count = (x_max - x_min).max(1.0);
                DataValue::Float(bounds.x0 + (cat_idx + 0.5) / cat_count * bounds.width())
            }
        });

        df.compute_column("py", |i, df| {
            if is_val_y {
                if let Some(v) = df.get_column("y").and_then(|c| c.as_f64(i)) {
                    DataValue::Float(bounds.y1 - (v - y_min) * y_scale)
                } else {
                    DataValue::Null
                }
            } else {
                let cat_idx = df
                    .get_column("cat_idx")
                    .and_then(|c| c.as_f64(i))
                    .unwrap_or(i as f64);
                let cat_count = (y_max - y_min).max(1.0);
                DataValue::Float(bounds.y0 + (cat_idx + 0.5) / cat_count * bounds.height())
            }
        });

        if has_stack_base {
            df.compute_column("pbase", |i, df| {
                let base = df
                    .get_column("stack_base")
                    .and_then(|c| c.as_f64(i))
                    .unwrap_or(0.0);
                if is_val_y {
                    DataValue::Float(bounds.y1 - (base - y_min).max(0.0) * y_scale)
                } else {
                    DataValue::Float(bounds.x0 + (base - x_min).max(0.0) * x_scale)
                }
            });
        }
    }
}
