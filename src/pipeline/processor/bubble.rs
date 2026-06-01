use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    option::{BubbleDataPoint, SeriesOption},
    pipeline::{
        accessors::{CartesianGeometry, StyleAccess},
        data_processor::{DataProcessor, DataProcessorInput},
        dataframe::{DataFrame, DataValue, Series},
        mapper::{CartesianMapper, CoordinateMapper},
    },
    visual::{
        FillStrokeStyle, Stroke, TextAlign, TextBaseline, VisualElement, Z_LABEL, Z_SERIES_POINT,
    },
};

pub struct BubbleProcessor;

impl BubbleProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl DataProcessor for BubbleProcessor {
    fn to_dataframe(
        &self,
        series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        let bubble = match series {
            SeriesOption::Bubble(b) => b,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Bubble series".into(),
                ));
            }
        };

        let mut df = DataFrame::new();

        let mut x_values = Vec::new();
        let mut y_values = Vec::new();
        let mut size_values = Vec::new();
        let mut name_values = Vec::new();

        for item in &bubble.data {
            let BubbleDataPoint { x, y, size, name } = item;
            x_values.push(DataValue::Float(*x));
            y_values.push(DataValue::Float(*y));
            size_values.push(size.map(|s| DataValue::Float(s)).unwrap_or(DataValue::Null));
            name_values.push(
                name.as_ref()
                    .map(|n| DataValue::String(n.clone()))
                    .unwrap_or(DataValue::Null),
            );
        }

        df.add_column(Series::new("x", x_values));
        df.add_column(Series::new("y", y_values));
        df.add_column(Series::new("size", size_values));
        df.add_column(Series::new("name", name_values));

        Ok(df)
    }

    fn transform(&self, mut df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame> {
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
            SeriesOption::Bubble(b) => b
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
        let bubble = match series {
            SeriesOption::Bubble(b) => b,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Bubble series".into(),
                ));
            }
        };

        let geom = CartesianGeometry::from_df(df)?;
        let style = StyleAccess::from_df(df, input.colors.get_default_color());
        let size_col = df.get_column("size");
        let name_col = df.get_column("name");
        let scale = bubble.symbol_size_scale.unwrap_or(1.0);

        let mut elements = Vec::new();

        for i in 0..geom.row_count() {
            let color = style.color(i);
            let px = geom.px(i);
            let py = geom.py(i);
            let size = size_col.and_then(|c| c.as_f64(i)).unwrap_or(20.0);
            let radius = size.sqrt() * scale;

            let mut fill_color = color;
            fill_color.a = 180;

            elements.push(VisualElement::Circle {
                center: Point::new(px, py),
                radius,
                style: FillStrokeStyle {
                    fill: Some(fill_color),
                    stroke: Some(Stroke {
                        color: input.colors.border_color,
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_POINT,
            });

            if let Some(name_col) = name_col {
                if let Some(name) = name_col.as_string(i) {
                    if !name.is_empty() {
                        elements.push(VisualElement::TextRun {
                            text: name.clone(),
                            position: Point::new(px, py),
                            style: crate::visual::TextStyle {
                                font_size: 10.0,
                                color: input.colors.text_color,
                                align: TextAlign::Center,
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
            }
        }

        Ok(elements)
    }
}
