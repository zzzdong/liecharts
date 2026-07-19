//! ChartSpec ↔ ChartOption 兼容转换层
//!
//! 将 ChartSpec（新管线核心类型）与 ChartOption（旧 API / JSON 配置类型）相互转换。
//! 保留此模块以支持 `examples/json_config.rs` 等基于 ChartOption 的入口。

use crate::{
    error::Result,
    option::{
        AxisConfig, AxisOption, AxisType, BarSeriesOption, BoxplotSeriesOption, BubbleDataPoint,
        BubbleSeriesOption, CandlestickSeriesOption, ChartOption, GaugeSeriesOption, GridConfig,
        GridOption, LegendOption, LineSeriesOption, PieSeriesOption, PolarBarSeriesOption,
        PolarScatterSeriesOption, PositionOption, PositionPreset, RadarSeriesOption,
        ScatterSeriesOption, SeriesOption, TableSeriesOption, TitleOption,
    },
    pipeline::{
        dataframe::DataValue,
        types::{
            self, AxisPosition as NewAxisPosition, AxisSpec as NewAxisSpec,
            AxisType as NewAxisType, ChartSpec, ChartType, GridSpec, SeriesConfig, SeriesSpec,
        },
    },
    sampling::SamplingOption,
    theme::Theme,
    visual::VisualElement,
};

// ── ChartSpec → ChartOption ──

/// 将 ChartSpec 转换为旧的 ChartOption
pub fn chart_spec_to_chart_option(spec: &ChartSpec) -> ChartOption {
    // Title
    let title = spec.title.as_ref().map(|t| TitleOption {
        text: t.text.clone(),
        subtext: t.subtext.clone(),
        left: Some(PositionOption::Preset(PositionPreset::Center)),
        top: Some(PositionOption::Pixel(20.0)),
        ..Default::default()
    });

    // Legend
    let legend = spec.legend.as_ref().map(|legend| LegendOption {
        show: Some(legend.show),
        data: Some(
            legend
                .data
                .iter()
                .cloned()
                .map(crate::option::LegendDataItem::Str)
                .collect(),
        ),
        ..Default::default()
    });

    // Grids
    let grid = {
        let grid_options: Vec<GridOption> = spec
            .grids
            .iter()
            .map(|g| GridOption {
                left: g.left.map(PositionOption::Pixel),
                right: g.right.map(PositionOption::Pixel),
                top: g.top.map(PositionOption::Pixel),
                bottom: g.bottom.map(PositionOption::Pixel),
                contain_label: Some(g.contain_label),
                ..Default::default()
            })
            .collect();
        if grid_options.is_empty() {
            GridConfig::default()
        } else {
            GridConfig::Multiple(grid_options)
        }
    };

    // X Axes
    let x_axis = {
        let x_axis_options: Vec<AxisOption> = spec.x_axes.iter().map(axis_spec_to_option).collect();
        if x_axis_options.is_empty() {
            AxisConfig::default()
        } else {
            AxisConfig::Multiple(x_axis_options)
        }
    };

    // Y Axes
    let y_axis = {
        let y_axis_options: Vec<AxisOption> = spec.y_axes.iter().map(axis_spec_to_option).collect();
        if y_axis_options.is_empty() {
            AxisConfig::default()
        } else {
            AxisConfig::Multiple(y_axis_options)
        }
    };

    // Series
    let mut series = Vec::new();
    for s in &spec.series {
        match s.chart_type {
            ChartType::Line => {
                let cfg = match &s.config {
                    SeriesConfig::Line(c) => c.clone(),
                    _ => Default::default(),
                };
                series.push(SeriesOption::Line(LineSeriesOption {
                    name: Some(s.name.clone()),
                    data: df_to_datapoints_by_cols(s, &cfg.x_col, &cfg.y_col),
                    smooth: Some(cfg.smooth),
                    stack: s.stack.clone(),
                    sampling: s
                        .sampling
                        .map(|(ty, threshold)| SamplingOption { ty, threshold }),
                    item_style: s.item_style.color.map(|c| crate::option::ItemStyleOption {
                        color: Some(crate::option::ColorOption::new(c.r, c.g, c.b)),
                        ..Default::default()
                    }),
                    y_axis_index: Some(s.y_axis_index),
                    grid_index: Some(s.grid_index),
                    ..Default::default()
                }));
            }
            ChartType::Bar => {
                let cfg = match &s.config {
                    SeriesConfig::Bar(c) => c.clone(),
                    _ => Default::default(),
                };
                series.push(SeriesOption::Bar(BarSeriesOption {
                    name: Some(s.name.clone()),
                    data: df_to_datapoints_by_cols(s, &cfg.x_col, &cfg.y_col),
                    stack: s.stack.clone(),
                    group_index: Some(s.group_index),
                    item_style: s.item_style.color.map(|c| crate::option::ItemStyleOption {
                        color: Some(crate::option::ColorOption::new(c.r, c.g, c.b)),
                        ..Default::default()
                    }),
                    y_axis_index: Some(s.y_axis_index),
                    grid_index: Some(s.grid_index),
                    ..Default::default()
                }));
            }
            ChartType::Scatter => {
                let cfg = match &s.config {
                    SeriesConfig::Scatter(c) => c.clone(),
                    _ => Default::default(),
                };
                series.push(SeriesOption::Scatter(ScatterSeriesOption {
                    name: Some(s.name.clone()),
                    data: df_to_datapoints_by_cols(s, &cfg.x_col, &cfg.y_col),
                    item_style: s.item_style.color.map(|c| crate::option::ItemStyleOption {
                        color: Some(crate::option::ColorOption::new(c.r, c.g, c.b)),
                        ..Default::default()
                    }),
                    symbol_size: Some(cfg.symbol_size),
                    y_axis_index: Some(s.y_axis_index),
                    grid_index: Some(s.grid_index),
                    ..Default::default()
                }));
            }
            ChartType::Pie => {
                series.push(SeriesOption::Pie(PieSeriesOption {
                    name: Some(s.name.clone()),
                    data: df_to_datapoints(s),
                    ..Default::default()
                }));
            }
            ChartType::Bubble => {
                let cfg = match &s.config {
                    SeriesConfig::Bubble(c) => c.clone(),
                    _ => Default::default(),
                };
                series.push(SeriesOption::Bubble(BubbleSeriesOption {
                    name: Some(s.name.clone()),
                    data: (0..s.data.row_count())
                        .map(|i| {
                            let x = s
                                .data
                                .get_column(&cfg.x_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0);
                            let y = s
                                .data
                                .get_column(&cfg.y_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0);
                            let size = cfg
                                .size_col
                                .as_ref()
                                .and_then(|sc| s.data.get_column(sc))
                                .and_then(|c| c.as_f64(i));
                            BubbleDataPoint {
                                x,
                                y,
                                size,
                                name: None,
                            }
                        })
                        .collect(),
                    symbol_size_scale: Some(cfg.symbol_size_scale),
                    y_axis_index: Some(s.y_axis_index),
                    grid_index: Some(s.grid_index),
                    ..Default::default()
                }));
            }
            ChartType::Candlestick => {
                let cfg = match &s.config {
                    SeriesConfig::Candlestick(c) => c.clone(),
                    _ => Default::default(),
                };
                series.push(SeriesOption::Candlestick(CandlestickSeriesOption {
                    name: Some(s.name.clone()),
                    data: (0..s.data.row_count())
                        .map(|i| crate::option::CandlestickDataPoint {
                            open: s
                                .data
                                .get_column(&cfg.open_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            close: s
                                .data
                                .get_column(&cfg.close_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            low: s
                                .data
                                .get_column(&cfg.low_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            high: s
                                .data
                                .get_column(&cfg.high_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            name: None,
                        })
                        .collect(),
                    y_axis_index: Some(s.y_axis_index),
                    grid_index: Some(s.grid_index),
                    ..Default::default()
                }));
            }
            ChartType::Boxplot => {
                let cfg = match &s.config {
                    SeriesConfig::Boxplot(c) => c.clone(),
                    _ => Default::default(),
                };
                series.push(SeriesOption::Boxplot(BoxplotSeriesOption {
                    name: Some(s.name.clone()),
                    data: (0..s.data.row_count())
                        .map(|i| crate::option::BoxplotDataPoint {
                            min: s
                                .data
                                .get_column(&cfg.min_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            q1: s
                                .data
                                .get_column(&cfg.q1_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            median: s
                                .data
                                .get_column(&cfg.median_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            q3: s
                                .data
                                .get_column(&cfg.q3_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            max: s
                                .data
                                .get_column(&cfg.max_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            name: s
                                .data
                                .get_column(&cfg.category_col)
                                .and_then(|c| c.as_string(i)),
                        })
                        .collect(),
                    y_axis_index: Some(s.y_axis_index),
                    grid_index: Some(s.grid_index),
                    ..Default::default()
                }));
            }
            ChartType::Radar => {
                let cfg = match &s.config {
                    SeriesConfig::Radar(c) => c.clone(),
                    _ => Default::default(),
                };
                series.push(SeriesOption::Radar(RadarSeriesOption {
                    name: Some(s.name.clone()),
                    data: (0..s.data.row_count())
                        .map(|i| crate::option::RadarDataOption {
                            value: vec![
                                s.data
                                    .get_column(&cfg.value_col)
                                    .and_then(|c| c.as_f64(i))
                                    .unwrap_or(0.0),
                            ],
                            name: None,
                        })
                        .collect(),
                    ..Default::default()
                }));
            }
            ChartType::PolarBar => {
                series.push(SeriesOption::PolarBar(PolarBarSeriesOption {
                    name: Some(s.name.clone()),
                    data: df_to_datapoints(s),
                    ..Default::default()
                }));
            }
            ChartType::PolarScatter => {
                let cfg = match &s.config {
                    SeriesConfig::PolarScatter(c) => c.clone(),
                    _ => Default::default(),
                };
                series.push(SeriesOption::PolarScatter(PolarScatterSeriesOption {
                    name: Some(s.name.clone()),
                    data: (0..s.data.row_count())
                        .map(|i| crate::option::PolarScatterDataPoint {
                            angle: s
                                .data
                                .get_column(&cfg.angle_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            radius: s
                                .data
                                .get_column(&cfg.radius_col)
                                .and_then(|c| c.as_f64(i))
                                .unwrap_or(0.0),
                            symbol_size: None,
                            name: None,
                        })
                        .collect(),
                    ..Default::default()
                }));
            }
            ChartType::Gauge => {
                let cfg = match &s.config {
                    SeriesConfig::Gauge(c) => c.clone(),
                    _ => Default::default(),
                };
                series.push(SeriesOption::Gauge(GaugeSeriesOption {
                    name: Some(s.name.clone()),
                    data: df_to_gauge_datapoints(s, &cfg.value_col),
                    min: Some(cfg.min),
                    max: Some(cfg.max),
                    ..Default::default()
                }));
            }
            ChartType::Table => {
                let cols = s.data.column_names().to_vec();
                let rows: Vec<Vec<serde_json::Value>> = (0..s.data.row_count())
                    .map(|i| {
                        s.data
                            .column_names()
                            .iter()
                            .map(|col_name| {
                                let val = s.data.get_column(col_name).and_then(|c| c.as_string(i));
                                val.map(|s| {
                                    s.parse::<f64>()
                                        .map(serde_json::Value::from)
                                        .unwrap_or_else(|_| serde_json::Value::String(s))
                                })
                                .unwrap_or(serde_json::Value::Null)
                            })
                            .collect()
                    })
                    .collect();
                series.push(SeriesOption::Table(TableSeriesOption {
                    name: Some(s.name.clone()),
                    columns: Some(cols),
                    data: Some(rows),
                    ..Default::default()
                }));
            }
        }
    }

    // Palette
    let color = if spec.palette.is_empty() {
        None
    } else {
        let colors: Vec<crate::option::ColorOption> = spec
            .palette
            .iter()
            .map(|c| crate::option::ColorOption::new(c.r, c.g, c.b))
            .collect();
        Some(crate::option::OneOrMany::Many(colors))
    };

    ChartOption {
        title,
        legend,
        grid,
        x_axis,
        y_axis,
        series,
        color,
        ..Default::default()
    }
}

fn axis_spec_to_option(a: &NewAxisSpec) -> AxisOption {
    AxisOption {
        axis_type: Some(match a.axis_type {
            NewAxisType::Category => AxisType::Category,
            NewAxisType::Value => AxisType::Value,
            NewAxisType::Time => AxisType::Time,
            NewAxisType::Log => AxisType::Log,
        }),
        position: Some(match a.position {
            NewAxisPosition::Left => crate::option::AxisPosition::Left,
            NewAxisPosition::Right => crate::option::AxisPosition::Right,
            NewAxisPosition::Bottom => crate::option::AxisPosition::Bottom,
            NewAxisPosition::Top => crate::option::AxisPosition::Top,
        }),
        grid_index: Some(a.grid_index),
        min: a.min,
        max: a.max,
        name: a.name.clone(),
        data: if a.categories.is_empty() {
            None
        } else {
            Some(a.categories.clone())
        },
        boundary_gap: Some(a.boundary_gap),
        ..Default::default()
    }
}

/// 将 SeriesSpec 的 DataFrame 数据转换为旧 DataPoint 列表
fn df_to_datapoints(s: &SeriesSpec) -> Vec<crate::option::DataPoint> {
    // 默认从 config 中读取列名
    let (x_col, y_col) = match &s.config {
        SeriesConfig::Line(c) => (&c.x_col, &c.y_col),
        SeriesConfig::Bar(c) => (&c.x_col, &c.y_col),
        SeriesConfig::Scatter(c) => (&c.x_col, &c.y_col),
        SeriesConfig::Pie(c) => (&c.category_col, &c.value_col),
        SeriesConfig::PolarBar(c) => (&c.angle_col, &c.radius_col),
        _ => return vec![],
    };
    df_to_datapoints_by_cols(s, x_col, y_col)
}

fn df_to_datapoints_by_cols(
    s: &SeriesSpec,
    x_col: &str,
    y_col: &str,
) -> Vec<crate::option::DataPoint> {
    let mut dps = Vec::new();

    let is_category = s
        .data
        .get_column(x_col)
        .and_then(|c| c.data.first())
        .map(|v| matches!(v, DataValue::String(_)))
        .unwrap_or(false);

    for i in 0..s.data.row_count() {
        let y_val = s
            .data
            .get_column(y_col)
            .and_then(|c| c.as_f64(i))
            .unwrap_or(0.0);

        if is_category {
            let name = s
                .data
                .get_column(x_col)
                .and_then(|c| c.as_string(i))
                .unwrap_or_default();
            dps.push(crate::option::DataPoint::Named(name, y_val));
        } else {
            let x_val = s
                .data
                .get_column(x_col)
                .and_then(|c| c.as_f64(i))
                .unwrap_or(i as f64);
            dps.push(crate::option::DataPoint::XY(x_val, y_val));
        }
    }
    dps
}

/// 将 SeriesSpec 的 DataFrame 数据转换为 GaugeDataPoint 列表
fn df_to_gauge_datapoints(s: &SeriesSpec, value_col: &str) -> Vec<crate::option::GaugeDataPoint> {
    (0..s.data.row_count())
        .map(|i| {
            let value = s
                .data
                .get_column(value_col)
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let name = s.data.get_column(value_col).and_then(|c| c.as_string(i));
            crate::option::GaugeDataPoint { value, name }
        })
        .collect()
}

// ── ChartOption → ChartSpec ──

/// 将旧的 ChartOption 转换为新的 ChartSpec（反向兼容）
pub fn chart_option_to_chart_spec(option: &ChartOption, width: u32, height: u32) -> ChartSpec {
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
    let x_axes: Vec<NewAxisSpec> = option
        .x_axis
        .iter()
        .map(|a| option_axis_to_new_spec(a, crate::option::AxisPosition::Bottom))
        .collect();

    // Y Axes
    let y_axes: Vec<NewAxisSpec> = option
        .y_axis
        .iter()
        .map(|a| option_axis_to_new_spec(a, crate::option::AxisPosition::Left))
        .collect();

    // Series — Unknown 系列会被跳过（不渲染，但解析不报错）
    let series: Vec<SeriesSpec> = option
        .series
        .iter()
        .enumerate()
        .filter_map(|(idx, s)| option_series_to_spec(s, idx))
        .collect();

    // Title
    let title = option.title.as_ref().map(|t| types::TitleSpec {
        text: t.text.clone(),
        subtext: t.subtext.clone(),
    });

    // Legend
    let legend = option.legend.as_ref().map(|l| types::LegendSpec {
        show: l.show.unwrap_or(true),
        data: l
            .data
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|i| i.name().to_string())
            .collect(),
        symbol_size: l.symbol_size.unwrap_or(10.0),
    });

    ChartSpec {
        width,
        height,
        grids,
        x_axes,
        y_axes,
        series,
        title,
        legend,
        background: crate::visual::Color::new(255, 255, 255),
        palette: vec![],
        theme_name: None,
    }
}

fn option_axis_to_new_spec(
    a: &AxisOption,
    default_pos: crate::option::AxisPosition,
) -> NewAxisSpec {
    let new_axis_type = match a.axis_type.unwrap_or(AxisType::Category) {
        AxisType::Value => NewAxisType::Value,
        AxisType::Category => NewAxisType::Category,
        AxisType::Time => NewAxisType::Time,
        AxisType::Log => NewAxisType::Log,
    };
    let pos = a.position.unwrap_or(default_pos);
    let new_position = match pos {
        crate::option::AxisPosition::Bottom => NewAxisPosition::Bottom,
        crate::option::AxisPosition::Top => NewAxisPosition::Top,
        crate::option::AxisPosition::Left => NewAxisPosition::Left,
        crate::option::AxisPosition::Right => NewAxisPosition::Right,
    };
    NewAxisSpec {
        axis_type: new_axis_type,
        position: new_position,
        grid_index: a.grid_index.unwrap_or(0),
        min: a.min,
        max: a.max,
        name: a.name.clone(),
        categories: a.data.clone().unwrap_or_default(),
        boundary_gap: a.boundary_gap.unwrap_or(true),
    }
}

fn option_series_to_spec(s: &SeriesOption, idx: usize) -> Option<SeriesSpec> {
    let mut df = crate::pipeline::dataframe::DataFrame::new();

    let (chart_type, stack, group_index, config, data_points) = match s {
        SeriesOption::Line(ls) => {
            let dps = ls.data.clone();
            let (x_col, y_col, data_cols) = datapoints_to_dataframe_cols(&dps);
            for (name, col) in data_cols {
                df.add_column(crate::pipeline::dataframe::Series::new(name, col));
            }
            let config = SeriesConfig::Line(crate::pipeline::types::LineConfig {
                x_col,
                y_col,
                smooth: ls.smooth.unwrap_or(false),
                line_width: 2.0,
                area: false,
                area_color: None,
                area_opacity: 0.5,
                symbol_type: Default::default(),
                symbol_size: 4.0,
                label_show: false,
                label_font_size: 12.0,
            });
            (ChartType::Line, ls.stack.clone(), 0usize, config, dps)
        }
        SeriesOption::Bar(bs) => {
            let dps = bs.data.clone();
            let (x_col, y_col, data_cols) = datapoints_to_dataframe_cols(&dps);
            for (name, col) in data_cols {
                df.add_column(crate::pipeline::dataframe::Series::new(name, col));
            }
            let config = SeriesConfig::Bar(crate::pipeline::types::BarConfig {
                x_col,
                y_col,
                bar_width: 0.6,
                label_show: false,
                label_font_size: 12.0,
            });
            (
                ChartType::Bar,
                bs.stack.clone(),
                bs.group_index.unwrap_or(0),
                config,
                dps,
            )
        }
        SeriesOption::Scatter(ss) => {
            let dps = ss.data.clone();
            let (x_col, y_col, data_cols) = datapoints_to_dataframe_cols(&dps);
            for (name, col) in data_cols {
                df.add_column(crate::pipeline::dataframe::Series::new(name, col));
            }
            let config = SeriesConfig::Scatter(crate::pipeline::types::ScatterConfig {
                x_col,
                y_col,
                symbol_size: ss.symbol_size.unwrap_or(10.0),
            });
            (ChartType::Scatter, None, 0usize, config, dps)
        }
        SeriesOption::Pie(ps) => {
            let dps = ps.data.clone();
            let (cat_col, val_col, data_cols) = datapoints_to_dataframe_cols(&dps);
            for (name, col) in data_cols {
                df.add_column(crate::pipeline::dataframe::Series::new(name, col));
            }
            let config = SeriesConfig::Pie(crate::pipeline::types::PieConfig {
                category_col: cat_col,
                value_col: val_col,
                center: (50.0, 50.0),
                radius: (0.0, 75.0),
                label_show: false,
                label_position: types::LabelPosition::Outside,
                label_font_size: 12.0,
            });
            (ChartType::Pie, None, 0usize, config, dps)
        }
        SeriesOption::Bubble(_) => {
            let config = SeriesConfig::Bubble(Default::default());
            (ChartType::Bubble, None, 0usize, config, vec![])
        }
        SeriesOption::Candlestick(_) => {
            let config = SeriesConfig::Candlestick(Default::default());
            (ChartType::Candlestick, None, 0usize, config, vec![])
        }
        SeriesOption::Boxplot(bs) => {
            let mut min_vals = Vec::new();
            let mut q1_vals = Vec::new();
            let mut median_vals = Vec::new();
            let mut q3_vals = Vec::new();
            let mut max_vals = Vec::new();
            let mut categories = Vec::new();
            for (i, dp) in bs.data.iter().enumerate() {
                categories.push(DataValue::from(
                    dp.name.clone().unwrap_or_else(|| (i + 1).to_string()),
                ));
                min_vals.push(DataValue::Float(dp.min));
                q1_vals.push(DataValue::Float(dp.q1));
                median_vals.push(DataValue::Float(dp.median));
                q3_vals.push(DataValue::Float(dp.q3));
                max_vals.push(DataValue::Float(dp.max));
            }
            df.add_column(crate::pipeline::dataframe::Series::new(
                "category", categories,
            ));
            df.add_column(crate::pipeline::dataframe::Series::new("min", min_vals));
            df.add_column(crate::pipeline::dataframe::Series::new("q1", q1_vals));
            df.add_column(crate::pipeline::dataframe::Series::new(
                "median",
                median_vals,
            ));
            df.add_column(crate::pipeline::dataframe::Series::new("q3", q3_vals));
            df.add_column(crate::pipeline::dataframe::Series::new("max", max_vals));
            let config = SeriesConfig::Boxplot(crate::pipeline::types::BoxplotConfig {
                category_col: "category".into(),
                min_col: "min".into(),
                q1_col: "q1".into(),
                median_col: "median".into(),
                q3_col: "q3".into(),
                max_col: "max".into(),
            });
            (ChartType::Boxplot, None, 0usize, config, vec![])
        }
        SeriesOption::Radar(_) => {
            let config = SeriesConfig::Radar(Default::default());
            (ChartType::Radar, None, 0usize, config, vec![])
        }
        SeriesOption::PolarBar(pbs) => {
            let dps = pbs.data.clone();
            let (angle_col, radius_col, data_cols) = datapoints_to_dataframe_cols(&dps);
            for (name, col) in data_cols {
                df.add_column(crate::pipeline::dataframe::Series::new(name, col));
            }
            let config = SeriesConfig::PolarBar(crate::pipeline::types::PolarBarConfig {
                angle_col,
                radius_col,
                pad_angle: 2.0,
                start_angle: 0.0,
            });
            (ChartType::PolarBar, None, 0usize, config, dps)
        }
        SeriesOption::PolarScatter(_) => {
            let config = SeriesConfig::PolarScatter(Default::default());
            (ChartType::PolarScatter, None, 0usize, config, vec![])
        }
        SeriesOption::Gauge(gs) => {
            // 仪表盘数据：Vec<GaugeDataPoint> 转 ("value", values)
            let mut values = Vec::new();
            for gdp in &gs.data {
                values.push(DataValue::Float(gdp.value));
            }
            df.add_column(crate::pipeline::dataframe::Series::new("value", values));
            let config = SeriesConfig::Gauge(crate::pipeline::types::GaugeConfig {
                value_col: "value".into(),
                min: gs.min.unwrap_or(0.0),
                max: gs.max.unwrap_or(100.0),
                center: (50.0, 75.0),
                radius: 75.0,
                start_angle: -225.0,
                end_angle: 45.0,
                split_number: 10,
            });
            (ChartType::Gauge, None, 0usize, config, vec![])
        }
        SeriesOption::Table(ts) => {
            // 表格：直接使用已有列
            if let Some(cols) = &ts.columns {
                for col_name in cols {
                    if let Some(data) = &ts.data {
                        let col_values: Vec<DataValue> = data
                            .iter()
                            .map(|row| {
                                // 简化处理：取第一列作为字符串
                                if let Some(v) = row.first() {
                                    match v {
                                        serde_json::Value::Number(n) => n
                                            .as_f64()
                                            .map(DataValue::Float)
                                            .unwrap_or(DataValue::Null),
                                        serde_json::Value::String(s) => {
                                            DataValue::String(s.clone())
                                        }
                                        _ => DataValue::Null,
                                    }
                                } else {
                                    DataValue::Null
                                }
                            })
                            .collect();
                        df.add_column(crate::pipeline::dataframe::Series::new(
                            col_name.clone(),
                            col_values,
                        ));
                    }
                }
            }
            let config = SeriesConfig::Table(Default::default());
            (ChartType::Table, None, 0usize, config, vec![])
        }
        SeriesOption::Unknown => {
            // 未识别的 series 类型（heatmap/funnel/treemap/...）：解析不报错，渲染时跳过
            return None;
        }
    };

    // 确保 DataFrame 至少有一列
    if df.column_names().is_empty() {
        let count = data_points.len();
        df.add_column(crate::pipeline::dataframe::Series::new_constant(
            "_dummy",
            DataValue::Float(0.0),
            count.max(1),
        ));
    }

    Some(SeriesSpec {
        name: format!("series_{}", idx),
        chart_type,
        data: df,
        grid_index: 0,
        x_axis_index: 0,
        y_axis_index: 0,
        stack,
        group_index,
        sampling: None,
        item_style: types::ItemStyleSpec::default(),
        config,
    })
}

/// 将 DataPoint 列表转换为 DataFrame 列数据，返回 (x_col_name, y_col_name, Vec<(col_name, Vec<DataValue>)>)
fn datapoints_to_dataframe_cols(
    dps: &[crate::option::DataPoint],
) -> (String, String, Vec<(String, Vec<DataValue>)>) {
    if dps.is_empty() {
        return ("x".into(), "y".into(), vec![]);
    }

    // 判断数据点类型
    let first = &dps[0];
    match first {
        crate::option::DataPoint::XY(_, _) => {
            let mut x_vals = Vec::with_capacity(dps.len());
            let mut y_vals = Vec::with_capacity(dps.len());
            for dp in dps {
                if let crate::option::DataPoint::XY(x, y) = dp {
                    x_vals.push(DataValue::Float(*x));
                    y_vals.push(DataValue::Float(*y));
                }
            }
            (
                "x".into(),
                "y".into(),
                vec![("x".into(), x_vals), ("y".into(), y_vals)],
            )
        }
        crate::option::DataPoint::Named(_, _) => {
            let mut cat_vals = Vec::with_capacity(dps.len());
            let mut y_vals = Vec::with_capacity(dps.len());
            for dp in dps {
                if let crate::option::DataPoint::Named(name, val) = dp {
                    cat_vals.push(DataValue::String(name.clone()));
                    y_vals.push(DataValue::Float(*val));
                }
            }
            (
                "category".into(),
                "value".into(),
                vec![("category".into(), cat_vals), ("value".into(), y_vals)],
            )
        }
        crate::option::DataPoint::Value(_) => {
            let mut y_vals = Vec::with_capacity(dps.len());
            for dp in dps {
                if let crate::option::DataPoint::Value(v) = dp {
                    y_vals.push(DataValue::Float(*v));
                }
            }
            // 对于纯数值，x 使用索引
            let x_vals: Vec<DataValue> =
                (0..dps.len()).map(|i| DataValue::Float(i as f64)).collect();
            (
                "x".into(),
                "y".into(),
                vec![("x".into(), x_vals), ("y".into(), y_vals)],
            )
        }
    }
}

// ── 旧 API 兼容入口 ──

/// 从 ChartOption 构建图表（旧 API 兼容入口）
pub fn build_chart(option: &ChartOption, width: u32, height: u32) -> Result<Vec<VisualElement>> {
    build_chart_with_theme(option, width, height, &Theme::echarts())
}

/// 从 ChartOption 构建图表（旧 API 兼容入口，指定主题）
pub fn build_chart_with_theme(
    option: &ChartOption,
    width: u32,
    height: u32,
    theme: &Theme,
) -> Result<Vec<VisualElement>> {
    let spec = chart_option_to_chart_spec(option, width, height);
    crate::pipeline::chart_pipeline::build_chart_from_spec(&spec, theme)
}
