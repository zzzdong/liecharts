//! ChartSpec → ChartOption 兼容转换层
//!
//! 将新的 ChartSpec 转换为旧的 ChartOption，以便复用基于旧类型的 processor 管线。
//! 这是临时的过渡方案，待所有 processor 迁移到新类型后移除。

use crate::{
    option::{
        self, AxisOption, AxisType, BarSeriesOption, BubbleDataPoint, BubbleSeriesOption,
        CandlestickSeriesOption, ChartOption, GaugeSeriesOption, GridOption, LegendOption,
        LineSeriesOption, PieSeriesOption, PolarBarSeriesOption, PolarScatterSeriesOption,
        PositionOption, PositionPreset, RadarSeriesOption, ScatterSeriesOption, SeriesOption,
        TableSeriesOption, TitleOption,
    },
    pipeline::{
        dataframe::DataValue,
        types::{ChartSpec, ChartType, GridSpec, SeriesSpec},
    },
    sampling::{SamplingOption, SamplingType},
};

/// 将 ChartSpec 转换为旧的 ChartOption
pub fn chart_spec_to_chart_option(spec: &ChartSpec) -> ChartOption {
    let mut option = ChartOption::default();

    // Title
    option.title = spec.title.as_ref().map(|t| TitleOption {
        text: t.text.clone(),
        subtext: t.subtext.clone(),
        left: Some(PositionOption::Preset(PositionPreset::Center)),
        top: Some(PositionOption::Pixel(20.0)),
        text_style: None,
        subtext_style: None,
    });

    // Legend
    if let Some(legend) = &spec.legend {
        option.legend = Some(LegendOption {
            show: Some(true),
            data: Some(legend.data.clone()),
            left: Some(PositionOption::Preset(PositionPreset::Center)),
            top: Some(PositionOption::Preset(PositionPreset::Auto)),
            orient: None,
            text_style: None,
            item_width: None,
            item_height: None,
            symbol_size: None,
        });
    }

    // Grids
    option.grid = spec.grids.iter().map(grid_to_grid_option).collect();

    // X Axes
    option.x_axis = spec.x_axes.iter().map(axis_to_axis_option).collect();

    // Y Axes
    option.y_axis = spec.y_axes.iter().map(axis_to_axis_option).collect();

    // Series
    option.series = spec.series.iter().map(series_to_series_option).collect();

    option
}

fn grid_to_grid_option(g: &GridSpec) -> GridOption {
    GridOption {
        left: g.left.map(PositionOption::Pixel),
        right: g.right.map(PositionOption::Pixel),
        top: g.top.map(PositionOption::Pixel),
        bottom: g.bottom.map(PositionOption::Pixel),
        contain_label: if g.contain_label { Some(true) } else { None },
    }
}

fn axis_to_axis_option(a: &crate::pipeline::types::AxisSpec) -> AxisOption {
    let old_type = match a.axis_type {
        crate::pipeline::types::AxisType::Value => AxisType::Value,
        crate::pipeline::types::AxisType::Category => AxisType::Category,
        crate::pipeline::types::AxisType::Time => AxisType::Time,
        crate::pipeline::types::AxisType::Log => AxisType::Log,
    };
    let old_position = match a.position {
        crate::pipeline::types::AxisPosition::Left => crate::option::AxisPosition::Left,
        crate::pipeline::types::AxisPosition::Right => crate::option::AxisPosition::Right,
        crate::pipeline::types::AxisPosition::Bottom => crate::option::AxisPosition::Bottom,
        crate::pipeline::types::AxisPosition::Top => crate::option::AxisPosition::Top,
    };

    let categories = if a.categories.is_empty() {
        None
    } else {
        Some(a.categories.clone())
    };

    AxisOption {
        axis_type: Some(old_type),
        data: categories,
        name: a.name.clone(),
        min: a.min,
        max: a.max,
        boundary_gap: Some(a.boundary_gap),
        position: Some(old_position),
        grid_index: Some(a.grid_index),
        ..Default::default()
    }
}

fn series_to_series_option(s: &SeriesSpec) -> SeriesOption {
    let data = dataframe_to_datapoints(&s.data, &s.x_col, &s.y_col);

    let sampling = s.sampling.map(|(ty, threshold)| {
        SamplingOption::new(
            match ty {
                crate::sampling::SamplingType::Lttb => SamplingType::Lttb,
                crate::sampling::SamplingType::Average => SamplingType::Average,
                crate::sampling::SamplingType::Max => SamplingType::Max,
                crate::sampling::SamplingType::Min => SamplingType::Min,
            },
            threshold,
        )
    });

    match s.chart_type {
        ChartType::Line => SeriesOption::Line(LineSeriesOption {
            name: Some(s.name.clone()),
            data,
            stack: s.stack.clone(),
            y_axis_index: Some(s.y_axis_index),
            grid_index: Some(s.grid_index),
            smooth: Some(s.smooth),
            sampling,
            ..Default::default()
        }),
        ChartType::Bar => SeriesOption::Bar(BarSeriesOption {
            name: Some(s.name.clone()),
            data,
            stack: s.stack.clone(),
            y_axis_index: Some(s.y_axis_index),
            grid_index: Some(s.grid_index),
            group_index: Some(s.group_index),
            sampling,
            ..Default::default()
        }),
        ChartType::Pie => SeriesOption::Pie(PieSeriesOption {
            name: Some(s.name.clone()),
            data,
            grid_index: Some(s.grid_index),
            ..Default::default()
        }),
        ChartType::Scatter => SeriesOption::Scatter(ScatterSeriesOption {
            name: Some(s.name.clone()),
            data,
            y_axis_index: Some(s.y_axis_index),
            grid_index: Some(s.grid_index),
            sampling,
            ..Default::default()
        }),
        ChartType::Bubble => SeriesOption::Bubble(BubbleSeriesOption {
            name: Some(s.name.clone()),
            data: data
                .into_iter()
                .map(|dp| match dp {
                    option::DataPoint::XY(x, y) => BubbleDataPoint {
                        x,
                        y,
                        size: None,
                        name: None,
                    },
                    option::DataPoint::Named(name, y) => BubbleDataPoint {
                        x: 0.0,
                        y,
                        size: None,
                        name: Some(name),
                    },
                    option::DataPoint::Value(y) => BubbleDataPoint {
                        x: 0.0,
                        y,
                        size: None,
                        name: None,
                    },
                })
                .collect(),
            y_axis_index: Some(s.y_axis_index),
            grid_index: Some(s.grid_index),
            ..Default::default()
        }),
        ChartType::Candlestick => SeriesOption::Candlestick(CandlestickSeriesOption {
            name: Some(s.name.clone()),
            data: vec![], // Candlestick 需要特殊处理
            grid_index: Some(s.grid_index),
            ..Default::default()
        }),
        ChartType::Radar => SeriesOption::Radar(RadarSeriesOption {
            name: Some(s.name.clone()),
            data: vec![],
            ..Default::default()
        }),
        ChartType::PolarBar => SeriesOption::PolarBar(PolarBarSeriesOption {
            name: Some(s.name.clone()),
            data: vec![],
            ..Default::default()
        }),
        ChartType::PolarScatter => SeriesOption::PolarScatter(PolarScatterSeriesOption {
            name: Some(s.name.clone()),
            data: vec![],
            ..Default::default()
        }),
        ChartType::Gauge => SeriesOption::Gauge(GaugeSeriesOption {
            name: Some(s.name.clone()),
            data: vec![],
            ..Default::default()
        }),
        ChartType::Table => SeriesOption::Table(TableSeriesOption {
            name: Some(s.name.clone()),
            ..Default::default()
        }),
    }
}

/// 从 DataFrame 中提取数据点，转换为旧的 DataPoint 格式
fn dataframe_to_datapoints(
    df: &crate::pipeline::dataframe::DataFrame,
    x_col: &str,
    y_col: &str,
) -> Vec<option::DataPoint> {
    let row_count = df.row_count();
    if row_count == 0 {
        return vec![];
    }

    let x_col_data = df.get_column(x_col);
    let y_col_data = df.get_column(y_col);

    let y_data: Vec<f64> = y_col_data
        .map(|c| (0..row_count).filter_map(|i| c.as_f64(i)).collect())
        .unwrap_or_default();

    // 判断 x 列类型
    let has_string_x = x_col_data
        .map(|c| matches!(c.data.first(), Some(DataValue::String(_))))
        .unwrap_or(false);

    let has_numeric_x = x_col_data
        .map(|c| {
            (0..row_count).any(|i| {
                matches!(
                    c.data.get(i),
                    Some(DataValue::Float(_) | DataValue::Integer(_))
                )
            })
        })
        .unwrap_or(false);

    if has_string_x {
        // Named 模式: ("category", value)
        (0..row_count)
            .map(|i| {
                let name = x_col_data
                    .and_then(|c| match c.data.get(i) {
                        Some(DataValue::String(s)) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let y = y_data.get(i).copied().unwrap_or(0.0);
                option::DataPoint::Named(name, y)
            })
            .collect()
    } else if has_numeric_x {
        // XY 模式: (x, y)
        (0..row_count)
            .map(|i| {
                let x = x_col_data.and_then(|c| c.as_f64(i)).unwrap_or(0.0);
                let y = y_data.get(i).copied().unwrap_or(0.0);
                option::DataPoint::XY(x, y)
            })
            .collect()
    } else {
        // Value 模式: y
        y_data.into_iter().map(option::DataPoint::Value).collect()
    }
}

/// 将旧的 ChartOption 转换为新的 ChartSpec（反向兼容）
pub fn chart_option_to_chart_spec(option: &ChartOption, width: u32, height: u32) -> ChartSpec {
    use crate::pipeline::{
        dataframe::DataFrame,
        types::{
            AxisPosition, AxisSpec, AxisType as NewAxisType, GridSpec, ItemStyleSpec, LegendSpec,
            SeriesSpec, TitleSpec,
        },
    };

    // Grids
    let grids: Vec<GridSpec> = option
        .grid
        .iter()
        .map(|g| {
            let left = g.left.as_ref().and_then(|p| match p {
                PositionOption::Pixel(v) => Some(*v),
                _ => None,
            });
            let right = g.right.as_ref().and_then(|p| match p {
                PositionOption::Pixel(v) => Some(*v),
                _ => None,
            });
            let top = g.top.as_ref().and_then(|p| match p {
                PositionOption::Pixel(v) => Some(*v),
                _ => None,
            });
            let bottom = g.bottom.as_ref().and_then(|p| match p {
                PositionOption::Pixel(v) => Some(*v),
                _ => None,
            });
            GridSpec {
                left,
                right,
                top,
                bottom,
                contain_label: g.contain_label.unwrap_or(false),
            }
        })
        .collect();

    // X Axes
    let x_axes: Vec<AxisSpec> = option
        .x_axis
        .iter()
        .map(|a| {
            let new_axis_type = match a.axis_type.unwrap_or(crate::option::AxisType::Category) {
                crate::option::AxisType::Value => NewAxisType::Value,
                crate::option::AxisType::Category => NewAxisType::Category,
                crate::option::AxisType::Time => NewAxisType::Time,
                crate::option::AxisType::Log => NewAxisType::Log,
            };
            let new_position = match a.position.unwrap_or(crate::option::AxisPosition::Bottom) {
                crate::option::AxisPosition::Bottom => AxisPosition::Bottom,
                crate::option::AxisPosition::Top => AxisPosition::Top,
                crate::option::AxisPosition::Left => AxisPosition::Left,
                crate::option::AxisPosition::Right => AxisPosition::Right,
            };
            AxisSpec {
                axis_type: new_axis_type,
                position: new_position,
                grid_index: a.grid_index.unwrap_or(0),
                min: a.min,
                max: a.max,
                name: a.name.clone(),
                categories: a.data.clone().unwrap_or_default(),
                boundary_gap: a.boundary_gap.unwrap_or(true),
            }
        })
        .collect();

    // Y Axes
    let y_axes: Vec<AxisSpec> = option
        .y_axis
        .iter()
        .map(|a| {
            let new_axis_type = match a.axis_type.unwrap_or(crate::option::AxisType::Value) {
                crate::option::AxisType::Value => NewAxisType::Value,
                crate::option::AxisType::Category => NewAxisType::Category,
                crate::option::AxisType::Time => NewAxisType::Time,
                crate::option::AxisType::Log => NewAxisType::Log,
            };
            let new_position = match a.position.unwrap_or(crate::option::AxisPosition::Left) {
                crate::option::AxisPosition::Left => AxisPosition::Left,
                crate::option::AxisPosition::Right => AxisPosition::Right,
                crate::option::AxisPosition::Bottom => AxisPosition::Bottom,
                crate::option::AxisPosition::Top => AxisPosition::Top,
            };
            AxisSpec {
                axis_type: new_axis_type,
                position: new_position,
                grid_index: a.grid_index.unwrap_or(0),
                min: a.min,
                max: a.max,
                name: a.name.clone(),
                categories: a.data.clone().unwrap_or_default(),
                boundary_gap: a.boundary_gap.unwrap_or(true),
            }
        })
        .collect();

    // Series
    let series: Vec<SeriesSpec> = option
        .series
        .iter()
        .map(|s| {
            let (chart_type, data, x_col, y_col, stack, group_index) = match s {
                SeriesOption::Line(ls) => (
                    ChartType::Line,
                    datapoints_to_dataframe(&ls.data, "x", "y"),
                    "x".into(),
                    "y".into(),
                    ls.stack.clone(),
                    0,
                ),
                SeriesOption::Bar(bs) => (
                    ChartType::Bar,
                    datapoints_to_dataframe(&bs.data, "x", "y"),
                    "x".into(),
                    "y".into(),
                    bs.stack.clone(),
                    bs.group_index.unwrap_or(0),
                ),
                SeriesOption::Pie(ps) => (
                    ChartType::Pie,
                    datapoints_to_dataframe(&ps.data, "name", "value"),
                    "name".into(),
                    "value".into(),
                    None,
                    0,
                ),
                SeriesOption::Scatter(ss) => (
                    ChartType::Scatter,
                    datapoints_to_dataframe(&ss.data, "x", "y"),
                    "x".into(),
                    "y".into(),
                    None,
                    0,
                ),
                _ => (
                    ChartType::Line,
                    DataFrame::new(),
                    "x".into(),
                    "y".into(),
                    None,
                    0,
                ),
            };

            let name = match s {
                SeriesOption::Line(ls) => ls.name.clone().unwrap_or_default(),
                SeriesOption::Bar(bs) => bs.name.clone().unwrap_or_default(),
                SeriesOption::Pie(ps) => ps.name.clone().unwrap_or_default(),
                SeriesOption::Scatter(ss) => ss.name.clone().unwrap_or_default(),
                SeriesOption::Bubble(bs) => bs.name.clone().unwrap_or_default(),
                SeriesOption::Candlestick(cs) => cs.name.clone().unwrap_or_default(),
                SeriesOption::Radar(rs) => rs.name.clone().unwrap_or_default(),
                SeriesOption::PolarBar(pb) => pb.name.clone().unwrap_or_default(),
                SeriesOption::PolarScatter(ps) => ps.name.clone().unwrap_or_default(),
                SeriesOption::Gauge(gs) => gs.name.clone().unwrap_or_default(),
                SeriesOption::Table(ts) => ts.name.clone().unwrap_or_default(),
            };

            let sampling = match s {
                SeriesOption::Line(ls) => ls.sampling.as_ref().map(|s| {
                    let ty = match s.ty {
                        SamplingType::Lttb => crate::sampling::SamplingType::Lttb,
                        SamplingType::Average => crate::sampling::SamplingType::Average,
                        SamplingType::Max => crate::sampling::SamplingType::Max,
                        SamplingType::Min => crate::sampling::SamplingType::Min,
                    };
                    (ty, s.threshold)
                }),
                SeriesOption::Bar(bs) => bs.sampling.as_ref().map(|s| {
                    let ty = match s.ty {
                        SamplingType::Lttb => crate::sampling::SamplingType::Lttb,
                        SamplingType::Average => crate::sampling::SamplingType::Average,
                        SamplingType::Max => crate::sampling::SamplingType::Max,
                        SamplingType::Min => crate::sampling::SamplingType::Min,
                    };
                    (ty, s.threshold)
                }),
                _ => None,
            };

            SeriesSpec {
                name,
                chart_type,
                data,
                x_col,
                y_col,
                grid_index: match s {
                    SeriesOption::Line(ls) => ls.grid_index.unwrap_or(0),
                    SeriesOption::Bar(bs) => bs.grid_index.unwrap_or(0),
                    SeriesOption::Pie(ps) => ps.grid_index.unwrap_or(0),
                    _ => 0,
                },
                x_axis_index: 0,
                y_axis_index: match s {
                    SeriesOption::Line(ls) => ls.y_axis_index.unwrap_or(0),
                    SeriesOption::Bar(bs) => bs.y_axis_index.unwrap_or(0),
                    _ => 0,
                },
                stack,
                group_index,
                sampling,
                smooth: match s {
                    SeriesOption::Line(ls) => ls.smooth.unwrap_or(false),
                    _ => false,
                },
                item_style: ItemStyleSpec::default(),
                // === Chart-type-specific config fields ===
                bar_width: match s {
                    SeriesOption::Bar(bs) => bs
                        .bar_width
                        .as_ref()
                        .and_then(|bw| bw.strip_suffix('%'))
                        .and_then(|pct| pct.parse::<f64>().ok())
                        .map(|v| v / 100.0),
                    _ => None,
                },
                line_width: match s {
                    SeriesOption::Line(ls) => ls.line_style.as_ref().and_then(|ls| ls.width),
                    _ => None,
                },
                area_color: match s {
                    SeriesOption::Line(ls) => ls
                        .area_style
                        .as_ref()
                        .and_then(|a| a.color)
                        .map(|c| crate::visual::Color::new(c.r, c.g, c.b)),
                    _ => None,
                },
                area_opacity: match s {
                    SeriesOption::Line(ls) => ls.area_style.as_ref().and_then(|a| a.opacity),
                    _ => None,
                },
                symbol_type: match s {
                    SeriesOption::Line(ls) => ls
                        .symbol
                        .as_ref()
                        .map(|s| format!("{:?}", s).to_lowercase()),
                    _ => None,
                },
                symbol_size: match s {
                    SeriesOption::Line(ls) => ls.symbol_size,
                    _ => None,
                },
                pie_center: match s {
                    SeriesOption::Pie(ps) => ps.center.clone(),
                    _ => None,
                },
                pie_radius: match s {
                    SeriesOption::Pie(ps) => ps.radius.clone(),
                    _ => None,
                },
                label_show: match s {
                    SeriesOption::Pie(ps) => ps.label.as_ref().and_then(|l| l.show),
                    _ => None,
                },
                label_position: match s {
                    SeriesOption::Pie(ps) => ps
                        .label
                        .as_ref()
                        .and_then(|l| l.position)
                        .map(|p| format!("{:?}", p).to_lowercase()),
                    _ => None,
                },
                label_font_size: match s {
                    SeriesOption::Pie(ps) => ps.label.as_ref().and_then(|l| l.font_size),
                    _ => None,
                },
                pad_angle: match s {
                    SeriesOption::PolarBar(pb) => pb.pad_angle,
                    _ => None,
                },
                start_angle: match s {
                    SeriesOption::PolarBar(pb) => pb.start_angle,
                    _ => None,
                },
                ..Default::default()
            }
        })
        .collect();

    ChartSpec {
        width,
        height,
        grids,
        x_axes,
        y_axes,
        series,
        title: option.title.as_ref().map(|t| TitleSpec {
            text: t.text.clone(),
            subtext: t.subtext.clone(),
        }),
        legend: option.legend.as_ref().map(|l| LegendSpec {
            show: l.show.unwrap_or(true),
            data: l.data.clone().unwrap_or_default(),
            symbol_size: l.symbol_size.unwrap_or(10.0),
        }),
        background: crate::visual::Color::new(255, 255, 255),
        palette: vec![],
        theme_name: None,
    }
}

/// 将旧的 Vec<DataPoint> 转换为 DataFrame
fn datapoints_to_dataframe(
    points: &[option::DataPoint],
    x_col: &str,
    y_col: &str,
) -> crate::pipeline::dataframe::DataFrame {
    use crate::pipeline::dataframe::{DataFrame, DataValue, Series as DfSeries};

    let mut df = DataFrame::new();

    if points.is_empty() {
        return df;
    }

    // 判断数据点类型
    let is_named = matches!(points[0], option::DataPoint::Named(_, _));
    let is_xy = matches!(points[0], option::DataPoint::XY(_, _));

    if is_named {
        let names: Vec<DataValue> = points
            .iter()
            .map(|p| {
                if let option::DataPoint::Named(name, _) = p {
                    DataValue::String(name.clone())
                } else {
                    DataValue::Null
                }
            })
            .collect();
        let values: Vec<DataValue> = points
            .iter()
            .map(|p| {
                let v = match p {
                    option::DataPoint::Named(_, y) => *y,
                    option::DataPoint::Value(y) => *y,
                    option::DataPoint::XY(_, y) => *y,
                };
                DataValue::Float(v)
            })
            .collect();
        df.add_column(DfSeries::new(x_col, names));
        df.add_column(DfSeries::new(y_col, values));
    } else if is_xy {
        let xs: Vec<DataValue> = points
            .iter()
            .map(|p| {
                if let option::DataPoint::XY(x, _) = p {
                    DataValue::Float(*x)
                } else {
                    DataValue::Null
                }
            })
            .collect();
        let ys: Vec<DataValue> = points
            .iter()
            .map(|p| {
                let v = match p {
                    option::DataPoint::XY(_, y) => *y,
                    option::DataPoint::Value(y) => *y,
                    option::DataPoint::Named(_, y) => *y,
                };
                DataValue::Float(v)
            })
            .collect();
        df.add_column(DfSeries::new(x_col, xs));
        df.add_column(DfSeries::new(y_col, ys));
    } else {
        // DataPoint::Value only
        let values: Vec<DataValue> = points
            .iter()
            .map(|p| {
                let v = match p {
                    option::DataPoint::Value(y) => *y,
                    option::DataPoint::XY(_, y) => *y,
                    option::DataPoint::Named(_, y) => *y,
                };
                DataValue::Float(v)
            })
            .collect();
        df.add_column(DfSeries::new(y_col, values));
    }

    df
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{
        dataframe::{DataFrame, DataValue, Series},
    };

    #[test]
    fn test_dataframe_to_datapoints_string_x() {
        let mut df = DataFrame::new();
        df.add_column(Series::new(
            "cat",
            vec![DataValue::String("A".into()), DataValue::String("B".into())],
        ));
        df.add_column(Series::new(
            "val",
            vec![DataValue::Float(10.0), DataValue::Float(20.0)],
        ));

        let points = dataframe_to_datapoints(&df, "cat", "val");
        assert_eq!(points.len(), 2);
        assert_eq!(points[0], option::DataPoint::Named("A".into(), 10.0));
        assert_eq!(points[1], option::DataPoint::Named("B".into(), 20.0));
    }

    #[test]
    fn test_dataframe_to_datapoints_numeric_x() {
        let mut df = DataFrame::new();
        df.add_column(Series::new(
            "x",
            vec![
                DataValue::Float(0.0),
                DataValue::Float(1.0),
                DataValue::Float(2.0),
            ],
        ));
        df.add_column(Series::new(
            "y",
            vec![
                DataValue::Float(100.0),
                DataValue::Float(200.0),
                DataValue::Float(300.0),
            ],
        ));

        let points = dataframe_to_datapoints(&df, "x", "y");
        assert_eq!(points.len(), 3);
        assert_eq!(points[0], option::DataPoint::XY(0.0, 100.0));
        assert_eq!(points[1], option::DataPoint::XY(1.0, 200.0));
    }

    #[test]
    fn test_dataframe_to_datapoints_value_only() {
        let mut df = DataFrame::new();
        df.add_column(Series::new(
            "val",
            vec![DataValue::Float(5.0), DataValue::Float(15.0)],
        ));
        df.add_column(Series::new("dummy", vec![DataValue::Null, DataValue::Null]));

        let points = dataframe_to_datapoints(&df, "dummy", "val");
        assert_eq!(points.len(), 2);
        assert_eq!(points[0], option::DataPoint::Value(5.0));
        assert_eq!(points[1], option::DataPoint::Value(15.0));
    }
}
