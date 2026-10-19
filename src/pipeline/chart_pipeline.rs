use lievisual::kurbo::Rect;

/// 基于 DataFrame 的新 Pipeline
///
/// 流程：
/// 1. GridPlanner: 计算每个 subplot 的布局边界
/// 2. AxisBindingResolver: 解析轴范围
/// 3. ColorAssigner: 分配颜色
/// 4. DataProcessor: 数据转换（采样等）
/// 5. Materializer: 将 SeriesSpec 转换为 TypedSeries
/// 6. Builder: 将 TypedSeries 组装为 SceneNode
/// 7. Decorator: 渲染标题、图例、轴名称等装饰元素
use crate::error::Result;
use crate::{
    Fill, FillStrokeStyle, SceneNode, Z_BACKGROUND,
    pipeline::{
        RenderContext,
        axis_binding_resolver::AxisBindingResolver,
        axis_label::{self},
        axis_renderer,
        builder::build_typed_series,
        color_assigner::ColorAssigner,
        data_processor, decorator,
        grid_planner::{GridPlanner, LayoutInput, PlanOutput},
        materializer::materialize_all,
        types::{
            ChartSpec, ChartType, ColorContext, FitMode, GridSpec, ResolvedAxisRanges, SubplotSpec,
            TextMeasurer,
        },
    },
    theme::Theme,
};

/// 布局求解完成的图表：元素 + 最终画布尺寸
///
/// [`FitMode::Hug`] / [`FitMode::HugMax`] 下最终尺寸可能大于 `ChartSpec` 的
/// 期望尺寸（画布按布局需求长大）；[`FitMode::Fixed`] 下两者恒等。
pub struct LaidOutChart {
    pub elements: Vec<SceneNode>,
    pub width: u32,
    pub height: u32,
    /// [`FitMode::HugMax`]：画布超出用户上限时，渲染阶段需整体等比缩放
    /// 回此目标尺寸（`fit_scene`）。仅"超上限"时为 `Some`。
    pub fit_max: Option<(f64, f64)>,
}

/// 布局迭代上限。
///
/// Hug 模式下需求随画布扩大单调递减（画布变宽 → 刻度槽变宽 → 旋转/抽稀
/// 决策放松 → 标签需求变小），实践中 1~2 轮收敛；此为防御性上限。
const MAX_LAYOUT_ITERS: usize = 4;

/// 从 ChartSpec 构建图表（新 API 入口）
///
/// 注意：[`FitMode::Hug`] 的 spec 请改用 [`build_chart_with_layout`]，
/// 本入口无法把长大后的画布尺寸带回给渲染器。
pub fn build_chart_from_spec(spec: &ChartSpec, theme: &Theme) -> Result<Vec<SceneNode>> {
    Ok(build_chart_with_layout(spec, theme)?.elements)
}

/// 布局求解 + 渲染：返回元素与最终画布尺寸
pub fn build_chart_with_layout(spec: &ChartSpec, theme: &Theme) -> Result<LaidOutChart> {
    let mut text_measurer = TextMeasurer::new();
    let (final_spec, plan) = resolve_layout(spec, theme, &mut text_measurer)?;
    let elements = render_chart(&final_spec, &plan, theme, &mut text_measurer)?;
    // HugMax：画布超出用户上限（原始 width/height）时，标记渲染阶段缩放回缩
    let fit_max = match spec.fit_mode {
        FitMode::HugMax if final_spec.width > spec.width || final_spec.height > spec.height => {
            Some((spec.width as f64, spec.height as f64))
        }
        _ => None,
    };
    Ok(LaidOutChart {
        elements,
        width: final_spec.width,
        height: final_spec.height,
        fit_max,
    })
}

/// 从 ChartOption 构建图表（旧 API 入口）
pub fn build_chart(
    option: &crate::option::ChartOption,
    width: u32,
    height: u32,
) -> Result<Vec<SceneNode>> {
    build_chart_with_theme(option, width, height, &Theme::echarts())
}

/// 从 ChartOption 构建图表（旧 API 入口，支持主题）
pub fn build_chart_with_theme(
    option: &crate::option::ChartOption,
    width: u32,
    height: u32,
    theme: &Theme,
) -> Result<Vec<SceneNode>> {
    // 「用户显式根 + rem」：`option.textStyle.fontSize` 显式给出时覆盖主题
    // `body_size` 作 1rem 根，各文本类型按主题系数（标题 1.5× 等）等比换算。
    let theme = apply_rem_root(theme, option.text_style.as_ref().and_then(|t| t.font_size));
    // 将 ChartOption 转换为 ChartSpec，然后使用新管线
    let spec = crate::pipeline::compat::chart_option_to_chart_spec(option, width, height);
    Ok(build_chart_with_layout(&spec, &theme)?.elements)
}

/// 把显式 rem 根字号应用到主题（Q2「用户显式根 + rem」）。
///
/// `root` 是用户显式给出的 1rem 基准：JSON 路径取 `option.textStyle.fontSize`，
/// api builder 路径取 [`Chart::body_font_size`](crate::api::Chart::body_font_size)。
/// `None` / 非正数 / 非有限值时原样返回主题（即回退主题自身的根字号）。
///
/// 应用方式：克隆主题并**仅重标** `tokens.text`——保留主题自身的 `*_rem` 系数、
/// `font_family`、`title_weight`，只把 `body_size` 换成根并按系数重算派生字号：
/// `title_size` = 根 × 1.5、`subtitle_size` = 根 × 1.0、`caption_size` = 根 × 0.833，
/// 整图文字随之等比缩放。
///
/// 注意：组件级显式 `fontSize`（如 `title.textStyle.fontSize`）不在此处折算，
/// 由各渲染解析层优先消费（完整优先级：组件显式 > 全局根 × 系数 > 主题令牌）。
pub fn apply_rem_root(theme: &Theme, root: Option<f64>) -> Theme {
    let root = root.filter(|r| r.is_finite() && *r > 0.0);
    let Some(root) = root else {
        return theme.clone();
    };
    let mut theme = theme.clone();
    let tokens = theme
        .tokens
        .get_or_insert_with(crate::theme::DesignTokens::echarts_v6);
    tokens.text.rescale_root(root);
    theme
}

/// 一轮布局求解的全部中间产物
struct RoundOutput {
    spec: ChartSpec,
    output: PlanOutput,
    axis_ranges: ResolvedAxisRanges,
    colors: ColorContext,
}

/// 需求求解：迭代扩大画布直至所有布局需求被满足（仅 Hug 模式会迭代）
///
/// 每轮流程：bind（无像素）→ resolve 轴范围（无像素）→ 生成轴标签文本
/// （无像素）→ 真实测量 + 画布切分 → 汇总需求 → 扩画布 + 写回目标边距。
fn resolve_layout(
    spec: &ChartSpec,
    theme: &Theme,
    measurer: &mut TextMeasurer,
) -> Result<(ChartSpec, RoundOutput)> {
    let mut working = spec.clone();
    let mut last: Option<RoundOutput> = None;

    for _ in 0..MAX_LAYOUT_ITERS {
        let mut round = plan_once(&working, theme, measurer)?;

        // 表格子图：行数 × 最小行高超出可用高度时上报画布增高需求（仅 Hug/HugMax）
        if matches!(working.fit_mode, FitMode::Hug | FitMode::HugMax) {
            apply_table_demands(&working, &round.output.specs, &mut round.output.demands);
        }

        let converged = !matches!(working.fit_mode, FitMode::Hug | FitMode::HugMax)
            || !round.output.demands.iter().any(|d| d.has_shortfall());
        if converged {
            last = Some(round);
            break;
        }

        apply_demands(&mut working, &round.output.demands);
        last = Some(round);
    }

    let mut round = last.ok_or_else(|| {
        crate::error::ChartError::RenderError("layout iteration produced no result".into())
    })?;
    // 迭代上限用尽时再做一次收敛轮，保证 specs 与最终画布一致
    if round.spec.width != working.width || round.spec.height != working.height {
        round = plan_once(&working, theme, measurer)?;
    }
    Ok((working, round))
}

/// 单轮布局：绑定 → 轴范围 → 轴标签文本 → 像素布局
fn plan_once(spec: &ChartSpec, theme: &Theme, measurer: &mut TextMeasurer) -> Result<RoundOutput> {
    // 0. 估计装饰元素占用：头部（标题）与底部（图例，v6 默认贴底）
    let header_height = decorator::estimate_header_height(spec, theme);
    let footer_height = decorator::estimate_footer_height(spec, theme, spec.width as f64);

    // 1. 绑定（无像素）：series / axis → subplot
    let planner = GridPlanner::new(spec.width, spec.height, header_height, &spec.grids)
        .with_footer_height(footer_height);
    let bindings = planner.bind(&spec.series, &spec.x_axes, &spec.y_axes);

    // 2. 解析轴范围（无像素，只依赖绑定关系）
    let resolver = AxisBindingResolver::new(&spec.x_axes, &spec.y_axes, &spec.series);
    let axis_ranges: ResolvedAxisRanges = resolver.resolve(&bindings);

    // 3. 分配颜色（不依赖布局）
    let colors: ColorContext = ColorAssigner.assign_with_theme(spec.series.len(), theme);

    // 4. 生成轴标签文本（无像素）：与 CartesianAxisRenderer 逐字一致
    let axis_labels = axis_label::collect_axis_labels(&spec.x_axes, &spec.y_axes, &axis_ranges);

    // 5. 像素布局：真实文本测量 + 画布切分
    let layout_input = LayoutInput {
        bindings,
        x_axes: &spec.x_axes,
        y_axes: &spec.y_axes,
        labels: &axis_labels,
        colors: &colors,
        fit_mode: spec.fit_mode,
    };
    let output = planner.plan(&layout_input, measurer);

    Ok(RoundOutput {
        spec: spec.clone(),
        output,
        axis_ranges,
        colors,
    })
}

/// 表格子图的最小高度需求：行数（含表头）× `TABLE_MIN_ROW_H` 超出
/// subplot 可用高度时，向画布增高（`grow_bottom`），底边距保持不变。
fn apply_table_demands(
    spec: &ChartSpec,
    specs: &[SubplotSpec],
    demands: &mut [crate::pipeline::grid_planner::SubplotDemand],
) {
    use crate::pipeline::builder::table::TABLE_MIN_ROW_H;

    for (i, subplot) in specs.iter().enumerate() {
        let is_table = !subplot.series_indices.is_empty()
            && subplot.series_indices.iter().all(|&idx| {
                spec.series
                    .get(idx)
                    .is_some_and(|s| s.chart_type() == ChartType::Table)
            });
        if !is_table {
            continue;
        }
        let max_rows = subplot
            .series_indices
            .iter()
            .filter_map(|&idx| spec.series.get(idx))
            .map(|s| s.data.row_count())
            .max()
            .unwrap_or(0);
        if max_rows == 0 {
            continue;
        }
        let needed_h = (max_rows + 1) as f64 * TABLE_MIN_ROW_H;
        let current_h = subplot.bounds.height();
        if needed_h > current_h {
            demands[i].grow_bottom += needed_h - current_h;
        }
    }
}

/// 把 Hug 需求写回 spec：目标边距写入对应 `GridSpec`，画布按缺口扩大
fn apply_demands(spec: &mut ChartSpec, demands: &[crate::pipeline::grid_planner::SubplotDemand]) {
    let mut grow_w = 0.0f64;
    let mut grow_h = 0.0f64;

    for (i, d) in demands.iter().enumerate() {
        if !d.has_shortfall() {
            continue;
        }
        if spec.grids.is_empty() && i == 0 {
            // 单默认 subplot：显式化，使目标边距可写回
            spec.grids.push(GridSpec {
                left: None,
                right: None,
                top: None,
                bottom: None,
                width: None,
                height: None,
                contain_label: false,
            });
        }
        if let Some(g) = spec.grids.get_mut(i) {
            if d.grid_left.is_some() {
                g.left = d.grid_left;
            }
            if d.grid_right.is_some() {
                g.right = d.grid_right;
            }
            if d.grid_top.is_some() {
                g.top = d.grid_top;
            }
            if d.grid_bottom.is_some() {
                g.bottom = d.grid_bottom;
            }
        }
        grow_w += d.grow_left + d.grow_right;
        grow_h += d.grow_top + d.grow_bottom;
    }

    if grow_w > 0.0 {
        spec.width = (spec.width as f64 + grow_w).ceil() as u32;
    }
    if grow_h > 0.0 {
        spec.height = (spec.height as f64 + grow_h).ceil() as u32;
    }
}

/// 把"自动"采样阈值（[`crate::sampling::AUTO_SAMPLING_THRESHOLD`]，即 `0`）
/// 解析为**绘图区像素尺寸**对应的目标点数。
///
/// 对齐 ECharts：`sampling:'lttb'`（不带阈值）时按坐标系像素宽度动态决定降采样
/// 目标，而非固定点数。这里取该系列所属 subplot 的 `max(宽, 高)` 作为目标点数
/// （每像素约 1 点；水平/垂直方向的稠密数据都能覆盖），并下限 3（LTTB 需要）。
///
/// 显式阈值（对象形式 `{type, threshold}`）保持不变。
fn resolve_sampling_thresholds(
    series: &mut [crate::pipeline::types::SeriesSpec],
    specs: &[SubplotSpec],
) {
    for s in series.iter_mut() {
        let Some((ty, threshold)) = s.sampling else {
            continue;
        };
        if threshold != crate::sampling::AUTO_SAMPLING_THRESHOLD {
            continue;
        }
        let px = specs
            .get(s.grid_index)
            .map(|sp| sp.bounds.width().max(sp.bounds.height()))
            .unwrap_or(0.0);
        let target = (px.round() as usize).max(3);
        s.sampling = Some((ty, target));
    }
}

/// 极坐标类系列的布局基准矩形。
///
/// - **饼图**：以**画布**为基准（ECharts 语义：饼图不参与 `grid`——`center`
///   相对画布取百分比，`radius` 的基准是 `min(画布宽, 画布高)/2`，标题/图例/
///   轴标签都不会把它挤小）。历史实现曾把它们塞进默认 grid 再 clamp 到「grid
///   内接半径」：竖向图例的 footer 预留把 grid 压扁后，`radius: ['0%','70%']`
///   在 800×600 下本应是 210px，实际被砍到 ~104px（官方同一份配置的输出是
///   263.2px / 70%）。
/// - **极坐标柱 / 极坐标散点**：以**绘图区**（planner bounds）为基准。与
///   `type:"bar" + coordinateSystem:"polar"` 的 JSON 路径一致——compat 对未显式
///   配 `grid` 的 JSON 套用 v6 默认边距（left 15% / right 10%），JSON 版因此
///   inset 于画布、顶部标签不侵入标题区；而 API 路径走整画布，同配置下两版
///   圆心/半径都不同（API 版顶部标签还会与副标题相撞）。统一到绘图区后两端
///   一致，且观感以 JSON 版（inset、不撞标题）为准。
///
/// 用户显式配置过 grid（任一边缘非 `None`，典型是多子图）时保持 grid 语义，
/// 既有多子图输出不受影响。
fn polar_series_bounds(subplot: &SubplotSpec, spec: &ChartSpec, width: u32, height: u32) -> Rect {
    let grid_explicit = spec.grids.get(subplot.id).is_some_and(|g| {
        g.left
            .or(g.right)
            .or(g.top)
            .or(g.bottom)
            .or(g.width)
            .or(g.height)
            .is_some()
    });
    // 仅饼图享有画布基准；极坐标柱/散点以绘图区为准（见上方文档）。
    let canvas_basis = !subplot.series_indices.is_empty()
        && subplot.series_indices.iter().all(|&idx| {
            spec.series
                .get(idx)
                .is_some_and(|s| matches!(s.config.chart_type(), ChartType::Pie))
        });
    if canvas_basis && !grid_explicit {
        Rect::new(0.0, 0.0, width as f64, height as f64)
    } else {
        subplot.bounds
    }
}

/// 渲染已求解的布局为视觉元素
fn render_chart(
    spec: &ChartSpec,
    round: &RoundOutput,
    theme: &Theme,
    text_measurer: &mut TextMeasurer,
) -> Result<Vec<SceneNode>> {
    let width = spec.width;
    let height = spec.height;
    let specs = &round.output.specs;
    let axis_ranges = &round.axis_ranges;
    let colors = &round.colors;

    // 数据转换（采样等）
    // 克隆 series 以便进行可变的数据处理，同时保持 spec 不变
    let mut processed_series = spec.series.clone();
    resolve_sampling_thresholds(&mut processed_series, specs);
    data_processor::process_series(&mut processed_series);
    // 创建包含处理后的数据的 ChartSpec（用于 Materialize 阶段）
    let data_spec = ChartSpec {
        series: processed_series,
        ..spec.clone()
    };

    // 收集所有 SceneNode
    let mut all_elements: Vec<SceneNode> = Vec::new();

    // 添加背景
    all_elements.push(crate::pipeline::builder::rect(
        Rect::new(0.0, 0.0, width as f64, height as f64),
        FillStrokeStyle {
            fill: Some(Fill::Solid(colors.background)),
            stroke: None,
        },
        Z_BACKGROUND,
    ));

    // 渲染轴（跳过纯表格子图，表格不需要坐标轴）
    //    text_measurer 自布局阶段起复用，测量口径与布局阶段一致。
    for subplot in specs {
        let is_table_subplot = subplot.series_indices.iter().all(|&idx| {
            spec.series
                .get(idx)
                .is_some_and(|s| s.chart_type() == ChartType::Table)
        });
        if is_table_subplot {
            continue;
        }
        let axis_elements = axis_renderer::render_axes(
            subplot,
            &spec.series,
            &spec.x_axes,
            &spec.y_axes,
            axis_ranges,
            colors,
            text_measurer,
        );
        all_elements.extend(axis_elements);
    }

    // 渲染系列数据（新流程：Materialize + Build）
    for subplot in specs {
        // 极坐标类系列的布局基准（饼图画布 / 极坐标柱散点绘图区，见 [`polar_series_bounds`]）
        let bounds = polar_series_bounds(subplot, spec, width, height);
        // 创建渲染上下文
        let ctx = RenderContext {
            colors,
            theme,
            bounds,
        };

        // Materialize 阶段：将 SeriesSpec 转换为 TypedSeries
        // 使用 data_spec（包含经过 DataProcessor 处理的系列数据）
        let typed_series_list = materialize_all(
            &subplot.series_indices,
            &data_spec,
            bounds,
            axis_ranges,
            colors,
        )?;

        // Build 阶段：将 TypedSeries 组装为 SceneNode
        // 指示器标签在每个 subplot 级别只绘制一次（取第一个雷达系列的 indicators），
        // 避免多系列雷达图时每个系列都重复绘制标签。
        let mut radar_indicators_drawn = false;
        for typed_series in typed_series_list {
            if !radar_indicators_drawn
                && let crate::pipeline::typed_series::TypedSeries::Radar(radar) = &typed_series
            {
                let indicator_elements =
                    crate::pipeline::builder::radar::build_radar_indicators(radar, subplot.bounds);
                all_elements.extend(indicator_elements);
                radar_indicators_drawn = true;
            }

            let mut elements = build_typed_series(&typed_series, &ctx)?;
            // `labelLayout.hideOverlap`：值标签以 `layout: None` 产出，需先排版
            // 才能计算包围盒；随后按系列内标签的重叠情况隐藏（ECharts 默认关闭）。
            if typed_series.label_hide_overlap() {
                decorator::compute_text_layouts(&mut elements);
                crate::pipeline::builder::hide_overlapping_labels(
                    &mut elements,
                    crate::pipeline::builder::Z_SERIES_LABEL,
                );
            }
            all_elements.extend(elements);
        }
    }

    // 渲染装饰元素（标题、图例、轴名称）
    let (decorator_elements, _title_height) =
        decorator::render_all_decorators(spec, width, height, specs, colors, theme);
    all_elements.extend(decorator_elements);

    // 计算文本布局
    decorator::compute_text_layouts(&mut all_elements);

    Ok(all_elements)
}

#[cfg(test)]
mod tests {
    use lievisual::Color;

    use super::*;
    use crate::pipeline::dataframe::DataFrame;

    #[test]
    fn test_build_pie_chart_v2() {
        let mut df = DataFrame::new();
        df.add_column(crate::pipeline::dataframe::Series::new(
            "name",
            vec![
                crate::pipeline::dataframe::DataValue::String("A".into()),
                crate::pipeline::dataframe::DataValue::String("B".into()),
                crate::pipeline::dataframe::DataValue::String("C".into()),
            ],
        ));
        df.add_column(crate::pipeline::dataframe::Series::new(
            "value",
            vec![
                crate::pipeline::dataframe::DataValue::Float(30.0),
                crate::pipeline::dataframe::DataValue::Float(50.0),
                crate::pipeline::dataframe::DataValue::Float(20.0),
            ],
        ));

        let spec = crate::pipeline::types::ChartSpec {
            width: 800,
            height: 600,
            grids: vec![crate::pipeline::types::GridSpec {
                left: None,
                right: None,
                top: None,
                bottom: None,
                width: None,
                height: None,
                contain_label: false,
            }],
            x_axes: vec![],
            y_axes: vec![],
            series: vec![crate::pipeline::types::SeriesSpec {
                name: "Sales".into(),
                data: df,
                grid_index: 0,
                x_axis_index: 0,
                y_axis_index: 0,
                stack: None,
                group_index: 0,
                sampling: None,
                item_style: crate::pipeline::types::ItemStyleSpec::default(),
                config: crate::pipeline::types::SeriesConfig::Pie(
                    crate::pipeline::types::PieConfig {
                        category_col: "name".into(),
                        value_col: "value".into(),
                        ..Default::default()
                    },
                ),
            }],
            title: Some(crate::pipeline::types::TitleSpec {
                text: Some("Pie Test".into()),
                ..Default::default()
            }),
            legend: None,
            background: Color::rgb(255, 255, 255),
            palette: vec![],
            theme_name: None,
            fit_mode: FitMode::Fixed,
        };

        let elements = build_chart_from_spec(&spec, &Theme::echarts()).unwrap();

        assert!(
            !elements.is_empty(),
            "Pie chart should produce visual elements"
        );

        let sector_count = elements
            .iter()
            .filter(|e| matches!(e.element(), Some(lievisual::scene::Element::Path { .. })))
            .count();
        assert!(sector_count >= 3, "Should have at least 3 pie sectors");
    }

    /// 构造带长数值刻度的 Y 轴测试 spec
    fn make_spec(fit_mode: FitMode) -> ChartSpec {
        use crate::pipeline::dataframe::Series;
        use crate::pipeline::types::{AxisSpec, AxisType, LineConfig, SeriesConfig, SeriesSpec};

        let mut df = DataFrame::new();
        df.add_column(Series::new(
            "x",
            Vec::<crate::pipeline::dataframe::DataValue>::new(),
        ));
        df.add_column(Series::new(
            "y",
            vec![
                crate::pipeline::dataframe::DataValue::Float(120_000_000.0),
                crate::pipeline::dataframe::DataValue::Float(240_000_000.0),
            ],
        ));

        ChartSpec {
            width: 300,
            height: 200,
            grids: vec![GridSpec {
                left: None,
                right: None,
                top: None,
                bottom: None,
                width: None,
                height: None,
                contain_label: false,
            }],
            x_axes: vec![AxisSpec {
                axis_type: AxisType::Value,
                position: crate::pipeline::types::AxisPosition::Bottom,
                grid_index: 0,
                min: None,
                max: None,
                name: None,
                name_location: None,
                categories: vec![],
                boundary_gap: true,
                inverse: false,
                split_number: None,
                label_show: false,
                label_formatter: None,
                label_rotate: None,
                name_gap: None,
                label_interval: None,
                decor: crate::pipeline::types::AxisDecor::echant_defaults(),
                z: None,
            }],
            y_axes: vec![AxisSpec {
                axis_type: AxisType::Value,
                position: crate::pipeline::types::AxisPosition::Left,
                grid_index: 0,
                min: None,
                max: None,
                name: None,
                name_location: None,
                categories: vec![],
                boundary_gap: true,
                inverse: false,
                split_number: None,
                label_show: true,
                label_formatter: None,
                label_rotate: None,
                name_gap: None,
                label_interval: None,
                decor: crate::pipeline::types::AxisDecor::echant_defaults(),
                z: None,
            }],
            series: vec![SeriesSpec {
                name: "s".into(),
                data: df,
                grid_index: 0,
                x_axis_index: 0,
                y_axis_index: 0,
                stack: None,
                group_index: 0,
                sampling: None,
                item_style: Default::default(),
                config: SeriesConfig::Line(LineConfig::default()),
            }],
            title: None,
            legend: None,
            background: Color::rgb(255, 255, 255),
            palette: vec![],
            theme_name: None,
            fit_mode,
        }
    }

    /// P2：`sampling` 不带阈值（哨兵 0）时按绘图区像素尺寸解析目标点数。
    #[test]
    fn auto_sampling_threshold_follows_plot_pixels() {
        use crate::pipeline::types::SubplotSpec;
        use crate::sampling::{AUTO_SAMPLING_THRESHOLD, SamplingType};

        let spec = make_spec(FitMode::Fixed);
        let mut series = spec.series.clone();
        series[0].sampling = Some((SamplingType::Lttb, AUTO_SAMPLING_THRESHOLD));
        let specs = vec![SubplotSpec {
            id: 0,
            bounds: Rect::new(0.0, 0.0, 640.0, 480.0),
            series_indices: vec![0],
            x_axis_indices: vec![0],
            y_axis_indices: vec![0],
        }];

        resolve_sampling_thresholds(&mut series, &specs);
        assert_eq!(
            series[0].sampling.unwrap().1,
            640,
            "自动阈值应取绘图区最大边长"
        );

        // 显式阈值不受影响
        series[0].sampling = Some((SamplingType::Lttb, 123));
        resolve_sampling_thresholds(&mut series, &specs);
        assert_eq!(series[0].sampling.unwrap().1, 123, "显式阈值不应被覆盖");
    }

    /// 回归：极坐标系列的布局基准——饼图以**画布**为基准（ECharts 语义：防
    /// grid 的 footer 预留把半径压扁）；极坐标柱/散点以**绘图区**为基准。
    ///
    /// 后者修复了 API 与 JSON 两路径的同配置不一致：compat 对未显式配 `grid`
    /// 的 JSON 套用 v6 默认边距（15%/10%），JSON 版 inset 于画布；而 API 版曾
    /// 走整画布基准，圆心/半径都不同（且顶部标签侵入副标题区）。
    #[test]
    fn polar_series_bounds_pie_uses_canvas_polar_bar_uses_plot_area() {
        use crate::pipeline::dataframe::DataFrame;
        use crate::pipeline::types::{
            GridEdge, GridSpec, ItemStyleSpec, PieConfig, PolarBarConfig, SeriesConfig, SeriesSpec,
            SubplotSpec,
        };

        let plot = Rect::new(120.0, 67.0, 720.0, 520.0);
        let canvas = Rect::new(0.0, 0.0, 800.0, 600.0);
        let subplot = SubplotSpec {
            id: 0,
            bounds: plot,
            series_indices: vec![0],
            x_axis_indices: vec![],
            y_axis_indices: vec![],
        };

        let mut spec = make_spec(FitMode::Fixed);
        spec.grids = vec![GridSpec {
            left: None,
            right: None,
            top: None,
            bottom: None,
            width: None,
            height: None,
            contain_label: false,
        }];
        spec.series = vec![SeriesSpec {
            name: "s".into(),
            data: DataFrame::new(),
            grid_index: 0,
            x_axis_index: 0,
            y_axis_index: 0,
            stack: None,
            group_index: 0,
            sampling: None,
            item_style: ItemStyleSpec::default(),
            config: SeriesConfig::PolarBar(PolarBarConfig {
                angle_col: "angle".into(),
                radius_col: "radius".into(),
                category_col: None,
                pad_angle: 2.0,
                start_angle: 0.0,
            }),
        }];

        // 极坐标柱：grid 未显式配置 → 绘图区（而非整画布）
        assert_eq!(polar_series_bounds(&subplot, &spec, 800, 600), plot);

        // 饼图：grid 未显式配置 → 画布（ECharts 饼图语义）
        spec.series[0].config = SeriesConfig::Pie(PieConfig::default());
        assert_eq!(polar_series_bounds(&subplot, &spec, 800, 600), canvas);

        // 显式配置过 grid（典型多子图）→ 绘图区
        spec.series[0].config = SeriesConfig::PolarBar(PolarBarConfig {
            angle_col: "angle".into(),
            radius_col: "radius".into(),
            category_col: None,
            pad_angle: 2.0,
            start_angle: 0.0,
        });
        spec.grids[0].left = Some(GridEdge::Pct(15.0));
        assert_eq!(polar_series_bounds(&subplot, &spec, 800, 600), plot);
    }

    #[test]
    fn test_hug_grows_canvas_for_long_value_labels() {
        // 千万级数值刻度（如 "12,000,000"）实测宽度超出默认 60px 左边距：
        // Fixed 只能收缩绘图区，Hug 应把画布加宽
        let fixed = build_chart_with_layout(&make_spec(FitMode::Fixed), &Theme::echarts()).unwrap();
        let hug = build_chart_with_layout(&make_spec(FitMode::Hug), &Theme::echarts()).unwrap();

        assert_eq!(fixed.width, 300, "Fixed 画布尺寸必须不变");
        assert!(hug.width > 300, "Hug 应加宽画布，实际 {}", hug.width);
        assert_eq!(hug.height, 200, "本例纵向无缺口，高度不变");
        assert!(!hug.elements.is_empty(), "Hug 输出应包含可视元素");
    }

    #[test]
    fn test_hug_converges_and_labels_stay_inside() {
        // Hug 收敛后：Y 轴标签不得越过绘图区左缘，整体位于绘图区与画布左缘之间。
        // Y 轴标签现以墨迹盒锚定（align=Left/Top，位置 = 锚点 − R(θ)·ink_center），
        // 块原点 = 画布内正值。
        let hug = build_chart_with_layout(&make_spec(FitMode::Hug), &Theme::echarts()).unwrap();

        use lievisual::scene::Element;
        // 本测试 X 轴 label_show=false，场景中的 Text 元素全部是 Y 轴刻度标签
        let text_xs: Vec<f64> = hug
            .elements
            .iter()
            .filter_map(|n| match n.element() {
                Some(Element::Text { position, .. }) => Some(position.x),
                _ => None,
            })
            .collect();
        assert!(!text_xs.is_empty(), "应存在 Y 轴刻度标签");
        // 画布加宽后绘图区左缘 = 目标边距，标签块原点应在画布内
        for &x in &text_xs {
            assert!(x > 0.0 && x < hug.width as f64, "标签锚点 x={x} 越界");
        }
    }
}
