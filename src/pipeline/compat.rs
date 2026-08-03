//! ChartOption → ChartSpec 单向转换层
//!
//! 将 ECharts JSON 反序列化而来的 ChartOption 转换为 ChartSpec（管线核心类型）。
//! 此模块是 ECharts 兼容层的核心，仅 ChartOption → ChartSpec 单向转换。

use crate::{
    option::{self, ChartOption, PositionOption, PositionPreset, SeriesOption},
    pipeline::{
        dataframe::DataValue,
        types::{
            ChartSpec, SeriesConfig,
        },
    },
};

/// 将旧的 ChartOption 转换为新的 ChartSpec（反向兼容）
pub fn chart_option_to_chart_spec(option: &ChartOption, width: u32, height: u32) -> ChartSpec {
    use crate::pipeline::types::{
        AxisPosition, AxisSpec, AxisType as NewAxisType, BarConfig, GridSpec, HeatmapConfig,
        ItemStyleSpec, LegendSpec, LineConfig, PieConfig, ScatterConfig, SeriesSpec, StepType,
        SymbolType, TitleSpec,
    };
    use crate::sampling::SamplingType;

    let total_w = width as f64;
    let total_h = height as f64;

    // Grids
    let grids: Vec<GridSpec> = option
        .grid
        .iter()
        .map(|g| {
            let left = g.left.as_ref().map(|p| resolve_position_option(p, total_w));
            let right = g.right.as_ref().map(|p| resolve_position_option(p, total_w));
            let top = g.top.as_ref().map(|p| resolve_position_option(p, total_h));
            let bottom = g.bottom.as_ref().map(|p| resolve_position_option(p, total_h));
            GridSpec {
                left,
                right,
                top,
                bottom,
                contain_label: g.contain_label.unwrap_or(false),
            }
        })
        .collect();
    let has_cartesian_series = option.series.iter().any(|s| {
        matches!(
            s,
            SeriesOption::Line(_)
                | SeriesOption::Bar(_)
                | SeriesOption::Scatter(_)
                | SeriesOption::Bubble(_)
                | SeriesOption::Candlestick(_)
                | SeriesOption::Boxplot(_)
                | SeriesOption::Heatmap(_)
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
                min: a.min.and_then(|m| match m {
                    crate::option::LenientAxisLimit::Value(v) => Some(v),
                    _ => None,
                }),
                max: a.max.and_then(|m| match m {
                    crate::option::LenientAxisLimit::Value(v) => Some(v),
                    _ => None,
                }),
                name: a.name.clone(),
                name_location: a.name_location.as_ref().map(|l| format!("{:?}", l)),
                categories: a.data.as_ref().map(|d| d.0.clone()).unwrap_or_default(),
                boundary_gap: a.boundary_gap.as_ref().is_none_or(|bg| match bg {
                    crate::option::LenientBoundaryGap::Bool(b) => *b,
                    crate::option::LenientBoundaryGap::Gap(_, _) => true,
                }),
                inverse: a.inverse.unwrap_or(false),
                split_number: None,
                label_show: a.axis_label.as_ref().map(|l| l.show.unwrap_or(true)).unwrap_or(true),
                label_formatter: a.axis_label.as_ref().and_then(|l| l.formatter.clone()),
                label_rotate: a.axis_label.as_ref().and_then(|l| l.rotate),
                axis_line_show: a.axis_line.as_ref().map(|l| l.show.unwrap_or(true)).unwrap_or(true),
                split_line_show: a.split_line.as_ref().map(|l| l.show.unwrap_or(true)).unwrap_or(true),
                z: a.z.or(a.zlevel),
            }
        })
        .collect();

    // Y Axes
    let y_axes: Vec<AxisSpec> = option
        .y_axis
        .iter()
        .enumerate()
        .map(|(axis_idx, a)| {
            let new_axis_type = match a.axis_type.unwrap_or(crate::option::AxisType::Value) {
                crate::option::AxisType::Value => NewAxisType::Value,
                crate::option::AxisType::Category => NewAxisType::Category,
                crate::option::AxisType::Time => NewAxisType::Time,
                crate::option::AxisType::Log => NewAxisType::Log,
            };
            // Y 轴位置缺省时：第一个在左、后续在右（ECharts 默认）
            let default_position = if axis_idx == 0 {
                crate::option::AxisPosition::Left
            } else {
                crate::option::AxisPosition::Right
            };
            let new_position = match a.position.unwrap_or(default_position) {
                crate::option::AxisPosition::Left => AxisPosition::Left,
                crate::option::AxisPosition::Right => AxisPosition::Right,
                crate::option::AxisPosition::Bottom => AxisPosition::Bottom,
                crate::option::AxisPosition::Top => AxisPosition::Top,
            };
            AxisSpec {
                axis_type: new_axis_type,
                position: new_position,
                grid_index: a.grid_index.unwrap_or(0),
                min: a.min.and_then(|m| match m {
                    crate::option::LenientAxisLimit::Value(v) => Some(v),
                    _ => None,
                }),
                max: a.max.and_then(|m| match m {
                    crate::option::LenientAxisLimit::Value(v) => Some(v),
                    _ => None,
                }),
                name: a.name.clone(),
                name_location: a.name_location.as_ref().map(|l| format!("{:?}", l)),
                categories: a.data.as_ref().map(|d| d.0.clone()).unwrap_or_default(),
                boundary_gap: a.boundary_gap.as_ref().is_none_or(|bg| match bg {
                    crate::option::LenientBoundaryGap::Bool(b) => *b,
                    crate::option::LenientBoundaryGap::Gap(_, _) => true,
                }),
                inverse: a.inverse.unwrap_or(false),
                split_number: None,
                label_show: a.axis_label.as_ref().map(|l| l.show.unwrap_or(true)).unwrap_or(true),
                label_formatter: a.axis_label.as_ref().and_then(|l| l.formatter.clone()),
                label_rotate: a.axis_label.as_ref().and_then(|l| l.rotate),
                axis_line_show: a.axis_line.as_ref().map(|l| l.show.unwrap_or(true)).unwrap_or(true),
                split_line_show: a.split_line.as_ref().map(|l| l.show.unwrap_or(true)).unwrap_or(true),
                z: a.z.or(a.zlevel),
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
            SeriesOption::Heatmap(hs) => (hs.dataset_index, hs.encode.clone()),
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
            && let Some(ds_df) = datasets.get(idx) {
                if let Some(enc) = &encode {
                    return extract_encoded_columns(ds_df, enc).0;
                }
                if ds_df.get_column("x").is_some() && ds_df.get_column("y").is_some() {
                    return ds_df.clone();
                }
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
                SeriesOption::Heatmap(hs) => hs.name.clone().unwrap_or_default(),
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
                        step: ls.step.as_ref().and_then(|s| match s {
                            crate::option::LenientStep::Bool(_) => None,
                            crate::option::LenientStep::Start => Some(StepType::Start),
                            crate::option::LenientStep::Middle => Some(StepType::Middle),
                            crate::option::LenientStep::End => Some(StepType::End),
                        }),
                        line_width: ls.line_style.as_ref().and_then(|l| l.width.as_ref().and_then(|w| w.as_number())).unwrap_or(2.0),
                        area: ls.area_style.is_some(),
                        area_color: ls
                            .area_style
                            .as_ref()
                            .and_then(|a| a.color.as_ref())
                            .and_then(|c| {
                                let v = c.as_vec();
                                v.first().map(|first| crate::visual::Color::new(first.r, first.g, first.b))
                            }),
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
                                crate::option::SymbolType::EmptyCircle => SymbolType::EmptyCircle,
                                crate::option::SymbolType::Rect => SymbolType::Rect,
                                crate::option::SymbolType::RoundRect => SymbolType::RoundRect,
                                crate::option::SymbolType::Triangle => SymbolType::Triangle,
                                crate::option::SymbolType::Diamond => SymbolType::Diamond,
                                crate::option::SymbolType::Pin => SymbolType::Pin,
                                crate::option::SymbolType::Arrow => SymbolType::Arrow,
                                crate::option::SymbolType::None => SymbolType::None,
                            })
                            .unwrap_or(SymbolType::EmptyCircle),
                        symbol_size: ls.symbol_size.as_ref().and_then(|v| v.as_number()).unwrap_or(4.0),
                        label_show: false,
                        label_font_size: 12.0,
                    };
                    SeriesSpec {
                        name,
                        
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
                    let y_axis_idx = bs.y_axis_index.unwrap_or(0);
                    let x_axis_idx = 0;
                    let is_horizontal = y_axes
                        .get(y_axis_idx)
                        .map(|a| matches!(a.axis_type, NewAxisType::Category))
                        .unwrap_or(false);

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
                        let df = datapoints_to_dataframe_horizontal(&bs.data);
                        (df, "x".into(), "y".into())
                    } else {
                        let df = datapoints_to_dataframe(&bs.data, "x", "y");
                        (df, "x".into(), "y".into())
                    };

                    let bar_width = bs
                        .bar_width
                        .as_ref()
                        .map(|bw| &bw.0)
                        .and_then(|bw| {
                            if let Some(pct) = bw.strip_suffix('%') {
                                pct.parse::<f64>().ok().map(|v| v / 100.0)
                            } else {
                                bw.parse::<f64>().ok().map(|v| v / 100.0)
                            }
                        })
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
                    let center = ps
                        .center
                        .as_ref()
                        .and_then(|c| {
                            if c.len() >= 2 {
                                let x = c[0].as_number()?;
                                let y = c[1].as_number()?;
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
                            let v = r.to_vec();
                            if v.len() >= 2 {
                                let inner = v[0].as_number()?;
                                let outer = v[1].as_number()?;
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
                    let data = crate::pipeline::dataframe::DataFrame::new();
                    let config = crate::pipeline::types::BubbleConfig {
                        x_col: "x".into(),
                        y_col: "y".into(),
                        size_col: None,
                        name_col: None,
                        symbol_size_scale: 1.0,
                    };
                    SeriesSpec {
                        name,
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
                    // ECharts K 线数据：[open, close, low, high]，名称缺省时用序号
                    let mut data = crate::pipeline::dataframe::DataFrame::new();
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "category",
                        cs.data
                            .iter()
                            .enumerate()
                            .map(|(i, d)| {
                                DataValue::from(
                                    d.name.clone().unwrap_or_else(|| (i + 1).to_string()),
                                )
                            })
                            .collect(),
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "open",
                        cs.data.iter().map(|d| DataValue::from(d.open)).collect(),
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "close",
                        cs.data.iter().map(|d| DataValue::from(d.close)).collect(),
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "low",
                        cs.data.iter().map(|d| DataValue::from(d.low)).collect(),
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "high",
                        cs.data.iter().map(|d| DataValue::from(d.high)).collect(),
                    ));
                    let config = crate::pipeline::types::CandlestickConfig {
                        category_col: "category".into(),
                        open_col: "open".into(),
                        close_col: "close".into(),
                        low_col: "low".into(),
                        high_col: "high".into(),
                    };
                    SeriesSpec {
                        name,
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
                    let mut data = crate::pipeline::dataframe::DataFrame::new();
                    let categories: Vec<DataValue> = bs
                        .data
                        .iter()
                        .enumerate()
                        .map(|(i, d)| {
                            DataValue::from(d.name.clone().unwrap_or_else(|| (i + 1).to_string()))
                        })
                        .collect();
                    data.add_column(crate::pipeline::dataframe::Series::new("category", categories));
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
                        border_width: bs.item_style.as_ref().and_then(|is| is.border_width.as_ref().and_then(|v| v.as_number())),
                        opacity: None,
                    };
                    SeriesSpec {
                        name,
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
                SeriesOption::Heatmap(hs) => {
                    let mut data = crate::pipeline::dataframe::DataFrame::new();
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "x",
                        hs.data.iter().map(|d| DataValue::Float(d.x)).collect(),
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "y",
                        hs.data.iter().map(|d| DataValue::Float(d.y)).collect(),
                    ));
                    data.add_column(crate::pipeline::dataframe::Series::new(
                        "value",
                        hs.data.iter().map(|d| DataValue::Float(d.value)).collect(),
                    ));

                    // visualMap → 热力图颜色映射
                    let (vm_min, vm_max, vm_colors) = resolve_visual_map(option, &data);

                    let item_style = ItemStyleSpec {
                        color: None,
                        border_color: hs.item_style.as_ref().and_then(|is| {
                            is.border_color
                                .as_ref()
                                .map(|c| crate::visual::Color::new(c.r, c.g, c.b))
                        }),
                        border_width: hs
                            .item_style
                            .as_ref()
                            .and_then(|is| is.border_width.as_ref().and_then(|v| v.as_number())),
                        opacity: hs
                            .item_style
                            .as_ref()
                            .and_then(|is| is.opacity),
                    };
                    let config = HeatmapConfig {
                        x_col: "x".into(),
                        y_col: "y".into(),
                        value_col: "value".into(),
                        min: vm_min,
                        max: vm_max,
                        colors: vm_colors,
                        border_color: item_style.border_color,
                        border_width: item_style.border_width.unwrap_or(0.0),
                        label_show: hs
                            .label
                            .as_ref()
                            .and_then(|l| l.show)
                            .unwrap_or(false),
                        label_font_size: hs
                            .label
                            .as_ref()
                            .and_then(|l| l.font_size)
                            .unwrap_or(12.0),
                    };
                    SeriesSpec {
                        name,
                        data,
                        grid_index: hs.grid_index.unwrap_or(0),
                        x_axis_index: hs.x_axis_index.unwrap_or(0),
                        y_axis_index: hs.y_axis_index.unwrap_or(0),
                        stack: None,
                        group_index: 0,
                        sampling: None,
                        item_style,
                        config: SeriesConfig::Heatmap(config),
                    }
                }
                SeriesOption::Radar(rs) => {
                    let mut data = crate::pipeline::dataframe::DataFrame::new();
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
                                    v.iter()
                                        .filter_map(|i| i.name.clone())
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default()
                        })
                        .unwrap_or_default();
                    let config = crate::pipeline::types::RadarConfig {
                        value_col: "value".into(),
                        indicators,
                    };
                    let grid_index = 0;
                    SeriesSpec {
                        name,
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
                    let data = crate::pipeline::dataframe::DataFrame::new();
                    let config = crate::pipeline::types::PolarScatterConfig {
                        angle_col: "angle".into(),
                        radius_col: "radius".into(),
                        symbol_size: 8.0,
                    };
                    SeriesSpec {
                        name,
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
                    let data = crate::pipeline::dataframe::DataFrame::new();
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
                    let data = crate::pipeline::dataframe::DataFrame::new();
                    let config = crate::pipeline::types::TableConfig;
                    SeriesSpec {
                        name,
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
            font_size: t.text_style.as_ref().and_then(|s| s.font_size),
            subfont_size: t.subtext_style.as_ref().and_then(|s| s.font_size),
            color: t.text_style.as_ref().and_then(|s| s.color.as_ref()).map(|c| crate::visual::Color::new(c.r, c.g, c.b)),
            subcolor: t.subtext_style.as_ref().and_then(|s| s.color.as_ref()).map(|c| crate::visual::Color::new(c.r, c.g, c.b)),
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
            symbol_size: l.symbol_size.as_ref().and_then(|v| v.as_number()).unwrap_or(10.0),
            item_gap: l.item_gap.unwrap_or(10.0),
        }),
        background: crate::visual::Color::new(255, 255, 255),
        palette: vec![],
        theme_name: None,
    }
}

/// 从 `option.visual_map` 解析热力图颜色映射。
///
/// 返回 `(min, max, 渐变颜色)`；min/max 缺失时由数据自动推断，
/// 颜色缺失时使用 ECharts heatmap 的默认三段渐变。
fn resolve_visual_map(
    option: &ChartOption,
    data: &crate::pipeline::dataframe::DataFrame,
) -> (Option<f64>, Option<f64>, Vec<crate::visual::Color>) {
    use crate::pipeline::dataframe::DataValue;

    let default_colors = vec![
        crate::visual::Color::new(80, 163, 186),  // #50a3ba
        crate::visual::Color::new(234, 199, 54),  // #eac736
        crate::visual::Color::new(217, 78, 93),   // #d94e5d
    ];

    // 数据范围（visualMap min/max 缺失时的回退值）
    let (data_min, data_max) = data
        .get_column("value")
        .map(|col| {
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;
            for v in &col.data {
                let f = match v {
                    DataValue::Float(f) => *f,
                    DataValue::Integer(i) => *i as f64,
                    _ => continue,
                };
                min = min.min(f);
                max = max.max(f);
            }
            if min.is_finite() && max.is_finite() {
                (Some(min), Some(max))
            } else {
                (None, None)
            }
        })
        .unwrap_or((None, None));

    let Some(vm) = option.visual_map.as_ref().and_then(|v| v.as_slice().first()) else {
        return (data_min, data_max, default_colors);
    };

    let min = vm.min.or(data_min);
    let max = vm.max.or(data_max);

    // in_range.color（OneOrMany）与顶层 color（Vec）均可作为渐变
    let raw_colors: Option<Vec<crate::option::ColorOption>> = vm
        .in_range
        .as_ref()
        .and_then(|r| r.color.as_ref())
        .map(|c| c.as_vec())
        .or_else(|| vm.color.clone());

    let colors: Vec<crate::visual::Color> = raw_colors
        .filter(|c| !c.is_empty())
        .map(|c| {
            c.iter()
                .map(|co| crate::visual::Color::new(co.r, co.g, co.b))
                .collect()
        })
        .unwrap_or(default_colors);

    (min, max, colors)
}

/// 将 PositionOption 解析为像素值
fn resolve_position_option(pos: &PositionOption, total: f64) -> f64 {
    match pos {
        PositionOption::Pixel(v) => *v,
        PositionOption::Percent(p) => total * p / 100.0,
        PositionOption::Preset(PositionPreset::Auto) => total * 0.1,
        PositionOption::Preset(PositionPreset::Center) => total / 2.0,
        PositionOption::Preset(PositionPreset::Left)
        | PositionOption::Preset(PositionPreset::Top) => 0.0,
        PositionOption::Preset(PositionPreset::Right)
        | PositionOption::Preset(PositionPreset::Bottom) => total,
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
fn datapoints_to_dataframe_horizontal(
    points: &[option::DataPoint],
) -> crate::pipeline::dataframe::DataFrame {
    use crate::pipeline::dataframe::{DataFrame, DataValue, Series as DfSeries};

    let mut df = DataFrame::new();

    if points.is_empty() {
        return df;
    }

    let is_named = matches!(points[0], option::DataPoint::Named(_, _));
    let is_xy = matches!(points[0], option::DataPoint::XY(_, _));

    if is_named {
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
    let data_start = if has_header && !source.is_empty() { 1 } else { 0 };

    let mut df = DataFrame::new();

    if data_start > 0 {
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

        for row_data in col_data.iter_mut() {
            while row_data.len() < source.len() - data_start {
                row_data.push(DataValue::Null);
            }
        }

        for (i, name) in col_names.iter().enumerate() {
            df.add_column(DfSeries::new(name.clone(), col_data[i].clone()));
        }
    } else {
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

/// 从 DataFrame 中按 encode 映射提取 x/y 列
fn extract_encoded_columns(
    df: &crate::pipeline::dataframe::DataFrame,
    encode: &option::SeriesEncodeOption,
) -> (crate::pipeline::dataframe::DataFrame, String, String) {
    use crate::pipeline::dataframe::{DataFrame, Series as DfSeries};

    let col_names = df.column_names().to_vec();
    let mut result_df = DataFrame::new();
    let mut x_col = String::from("x");
    let mut y_col = String::from("y");

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

    fn is_empty_or_none(
        v: &Option<option::OneOrMany<option::StringOrInt>>,
    ) -> bool {
        match v {
            None => true,
            Some(option::OneOrMany::One(_)) => false,
            Some(option::OneOrMany::Many(vec)) => vec.is_empty(),
        }
    }

    if let Some(src_name) = first_column_name(&encode.x, &col_names)
        && let Some(col) = df.get_column(&src_name) {
            result_df.add_column(DfSeries::new("x", col.data.clone()));
            x_col = "x".into();
        }

    if let Some(src_name) = first_column_name(&encode.y, &col_names)
        && let Some(col) = df.get_column(&src_name) {
            result_df.add_column(DfSeries::new("y", col.data.clone()));
            y_col = "y".into();
        }

    if is_empty_or_none(&encode.x)
        && let Some(src_name) = first_column_name(&encode.item_name, &col_names)
            && let Some(col) = df.get_column(&src_name)
                && result_df.get_column("x").is_none() {
                    result_df.add_column(DfSeries::new("x", col.data.clone()));
                    x_col = "x".into();
                }

    if is_empty_or_none(&encode.y)
        && let Some(src_name) = first_column_name(&encode.value, &col_names)
            && let Some(col) = df.get_column(&src_name)
                && result_df.get_column("y").is_none() {
                    result_df.add_column(DfSeries::new("y", col.data.clone()));
                    y_col = "y".into();
                }

    if result_df.column_count() == 0 {
        return (df.clone(), "x".into(), "y".into());
    }

    (result_df, x_col, y_col)
}
