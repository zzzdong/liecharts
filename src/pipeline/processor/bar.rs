use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    option::{DataPoint, SeriesOption},
    pipeline::{
        accessors::{CartesianGeometry, GroupInfo, StyleAccess},
        data_processor::{DataProcessor, DataProcessorInput},
        dataframe::{DataFrame, DataValue, Series},
        mapper::{CartesianMapper, CoordinateMapper},
        sampling::SamplingProcessor,
        types::{AxisType, SeriesSpec},
    },
    visual::{
        FillStrokeStyle, Stroke, TextAlign, TextBaseline, VisualElement, Z_LABEL, Z_SERIES_FILL,
    },
};

pub struct BarProcessor;

impl Default for BarProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BarProcessor {
    pub fn new() -> Self {
        Self
    }

    fn extract_value(dp: &DataPoint) -> f64 {
        match dp {
            DataPoint::Value(v) => *v,
            DataPoint::Named(_, v) => *v,
            DataPoint::XY(_, y) => *y,
        }
    }

    /// 从 series_spec 获取 bar_width_ratio
    fn get_bar_width(input: &DataProcessorInput) -> f64 {
        input
            .series_spec
            .and_then(|s| s.bar_width)
            .unwrap_or(0.6)
    }

    /// 检查是否为水平柱状图（Y轴为 Category 类型）
    fn is_horizontal(input: &DataProcessorInput, series_spec: &SeriesSpec) -> bool {
        input
            .chart_spec
            .and_then(|cs| cs.y_axes.get(series_spec.y_axis_index))
            .map(|a| a.axis_type == AxisType::Category)
            .unwrap_or(false)
    }
}

impl DataProcessor for BarProcessor {
    fn to_dataframe(
        &self,
        series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        let bar = match series {
            SeriesOption::Bar(b) => b,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Bar series".into(),
                ));
            }
        };

        let mut df = DataFrame::new();

        let y_values: Vec<DataValue> = bar
            .data
            .iter()
            .map(|dp| DataValue::Float(Self::extract_value(dp)))
            .collect();

        let x_values: Vec<DataValue> = bar
            .data
            .iter()
            .enumerate()
            .map(|(i, dp)| match dp {
                DataPoint::Named(name, _) => DataValue::String(name.clone()),
                _ => DataValue::Integer(i as i64),
            })
            .collect();

        let cat_idx_values: Vec<DataValue> = (0..bar.data.len())
            .map(|i| DataValue::Integer(i as i64))
            .collect();

        df.add_column(Series::new("x", x_values));
        df.add_column(Series::new("y", y_values));
        df.add_column(Series::new("cat_idx", cat_idx_values));

        Ok(df)
    }

    fn transform(&self, mut df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame> {
        // 获取 series_spec（当前路径下始终有值）
        let series_spec = input.series_spec.as_ref().ok_or_else(|| {
            crate::error::ChartError::DataError("BarProcessor: missing series_spec".into())
        })?;

        // 应用采样（如果配置了）
        if let Some((sampling_type, threshold)) = &series_spec.sampling {
            df = SamplingProcessor::sample(&df, *threshold, *sampling_type);
        }

        if df.get_column("color").is_none() {
            let series_color = input.colors.get_series_color(input.series_idx);
            df.add_column(Series::new_constant(
                "color",
                DataValue::Color(series_color),
                df.row_count(),
            ));
        }

        if df.get_column("group_total").is_none() {
            df.add_column(Series::new_constant(
                "group_total",
                DataValue::Integer(1),
                df.row_count(),
            ));
        }
        if df.get_column("group_position").is_none() {
            df.add_column(Series::new_constant(
                "group_position",
                DataValue::Integer(0),
                df.row_count(),
            ));
        }

        let bar_width_ratio = Self::get_bar_width(input);
        df.add_column(Series::new_constant(
            "bar_width_ratio",
            DataValue::Float(bar_width_ratio),
            df.row_count(),
        ));

        let is_horizontal = Self::is_horizontal(input, series_spec);

        if is_horizontal {
            let y_vals: Vec<DataValue> = (0..df.row_count())
                .map(|i| {
                    df.get_column("y")
                        .and_then(|c| c.data.get(i))
                        .cloned()
                        .unwrap_or(DataValue::Null)
                })
                .collect();
            df.add_column(Series::new("x", y_vals));
        }

        Ok(df)
    }

    fn resolve_y_axis_idx(&self, _series: &SeriesOption, input: &DataProcessorInput) -> usize {
        input
            .series_spec
            .map(|s| s.y_axis_index)
            .or_else(|| input.spec.y_axis_indices.first().copied())
            .unwrap_or(0)
    }

    fn mapper(&self) -> Box<dyn CoordinateMapper> {
        Box::new(CartesianMapper)
    }

    fn to_visual_elements(
        &self,
        df: &DataFrame,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let series_spec = input.series_spec.as_ref().ok_or_else(|| {
            crate::error::ChartError::DataError("BarProcessor: missing series_spec".into())
        })?;

        let geom = CartesianGeometry::from_df(df)?;
        let group = GroupInfo::from_df(df);
        let style = StyleAccess::from_df(df, input.colors.get_default_color());

        let y_col = df.get_column("y").expect("y column should exist");
        let bar_width_ratio = df
            .get_column("bar_width_ratio")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(0.6);

        let spec = input.spec;
        let bounds = spec.bounds;

        let y_axis_idx = series_spec.y_axis_index;
        let is_horizontal = Self::is_horizontal(input, series_spec);

        let y_range = input.axis_ranges.get_y_range(y_axis_idx);
        let (y_min, y_max) = y_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 100.0));

        let row_count = geom.row_count();
        let mut elements = Vec::new();
        let mut label_elements = Vec::new();

        for i in 0..row_count {
            let value = y_col.as_f64(i).unwrap_or(0.0);
            let color = style.color(i);
            let center_offset = group.center_offset(i);

            if is_horizontal {
                let cat_count = (y_max - y_min).max(1.0);
                let cat_height = bounds.height() / cat_count;
                let group_height = cat_height * bar_width_ratio;
                let bar_height = group_height / group.total() as f64;

                let category_center = geom.py(i);
                let group_offset = center_offset * group_height;
                let center_y = category_center + group_offset;

                let right_x = geom.px(i);
                let left_x = geom.pbase(i, bounds.x0);
                let bar_left = left_x.min(right_x);
                let bar_w = (right_x - left_x).abs();
                let y = center_y - bar_height / 2.0;

                elements.push(VisualElement::Rect {
                    rect: Rect::new(bar_left, y, bar_left + bar_w, y + bar_height),
                    style: FillStrokeStyle {
                        fill: Some(color),
                        stroke: Some(Stroke {
                            color: input.colors.border_color,
                            width: 1.0,
                        }),
                    },
                    z_index: Z_SERIES_FILL,
                });

                let label_text = format!("{:.0}", value);
                let label_x = right_x + 5.0;
                label_elements.push(VisualElement::TextRun {
                    text: label_text,
                    position: Point::new(label_x, center_y),
                    style: crate::visual::TextStyle {
                        font_size: 11.0,
                        color,
                        align: TextAlign::Left,
                        vertical_align: TextBaseline::Middle,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                    z_index: Z_LABEL,
                });
            } else {
                let x_axis_idx = spec.x_axis_indices.first().copied().unwrap_or(0);
                let x_range = input.axis_ranges.get_x_range(x_axis_idx);
                let (x_min, x_max) = x_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 1.0));
                let cat_count = (x_max - x_min).max(1.0);
                let cat_width = bounds.width() / cat_count;
                let group_width = cat_width * bar_width_ratio;
                let bar_width = group_width / group.total() as f64;

                let group_offset = center_offset * group_width;
                let center_x = geom.px(i) + group_offset;

                let top_y = geom.py(i);
                let bottom_y = geom.pbase(i, bounds.y1);
                let bar_top = top_y.min(bottom_y);
                let bar_h = (top_y - bottom_y).abs();
                let x = center_x - bar_width / 2.0;

                elements.push(VisualElement::Rect {
                    rect: Rect::new(x, bar_top, x + bar_width, bar_top + bar_h),
                    style: FillStrokeStyle {
                        fill: Some(color),
                        stroke: Some(Stroke {
                            color: input.colors.border_color,
                            width: 1.0,
                        }),
                    },
                    z_index: Z_SERIES_FILL,
                });

                let label_text = format!("{:.0}", value);
                let (label_y, label_color, valign) = if bar_h > 25.0 {
                    (bar_top + 14.0, input.colors.border_color, TextBaseline::Top)
                } else {
                    (bar_top - 6.0, color, TextBaseline::Bottom)
                };
                label_elements.push(VisualElement::TextRun {
                    text: label_text,
                    position: Point::new(center_x, label_y),
                    style: crate::visual::TextStyle {
                        font_size: 11.0,
                        color: label_color,
                        align: TextAlign::Center,
                        vertical_align: valign,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                    z_index: Z_LABEL,
                });
            }
        }

        elements.extend(label_elements);
        Ok(elements)
    }

    /// 从 SeriesSpec 直接处理（跳过 to_dataframe，数据已在 DataFrame 中）
    fn process_from_spec(
        &self,
        series: &SeriesSpec,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let mut df = series.data.clone();

        // 添加必要的计算列
        // 应用采样（如果配置了）
        if let Some((sampling_type, threshold)) = &series.sampling {
            df = SamplingProcessor::sample(&df, *threshold, *sampling_type);
        }

        if df.get_column("color").is_none() {
            let series_color = input.colors.get_series_color(input.series_idx);
            df.add_column(Series::new_constant(
                "color",
                DataValue::Color(series_color),
                df.row_count(),
            ));
        }

        if df.get_column("group_total").is_none() {
            df.add_column(Series::new_constant(
                "group_total",
                DataValue::Integer(1),
                df.row_count(),
            ));
        }
        if df.get_column("group_position").is_none() {
            df.add_column(Series::new_constant(
                "group_position",
                DataValue::Integer(0),
                df.row_count(),
            ));
        }

        let bar_width_ratio = series.bar_width.unwrap_or(0.6);
        df.add_column(Series::new_constant(
            "bar_width_ratio",
            DataValue::Float(bar_width_ratio),
            df.row_count(),
        ));

        let is_horizontal = Self::is_horizontal(input, series);

        if is_horizontal {
            let y_vals: Vec<DataValue> = (0..df.row_count())
                .map(|i| {
                    df.get_column("y")
                        .and_then(|c| c.data.get(i))
                        .cloned()
                        .unwrap_or(DataValue::Null)
                })
                .collect();
            df.add_column(Series::new("x", y_vals));
        }

        // 坐标系映射
        self.mapper()
            .map_coordinates(&mut df, input, series.x_axis_index, series.y_axis_index);

        // 生成视觉元素
        self.to_visual_elements(&df, input)
    }
}