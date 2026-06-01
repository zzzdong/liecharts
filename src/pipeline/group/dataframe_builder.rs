use crate::pipeline::dataframe::{DataFrame, DataValue, Series};

pub struct GroupedBarProcessor;

impl GroupedBarProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn combine_to_dataframe(
        plan: &super::analyzer::GroupPlan,
        option: &crate::option::ChartOption,
        colors: &crate::pipeline::types::ColorContext,
    ) -> DataFrame {
        use crate::option::{DataPoint, SeriesOption};

        let is_stacked = plan.group_type == super::analyzer::GroupType::Stacked;

        let mut all_x: Vec<DataValue> = Vec::new();
        let mut all_y: Vec<DataValue> = Vec::new();
        let mut all_cat: Vec<DataValue> = Vec::new();
        let mut all_color: Vec<DataValue> = Vec::new();
        let mut all_pos: Vec<DataValue> = Vec::new();
        let mut all_base: Vec<DataValue> = Vec::new();

        if is_stacked {
            let max_cats = plan
                .series_indices
                .iter()
                .filter_map(|&idx| match &option.series[idx] {
                    SeriesOption::Bar(b) => Some(b.data.len()),
                    _ => None,
                })
                .max()
                .unwrap_or(0);

            let mut stack_cums: Vec<f64> = vec![0.0; max_cats];

            for (_pos, &global_idx) in plan.series_indices.iter().enumerate() {
                let bar = match &option.series[global_idx] {
                    SeriesOption::Bar(b) => b,
                    _ => continue,
                };
                let color = colors.get_series_color(global_idx);
                let color_val = DataValue::Color(color);

                for (cat_idx, dp) in bar.data.iter().enumerate() {
                    let x_val = match dp {
                        DataPoint::Named(name, _) => DataValue::String(name.clone()),
                        _ => DataValue::Integer(cat_idx as i64),
                    };
                    let raw_val = match dp {
                        DataPoint::Value(v) => *v,
                        DataPoint::Named(_, v) => *v,
                        DataPoint::XY(_, y) => *y,
                    };
                    let base = stack_cums[cat_idx];
                    let cum_val = base + raw_val;
                    stack_cums[cat_idx] = cum_val;

                    all_x.push(x_val);
                    all_y.push(DataValue::Float(cum_val));
                    all_cat.push(DataValue::Integer(cat_idx as i64));
                    all_color.push(color_val.clone());
                    all_pos.push(DataValue::Integer(0));
                    all_base.push(DataValue::Float(base));
                }
            }
        } else {
            for (pos, &global_idx) in plan.series_indices.iter().enumerate() {
                let bar = match &option.series[global_idx] {
                    SeriesOption::Bar(b) => b,
                    _ => continue,
                };
                let color = colors.get_series_color(global_idx);
                let color_val = DataValue::Color(color);

                for (cat_idx, dp) in bar.data.iter().enumerate() {
                    let x_val = match dp {
                        DataPoint::Named(name, _) => DataValue::String(name.clone()),
                        _ => DataValue::Integer(cat_idx as i64),
                    };
                    let y_val = DataValue::Float(match dp {
                        DataPoint::Value(v) => *v,
                        DataPoint::Named(_, v) => *v,
                        DataPoint::XY(_, y) => *y,
                    });

                    all_x.push(x_val);
                    all_y.push(y_val);
                    all_cat.push(DataValue::Integer(cat_idx as i64));
                    all_color.push(color_val.clone());
                    all_pos.push(DataValue::Integer(pos as i64));
                    all_base.push(DataValue::Float(0.0));
                }
            }
        }

        let row_count = all_x.len();
        let display_group_total = if is_stacked {
            1
        } else {
            plan.series_indices.len()
        };

        let mut df = DataFrame::new();
        df.add_column(Series::new("x", all_x));
        df.add_column(Series::new("y", all_y));
        df.add_column(Series::new("cat_idx", all_cat));
        df.add_column(Series::new("color", all_color));
        df.add_column(Series::new("group_position", all_pos));
        df.add_column(Series::new_constant(
            "group_total",
            DataValue::Integer(display_group_total as i64),
            row_count,
        ));
        df.add_column(Series::new("stack_base", all_base));

        df
    }
}
