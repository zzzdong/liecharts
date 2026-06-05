use vello_cpu::kurbo::{BezPath, Point};

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
        Color, FillStrokeStyle, Stroke, StrokeStyle, VisualElement, Z_SERIES_FILL, Z_SERIES_LINE,
        Z_SERIES_POINT,
    },
};

pub struct LineProcessor;

impl Default for LineProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl LineProcessor {
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

    fn build_smooth_path(points: &[Point]) -> BezPath {
        let n = points.len();
        let mut path = BezPath::new();
        let tension = 0.5;

        path.move_to(points[0]);

        for i in 0..n - 1 {
            let p0 = if i == 0 { points[0] } else { points[i - 1] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < n {
                points[i + 2]
            } else {
                points[n - 1]
            };

            let cp1_x = p1.x + (p2.x - p0.x) * tension / 3.0;
            let cp1_y = p1.y + (p2.y - p0.y) * tension / 3.0;
            let cp2_x = p2.x - (p3.x - p1.x) * tension / 3.0;
            let cp2_y = p2.y - (p3.y - p1.y) * tension / 3.0;

            path.curve_to(Point::new(cp1_x, cp1_y), Point::new(cp2_x, cp2_y), p2);
        }

        path
    }

    fn build_area_path(points: &[Point], baseline_y: f64) -> BezPath {
        let mut path = BezPath::new();
        path.move_to(points[0]);
        for p in &points[1..] {
            path.line_to(*p);
        }
        path.line_to(Point::new(points.last().unwrap().x, baseline_y));
        path.line_to(Point::new(points[0].x, baseline_y));
        path.close_path();
        path
    }
}

impl DataProcessor for LineProcessor {
    fn to_dataframe(
        &self,
        series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        let line = match series {
            SeriesOption::Line(l) => l,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Line series".into(),
                ));
            }
        };

        let mut df = DataFrame::new();

        let has_numeric_x = line.data.iter().any(|d| matches!(d, DataPoint::XY(_, _)));

        let x_values: Vec<DataValue> = if has_numeric_x {
            line.data
                .iter()
                .map(|dp| match dp {
                    DataPoint::XY(x, _) => DataValue::Float(*x),
                    _ => DataValue::Null,
                })
                .collect()
        } else {
            (0..line.data.len())
                .map(|i| DataValue::Integer(i as i64))
                .collect()
        };

        let y_values: Vec<DataValue> = line
            .data
            .iter()
            .map(|dp| DataValue::Float(Self::extract_value(dp)))
            .collect();

        df.add_column(Series::new("x", x_values));
        df.add_column(Series::new("y", y_values));

        Ok(df)
    }

    fn transform(&self, mut df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame> {
        let series = &input.option.series[input.series_idx];
        let line = match series {
            SeriesOption::Line(l) => l,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Line series".into(),
                ));
            }
        };

        // 应用采样（如果配置了）
        if let Some(sampling) = &line.sampling {
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
            SeriesOption::Line(l) => l
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
        let line = match series {
            SeriesOption::Line(l) => l,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Line series".into(),
                ));
            }
        };

        let geom = CartesianGeometry::from_df(df)?;
        let style = StyleAccess::from_df(df, input.colors.get_default_color());

        let points = geom.collect_points();
        if points.is_empty() {
            return Ok(Vec::new());
        }

        let series_color = style.color(0);

        let line_width = line
            .line_style
            .as_ref()
            .and_then(|ls| ls.width)
            .unwrap_or(2.0);

        let smooth = line.smooth.unwrap_or(false);

        let (area_color, area_opacity) = line
            .area_style
            .as_ref()
            .map(|a| {
                let color = a
                    .color
                    .map(|c| Color::new(c.r, c.g, c.b))
                    .unwrap_or(series_color);
                let opacity = a.opacity.unwrap_or(0.5);
                (color, opacity)
            })
            .map(|(c, o)| (Some(c), Some(o)))
            .unwrap_or((None, None));

        let mut elements = Vec::new();

        if points.len() >= 2
            && area_color.is_some()
            && let Some(ac) = area_color
        {
            let opacity = area_opacity.unwrap_or(0.5);
            let alpha = (255.0 * opacity).clamp(0.0, 255.0) as u8;
            let mut fill_color = ac;
            fill_color.a = alpha;

            let path = Self::build_area_path(&points, input.spec.bounds.y1);
            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(fill_color),
                    stroke: None,
                },
                z_index: Z_SERIES_FILL,
            });
        }

        if points.len() >= 2 {
            if smooth {
                let path = Self::build_smooth_path(&points);
                elements.push(VisualElement::Path {
                    path,
                    style: FillStrokeStyle {
                        fill: None,
                        stroke: Some(Stroke {
                            color: series_color,
                            width: line_width,
                        }),
                    },
                    z_index: Z_SERIES_LINE,
                });
            } else {
                elements.push(VisualElement::Polyline {
                    points: points.clone(),
                    style: StrokeStyle {
                        color: series_color,
                        width: line_width,
                    },
                    z_index: Z_SERIES_LINE,
                });
            }
        }

        let show_symbol = line
            .symbol
            .as_ref()
            .map(|s| !matches!(s, crate::option::SymbolType::None))
            .unwrap_or(true);
        let symbol_size = line.symbol_size.unwrap_or(8.0);

        if show_symbol {
            for pt in &points {
                elements.push(VisualElement::Circle {
                    center: *pt,
                    radius: symbol_size / 2.0,
                    style: FillStrokeStyle {
                        fill: Some(input.colors.border_color),
                        stroke: Some(Stroke {
                            color: series_color,
                            width: 2.0,
                        }),
                    },
                    z_index: Z_SERIES_POINT,
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

        // 读取 SeriesSpec 中的配置（从 config 字段获取）
        let (line_width, smooth, area_color, area_opacity, show_symbol, symbol_size) = match &series
            .config
        {
            crate::pipeline::types::SeriesConfig::Line(cfg) => {
                let show_sym = !matches!(cfg.symbol_type, crate::pipeline::types::SymbolType::None);
                (
                    cfg.line_width,
                    cfg.smooth,
                    cfg.area_color,
                    cfg.area_opacity,
                    show_sym,
                    cfg.symbol_size,
                )
            }
            _ => (2.0, false, None, 0.5, true, 8.0),
        };

        let geom = CartesianGeometry::from_df(&df)?;
        let style = StyleAccess::from_df(&df, input.colors.get_default_color());

        let points = geom.collect_points();
        if points.is_empty() {
            return Ok(Vec::new());
        }

        let series_color = style.color(0);
        let mut elements = Vec::new();

        // 面积填充
        if points.len() >= 2
            && let Some(ac) = area_color
        {
            let alpha = (255.0 * area_opacity).clamp(0.0, 255.0) as u8;
            let mut fill_color = ac;
            fill_color.a = alpha;

            let path = Self::build_area_path(&points, input.spec.bounds.y1);
            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(fill_color),
                    stroke: None,
                },
                z_index: Z_SERIES_FILL,
            });
        }

        // 折线
        if points.len() >= 2 {
            if smooth {
                let path = Self::build_smooth_path(&points);
                elements.push(VisualElement::Path {
                    path,
                    style: FillStrokeStyle {
                        fill: None,
                        stroke: Some(Stroke {
                            color: series_color,
                            width: line_width,
                        }),
                    },
                    z_index: Z_SERIES_LINE,
                });
            } else {
                elements.push(VisualElement::Polyline {
                    points: points.clone(),
                    style: StrokeStyle {
                        color: series_color,
                        width: line_width,
                    },
                    z_index: Z_SERIES_LINE,
                });
            }
        }

        // 标记点
        if show_symbol {
            for pt in &points {
                elements.push(VisualElement::Circle {
                    center: *pt,
                    radius: symbol_size / 2.0,
                    style: FillStrokeStyle {
                        fill: Some(input.colors.border_color),
                        stroke: Some(Stroke {
                            color: series_color,
                            width: 2.0,
                        }),
                    },
                    z_index: Z_SERIES_POINT,
                });
            }
        }

        Ok(elements)
    }
}
