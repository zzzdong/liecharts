//! ChartSpec → ChartOption 兼容转换层
//!
//! 将新的 ChartSpec 转换为旧的 ChartOption，以便复用基于旧类型的 processor 管线。
//! 这是临时的过渡方案，待所有 processor 迁移到新类型后移除。

use crate::{
    option::{
        self, AxisConfig, AxisOption, AxisType, BarSeriesOption, BoxplotSeriesOption,
        BubbleDataPoint, BubbleSeriesOption, CandlestickSeriesOption, ChartOption,
        GaugeSeriesOption, GridConfig, GridOption, LegendOption, LineSeriesOption, PieSeriesOption,
        PolarBarSeriesOption, PolarScatterSeriesOption, PositionOption, PositionPreset,
        RadarIndicatorOption, RadarOption, RadarSeriesOption, ScatterSeriesOption, SeriesOption,
        TableSeriesOption, TitleOption,
    },
    pipeline::{
        dataframe::DataValue,
        types::{
            BarConfig, ChartSpec, ChartType, GridSpec, LineConfig, PieConfig, ScatterConfig,
            SeriesConfig, SeriesSpec, SymbolType,
        },
    },
    sampling::{SamplingOption, SamplingType},
};

/// 将 ChartSpec 转换为旧的 ChartOption
pub fn chart_spec_to_chart_option(spec: &ChartSpec) -> ChartOption {
    // Title
    let title = spec.title.as_ref().map(|t| TitleOption {
        text: t.text.clone(),
        subtext: t.subtext.clone(),
        left: Some(PositionOption::Preset(PositionPreset::Center)),
        top: Some(PositionOption::Pixel(20.0)),
        text_style: None,
        subtext_style: None,
        ..Default::default()
    });

    // Legend
    let legend = spec.legend.as_ref().map(|legend| LegendOption {
        show: Some(true),
        data: Some(
            legend
                .data
                .iter()
                .cloned()
                .map(crate::option::LegendDataItem::Str)
                .collect(),
        ),
        left: Some(PositionOption::Preset(PositionPreset::Center)),
        top: Some(PositionOption::Preset(PositionPreset::Auto)),
        ..Default::default()
    });

    // Grids
    let grid = {
        let grid_options: Vec<GridOption> = spec.grids.iter().map(grid_to_grid_option).collect();
        if grid_options.is_empty() {
            GridConfig::default()
        } else {
            GridConfig::Multiple(grid_options)
        }
    };

    // X Axes
    let x_axis = {
        let x_axis_options: Vec<AxisOption> = spec.x_axes.iter().map(axis_to_axis_option).collect();
        if x_axis_options.is_empty() {
            AxisConfig::default()
        } else {
            AxisConfig::Multiple(x_axis_options)
        }
    };

    // Y Axes
    let y_axis = {
        let y_axis_options: Vec<AxisOption> = spec.y_axes.iter().map(axis_to_axis_option).collect();
        if y_axis_options.is_empty() {
            AxisConfig::default()
        } else {
            AxisConfig::Multiple(y_axis_options)
        }
    };

    // Series
    let series = spec.series.iter().map(series_to_series_option).collect();

    // Radar: 如果有雷达图系列，设置 radar 配置
    let radar = spec.series.iter().find_map(|series| {
        if let SeriesConfig::Radar(cfg) = &series.config {
            let indicators: Vec<RadarIndicatorOption> = cfg
                .indicators
                .iter()
                .map(|name| RadarIndicatorOption {
                    name: Some(name.clone()),
                    max: Some(100.0),
                })
                .collect();
            Some(RadarOption {
                indicator: Some(indicators),
                ..Default::default()
            })
        } else {
            None
        }
    });

    ChartOption {
        title,
        legend,
        grid,
        x_axis,
        y_axis,
        series,
        radar,
        ..Default::default()
    }
}

fn grid_to_grid_option(g: &GridSpec) -> GridOption {
    GridOption {
        left: g.left.map(PositionOption::Pixel),
        right: g.right.map(PositionOption::Pixel),
        top: g.top.map(PositionOption::Pixel),
        bottom: g.bottom.map(PositionOption::Pixel),
        contain_label: if g.contain_label { Some(true) } else { None },
        ..Default::default()
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
    let x_col = s.config.x_col_name();
    let y_col = s.config.y_col_name();
    let data = dataframe_to_datapoints(&s.data, x_col, y_col);

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
        ChartType::Line => {
            let smooth = match &s.config {
                SeriesConfig::Line(cfg) => Some(cfg.smooth),
                _ => Some(false),
            };
            SeriesOption::Line(LineSeriesOption {
                name: Some(s.name.clone()),
                data,
                stack: s.stack.clone(),
                y_axis_index: Some(s.y_axis_index),
                grid_index: Some(s.grid_index),
                smooth,
                sampling,
                ..Default::default()
            })
        }
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
        ChartType::Boxplot => SeriesOption::Boxplot(BoxplotSeriesOption {
            name: Some(s.name.clone()),
            data: vec![], // Boxplot 需要特殊处理
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

/// 将 PositionOption 解析为像素值
fn resolve_position_option(pos: &PositionOption, total: f64) -> f64 {
    match pos {
        PositionOption::Pixel(v) => *v,
        PositionOption::Percent(p) => total * p / 100.0,
        PositionOption::Preset(PositionPreset::Auto) => total * 0.1, // fallback: 10%
        PositionOption::Preset(PositionPreset::Center) => total / 2.0,
        PositionOption::Preset(PositionPreset::Left)
        | PositionOption::Preset(PositionPreset::Top) => 0.0,
        PositionOption::Preset(PositionPreset::Right)
        | PositionOption::Preset(PositionPreset::Bottom) => total,
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

    let total_w = width as f64;
    let total_h = height as f64;

    // Grids
    let grids: Vec<GridSpec> = option
        .grid
        .iter()
        .map(|g| {
            let left = g.left.as_ref().map(|p| resolve_position_option(p, total_w));
            let right = g
                .right
                .as_ref()
                .map(|p| resolve_position_option(p, total_w));
            let top = g.top.as_ref().map(|p| resolve_position_option(p, total_h));
            let bottom = g
                .bottom
                .as_ref()
                .map(|p| resolve_position_option(p, total_h));
            GridSpec {
                left,
                right,
                top,
                bottom,
                contain_label: g.contain_label.unwrap_or(false),
            }
        })
        .collect();
    // 如果没有指定 grid，为所有非极坐标/非雷达/非仪表盘系列创建一个默认 grid
    let has_cartesian_series = option.series.iter().any(|s| {
        matches!(
            s,
            SeriesOption::Line(_)
                | SeriesOption::Bar(_)
                | SeriesOption::Scatter(_)
                | SeriesOption::Bubble(_)
                | SeriesOption::Candlestick(_)
                | SeriesOption::Boxplot(_)
        )
    });
    let grids = if grids.is_empty() && has_cartesian_series {
        vec![GridSpec {
            left: Some(60.0),
            right: Some(60.0),
            top: Some(60.0),
            bottom: Some(60.0),
            contain_label: false,
        }]
    } else {
        grids
    };

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

    // Resolve datasets
    let datasets: Vec<crate::pipeline::dataframe::DataFrame> = option
        .dataset
        .as_ref()
        .map(|ds| ds.as_slice().iter().map(dataset_to_dataframe).collect())
        .unwrap_or_default();

    /// 尝试从系列中获取 dataset_index 和 encode
    fn get_series_dataset_info(
        s: &SeriesOption,
    ) -> (Option<usize>, Option<option::SeriesEncodeOption>) {
        match s {
            SeriesOption::Line(ls) => (ls.dataset_index, ls.encode.clone()),
            SeriesOption::Bar(bs) => (bs.dataset_index, bs.encode.clone()),
            SeriesOption::Pie(ps) => (ps.dataset_index, ps.encode.clone()),
            SeriesOption::Scatter(ss) => (ss.dataset_index, ss.encode.clone()),
            _ => (None, None),
        }
    }

    /// 从 dataset 解析数据，或回退到 series.data
    fn resolve_series_data<'a>(
        s: &'a SeriesOption,
        datasets: &'a [crate::pipeline::dataframe::DataFrame],
        fallback_data: impl FnOnce() -> crate::pipeline::dataframe::DataFrame,
        x_col: &str,
        y_col: &str,
    ) -> crate::pipeline::dataframe::DataFrame {
        let (ds_idx, encode) = get_series_dataset_info(s);
        if let Some(idx) = ds_idx
            && let Some(ds_df) = datasets.get(idx)
        {
            if let Some(enc) = &encode {
                return extract_encoded_columns(ds_df, enc).0;
            }
            // 没有 encode：直接使用 dataset 的 x/y 列（如果存在）
            if ds_df.get_column("x").is_some() && ds_df.get_column("y").is_some() {
                return ds_df.clone();
            }
            // 如果 dataset 有列名，尝试用 fallback 的列名
            if ds_df.column_count() >= 2 {
                let cols = ds_df.column_names().to_vec();
                let mut df = crate::pipeline::dataframe::DataFrame::new();
                df.add_column(crate::pipeline::dataframe::Series::new(
                    x_col,
                    ds_df.get_column(&cols[0]).unwrap().data.clone(),
                ));
                df.add_column(crate::pipeline::dataframe::Series::new(
                    y_col,
                    ds_df.get_column(&cols[1]).unwrap().data.clone(),
                ));
                return df;
            }
            return ds_df.clone();
        }
        fallback_data()
    }

    // Series — Unknown 系列会被跳过（不渲染，但解析不报错）
    let series: Vec<SeriesSpec> = option
        .series
        .iter()
        .filter_map(|s| {
            // 未识别的 series 类型（heatmap/funnel/treemap/...）：解析不报错，渲染时跳过
            if matches!(s, SeriesOption::Unknown) {
                return None;
            }
            let name = match s {
                SeriesOption::Line(ls) => ls.name.clone().unwrap_or_default(),
                SeriesOption::Bar(bs) => bs.name.clone().unwrap_or_default(),
                SeriesOption::Pie(ps) => ps.name.clone().unwrap_or_default(),
                SeriesOption::Scatter(ss) => ss.name.clone().unwrap_or_default(),
                SeriesOption::Bubble(bs) => bs.name.clone().unwrap_or_default(),
                SeriesOption::Candlestick(cs) => cs.name.clone().unwrap_or_default(),
                SeriesOption::Boxplot(bs) => bs.name.clone().unwrap_or_default(),
                SeriesOption::Radar(rs) => rs.name.clone().unwrap_or_default(),
                SeriesOption::PolarBar(pb) => pb.name.clone().unwrap_or_default(),
                SeriesOption::PolarScatter(ps) => ps.name.clone().unwrap_or_default(),
                SeriesOption::Gauge(gs) => gs.name.clone().unwrap_or_default(),
                SeriesOption::Table(ts) => ts.name.clone().unwrap_or_default(),
                SeriesOption::Unknown => unreachable!(),
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

            let spec = match s {
                SeriesOption::Line(ls) => {
                    let data = resolve_series_data(
                        s,
                        &datasets,
                        || datapoints_to_dataframe(&ls.data, "x", "y"),
                        "x",
                        "y",
                    );
                    let config = LineConfig {
                        x_col: "x".into(),
                        y_col: "y".into(),
                        smooth: ls.smooth.unwrap_or(false),
                        line_width: ls.line_style.as_ref().and_then(|l| l.width).unwrap_or(2.0),
                        area: ls.area_style.is_some(),
                        area_color: ls
                            .area_style
                            .as_ref()
                            .and_then(|a| a.color)
                            .map(|c| crate::visual::Color::new(c.r, c.g, c.b)),
                        area_opacity: ls
                            .area_style
                            .as_ref()
                            .and_then(|a| a.opacity)
                            .unwrap_or(0.5),
                        symbol_type: ls
                            .symbol
                            .as_ref()
                            .map(|s| match s {
                                crate::option::SymbolType::Circle => SymbolType::Circle,
                                crate::option::SymbolType::Rect => SymbolType::Rect,
                                crate::option::SymbolType::RoundRect => SymbolType::RoundRect,
                                crate::option::SymbolType::Triangle => SymbolType::Triangle,
                                crate::option::SymbolType::Diamond => SymbolType::Diamond,
                                crate::option::SymbolType::Pin => SymbolType::Pin,
                                crate::option::SymbolType::Arrow => SymbolType::Arrow,
                                crate::option::SymbolType::None => SymbolType::None,
                            })
                            .unwrap_or_default(),
                        symbol_size: ls.symbol_size.unwrap_or(4.0),
                        label_show: false,
                        label_font_size: 12.0,
                    };
                    SeriesSpec {
                        name,
                        chart_type: ChartType::Line,
                        data,
                        grid_index: ls.grid_index.unwrap_or(0),
                        x_axis_index: 0,
                        y_axis_index: ls.y_axis_index.unwrap_or(0),
                        stack: ls.stack.clone(),
                        group_index: 0,
                        sampling,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::Line(config),
                    }
                }
                SeriesOption::Bar(bs) => {
                    // 判断是否为水平柱状图：Y轴是分类轴
                    let y_axis_idx = bs.y_axis_index.unwrap_or(0);
                    let x_axis_idx = 0; // Bar 系列目前只支持 xAxisIndex=0
                    let is_horizontal = y_axes
                        .get(y_axis_idx)
                        .map(|a| matches!(a.axis_type, NewAxisType::Category))
                        .unwrap_or(false);

                    // dataset 模式下数据来自 dataset，不需要水平/垂直转换
                    let (data, x_col, y_col) = if bs.dataset_index.is_some() {
                        let df = resolve_series_data(
                            s,
                            &datasets,
                            || datapoints_to_dataframe(&bs.data, "x", "y"),
                            "x",
                            "y",
                        );
                        (df, "x".into(), "y".into())
                    } else if is_horizontal {
                        // 水平柱状图：X轴是数值，Y轴是分类
                        // 将数据放入 X 列，索引放入 Y 列
                        let df = datapoints_to_dataframe_horizontal(&bs.data);
                        (df, "x".into(), "y".into())
                    } else {
                        // 纵向柱状图：X轴是分类，Y轴是数值
                        let df = datapoints_to_dataframe(&bs.data, "x", "y");
                        (df, "x".into(), "y".into())
                    };

                    let bar_width = bs
                        .bar_width
                        .as_ref()
                        .and_then(|bw| bw.strip_suffix('%'))
                        .and_then(|pct| pct.parse::<f64>().ok())
                        .map(|v| v / 100.0)
                        .unwrap_or(0.6);
                    let config = BarConfig {
                        x_col,
                        y_col,
                        bar_width,
                        label_show: false,
                        label_font_size: 12.0,
                    };
                    SeriesSpec {
                        name,
                        chart_type: ChartType::Bar,
                        data,
                        grid_index: bs.grid_index.unwrap_or(0),
                        x_axis_index: x_axis_idx,
                        y_axis_index: y_axis_idx,
                        stack: bs.stack.clone(),
                        group_index: bs.group_index.unwrap_or(0),
                        sampling,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::Bar(config),
                    }
                }
                SeriesOption::Pie(ps) => {
                    let data = resolve_series_data(
                        s,
                        &datasets,
                        || datapoints_to_dataframe(&ps.data, "name", "value"),
                        "name",
                        "value",
                    );
                    let label = ps.label.as_ref();
                    // 解析 center 和 radius (从 Vec<String> 转换为 (f64, f64))
                    let center = ps
                        .center
                        .as_ref()
                        .and_then(|c| {
                            if c.len() >= 2 {
                                let x = c[0].trim_end_matches('%').parse::<f64>().ok()?;
                                let y = c[1].trim_end_matches('%').parse::<f64>().ok()?;
                                Some((x, y))
                            } else {
                                None
                            }
                        })
                        .unwrap_or((50.0, 50.0));
                    let radius = ps
                        .radius
                        .as_ref()
                        .and_then(|r| {
                            if r.len() >= 2 {
                                let inner = r[0].trim_end_matches('%').parse::<f64>().ok()?;
                                let outer = r[1].trim_end_matches('%').parse::<f64>().ok()?;
                                Some((inner, outer))
                            } else {
                                None
                            }
                        })
                        .unwrap_or((0.0, 75.0));
                    let config = PieConfig {
                        category_col: "name".into(),
                        value_col: "value".into(),
                        center,
                        radius,
                        label_show: label.and_then(|l| l.show).unwrap_or(false),
                        label_position: label
                            .and_then(|l| l.position)
                            .map(|p| match p {
                                crate::option::LabelPosition::Outside => {
                                    crate::pipeline::types::LabelPosition::Outside
                                }
                                crate::option::LabelPosition::Inside => {
                                    crate::pipeline::types::LabelPosition::Inside
                                }
                                _ => crate::pipeline::types::LabelPosition::Outside,
                            })
                            .unwrap_or(crate::pipeline::types::LabelPosition::Outside),
                        label_font_size: label.and_then(|l| l.font_size).unwrap_or(12.0),
                    };
                    SeriesSpec {
                        name,
                        chart_type: ChartType::Pie,
                        data,
                        grid_index: ps.grid_index.unwrap_or(0),
                        x_axis_index: 0,
                        y_axis_index: 0,
                        stack: None,
                        group_index: 0,
                        sampling,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::Pie(config),
                    }
                }
                SeriesOption::Scatter(ss) => {
                    let data = resolve_series_data(
                        s,
                        &datasets,
                        || datapoints_to_dataframe(&ss.data, "x", "y"),
                        "x",
                        "y",
                    );
                    let config = ScatterConfig {
                        x_col: "x".into(),
                        y_col: "y".into(),
                        symbol_size: 10.0,
                    };
                    SeriesSpec {
                        name,
                        chart_type: ChartType::Scatter,
                        data,
                        grid_index: ss.grid_index.unwrap_or(0),
                        x_axis_index: 0,
                        y_axis_index: ss.y_axis_index.unwrap_or(0),
                        stack: None,
                        group_index: 0,
                        sampling,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::Scatter(config),
                    }
                }
                SeriesOption::Bubble(bs) => {
                    let data = DataFrame::new();
                    let config = crate::pipeline::types::BubbleConfig {
                        x_col: "x".into(),
                        y_col: "y".into(),
                        size_col: None,
                        name_col: None,
                        symbol_size_scale: 1.0,
                    };
                    SeriesSpec {
                        name,
                        chart_type: ChartType::Bubble,
                        data,
                        grid_index: bs.grid_index.unwrap_or(0),
                        x_axis_index: bs.x_axis_index.unwrap_or(0),
                        y_axis_index: bs.y_axis_index.unwrap_or(0),
                        stack: None,
                        group_index: 0,
                        sampling: None,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::Bubble(config),
                    }
                }
                SeriesOption::Candlestick(cs) => {
                    let data = DataFrame::new();
                    let config = crate::pipeline::types::CandlestickConfig {
                        category_col: "category".into(),
                        open_col: "open".into(),
                        close_col: "close".into(),
                        low_col: "low".into(),
                        high_col: "high".into(),
                    };
                    SeriesSpec {
                        name,
                        chart_type: ChartType::Candlestick,
                        data,
                        grid_index: cs.grid_index.unwrap_or(0),
                        x_axis_index: cs.x_axis_index.unwrap_or(0),
                        y_axis_index: cs.y_axis_index.unwrap_or(0),
                        stack: None,
                        group_index: 0,
                        sampling: None,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::Candlestick(config),
                    }
                }
                SeriesOption::Boxplot(bs) => {
                    let mut data = DataFrame::new();
                    let categories: Vec<DataValue> = bs
                        .data
                        .iter()
                        .enumerate()
                        .map(|(i, d)| {
                            DataValue::from(d.name.clone().unwrap_or_else(|| (i + 1).to_string()))
                        })
                        .collect();
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "category", categories,
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "min",
                        bs.data.iter().map(|d| DataValue::from(d.min)).collect(),
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "q1",
                        bs.data.iter().map(|d| DataValue::from(d.q1)).collect(),
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "median",
                        bs.data.iter().map(|d| DataValue::from(d.median)).collect(),
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "q3",
                        bs.data.iter().map(|d| DataValue::from(d.q3)).collect(),
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "max",
                        bs.data.iter().map(|d| DataValue::from(d.max)).collect(),
                    ));
                    let config = crate::pipeline::types::BoxplotConfig {
                        category_col: "category".into(),
                        min_col: "min".into(),
                        q1_col: "q1".into(),
                        median_col: "median".into(),
                        q3_col: "q3".into(),
                        max_col: "max".into(),
                    };
                    let item_style = ItemStyleSpec {
                        color: bs.item_style.as_ref().and_then(|is| {
                            is.color
                                .as_ref()
                                .map(|c| crate::visual::Color::new(c.r, c.g, c.b))
                        }),
                        border_color: bs.item_style.as_ref().and_then(|is| {
                            is.border_color
                                .as_ref()
                                .map(|c| crate::visual::Color::new(c.r, c.g, c.b))
                        }),
                        border_width: bs.item_style.as_ref().and_then(|is| is.border_width),
                        opacity: None,
                    };
                    SeriesSpec {
                        name,
                        chart_type: ChartType::Boxplot,
                        data,
                        grid_index: bs.grid_index.unwrap_or(0),
                        x_axis_index: bs.x_axis_index.unwrap_or(0),
                        y_axis_index: bs.y_axis_index.unwrap_or(0),
                        stack: None,
                        group_index: 0,
                        sampling: None,
                        item_style,
                        config: SeriesConfig::Boxplot(config),
                    }
                }
                SeriesOption::Radar(rs) => {
                    let mut data = DataFrame::new();
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "value",
                        rs.data
                            .iter()
                            .flat_map(|d| d.value.iter().cloned().map(DataValue::from))
                            .collect(),
                    ));
                    let indicators: Vec<String> = option
                        .radar
                        .as_ref()
                        .map(|r| {
                            r.indicator
                                .as_ref()
                                .map(|v| {
                                    v.iter().filter_map(|i| i.name.clone()).collect::<Vec<_>>()
                                })
                                .unwrap_or_default()
                        })
                        .unwrap_or_default();
                    let config = crate::pipeline::types::RadarConfig {
                        value_col: "value".into(),
                        indicators,
                    };
                    // 雷达图不使用 grid/axis，但需要设置默认索引
                    let grid_index = 0;
                    SeriesSpec {
                        name,
                        chart_type: ChartType::Radar,
                        data,
                        grid_index,
                        x_axis_index: 0,
                        y_axis_index: 0,
                        stack: None,
                        group_index: 0,
                        sampling: None,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::Radar(config),
                    }
                }
                SeriesOption::PolarBar(pb) => {
                    let data = datapoints_to_dataframe(&pb.data, "angle", "radius");
                    let config = crate::pipeline::types::PolarBarConfig {
                        angle_col: "angle".into(),
                        radius_col: "radius".into(),
                        pad_angle: 2.0,
                        start_angle: 90.0,
                    };
                    SeriesSpec {
                        name,
                        chart_type: ChartType::PolarBar,
                        data,
                        grid_index: 0,
                        x_axis_index: 0,
                        y_axis_index: 0,
                        stack: None,
                        group_index: 0,
                        sampling: None,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::PolarBar(config),
                    }
                }
                SeriesOption::PolarScatter(_ps) => {
                    let data = DataFrame::new();
                    let config = crate::pipeline::types::PolarScatterConfig {
                        angle_col: "angle".into(),
                        radius_col: "radius".into(),
                        symbol_size: 8.0,
                    };
                    SeriesSpec {
                        name,
                        chart_type: ChartType::PolarScatter,
                        data,
                        grid_index: 0,
                        x_axis_index: 0,
                        y_axis_index: 0,
                        stack: None,
                        group_index: 0,
                        sampling: None,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::PolarScatter(config),
                    }
                }
                SeriesOption::Gauge(_gs) => {
                    let data = DataFrame::new();
                    let config = crate::pipeline::types::GaugeConfig {
                        value_col: "value".into(),
                        min: 0.0,
                        max: 100.0,
                        center: (50.0, 50.0),
                        radius: 75.0,
                        start_angle: 225.0,
                        end_angle: -45.0,
                        split_number: 10,
                    };
                    SeriesSpec {
                        name,
                        chart_type: ChartType::Gauge,
                        data,
                        grid_index: 0,
                        x_axis_index: 0,
                        y_axis_index: 0,
                        stack: None,
                        group_index: 0,
                        sampling: None,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::Gauge(config),
                    }
                }
                SeriesOption::Table(_ts) => {
                    let data = DataFrame::new();
                    let config = crate::pipeline::types::TableConfig;
                    SeriesSpec {
                        name,
                        chart_type: ChartType::Table,
                        data,
                        grid_index: 0,
                        x_axis_index: 0,
                        y_axis_index: 0,
                        stack: None,
                        group_index: 0,
                        sampling: None,
                        item_style: ItemStyleSpec::default(),
                        config: SeriesConfig::Table(config),
                    }
                }
                SeriesOption::Unknown => unreachable!(),
            };
            Some(spec)
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
            data: l
                .data
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|i| i.name().to_string())
                .collect(),
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
        // DataPoint::Value only — add x as index + y column
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
        let xs: Vec<DataValue> = (0..points.len())
            .map(|i| DataValue::Float(i as f64))
            .collect();
        df.add_column(DfSeries::new(x_col, xs));
        df.add_column(DfSeries::new(y_col, values));
    }

    df
}

/// 为水平柱状图转换数据点
/// 水平柱状图：X轴是数值轴，Y轴是分类轴
/// 将数据值放入 X 列，索引放入 Y 列
fn datapoints_to_dataframe_horizontal(
    points: &[option::DataPoint],
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
        // Named 模式: (category, value) -> X=value, Y=index
        let xs: Vec<DataValue> = points
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
        let ys: Vec<DataValue> = (0..points.len())
            .map(|i| DataValue::Float(i as f64))
            .collect();
        df.add_column(DfSeries::new("x", xs));
        df.add_column(DfSeries::new("y", ys));
    } else if is_xy {
        // XY 模式: (x, y) -> X=x, Y=index
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
        let ys: Vec<DataValue> = (0..points.len())
            .map(|i| DataValue::Float(i as f64))
            .collect();
        df.add_column(DfSeries::new("x", xs));
        df.add_column(DfSeries::new("y", ys));
    } else {
        // Value 模式: value -> X=value, Y=index
        let xs: Vec<DataValue> = points
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
        let ys: Vec<DataValue> = (0..points.len())
            .map(|i| DataValue::Float(i as f64))
            .collect();
        df.add_column(DfSeries::new("x", xs));
        df.add_column(DfSeries::new("y", ys));
    }

    df
}

/// 将 serde_json::Value 转换为 DataValue
fn serde_value_to_data_value(v: &serde_json::Value) -> DataValue {
    match v {
        serde_json::Value::Null => DataValue::Null,
        serde_json::Value::Bool(b) => DataValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                DataValue::Float(f)
            } else {
                DataValue::Null
            }
        }
        serde_json::Value::String(s) => DataValue::String(s.clone()),
        _ => DataValue::Null,
    }
}

/// 将 DatasetOption.source 转换为 DataFrame
///
/// 如果 source_header 为 true（默认），第一行作为列名；
/// 否则使用 column0, column1, ... 作为列名。
fn dataset_to_dataframe(dataset: &option::DatasetOption) -> crate::pipeline::dataframe::DataFrame {
    use crate::pipeline::dataframe::{DataFrame, Series as DfSeries};

    let source = match &dataset.source {
        Some(s) => s,
        None => return DataFrame::new(),
    };

    if source.is_empty() {
        return DataFrame::new();
    }

    let has_header = dataset.source_header.unwrap_or(true);
    let data_start = if has_header && !source.is_empty() {
        1
    } else {
        0
    };

    let mut df = DataFrame::new();

    if data_start > 0 {
        // 第一行是列名
        let header_row = &source[0];
        let col_names: Vec<String> = header_row
            .iter()
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                other => format!("{}", other),
            })
            .collect();

        let num_cols = col_names.len();
        let mut col_data: Vec<Vec<DataValue>> = vec![Vec::new(); num_cols];

        for row in source.iter().skip(data_start) {
            for (i, val) in row.iter().enumerate() {
                if i < num_cols {
                    col_data[i].push(serde_value_to_data_value(val));
                }
            }
        }

        // 补齐短行
        for row_data in col_data.iter_mut() {
            while row_data.len() < source.len() - data_start {
                row_data.push(DataValue::Null);
            }
        }

        for (i, name) in col_names.iter().enumerate() {
            df.add_column(DfSeries::new(name.clone(), col_data[i].clone()));
        }
    } else {
        // 无列名，使用 column0, column1, ...
        let num_cols = source[0].len();
        let mut col_data: Vec<Vec<DataValue>> = vec![Vec::new(); num_cols];

        for row in source.iter() {
            for (i, val) in row.iter().enumerate() {
                if i < num_cols {
                    col_data[i].push(serde_value_to_data_value(val));
                }
            }
        }

        for (i, data) in col_data.iter().enumerate() {
            df.add_column(DfSeries::new(format!("column{}", i), data.clone()));
        }
    }

    df
}

/// 从 DataFrame 中按 encode 映射提取 x/y 列，构造新的 DataFrame
fn extract_encoded_columns(
    df: &crate::pipeline::dataframe::DataFrame,
    encode: &option::SeriesEncodeOption,
) -> (crate::pipeline::dataframe::DataFrame, String, String) {
    use crate::pipeline::dataframe::{DataFrame, Series as DfSeries};

    let col_names = df.column_names().to_vec();
    let mut result_df = DataFrame::new();
    let mut x_col = String::from("x");
    let mut y_col = String::from("y");

    /// 从 OneOrMany<StringOrInt> 中取第一个值并解析为列名（既支持索引，也支持列名字符串）
    fn first_column_name(
        v: &Option<option::OneOrMany<option::StringOrInt>>,
        col_names: &[String],
    ) -> Option<String> {
        let first: Option<&option::StringOrInt> = match v {
            Some(option::OneOrMany::One(item)) => Some(item),
            Some(option::OneOrMany::Many(vec)) => vec.first(),
            None => None,
        };
        first.and_then(|si| match si {
            option::StringOrInt::Str(s) => Some(s.clone()),
            option::StringOrInt::Int(idx) => col_names.get(*idx).cloned(),
        })
    }

    /// 判断 OneOrMany 是否为空或 None
    fn is_empty_or_none(v: &Option<option::OneOrMany<option::StringOrInt>>) -> bool {
        match v {
            None => true,
            Some(option::OneOrMany::One(_)) => false,
            Some(option::OneOrMany::Many(vec)) => vec.is_empty(),
        }
    }

    // 处理 encode.x → 取对应列重命名为 "x"
    if let Some(src_name) = first_column_name(&encode.x, &col_names)
        && let Some(col) = df.get_column(&src_name)
    {
        result_df.add_column(DfSeries::new("x", col.data.clone()));
        x_col = "x".into();
    }

    // 处理 encode.y → 取对应列重命名为 "y"
    if let Some(src_name) = first_column_name(&encode.y, &col_names)
        && let Some(col) = df.get_column(&src_name)
    {
        result_df.add_column(DfSeries::new("y", col.data.clone()));
        y_col = "y".into();
    }

    // 如果 encode 没有指定 x，但指定了 itemName，用 itemName 作为 x
    if is_empty_or_none(&encode.x)
        && let Some(src_name) = first_column_name(&encode.item_name, &col_names)
        && let Some(col) = df.get_column(&src_name)
        && result_df.get_column("x").is_none()
    {
        result_df.add_column(DfSeries::new("x", col.data.clone()));
        x_col = "x".into();
    }

    // 如果 encode 没有指定 y，但指定了 value，用 value 作为 y
    if is_empty_or_none(&encode.y)
        && let Some(src_name) = first_column_name(&encode.value, &col_names)
        && let Some(col) = df.get_column(&src_name)
        && result_df.get_column("y").is_none()
    {
        result_df.add_column(DfSeries::new("y", col.data.clone()));
        y_col = "y".into();
    }

    // 如果 result_df 为空（没有 encode 或 encode 无效），复制整个 DataFrame
    if result_df.column_count() == 0 {
        return (df.clone(), "x".into(), "y".into());
    }

    (result_df, x_col, y_col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::dataframe::{DataFrame, DataValue, Series};

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
