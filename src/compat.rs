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
use crate::{error::Result, theme::Theme, visual::VisualElement};

/// 将 ChartSpec 转换为旧的 ChartOption
pub fn chart_spec_to_chart_option(spec: &ChartSpec) -> ChartOption {
    let mut option = ChartOption::default();

    // Title
    option.title = spec.title.as_ref().map(|t| TitleOption {
        text: t.text.clone(),
        subtext: t.subtext.clone(),
        left: Some(PositionOption::Preset(PositionPreset::Center)),
        top: Some(PositionOption::Pixel(20.0)),
        ..Default::default()
    });

    // Legend
    if let Some(ref legend) = spec.legend {
        option.legend = Some(LegendOption {
            show: Some(legend.show),
            data: Some(legend.data.clone()),
            ..Default::default()
        });
    }

    // Grids
    for g in &spec.grids {
        option.grid.push(GridOption {
            left: g.left.map(PositionOption::Pixel),
            right: g.right.map(PositionOption::Pixel),
            top: g.top.map(PositionOption::Pixel),
            bottom: g.bottom.map(PositionOption::Pixel),
            contain_label: Some(g.contain_label),
        });
    }

    // X Axes
    for a in &spec.x_axes {
        option.x_axis.push(AxisOption {
            axis_type: Some(match a.axis_type {
                crate::pipeline::types::AxisType::Category => AxisType::Category,
                crate::pipeline::types::AxisType::Value => AxisType::Value,
                crate::pipeline::types::AxisType::Time => AxisType::Time,
                crate::pipeline::types::AxisType::Log => AxisType::Log,
            }),
            position: Some(match a.position {
                crate::pipeline::types::AxisPosition::Left => crate::option::AxisPosition::Left,
                crate::pipeline::types::AxisPosition::Right => crate::option::AxisPosition::Right,
                crate::pipeline::types::AxisPosition::Bottom => crate::option::AxisPosition::Bottom,
                crate::pipeline::types::AxisPosition::Top => crate::option::AxisPosition::Top,
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
        });
    }

    // Y Axes
    for a in &spec.y_axes {
        option.y_axis.push(AxisOption {
            axis_type: Some(match a.axis_type {
                crate::pipeline::types::AxisType::Category => AxisType::Category,
                crate::pipeline::types::AxisType::Value => AxisType::Value,
                crate::pipeline::types::AxisType::Time => AxisType::Time,
                crate::pipeline::types::AxisType::Log => AxisType::Log,
            }),
            position: Some(match a.position {
                crate::pipeline::types::AxisPosition::Left => crate::option::AxisPosition::Left,
                crate::pipeline::types::AxisPosition::Right => crate::option::AxisPosition::Right,
                crate::pipeline::types::AxisPosition::Bottom => crate::option::AxisPosition::Bottom,
                crate::pipeline::types::AxisPosition::Top => crate::option::AxisPosition::Top,
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
        });
    }

    // Background color
    option.background_color = None;

    // Series
    for s in &spec.series {
        match s.chart_type {
            ChartType::Line => {
                let mut ls = LineSeriesOption::default();
                ls.name = Some(s.name.clone());
                ls.data = Some(df_to_datapoints(s));
                ls.smooth = Some(s.smooth);
                ls.stack = s.stack.clone();
                ls.sampling = s.sampling.map(|(ty, threshold)| {
                    SamplingOption {
                        ty,
                        threshold,
                    }
                });
                ls.item_style = s.item_style.color.map(|c| crate::option::ItemStyleOption {
                    color: Some(crate::option::ColorOption::new(c.r, c.g, c.b)),
                    ..Default::default()
                });
                ls.y_axis_index = Some(s.y_axis_index);
                ls.grid_index = Some(s.grid_index);
                option.series.push(SeriesOption::Line(ls));
            }
            ChartType::Bar => {
                let mut bs = BarSeriesOption::default();
                bs.name = Some(s.name.clone());
                bs.data = Some(df_to_datapoints(s));
                bs.stack = s.stack.clone();
                bs.group_index = Some(s.group_index);
                bs.item_style = s.item_style.color.map(|c| crate::option::ItemStyleOption {
                    color: Some(crate::option::ColorOption::new(c.r, c.g, c.b)),
                    ..Default::default()
                });
                bs.y_axis_index = Some(s.y_axis_index);
                bs.grid_index = Some(s.grid_index);
                option.series.push(SeriesOption::Bar(bs));
            }
            ChartType::Scatter => {
                let mut ss = ScatterSeriesOption::default();
                ss.name = Some(s.name.clone());
                ss.data = Some(df_to_datapoints(s));
                ss.item_style = s.item_style.color.map(|c| crate::option::ItemStyleOption {
                    color: Some(crate::option::ColorOption::new(c.r, c.g, c.b)),
                    ..Default::default()
                });
                ss.symbol_size = Some(10.0);
                ss.y_axis_index = Some(s.y_axis_index);
                ss.grid_index = Some(s.grid_index);
                option.series.push(SeriesOption::Scatter(ss));
            }
            ChartType::Pie => {
                let mut ps = PieSeriesOption::default();
                ps.name = Some(s.name.clone());
                ps.data = Some(df_to_datapoints(s));
                option.series.push(SeriesOption::Pie(ps));
            }
            ChartType::Bubble => {
                let mut bs = BubbleSeriesOption::default();
                bs.name = Some(s.name.clone());
                // Bubble data points from DataFrame
                let dps: Vec<BubbleDataPoint> = (0..s.data.row_count())
                    .map(|i| {
                        let x = s.data.get_column(&s.x_col)
                            .and_then(|c| c.as_f64(i))
                            .unwrap_or(0.0);
                        let y = s.data.get_column(&s.y_col)
                            .and_then(|c| c.as_f64(i))
                            .unwrap_or(0.0);
                        BubbleDataPoint {
                            x,
                            y,
                            value: y,
                            symbol_size: 10.0,
                            name: None,
                            item_style: None,
                        }
                    })
                    .collect();
                bs.data = Some(dps);
                bs.y_axis_index = Some(s.y_axis_index);
                bs.grid_index = Some(s.grid_index);
                option.series.push(SeriesOption::Bubble(bs));
            }
            ChartType::Candlestick => {
                let mut cs = CandlestickSeriesOption::default();
                cs.name = Some(s.name.clone());
                let dps: Vec<crate::option::CandlestickDataPoint> = (0..s.data.row_count())
                    .map(|i| {
                        crate::option::CandlestickDataPoint {
                            open: s.data.get_column("open").and_then(|c| c.as_f64(i)).unwrap_or(0.0),
                            close: s.data.get_column("close").and_then(|c| c.as_f64(i)).unwrap_or(0.0),
                            low: s.data.get_column("low").and_then(|c| c.as_f64(i)).unwrap_or(0.0),
                            high: s.data.get_column("high").and_then(|c| c.as_f64(i)).unwrap_or(0.0),
                            name: None,
                            item_style: None,
                        }
                    })
                    .collect();
                cs.data = Some(dps);
                cs.y_axis_index = Some(s.y_axis_index);
                cs.grid_index = Some(s.grid_index);
                option.series.push(SeriesOption::Candlestick(cs));
            }
            ChartType::Radar => {
                let mut rs = RadarSeriesOption::default();
                rs.name = Some(s.name.clone());
                let dps: Vec<crate::option::RadarDataOption> = (0..s.data.row_count())
                    .map(|i| {
                        crate::option::RadarDataOption {
                            value: vec![s.data.get_column(&s.y_col).and_then(|c| c.as_f64(i)).unwrap_or(0.0)],
                            name: None,
                        }
                    })
                    .collect();
                rs.data = Some(dps);
                option.series.push(SeriesOption::Radar(rs));
            }
            ChartType::PolarBar => {
                let mut pbs = PolarBarSeriesOption::default();
                pbs.name = Some(s.name.clone());
                pbs.data = Some(df_to_datapoints(s));
                option.series.push(SeriesOption::PolarBar(pbs));
            }
            ChartType::PolarScatter => {
                let mut pss = PolarScatterSeriesOption::default();
                pss.name = Some(s.name.clone());
                let dps: Vec<crate::option::PolarScatterDataPoint> = (0..s.data.row_count())
                    .map(|i| {
                        crate::option::PolarScatterDataPoint {
                            angle: s.data.get_column(&s.x_col).and_then(|c| c.as_f64(i)).unwrap_or(0.0),
                            radius: s.data.get_column(&s.y_col).and_then(|c| c.as_f64(i)).unwrap_or(0.0),
                            value: s.data.get_column(&s.y_col).and_then(|c| c.as_f64(i)).unwrap_or(0.0),
                            name: None,
                            item_style: None,
                        }
                    })
                    .collect();
                pss.data = Some(dps);
                option.series.push(SeriesOption::PolarScatter(pss));
            }
            ChartType::Gauge => {
                let mut gs = GaugeSeriesOption::default();
                gs.name = Some(s.name.clone());
                gs.data = Some(df_to_gauge_datapoints(s));
                option.series.push(SeriesOption::Gauge(gs));
            }
            ChartType::Table => {
                let mut ts = TableSeriesOption::default();
                ts.name = Some(s.name.clone());
                let cols = s.data.column_names().to_vec();
                ts.columns = Some(cols);
                let mut rows = Vec::new();
                for i in 0..s.data.row_count() {
                    let mut row = Vec::new();
                    for col_name in s.data.column_names() {
                        let val = s.data.get_column(col_name).and_then(|c| c.as_string(i));
                        let json_val: serde_json::Value = val
                            .map(|s| s.parse::<f64>()
                                .map(serde_json::Value::from)
                                .unwrap_or_else(|_| serde_json::Value::String(s)))
                            .unwrap_or(serde_json::Value::Null);
                        row.push(json_val);
                    }
                    rows.push(row);
                }
                ts.data = Some(rows);
                option.series.push(SeriesOption::Table(ts));
            }
        }
    }

    // Palette
    if !spec.palette.is_empty() {
        option.color = Some(
            spec.palette
                .iter()
                .map(|c| crate::option::ColorOption::new(c.r, c.g, c.b))
                .collect(),
        );
    }

    option
}

/// 将 SeriesSpec 的 DataFrame 数据转换为旧 DataPoint 列表
fn df_to_datapoints(s: &SeriesSpec) -> Vec<crate::option::DataPoint> {
    let x_col = &s.x_col;
    let y_col = &s.y_col;
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
fn df_to_gauge_datapoints(s: &SeriesSpec) -> Vec<crate::option::GaugeDataPoint> {
    (0..s.data.row_count())
        .map(|i| {
            let value = s
                .data
                .get_column(&s.y_col)
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let name = s
                .data
                .get_column(&s.x_col)
                .and_then(|c| c.as_string(i));
            crate::option::GaugeDataPoint { value, name }
        })
        .collect()
}

/// 将旧的 ChartOption 转换为新的 ChartSpec（反向兼容）
pub fn chart_option_to_chart_spec(option: &ChartOption, width: u32, height: u32) -> ChartSpec {
    use crate::pipeline::types::{
        AxisPosition, AxisSpec, AxisType as NewAxisType, GridSpec, ItemStyleSpec, LegendSpec,
        SeriesSpec, TitleSpec,
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
            let new_axis_type = match a.axis_type.unwrap_or(AxisType::Category) {
                AxisType::Value => NewAxisType::Value,
                AxisType::Category => NewAxisType::Category,
                AxisType::Time => NewAxisType::Time,
                AxisType::Log => NewAxisType::Log,
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
            let new_axis_type = match a.axis_type.unwrap_or(AxisType::Value) {
                AxisType::Value => NewAxisType::Value,
                AxisType::Category => NewAxisType::Category,
                AxisType::Time => NewAxisType::Time,
                AxisType::Log => NewAxisType::Log,
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
        .enumerate()
        .map(|(idx, s)| {
            let (chart_type, x_col, y_col, stack, group_index) = match s {
                SeriesOption::Line(ls) => {
                    let chart_type = ChartType::Line;
                    (chart_type, String::new(), String::new(), ls.stack.clone(), 0usize)
                }
                SeriesOption::Bar(bs) => {
                    let chart_type = ChartType::Bar;
                    (chart_type, String::new(), String::new(), bs.stack.clone(), bs.group_index.unwrap_or(0) as usize)
                }
                SeriesOption::Scatter(ss) => {
                    let chart_type = ChartType::Scatter;
                    (chart_type, String::new(), String::new(), None, 0usize)
                }
                SeriesOption::Pie(_) => {
                    (ChartType::Pie, String::new(), String::new(), None, 0usize)
                }
                SeriesOption::Bubble(_) => {
                    (ChartType::Bubble, String::new(), String::new(), None, 0usize)
                }
                SeriesOption::Candlestick(_) => {
                    (ChartType::Candlestick, String::new(), String::new(), None, 0usize)
                }
                SeriesOption::Radar(_) => {
                    (ChartType::Radar, String::new(), String::new(), None, 0usize)
                }
                SeriesOption::PolarBar(_) => {
                    (ChartType::PolarBar, String::new(), String::new(), None, 0usize)
                }
                SeriesOption::PolarScatter(_) => {
                    (ChartType::PolarScatter, String::new(), String::new(), None, 0usize)
                }
                SeriesOption::Gauge(_) => {
                    (ChartType::Gauge, String::new(), String::new(), None, 0usize)
                }
                SeriesOption::Table(_) => {
                    (ChartType::Table, String::new(), String::new(), None, 0usize)
                }
            };

            // Build DataFrame from the old series data
            let mut df = crate::pipeline::dataframe::DataFrame::new();

            // Simplified: for now just store minimal data
            // Each series will be processed by compatible processors
            df.add_column(crate::pipeline::dataframe::Series::new_constant(
                "_dummy",
                DataValue::Float(0.0),
                0,
            ));

            SeriesSpec {
                name: format!("series_{}", idx),
                chart_type,
                data: df,
                x_col,
                y_col,
                grid_index: 0,
                x_axis_index: idx.min(option.x_axis.len().saturating_sub(1)),
                y_axis_index: idx.min(option.y_axis.len().saturating_sub(1)),
                stack,
                group_index,
                sampling: None,
                smooth: false,
                item_style: ItemStyleSpec::default(),
                ..Default::default()
            }
        })
        .collect();

    // Title
    let title = option.title.as_ref().map(|t| TitleSpec {
        text: t.text.clone(),
        subtext: t.subtext.clone(),
    });

    // Legend
    let legend = option.legend.as_ref().map(|l| LegendSpec {
        show: l.show.unwrap_or(true),
        data: l.data.clone().unwrap_or_default(),
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
    // 将 ChartOption 转换为 ChartSpec，然后使用新管线
    let spec = chart_option_to_chart_spec(option, width, height);
    crate::pipeline::pipeline::build_chart_from_spec(&spec, theme)
}