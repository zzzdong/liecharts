use std::f64::consts::PI;

use vello_cpu::kurbo::Point;

use crate::{
    error::Result,
    option::SeriesOption,
    pipeline::{
        accessors::StyleAccess,
        data_processor::{DataProcessor, DataProcessorInput},
        dataframe::{DataFrame, DataValue, Series},
        mapper::{CoordinateMapper, PolarMapper},
    },
    visual::{
        FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement, Z_AXIS,
        Z_LABEL, Z_SERIES_POINT,
    },
};

pub struct PolarScatterProcessor;

impl Default for PolarScatterProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PolarScatterProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl DataProcessor for PolarScatterProcessor {
    fn to_dataframe(
        &self,
        series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        let polar_scatter = match series {
            SeriesOption::PolarScatter(p) => p,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected PolarScatter series".into(),
                ));
            }
        };

        let mut df = DataFrame::new();

        let angles: Vec<DataValue> = polar_scatter
            .data
            .iter()
            .map(|d| DataValue::Float(d.angle))
            .collect();
        let radii: Vec<DataValue> = polar_scatter
            .data
            .iter()
            .map(|d| DataValue::Float(d.radius))
            .collect();
        let symbol_sizes: Vec<DataValue> = polar_scatter
            .data
            .iter()
            .map(|d| DataValue::Float(d.symbol_size.unwrap_or(10.0)))
            .collect();

        df.add_column(Series::new("angle", angles));
        df.add_column(Series::new("radius", radii));
        df.add_column(Series::new("symbol_size", symbol_sizes));

        Ok(df)
    }

    fn transform(&self, df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame> {
        let series = &input.option.series[input.series_idx];
        let polar_scatter = match series {
            SeriesOption::PolarScatter(p) => p,
            _ => return Ok(df),
        };

        let max_data_radius = polar_scatter
            .data
            .iter()
            .map(|d| d.radius)
            .fold(0.0_f64, f64::max)
            .max(1.0);

        let mut df = df;
        df.add_column(Series::new_constant(
            "max_data_radius",
            DataValue::Float(max_data_radius),
            df.row_count(),
        ));

        Ok(df)
    }

    fn mapper(&self) -> Box<dyn CoordinateMapper> {
        Box::new(PolarMapper::new(0.8))
    }

    fn to_visual_elements(
        &self,
        df: &DataFrame,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let series = &input.option.series[input.series_idx];
        let polar_scatter = match series {
            SeriesOption::PolarScatter(p) => p,
            _ => return Ok(Vec::new()),
        };

        let colors = &input.colors;
        let style = StyleAccess::from_df(df, colors.get_default_color());

        let cx = df
            .get_column("center_x")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(400.0);
        let cy = df
            .get_column("center_y")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(300.0);
        let max_radius = df
            .get_column("max_radius")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(150.0);
        let max_data_radius = df
            .get_column("max_data_radius")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(1.0);

        let center = Point::new(cx, cy);
        let mut elements = Vec::new();

        let grid_levels = 5;
        for i in 1..=grid_levels {
            let r = max_radius * i as f64 / grid_levels as f64;
            elements.push(VisualElement::Circle {
                center,
                radius: r,
                style: FillStrokeStyle {
                    fill: None,
                    stroke: Some(Stroke {
                        color: colors.grid_line_color,
                        width: 1.0,
                    }),
                },
                z_index: Z_AXIS,
            });

            let label_value = max_data_radius * i as f64 / grid_levels as f64;
            elements.push(VisualElement::TextRun {
                text: format!("{:.0}", label_value),
                position: Point::new(center.x + r + 4.0, center.y),
                style: crate::visual::TextStyle {
                    font_size: 10.0,
                    color: colors.axis_label_color,
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

        let angle_labels = vec![
            (0.0, "N"),
            (45.0, "NE"),
            (90.0, "E"),
            (135.0, "SE"),
            (180.0, "S"),
            (225.0, "SW"),
            (270.0, "W"),
            (315.0, "NW"),
        ];

        for (angle_deg, label) in &angle_labels {
            let angle_rad = angle_deg * PI / 180.0 - PI / 2.0;
            let end_x = center.x + max_radius * angle_rad.cos();
            let end_y = center.y + max_radius * angle_rad.sin();

            elements.push(VisualElement::Line {
                start: center,
                end: Point::new(end_x, end_y),
                style: StrokeStyle::new(colors.grid_line_color, 1.0),
                z_index: Z_AXIS,
            });

            let label_radius = max_radius + 20.0;
            let label_x = center.x + label_radius * angle_rad.cos();
            let label_y = center.y + label_radius * angle_rad.sin();

            elements.push(VisualElement::TextRun {
                text: label.to_string(),
                position: Point::new(label_x, label_y),
                style: crate::visual::TextStyle {
                    font_size: 11.0,
                    color: colors.axis_label_color,
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

        for dp in &polar_scatter.data {
            let angle_rad = dp.angle * PI / 180.0 - PI / 2.0;
            let r_ratio = dp.radius / max_data_radius;
            let r = max_radius * r_ratio;

            let px = center.x + r * angle_rad.cos();
            let py = center.y + r * angle_rad.sin();
            let symbol_size = dp.symbol_size.unwrap_or(10.0);

            elements.push(VisualElement::Circle {
                center: Point::new(px, py),
                radius: symbol_size / 2.0,
                style: FillStrokeStyle {
                    fill: Some(style.color(0).set_alpha(0.7)),
                    stroke: Some(Stroke {
                        color: colors.border_color,
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_POINT,
            });
        }

        Ok(elements)
    }
}
