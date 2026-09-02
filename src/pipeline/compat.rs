//! ChartOption → ChartSpec 单向转换层
//!
//! 将 ECharts JSON 反序列化而来的 ChartOption 转换为 ChartSpec（管线核心类型）。
//! 此模块是 ECharts 兼容层的核心，仅 ChartOption → ChartSpec 单向转换。

use lievisual::Color;

use crate::{
    option::{self, ChartOption, PositionOption, PositionPreset, SeriesOption},
    pipeline::{
        dataframe::DataValue,
        types::{ChartSpec, SeriesConfig},
    },
};

/// 将旧的 ChartOption 转换为新的 ChartSpec（反向兼容）
pub fn chart_option_to_chart_spec(option: &ChartOption, width: u32, height: u32) -> ChartSpec {
    use crate::{
        pipeline::types::{
            AxisPosition, AxisSpec, AxisType as NewAxisType, BarConfig, GridSpec, HeatmapConfig,
            ItemStyleSpec, LegendSpec, LineConfig, PieConfig, ScatterConfig, SeriesSpec, StepType,
            SymbolType, TitleSpec,
        },
        sampling::SamplingType,
    };

    // 注：P2b 起 grid 边距不再在此处折算（`total_w` / `total_h` 已无使用者），
    // 原始语义交由 `GridPlanner` 在布局阶段按当前画布解析。
    let grids: Vec<GridSpec> = option
        .grid
        .iter()
        .map(|g| {
            let left = g.left.as_ref().map(position_option_to_edge);
            let right = g.right.as_ref().map(position_option_to_edge);
            let top = g.top.as_ref().map(position_option_to_edge);
            let bottom = g.bottom.as_ref().map(position_option_to_edge);
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
            left: Some(crate::pipeline::types::GridEdge::Px(60.0)),
            right: Some(crate::pipeline::types::GridEdge::Px(60.0)),
            top: Some(crate::pipeline::types::GridEdge::Px(60.0)),
            bottom: Some(crate::pipeline::types::GridEdge::Px(60.0)),
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
                split_number: a.split_number,
                label_show: a
                    .axis_label
                    .as_ref()
                    .map(|l| l.show.unwrap_or(true))
                    .unwrap_or(true),
                label_formatter: a.axis_label.as_ref().and_then(|l| l.formatter.clone()),
                label_rotate: a.axis_label.as_ref().and_then(|l| l.rotate),
                axis_line_show: a
                    .axis_line
                    .as_ref()
                    .map(|l| l.show.unwrap_or(true))
                    .unwrap_or(true),
                split_line_show: a
                    .split_line
                    .as_ref()
                    .map(|l| l.show.unwrap_or(true))
                    .unwrap_or(true),
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
                split_number: a.split_number,
                label_show: a
                    .axis_label
                    .as_ref()
                    .map(|l| l.show.unwrap_or(true))
                    .unwrap_or(true),
                label_formatter: a.axis_label.as_ref().and_then(|l| l.formatter.clone()),
                label_rotate: a.axis_label.as_ref().and_then(|l| l.rotate),
                axis_line_show: a
                    .axis_line
                    .as_ref()
                    .map(|l| l.show.unwrap_or(true))
                    .unwrap_or(true),
                split_line_show: a
                    .split_line
                    .as_ref()
                    .map(|l| l.show.unwrap_or(true))
                    .unwrap_or(true),
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
            && let Some(ds_df) = datasets.get(idx)
        {
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
                    let x_is_time = axis_is_time(&x_axes, ls.x_axis_index.unwrap_or(0));
                    let data = resolve_series_data(
                        s,
                        &datasets,
                        || datapoints_to_dataframe(&ls.data, "x", "y", x_is_time),
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
                        line_width: ls
                            .line_style
                            .as_ref()
                            .and_then(|l| l.width.as_ref().and_then(|w| w.as_number()))
                            .unwrap_or(2.0),
                        area: ls.area_style.is_some(),
                        area_color: ls
                            .area_style
                            .as_ref()
                            .and_then(|a| a.color.as_ref())
                            .and_then(|c| {
                                let v = c.as_vec();
                                v.first().map(|first| Color::rgb(first.r, first.g, first.b))
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
                        symbol_size: ls
                            .symbol_size
                            .as_ref()
                            .and_then(|v| v.as_number())
                            .unwrap_or(4.0),
                        label_show: ls.label.as_ref().and_then(|l| l.show).unwrap_or(false),
                        label_font_size: ls
                            .label
                            .as_ref()
                            .and_then(|l| l.font_size)
                            .unwrap_or(12.0),
                        label_formatter: ls.label.as_ref().and_then(|l| l.formatter.clone()),
                        label_position: parse_value_label_position(ls.label.as_ref()),
                        label_color: parse_label_color(ls.label.as_ref()),
                        mark_line: parse_mark_line(ls.mark_line.as_ref()),
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
                    // ECharts 标准极坐标写法：type:"bar" + coordinateSystem:"polar"
                    if bs
                        .coordinate_system
                        .as_deref()
                        .map(|cs| cs.eq_ignore_ascii_case("polar"))
                        .unwrap_or(false)
                    {
                        let data = polar_datapoints_to_dataframe(&bs.data, "angle", "radius");
                        let config = crate::pipeline::types::PolarBarConfig {
                            angle_col: "angle".into(),
                            radius_col: "radius".into(),
                            category_col: None,
                            pad_angle: 2.0,
                            start_angle: 0.0,
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
                    } else {
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
                                || datapoints_to_dataframe(&bs.data, "x", "y", false),
                                "x",
                                "y",
                            );
                            (df, "x".into(), "y".into())
                        } else if is_horizontal {
                            let df = datapoints_to_dataframe_horizontal(&bs.data);
                            (df, "x".into(), "y".into())
                        } else {
                            let df = datapoints_to_dataframe(&bs.data, "x", "y", false);
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
                            label_show: bs.label.as_ref().and_then(|l| l.show).unwrap_or(false),
                            label_font_size: bs
                                .label
                                .as_ref()
                                .and_then(|l| l.font_size)
                                .unwrap_or(12.0),
                            label_formatter: bs.label.as_ref().and_then(|l| l.formatter.clone()),
                            label_position: parse_value_label_position(bs.label.as_ref()),
                            label_color: parse_label_color(bs.label.as_ref()),
                            mark_line: parse_mark_line(bs.mark_line.as_ref()),
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
                }
                SeriesOption::Pie(ps) => {
                    let data = resolve_series_data(
                        s,
                        &datasets,
                        || datapoints_to_dataframe(&ps.data, "name", "value", false),
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
                    // P2a：radius 百分比折算为绝对像素（基准 = 画布 min/2）
                    let radius = parse_pie_radius(ps.radius.as_ref()).unwrap_or((0.0, 75.0));
                    let radius = (
                        radius_percent_to_abs_px(radius.0, width, height),
                        radius_percent_to_abs_px(radius.1, width, height),
                    );
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
                        label_formatter: label.and_then(|l| l.formatter.clone()),
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
                    // ECharts 标准极坐标写法：type:"scatter" + coordinateSystem:"polar"
                    if ss
                        .coordinate_system
                        .as_deref()
                        .map(|cs| cs.eq_ignore_ascii_case("polar"))
                        .unwrap_or(false)
                    {
                        let data = polar_datapoints_to_dataframe(&ss.data, "angle", "radius");
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
                    } else {
                        let data = resolve_series_data(
                            s,
                            &datasets,
                            || datapoints_to_dataframe(&ss.data, "x", "y", false),
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
                    // 优先 dataset + datasetIndex（列名/encode 匹配）；否则用 series.data
                    let ds_idx = resolve_dataset_index(cs.dataset_index, !cs.data.is_empty());
                    let data = match candlestick_dataset_df(ds_idx, &cs.encode, &datasets) {
                        Some(df) => df,
                        None => {
                            // ECharts K 线数据：[open, close, low, high]，名称缺省时用序号
                            let mut d = crate::pipeline::dataframe::DataFrame::new();
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "category",
                                cs.data
                                    .iter()
                                    .enumerate()
                                    .map(|(i, dp)| {
                                        DataValue::from(
                                            dp.name.clone().unwrap_or_else(|| (i + 1).to_string()),
                                        )
                                    })
                                    .collect(),
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "open",
                                cs.data.iter().map(|dp| DataValue::from(dp.open)).collect(),
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "close",
                                cs.data.iter().map(|dp| DataValue::from(dp.close)).collect(),
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "low",
                                cs.data.iter().map(|dp| DataValue::from(dp.low)).collect(),
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "high",
                                cs.data.iter().map(|dp| DataValue::from(dp.high)).collect(),
                            ));
                            d
                        }
                    };
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
                    // 优先 dataset + datasetIndex；否则用 series.data
                    let ds_idx = resolve_dataset_index(bs.dataset_index, !bs.data.is_empty());
                    let data = match boxplot_dataset_df(ds_idx, &bs.encode, &datasets) {
                        Some(df) => df,
                        None => {
                            let mut d = crate::pipeline::dataframe::DataFrame::new();
                            let categories: Vec<DataValue> = bs
                                .data
                                .iter()
                                .enumerate()
                                .map(|(i, dp)| {
                                    DataValue::from(
                                        dp.name.clone().unwrap_or_else(|| (i + 1).to_string()),
                                    )
                                })
                                .collect();
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "category", categories,
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "min",
                                bs.data.iter().map(|dp| DataValue::from(dp.min)).collect(),
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "q1",
                                bs.data.iter().map(|dp| DataValue::from(dp.q1)).collect(),
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "median",
                                bs.data
                                    .iter()
                                    .map(|dp| DataValue::from(dp.median))
                                    .collect(),
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "q3",
                                bs.data.iter().map(|dp| DataValue::from(dp.q3)).collect(),
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "max",
                                bs.data.iter().map(|dp| DataValue::from(dp.max)).collect(),
                            ));
                            d
                        }
                    };
                    let config = crate::pipeline::types::BoxplotConfig {
                        category_col: "category".into(),
                        min_col: "min".into(),
                        q1_col: "q1".into(),
                        median_col: "median".into(),
                        q3_col: "q3".into(),
                        max_col: "max".into(),
                    };
                    let item_style =
                        ItemStyleSpec {
                            color: bs.item_style.as_ref().and_then(|is| {
                                is.color.as_ref().map(|c| Color::rgb(c.r, c.g, c.b))
                            }),
                            border_color: bs.item_style.as_ref().and_then(|is| {
                                is.border_color.as_ref().map(|c| Color::rgb(c.r, c.g, c.b))
                            }),
                            border_width: bs.item_style.as_ref().and_then(|is| {
                                is.border_width.as_ref().and_then(|v| v.as_number())
                            }),
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
                    // 优先 dataset + datasetIndex；否则用 series.data
                    let ds_idx = resolve_dataset_index(hs.dataset_index, !hs.data.is_empty());
                    let data = match heatmap_dataset_df(ds_idx, &hs.encode, &datasets) {
                        Some(df) => df,
                        None => {
                            let mut d = crate::pipeline::dataframe::DataFrame::new();
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "x",
                                hs.data.iter().map(|dp| DataValue::Float(dp.x)).collect(),
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "y",
                                hs.data.iter().map(|dp| DataValue::Float(dp.y)).collect(),
                            ));
                            d.add_column(crate::pipeline::dataframe::Series::new(
                                "value",
                                hs.data
                                    .iter()
                                    .map(|dp| DataValue::Float(dp.value))
                                    .collect(),
                            ));
                            d
                        }
                    };

                    // visualMap → 热力图颜色映射
                    let (vm_min, vm_max, vm_colors) = resolve_visual_map(option, &data);

                    let item_style = ItemStyleSpec {
                        color: None,
                        border_color: hs.item_style.as_ref().and_then(|is| {
                            is.border_color.as_ref().map(|c| Color::rgb(c.r, c.g, c.b))
                        }),
                        border_width: hs
                            .item_style
                            .as_ref()
                            .and_then(|is| is.border_width.as_ref().and_then(|v| v.as_number())),
                        opacity: hs.item_style.as_ref().and_then(|is| is.opacity),
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
                        label_show: hs.label.as_ref().and_then(|l| l.show).unwrap_or(false),
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
                                    v.iter().filter_map(|i| i.name.clone()).collect::<Vec<_>>()
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
                    let data = polar_datapoints_to_dataframe(&pb.data, "angle", "radius");
                    let config = crate::pipeline::types::PolarBarConfig {
                        angle_col: "angle".into(),
                        radius_col: "radius".into(),
                        category_col: None,
                        pad_angle: 2.0,
                        start_angle: 0.0,
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
                SeriesOption::PolarScatter(ps) => {
                    let symbol_size = ps
                        .symbol_size
                        .as_ref()
                        .map(|v| v.to_vec())
                        .and_then(|v| v.first().and_then(|n| n.as_number()))
                        .unwrap_or(8.0);
                    let data = polar_scatter_datapoints_to_dataframe(
                        &ps.data,
                        "angle",
                        "radius",
                        symbol_size,
                    );
                    let config = crate::pipeline::types::PolarScatterConfig {
                        angle_col: "angle".into(),
                        radius_col: "radius".into(),
                        symbol_size,
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
                SeriesOption::Gauge(gs) => {
                    let data = gauge_datapoints_to_dataframe(&gs.data, "value");
                    let (center_x, center_y) = parse_gauge_center(gs.center.as_deref());
                    // P2a：radius 百分比折算为绝对像素（基准 = 画布 min/2）
                    let radius = radius_percent_to_abs_px(
                        parse_gauge_radius(gs.radius.as_ref()).unwrap_or(75.0),
                        width,
                        height,
                    );
                    let config = crate::pipeline::types::GaugeConfig {
                        value_col: "value".into(),
                        min: gs.min.unwrap_or(0.0),
                        max: gs.max.unwrap_or(100.0),
                        center: (center_x, center_y),
                        radius,
                        start_angle: gs.start_angle.unwrap_or(225.0),
                        end_angle: gs.end_angle.unwrap_or(-45.0),
                        split_number: gs.split_number.unwrap_or(10),
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

    // 预计算自动图例名（在 series 被 move 进 ChartSpec 之前）
    let auto_legend_names = collect_legend_names(&series);

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
            color: t
                .text_style
                .as_ref()
                .and_then(|s| s.color.as_ref())
                .map(|c| Color::rgb(c.r, c.g, c.b)),
            subcolor: t
                .subtext_style
                .as_ref()
                .and_then(|s| s.color.as_ref())
                .map(|c| Color::rgb(c.r, c.g, c.b)),
        }),
        legend: option.legend.as_ref().map(|l| LegendSpec {
            show: l.show.unwrap_or(true),
            // 显式提供的 data 优先；否则自动从系列回填展示名（与 ECharts 一致）
            data: {
                let explicit: Vec<String> = l
                    .data
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .map(|i| i.name().to_string())
                    .collect();
                if explicit.is_empty() {
                    auto_legend_names.clone()
                } else {
                    explicit
                }
            },
            symbol_size: l
                .symbol_size
                .as_ref()
                .and_then(|v| v.as_number())
                .unwrap_or(10.0),
            item_gap: l.item_gap.unwrap_or(10.0),
            formatter: l.formatter.clone(),
        }),
        background: Color::rgb(255, 255, 255),
        palette: vec![],
        theme_name: None,
        fit_mode: crate::pipeline::types::FitMode::Fixed,
    }
}

/// 将 `LenientNumber` 解析为半径百分比数值。
///
/// 兼容两种写法：
/// - `LenientNumber::Number(n)` → `n`（纯数字，直接作为百分比值）
/// - `LenientNumber::Percent(p)` → `p`（"60%" 字符串）
fn radius_num_to_percent(v: &option::LenientNumber) -> Option<f64> {
    match v {
        option::LenientNumber::Number(n) => Some(*n),
        option::LenientNumber::Percent(p) => Some(*p),
        _ => None,
    }
}

/// 解析饼图/环形图的 `radius` 字段为 `(inner, outer)` 百分比值。
///
/// ECharts 支持两种形式：
/// - 单个值（`"60%"` 或 `60`）：仅设置外半径，内半径为 0（实心饼图）
/// - 数组（`["0%", "70%"]` 或 `[40, 70]`）：`[内半径, 外半径]`，用于环形图
pub(crate) fn parse_pie_radius(
    radius: Option<&option::SingleOrArray<option::LenientNumber>>,
) -> Option<(f64, f64)> {
    let values = radius?.to_vec();
    match values.len() {
        0 => None,
        1 => {
            let outer = radius_num_to_percent(&values[0])?;
            Some((0.0, outer))
        }
        _ => {
            let inner = radius_num_to_percent(&values[0])?;
            let outer = radius_num_to_percent(&values[1])?;
            Some((inner, outer))
        }
    }
}

/// 将 `GaugeSeriesOption.data`（`Vec<GaugeDataPoint>`）转换为以 `value` 列为单列的数据。
fn gauge_datapoints_to_dataframe(
    points: &[option::GaugeDataPoint],
    value_col: &str,
) -> crate::pipeline::dataframe::DataFrame {
    use crate::pipeline::dataframe::{DataFrame, DataValue, Series as DfSeries};

    let mut df = DataFrame::new();
    let values: Vec<DataValue> = points.iter().map(|p| DataValue::Float(p.value)).collect();
    df.add_column(DfSeries::new(value_col, values));
    df
}

/// 解析 gauge `center` 配置（百分比），返回 `(x%, y%)`，默认 (50, 50)。
fn parse_gauge_center(center: Option<&[option::LenientNumber]>) -> (f64, f64) {
    let Some(c) = center else {
        return (50.0, 50.0);
    };
    if c.len() < 2 {
        return (50.0, 50.0);
    }
    let x = radius_num_to_percent(&c[0]).unwrap_or(50.0);
    let y = radius_num_to_percent(&c[1]).unwrap_or(50.0);
    (x, y)
}

/// P2a：把百分比半径折算为绝对像素（基准 = 画布 min/2）。
///
/// 与 `api::Chart` 的 `size_to_abs_px` 语义一致：pipeline 只消费绝对像素。
fn radius_percent_to_abs_px(v: f64, width: u32, height: u32) -> f64 {
    (width.min(height) as f64 * 0.5) * v / 100.0
}

/// 解析 gauge `radius` 配置（单值或数组），返回百分比外半径，默认 75。
fn parse_gauge_radius(
    radius: Option<&option::SingleOrArray<option::LenientNumber>>,
) -> Option<f64> {
    let values = radius?.to_vec();
    let v = values.last()?;
    radius_num_to_percent(v)
}

/// 解析 `series.markLine.data` 为标注线配置列表。
///
/// 支持 `type: average / min / max` 三种类型；`OneOrMany` 兼容单对象与数组两种写法。
fn parse_mark_line(
    mark_line: Option<&crate::option::MarkLineOption>,
) -> Vec<crate::pipeline::types::MarkLineSpec> {
    use crate::pipeline::types::{MarkLineSpec, MarkLineType};

    let Some(ml) = mark_line else {
        return Vec::new();
    };
    let Some(data) = &ml.data else {
        return Vec::new();
    };

    let mut specs = Vec::new();
    for item in data {
        for d in item.as_vec() {
            let data_type = match d.data_type.as_deref() {
                Some("average") => MarkLineType::Average,
                Some("min") => MarkLineType::Min,
                Some("max") => MarkLineType::Max,
                _ => continue,
            };
            specs.push(MarkLineSpec {
                data_type,
                name: d.name.clone(),
            });
        }
    }
    specs
}

/// 将 ECharts `label.position` 映射为笛卡尔系列（line/bar）的值标签位置。
///
/// 仅支持 Top/Bottom/Inside 三种垂直语义；其余（Left/Right/Center/Start/...）
/// 在纵向笛卡尔系列上无明确对应，统一按 ECharts 默认降级为 `Top`。
fn parse_value_label_position(
    label: Option<&crate::option::LabelOption>,
) -> Option<crate::pipeline::types::ValueLabelPos> {
    use crate::option::LabelPosition as In;
    use crate::pipeline::types::ValueLabelPos as Out;

    label.and_then(|l| l.position).map(|p| match p {
        In::Bottom => Out::Bottom,
        In::Inside | In::Center | In::Middle => Out::Inside,
        _ => Out::Top,
    })
}

/// 提取 ECharts `label.color` 为标签颜色；未配置时返回 None（由 Builder 取语义默认色）。
fn parse_label_color(label: Option<&crate::option::LabelOption>) -> Option<Color> {
    label
        .and_then(|l| l.color.as_ref())
        .map(|c| Color::rgba(c.r, c.g, c.b, c.a))
}

/// 当 legend 未显式提供 data 时，自动从系列收集展示名（与 ECharts 行为一致）。
///
/// - 饼图/环形图/极坐标柱状图：按数据点取色，图例项为**数据点名**（category 列）
/// - 其他系列：图例项为**系列名**（series.name）
pub(crate) fn collect_legend_names(series: &[crate::pipeline::types::SeriesSpec]) -> Vec<String> {
    use crate::pipeline::types::SeriesConfig;

    let mut names = Vec::new();
    for s in series {
        match &s.config {
            // 按数据点着色的类型：图例显示数据点名
            SeriesConfig::Pie(cfg) => {
                if let Some(col) = s.data.get_column(&cfg.category_col) {
                    for i in 0..s.data.row_count() {
                        if let Some(v) = col.as_string(i) {
                            names.push(v);
                        }
                    }
                }
            }
            SeriesConfig::PolarBar(cfg) => {
                if let Some(col) = s.data.get_column(&cfg.angle_col) {
                    for i in 0..s.data.row_count() {
                        if let Some(v) = col.as_string(i) {
                            names.push(v);
                        }
                    }
                }
            }
            _ => {
                if !s.name.is_empty() {
                    names.push(s.name.clone());
                }
            }
        }
    }
    names
}

/// 从 `option.visual_map` 解析热力图颜色映射。
///
/// 返回 `(min, max, 渐变颜色)`；min/max 缺失时由数据自动推断，
/// 颜色缺失时使用 ECharts heatmap 的默认三段渐变。
fn resolve_visual_map(
    option: &ChartOption,
    data: &crate::pipeline::dataframe::DataFrame,
) -> (Option<f64>, Option<f64>, Vec<Color>) {
    use crate::pipeline::dataframe::DataValue;

    let default_colors = vec![
        Color::rgb(80, 163, 186), // #50a3ba
        Color::rgb(234, 199, 54), // #eac736
        Color::rgb(217, 78, 93),  // #d94e5d
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

    let Some(vm) = option
        .visual_map
        .as_ref()
        .and_then(|v| v.as_slice().first())
    else {
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

    let colors: Vec<Color> = raw_colors
        .filter(|c| !c.is_empty())
        .map(|c| c.iter().map(|co| Color::rgb(co.r, co.g, co.b)).collect())
        .unwrap_or(default_colors);

    (min, max, colors)
}

/// 将 PositionOption 转换为可延迟解析的 [`GridEdge`]（P2b）。
///
/// 与旧的 [`resolve_position_option`] 结果语义一致，但 `Percent` 保留比例
/// 由 `GridPlanner` 在布局阶段解析（画布变化时随比例缩放）。
fn position_option_to_edge(pos: &PositionOption) -> crate::pipeline::types::GridEdge {
    match pos {
        PositionOption::Pixel(v) => crate::pipeline::types::GridEdge::Px(*v),
        PositionOption::Percent(p) => crate::pipeline::types::GridEdge::Pct(*p),
        PositionOption::Preset(PositionPreset::Auto) => {
            crate::pipeline::types::GridEdge::Pct(10.0) // 旧行为 = total*0.1
        }
        PositionOption::Preset(PositionPreset::Center) => {
            crate::pipeline::types::GridEdge::Pct(50.0)
        }
        PositionOption::Preset(PositionPreset::Left)
        | PositionOption::Preset(PositionPreset::Top) => crate::pipeline::types::GridEdge::Px(0.0),
        PositionOption::Preset(PositionPreset::Right)
        | PositionOption::Preset(PositionPreset::Bottom) => {
            // 贴边：占满对应维度（等价于旧的 `total`），Hug 长大时自动跟随
            crate::pipeline::types::GridEdge::Pct(100.0)
        }
    }
}

/// 将旧的 Vec<DataPoint> 转换为 DataFrame
/// 将系列数据转换为极坐标柱状图/散点图所需的 `(angle, radius)` 数据。
///
/// 与 `datapoints_to_dataframe` 不同，这里针对纯数组/命名数据做「角度均分」：
/// - `Value(v)` / `Named(name, v)`（无显式角度）：按 `360/N` 均匀分布角度，避免所有柱子重叠在 0°
/// - `XY(angle, radius)`：使用显式的角度/半径
///
/// 角度使用 ECharts 极坐标语义：`0°` 在顶部，顺时针增加。
fn polar_datapoints_to_dataframe(
    points: &[option::DataPoint],
    angle_col: &str,
    radius_col: &str,
) -> crate::pipeline::dataframe::DataFrame {
    use crate::pipeline::dataframe::{DataFrame, DataValue, Series as DfSeries};

    let mut df = DataFrame::new();
    if points.is_empty() {
        return df;
    }

    let n = points.len();
    let is_xy = matches!(points[0], option::DataPoint::XY(_, _));

    if is_xy {
        // 显式角度/半径
        let angles: Vec<DataValue> = points
            .iter()
            .map(|p| match p {
                option::DataPoint::XY(x, _) => DataValue::Float(*x),
                _ => DataValue::Null,
            })
            .collect();
        let radii: Vec<DataValue> = points
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
        df.add_column(DfSeries::new(angle_col, angles));
        df.add_column(DfSeries::new(radius_col, radii));
    } else {
        // 纯数组 / 命名数据：角度均分 360°
        // 起点在顶部（-90°），顺时针
        let start_deg = -90.0_f64;
        let step = 360.0 / n as f64;
        let angles: Vec<DataValue> = (0..n)
            .map(|i| DataValue::Float(start_deg + i as f64 * step))
            .collect();
        let radii: Vec<DataValue> = points
            .iter()
            .map(|p| {
                let v = match p {
                    option::DataPoint::Value(y) => *y,
                    option::DataPoint::Named(_, y) => *y,
                    option::DataPoint::XY(_, y) => *y,
                };
                DataValue::Float(v)
            })
            .collect();
        df.add_column(DfSeries::new(angle_col, angles));
        df.add_column(DfSeries::new(radius_col, radii));
    }

    df
}

/// 将 `PolarScatterSeriesOption.data`（显式角度/半径）转换为 `(angle, radius)` 数据。
fn polar_scatter_datapoints_to_dataframe(
    points: &[option::PolarScatterDataPoint],
    angle_col: &str,
    radius_col: &str,
    _default_symbol_size: f64,
) -> crate::pipeline::dataframe::DataFrame {
    use crate::pipeline::dataframe::{DataFrame, DataValue, Series as DfSeries};

    let mut df = DataFrame::new();
    let angles: Vec<DataValue> = points.iter().map(|p| DataValue::Float(p.angle)).collect();
    let radii: Vec<DataValue> = points.iter().map(|p| DataValue::Float(p.radius)).collect();
    df.add_column(DfSeries::new(angle_col, angles));
    df.add_column(DfSeries::new(radius_col, radii));
    df
}

/// 判断指定 x 轴是否为 Time 轴
fn axis_is_time(x_axes: &[crate::pipeline::types::AxisSpec], x_axis_index: usize) -> bool {
    x_axes
        .get(x_axis_index)
        .map(|a| matches!(a.axis_type, crate::pipeline::types::AxisType::Time))
        .unwrap_or(false)
}

/// 将常见日期字符串解析为时间戳（Unix 秒）。
///
/// 支持的格式：
/// - `YYYY-MM-DD`
/// - `YYYY-MM-DD HH:mm:ss`
/// - `YYYY/MM/DD`
/// - `YYYY-MM-DDTHH:mm:ss`（ISO）
///
/// 解析失败返回 `None`，调用方回退为字符串。
fn parse_date_string(s: &str) -> Option<i64> {
    let s = s.trim();
    let (date_part, time_part) = match s.split_once('T').or_else(|| s.split_once(' ')) {
        Some((d, t)) => (d, t),
        None => (s, ""),
    };

    let mut nums = Vec::new();
    for part in date_part.split(['-', '/', '.']) {
        if let Ok(n) = part.parse::<i32>() {
            nums.push(n);
        }
    }
    if nums.len() != 3 {
        return None;
    }
    let (year, month, day) = (nums[0] as i64, nums[1] as i64, nums[2] as i64);
    if year < 1970 || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let (hour, minute, second) = if time_part.is_empty() {
        (0, 0, 0)
    } else {
        let t_nums: Vec<i64> = time_part
            .split(':')
            .filter_map(|x| x.parse::<i64>().ok())
            .collect();
        match t_nums.as_slice() {
            [h] => (*h, 0, 0),
            [h, m] => (*h, *m, 0),
            [h, m, sec] => (*h, *m, *sec),
            _ => return None,
        }
    };

    // 使用简单公历转儒略日（忽略时区，视为 UTC）
    let y = year;
    let m = month;
    let d = day;
    let a = (14 - m) / 12;
    let y2 = y + 4800 - a;
    let m2 = m + 12 * a - 3;
    let jdn = d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045;
    Some((jdn - 2440588) * 86400 + hour * 3600 + minute * 60 + second)
}

fn datapoints_to_dataframe(
    points: &[option::DataPoint],
    x_col: &str,
    y_col: &str,
    x_is_time: bool,
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
                    if x_is_time {
                        // 时间轴：将日期字符串解析为时间戳（秒）
                        if let Some(ts) = parse_date_string(name) {
                            DataValue::Float(ts as f64)
                        } else {
                            DataValue::String(name.clone())
                        }
                    } else {
                        DataValue::String(name.clone())
                    }
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
    let data_start = if has_header && !source.is_empty() {
        1
    } else {
        0
    };

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

        // 行不齐（某行比首行短）时用 Null 补齐，保证各列等长——
        // 与有表头分支一致，否则 DataFrame::add_column 会对长度不一的列 panic。
        for row_data in col_data.iter_mut() {
            while row_data.len() < source.len() {
                row_data.push(DataValue::Null);
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

    fn is_empty_or_none(v: &Option<option::OneOrMany<option::StringOrInt>>) -> bool {
        match v {
            None => true,
            Some(option::OneOrMany::One(_)) => false,
            Some(option::OneOrMany::Many(vec)) => vec.is_empty(),
        }
    }

    if let Some(src_name) = first_column_name(&encode.x, &col_names)
        && let Some(col) = df.get_column(&src_name)
    {
        result_df.add_column(DfSeries::new("x", col.data.clone()));
        x_col = "x".into();
    }

    if let Some(src_name) = first_column_name(&encode.y, &col_names)
        && let Some(col) = df.get_column(&src_name)
    {
        result_df.add_column(DfSeries::new("y", col.data.clone()));
        y_col = "y".into();
    }

    if is_empty_or_none(&encode.x)
        && let Some(src_name) = first_column_name(&encode.item_name, &col_names)
        && let Some(col) = df.get_column(&src_name)
        && result_df.get_column("x").is_none()
    {
        result_df.add_column(DfSeries::new("x", col.data.clone()));
        x_col = "x".into();
    }

    if is_empty_or_none(&encode.y)
        && let Some(src_name) = first_column_name(&encode.value, &col_names)
        && let Some(col) = df.get_column(&src_name)
        && result_df.get_column("y").is_none()
    {
        result_df.add_column(DfSeries::new("y", col.data.clone()));
        y_col = "y".into();
    }

    if result_df.column_count() == 0 {
        return (df.clone(), "x".into(), "y".into());
    }

    (result_df, x_col, y_col)
}

// ═══ H3: candlestick / boxplot / heatmap 支持 dataset + datasetIndex ═══

/// 解析系列实际使用的 dataset 下标。
///
/// ECharts 语义：`datasetIndex` 缺省为 **0**，因此只要系列没有显式提供 `data`，
/// 就默认读第 0 个 dataset；显式给了 `series.data` 时 data 优先，返回 `None`
/// 让调用方回退到 series.data。
fn resolve_dataset_index(dataset_index: Option<usize>, has_series_data: bool) -> Option<usize> {
    match dataset_index {
        Some(i) => Some(i),
        None if !has_series_data => Some(0),
        None => None,
    }
}

/// 解析 encode 某一维的首个列名（字符串按名、整数按下标）。
fn encode_dim_first(
    dim: &Option<option::OneOrMany<option::StringOrInt>>,
    df: &crate::pipeline::dataframe::DataFrame,
) -> Option<String> {
    let item = match dim {
        Some(option::OneOrMany::One(i)) => i,
        Some(option::OneOrMany::Many(v)) => v.first()?,
        None => return None,
    };
    match item {
        option::StringOrInt::Str(s) => Some(s.clone()),
        option::StringOrInt::Int(idx) => df.column_names().get(*idx).cloned(),
    }
}

/// encode.y 的多维名列表（如 candlestick 的 [open,close,low,high]）。
fn encode_y_names(
    encode: &Option<option::SeriesEncodeOption>,
    df: &crate::pipeline::dataframe::DataFrame,
) -> Vec<String> {
    let Some(y) = encode.as_ref().and_then(|e| e.y.clone()) else {
        return Vec::new();
    };
    let to_name = |i: &option::StringOrInt| -> Option<String> {
        match i {
            option::StringOrInt::Str(s) => Some(s.clone()),
            option::StringOrInt::Int(idx) => df.column_names().get(*idx).cloned(),
        }
    };
    match y {
        option::OneOrMany::One(i) => to_name(&i).into_iter().collect(),
        option::OneOrMany::Many(items) => items.iter().filter_map(to_name).collect(),
    }
}

/// 从 dataset 数据框取列：优先 encode 指定列名，其次按别名（大小写不敏感）匹配。
/// 无法匹配时返回 `None`——调用方回退 series.data。
fn pick_dataset_col(
    df: &crate::pipeline::dataframe::DataFrame,
    encode_name: Option<&String>,
    aliases: &[&str],
) -> Option<Vec<crate::pipeline::dataframe::DataValue>> {
    if let Some(name) = encode_name
        && let Some(col) = df.get_column(name)
    {
        return Some(col.data.clone());
    }
    for a in aliases {
        if let Some(col) = df.get_column(a) {
            return Some(col.data.clone());
        }
        if let Some(n) = df.column_names().iter().find(|n| n.eq_ignore_ascii_case(a))
            && let Some(col) = df.get_column(n)
        {
            return Some(col.data.clone());
        }
    }
    None
}

/// K 线图：dataset 数据框 → [category, open, close, low, high] 子数据框。
fn candlestick_dataset_df(
    ds_idx: Option<usize>,
    encode: &Option<option::SeriesEncodeOption>,
    datasets: &[crate::pipeline::dataframe::DataFrame],
) -> Option<crate::pipeline::dataframe::DataFrame> {
    use crate::pipeline::dataframe::{DataFrame, Series as DfSeries};
    let df = datasets.get(ds_idx?)?;
    let y_names = encode_y_names(encode, df);
    let x_name = encode.as_ref().and_then(|e| encode_dim_first(&e.x, df));

    let category = pick_dataset_col(df, x_name.as_ref(), &["category", "name", "x", "date"])?;
    let open = pick_dataset_col(df, y_names.first(), &["open"])?;
    let close = pick_dataset_col(df, y_names.get(1), &["close"])?;
    let low = pick_dataset_col(df, y_names.get(2), &["low", "lowest"])?;
    let high = pick_dataset_col(df, y_names.get(3), &["high", "highest"])?;

    let mut out = DataFrame::new();
    out.add_column(DfSeries::new("category", category));
    out.add_column(DfSeries::new("open", open));
    out.add_column(DfSeries::new("close", close));
    out.add_column(DfSeries::new("low", low));
    out.add_column(DfSeries::new("high", high));
    Some(out)
}

/// 箱线图：dataset 数据框 → [category, min, q1, median, q3, max] 子数据框。
fn boxplot_dataset_df(
    ds_idx: Option<usize>,
    encode: &Option<option::SeriesEncodeOption>,
    datasets: &[crate::pipeline::dataframe::DataFrame],
) -> Option<crate::pipeline::dataframe::DataFrame> {
    use crate::pipeline::dataframe::{DataFrame, Series as DfSeries};
    let df = datasets.get(ds_idx?)?;
    let y_names = encode_y_names(encode, df);
    let x_name = encode.as_ref().and_then(|e| encode_dim_first(&e.x, df));

    let category = pick_dataset_col(df, x_name.as_ref(), &["category", "name", "x"])?;
    let min = pick_dataset_col(df, y_names.first(), &["min"])?;
    let q1 = pick_dataset_col(df, y_names.get(1), &["q1", "quartile1"])?;
    let median = pick_dataset_col(df, y_names.get(2), &["median"])?;
    let q3 = pick_dataset_col(df, y_names.get(3), &["q3", "quartile3"])?;
    let max = pick_dataset_col(df, y_names.get(4), &["max"])?;

    let mut out = DataFrame::new();
    out.add_column(DfSeries::new("category", category));
    out.add_column(DfSeries::new("min", min));
    out.add_column(DfSeries::new("q1", q1));
    out.add_column(DfSeries::new("median", median));
    out.add_column(DfSeries::new("q3", q3));
    out.add_column(DfSeries::new("max", max));
    Some(out)
}

/// 热力图：dataset 数据框 → [x, y, value] 子数据框。
fn heatmap_dataset_df(
    ds_idx: Option<usize>,
    encode: &Option<option::SeriesEncodeOption>,
    datasets: &[crate::pipeline::dataframe::DataFrame],
) -> Option<crate::pipeline::dataframe::DataFrame> {
    use crate::pipeline::dataframe::{DataFrame, Series as DfSeries};
    let df = datasets.get(ds_idx?)?;
    let e = encode.as_ref();
    let x = pick_dataset_col(
        df,
        e.and_then(|e| encode_dim_first(&e.x, df)).as_ref(),
        &["x"],
    )?;
    let y = pick_dataset_col(
        df,
        e.and_then(|e| encode_dim_first(&e.y, df)).as_ref(),
        &["y"],
    )?;
    let value = pick_dataset_col(
        df,
        e.and_then(|e| encode_dim_first(&e.value, df)).as_ref(),
        &["value", "val"],
    )?;

    let mut out = DataFrame::new();
    out.add_column(DfSeries::new("x", x));
    out.add_column(DfSeries::new("y", y));
    out.add_column(DfSeries::new("value", value));
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polar_datapoints_to_dataframe_distributes_angles_evenly() {
        // 纯数组数据：3 个值应均匀分布在 0°/120°/240°
        let pts = vec![
            option::DataPoint::Value(10.0),
            option::DataPoint::Value(20.0),
            option::DataPoint::Value(30.0),
        ];
        let df = polar_datapoints_to_dataframe(&pts, "angle", "radius");
        let angle = df.get_column("angle").unwrap();
        let radius = df.get_column("radius").unwrap();
        assert_eq!(df.row_count(), 3);
        // 角度从 -90° 起，步进 120°
        assert!((angle.as_f64(0).unwrap() - (-90.0)).abs() < 1e-6);
        assert!((angle.as_f64(1).unwrap() - 30.0).abs() < 1e-6);
        assert!((angle.as_f64(2).unwrap() - 150.0).abs() < 1e-6);
        // 半径保持原值
        assert!((radius.as_f64(0).unwrap() - 10.0).abs() < 1e-6);
        assert!((radius.as_f64(2).unwrap() - 30.0).abs() < 1e-6);
    }

    #[test]
    fn polar_datapoints_to_dataframe_keeps_explicit_angles() {
        // 显式 XY 角度/半径：保持原角度
        let pts = vec![
            option::DataPoint::XY(0.0, 5.0),
            option::DataPoint::XY(90.0, 15.0),
        ];
        let df = polar_datapoints_to_dataframe(&pts, "angle", "radius");
        let angle = df.get_column("angle").unwrap();
        assert!((angle.as_f64(0).unwrap() - 0.0).abs() < 1e-6);
        assert!((angle.as_f64(1).unwrap() - 90.0).abs() < 1e-6);
    }

    #[test]
    fn parse_date_string_handles_common_formats() {
        // 2026-08-01 的 Unix 秒
        let ts = parse_date_string("2026-08-01").unwrap();
        assert_eq!(ts, 1785542400);
        // 带时间
        let ts2 = parse_date_string("2026-08-01 10:00:00").unwrap();
        assert_eq!(ts2, 1785542400 + 10 * 3600);
        // 非法输入返回 None
        assert!(parse_date_string("not-a-date").is_none());
    }

    #[test]
    fn ragged_no_header_dataset_does_not_panic() {
        // H1 回归：无表头 dataset 行不齐时（某行比首行短/长）不得因列长不齐 panic，
        // 缺失单元格应补 Null。此前 dataset_to_dataframe 无补位 → DataFrame::add_column panic。
        let json = r#"{
            "dataset": {
                "source": [[1, 2], [3], [4, 5, 6]],
                "sourceHeader": false
            },
            "series": [{ "type": "line", "data": [] }]
        }"#;
        let opt: crate::option::ChartOption = serde_json::from_str(json).unwrap();
        let spec = chart_option_to_chart_spec(&opt, 800, 600);
        // datasets 解析成功即可；显式验证 dataset 数据框各列等长（3 行）
        let ds = opt.dataset.as_ref().unwrap().as_slice()[0].clone();
        let df = dataset_to_dataframe(&ds);
        assert_eq!(df.row_count(), 3);
        for name in df.column_names() {
            assert_eq!(
                df.get_column(name).map(|c| c.len()).unwrap_or(0),
                3,
                "列 {name} 应补齐到 3 行"
            );
        }
        let _ = spec;
    }

    #[test]
    fn candlestick_reads_dataset_by_index() {
        // H3 回归：candlestick 应支持 dataset + datasetIndex，而非静默空图
        let json = r#"{
            "dataset": {
                "source": [
                    ["category", "open", "close", "low", "high"],
                    ["d1", 10, 20, 5, 25],
                    ["d2", 20, 15, 10, 22]
                ]
            },
            "xAxis": [{ "type": "category" }],
            "yAxis": [{ "type": "value" }],
            "series": [{ "type": "candlestick", "datasetIndex": 0 }]
        }"#;
        let opt: crate::option::ChartOption =
            serde_json::from_str(json).expect("candlestick JSON 应可解析");
        let spec = chart_option_to_chart_spec(&opt, 800, 600);
        assert_eq!(
            spec.series.len(),
            1,
            "应解析出 1 个系列，实际 {}",
            spec.series.len()
        );
        let s = spec.series.first().unwrap();
        assert_eq!(s.data.row_count(), 2, "应从 dataset 读取 2 行 K 线数据");
        assert!(s.data.get_column("open").is_some());
        assert!(s.data.get_column("high").is_some());
    }

    #[test]
    fn candlestick_defaults_dataset_index_zero_when_data_omitted() {
        // H3 回归：存在 dataset 且系列省略 datasetIndex/data 时，应默认读第 0 个
        // dataset（ECharts 语义），而非回退到空 series.data。
        let json = r#"{
            "dataset": {
                "source": [
                    ["category", "open", "close", "low", "high"],
                    ["d1", 10, 20, 5, 25]
                ]
            },
            "xAxis": [{ "type": "category" }],
            "yAxis": [{ "type": "value" }],
            "series": [{ "type": "candlestick" }]
        }"#;
        let opt: crate::option::ChartOption =
            serde_json::from_str(json).expect("candlestick JSON 应可解析");
        let spec = chart_option_to_chart_spec(&opt, 800, 600);
        let s = spec.series.first().unwrap();
        assert_eq!(
            s.data.row_count(),
            1,
            "未写 datasetIndex/data 时也应按默认 datasetIndex=0 读取"
        );
    }

    #[test]
    fn explicit_series_data_overrides_dataset() {
        // ECharts 语义：显式给了 series.data 时优先于 dataset（不因缺省 datasetIndex=0
        // 而被 dataset 覆盖）。
        let json = r#"{
            "dataset": {
                "source": [
                    ["category", "open", "close", "low", "high"],
                    ["d1", 10, 20, 5, 25]
                ]
            },
            "xAxis": [{ "type": "category" }],
            "yAxis": [{ "type": "value" }],
            "series": [{
                "type": "candlestick",
                "data": [
                    { "name": "e1", "open": 1, "close": 2, "low": 0.5, "high": 3 }
                ]
            }]
        }"#;
        let opt: crate::option::ChartOption =
            serde_json::from_str(json).expect("candlestick JSON 应可解析");
        let spec = chart_option_to_chart_spec(&opt, 800, 600);
        let s = spec.series.first().unwrap();
        assert_eq!(s.data.row_count(), 1);
        // 值应来自 series.data 的 e1（open=1），而不是 dataset 的 d1（open=10）
        let open = s.data.get_column("open").and_then(|c| c.as_f64(0));
        assert!(
            (open.unwrap_or(0.0) - 1.0).abs() < 1e-9,
            "series.data 应覆盖 dataset，open 应为 1，实际 {open:?}"
        );
    }
}
