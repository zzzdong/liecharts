use crate::pipeline::{
    dataframe::{DataFrame, DataValue, Series},
    types::{ChartSpec, ChartType, SeriesConfig},
};

pub struct GroupedBarProcessor;

impl Default for GroupedBarProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupedBarProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn combine_to_dataframe(
        plan: &super::analyzer::GroupPlan,
        spec: &ChartSpec,
        colors: &crate::pipeline::types::ColorContext,
    ) -> DataFrame {
        let is_stacked = plan.group_type == super::analyzer::GroupType::Stacked;

        let mut all_x: Vec<DataValue> = Vec::new();
        let mut all_y: Vec<DataValue> = Vec::new();
        let mut all_cat: Vec<DataValue> = Vec::new();
        let mut all_color: Vec<DataValue> = Vec::new();
        let mut all_pos: Vec<DataValue> = Vec::new();
        let mut all_base: Vec<DataValue> = Vec::new();

        if is_stacked {
            // 获取所有 bar 系列的最大类别数
            let max_cats = plan
                .series_indices
                .iter()
                .filter_map(|&idx| {
                    let s = &spec.series[idx];
                    if s.chart_type == ChartType::Bar {
                        Some(s.data.row_count())
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(0);

            let mut stack_cums: Vec<f64> = vec![0.0; max_cats];

            for &global_idx in plan.series_indices.iter() {
                let s = &spec.series[global_idx];
                if s.chart_type != ChartType::Bar {
                    continue;
                }
                let color = colors.get_series_color(global_idx);
                let color_val = DataValue::Color(color);

                // 从 config 获取列名
                let (x_col_name, y_col_name) = match &s.config {
                    SeriesConfig::Bar(cfg) => (cfg.x_col.as_str(), cfg.y_col.as_str()),
                    _ => ("x", "y"),
                };
                let x_col = s.data.get_column(x_col_name);
                let y_col = s.data.get_column(y_col_name);

                if let (Some(x_series), Some(y_series)) = (x_col, y_col) {
                    for cat_idx in 0..s.data.row_count() {
                        let x_val = x_series
                            .data
                            .get(cat_idx)
                            .cloned()
                            .unwrap_or(DataValue::Integer(cat_idx as i64));
                        let raw_val = y_series.as_f64(cat_idx).unwrap_or(0.0);
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
            }
        } else {
            for (pos, &global_idx) in plan.series_indices.iter().enumerate() {
                let s = &spec.series[global_idx];
                if s.chart_type != ChartType::Bar {
                    continue;
                }
                let color = colors.get_series_color(global_idx);
                let color_val = DataValue::Color(color);

                // 从 config 获取列名
                let (x_col_name, y_col_name) = match &s.config {
                    SeriesConfig::Bar(cfg) => (cfg.x_col.as_str(), cfg.y_col.as_str()),
                    _ => ("x", "y"),
                };
                let x_col = s.data.get_column(x_col_name);
                let y_col = s.data.get_column(y_col_name);

                if let (Some(x_series), Some(y_series)) = (x_col, y_col) {
                    for cat_idx in 0..s.data.row_count() {
                        let x_val = x_series
                            .data
                            .get(cat_idx)
                            .cloned()
                            .unwrap_or(DataValue::Integer(cat_idx as i64));
                        let y_val = DataValue::Float(y_series.as_f64(cat_idx).unwrap_or(0.0));

                        all_x.push(x_val);
                        all_y.push(y_val);
                        all_cat.push(DataValue::Integer(cat_idx as i64));
                        all_color.push(color_val.clone());
                        all_pos.push(DataValue::Integer(pos as i64));
                        all_base.push(DataValue::Float(0.0));
                    }
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
