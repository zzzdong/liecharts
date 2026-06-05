use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    option::{DataPoint, SeriesOption},
    pipeline::{
        accessors::{CartesianGeometry, StyleAccess},
        data_processor::{DataProcessor, DataProcessorInput},
        dataframe::{DataFrame, DataValue, Series},
        mapper::{CartesianMapper, CoordinateMapper},
        sampling::SamplingProcessor,
        types::SeriesSpec,
    },
    visual::{
        FillStrokeStyle, Stroke, TextAlign, TextBaseline, VisualElement, Z_LABEL, Z_SERIES_POINT,
    },
};

pub struct ScatterProcessor;

impl Default for ScatterProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ScatterProcessor {
    pub fn new() -> Self {
        Self
    }

    fn extract_xy(dp: &DataPoint) -> Option<(f64, f64)> {
        match dp {
            DataPoint::XY(x, y) => Some((*x, *y)),
            DataPoint::Named(_, v) => Some((0.0, *v)),
            DataPoint::Value(v) => Some((0.0, *v)),
        }
    }

    fn extract_name(dp: &DataPoint) -> Option<String> {
        match dp {
            DataPoint::Named(name, _) => Some(name.clone()),
            _ => None,
        }
    }
}

impl DataProcessor for ScatterProcessor {
    fn to_dataframe(
        &self,
        series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        let scatter = match series {
            SeriesOption::Scatter(s) => s,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Scatter series".into(),
                ));
            }
        };

        let mut df = DataFrame::new();

        let mut x_values = Vec::new();
        let mut y_values = Vec::new();
        let mut name_values = Vec::new();

        for dp in &scatter.data {
            if let Some((x, y)) = Self::extract_xy(dp) {
                x_values.push(DataValue::Float(x));
                y_values.push(DataValue::Float(y));
                name_values.push(
                    Self::extract_name(dp)
                        .map(DataValue::String)
                        .unwrap_or(DataValue::Null),
                );
            }
        }

        df.add_column(Series::new("x", x_values));
        df.add_column(Series::new("y", y_values));
        df.add_column(Series::new("name", name_values));

        Ok(df)
    }

    fn transform(&self, mut df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame> {
        let series = &input.option.series[input.series_idx];
        let scatter = match series {
            SeriesOption::Scatter(s) => s,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Scatter series".into(),
                ));
            }
        };

        // 应用采样（如果配置了）
        if let Some(sampling) = &scatter.sampling {
            df = SamplingProcessor::sample(&df, sampling.threshold, sampling.ty);
        }

        let series_color = input.colors.get_series_color(input.series_idx);
        df.add_column(Series::new_constant(
            "color",
            DataValue::Color(series_color),
            df.row_count(),
        ));

        Ok(df)
    }

    fn resolve_y_axis_idx(&self, series: &SeriesOption, input: &DataProcessorInput) -> usize {
        match series {
            SeriesOption::Scatter(s) => s
                .y_axis_index
                .or_else(|| input.spec.y_axis_indices.first().copied())
                .unwrap_or(0),
            _ => input.spec.y_axis_indices.first().copied().unwrap_or(0),
        }
    }

    fn mapper(&self) -> Box<dyn CoordinateMapper> {
        Box::new(CartesianMapper)
    }

    fn to_visual_elements(
        &self,
        df: &DataFrame,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let series = &input.option.series[input.series_idx];
        let scatter = match series {
            SeriesOption::Scatter(s) => s,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Scatter series".into(),
                ));
            }
        };

        let geom = CartesianGeometry::from_df(df)?;
        let style = StyleAccess::from_df(df, input.colors.get_default_color());
        let name_col = df.get_column("name");
        let symbol_size = scatter.symbol_size.unwrap_or(10.0);

        let mut elements = Vec::new();

        for i in 0..geom.row_count() {
            let color = style.color(i);
            let px = geom.px(i);
            let py = geom.py(i);

            elements.push(VisualElement::Circle {
                center: Point::new(px, py),
                radius: symbol_size / 2.0,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: Some(Stroke {
                        color: input.colors.border_color,
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_POINT,
            });

            if let Some(name_col) = name_col
                && let Some(name) = name_col.as_string(i)
                && !name.is_empty()
            {
                elements.push(VisualElement::TextRun {
                    text: name,
                    position: Point::new(px + symbol_size / 2.0 + 3.0, py),
                    style: crate::visual::TextStyle {
                        font_size: 10.0,
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
            }
        }

        Ok(elements)
    }

    /// 从 SeriesSpec 直接处理（跳过 to_dataframe，数据已在 DataFrame 中）
    fn process_from_spec(
        &self,
        series: &SeriesSpec,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let mut df = series.data.clone();

        // 应用采样（如果配置了）
        if let Some((sampling_type, threshold)) = &series.sampling {
            df = SamplingProcessor::sample(&df, *threshold, *sampling_type);
        }

        // 添加颜色列
        if df.get_column("color").is_none() {
            let series_color = input.colors.get_series_color(input.series_idx);
            df.add_column(Series::new_constant(
                "color",
                DataValue::Color(series_color),
                df.row_count(),
            ));
        }

        // 坐标系映射
        self.mapper()
            .map_coordinates(&mut df, input, series.x_axis_index, series.y_axis_index);

        let symbol_size = match &series.config {
            crate::pipeline::types::SeriesConfig::Scatter(cfg) => cfg.symbol_size,
            _ => 10.0,
        };
        let geom = CartesianGeometry::from_df(&df)?;
        let style = StyleAccess::from_df(&df, input.colors.get_default_color());
        let name_col = df.get_column("name");

        let mut elements = Vec::new();

        for i in 0..geom.row_count() {
            let color = style.color(i);
            let px = geom.px(i);
            let py = geom.py(i);

            elements.push(VisualElement::Circle {
                center: Point::new(px, py),
                radius: symbol_size / 2.0,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: Some(Stroke {
                        color: input.colors.border_color,
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_POINT,
            });

            if let Some(name_col) = name_col
                && let Some(name) = name_col.as_string(i)
                && !name.is_empty()
            {
                elements.push(VisualElement::TextRun {
                    text: name,
                    position: Point::new(px + symbol_size / 2.0 + 3.0, py),
                    style: crate::visual::TextStyle {
                        font_size: 10.0,
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
            }
        }

        Ok(elements)
    }
}
