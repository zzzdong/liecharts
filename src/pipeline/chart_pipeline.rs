use vello_cpu::kurbo::Rect;

/// 基于 DataFrame 的新 Pipeline
///
/// 流程：
/// 1. GridPlanner: 计算每个 subplot 的布局边界
/// 2. AxisBindingResolver: 解析轴范围
/// 3. ColorAssigner: 分配颜色
/// 4. DataProcessor: 数据转换（采样等）
/// 5. Materializer: 将 SeriesSpec 转换为 TypedSeries
/// 6. Builder: 将 TypedSeries 组装为 VisualElement
/// 7. Decorator: 渲染标题、图例、轴名称等装饰元素
use crate::error::Result;
use crate::{
    pipeline::{
        RenderContext,
        axis_binding_resolver::AxisBindingResolver,
        axis_renderer,
        builder::build_typed_series,
        color_assigner::ColorAssigner,
        data_processor, decorator,
        grid_planner::GridPlanner,
        materializer::materialize_all,
        types::{
            ChartSpec, ChartType, ColorContext, ResolvedAxisRanges, SubplotSpec, TextMeasurer,
        },
    },
    theme::Theme,
    visual::{Fill, FillStrokeStyle, VisualElement, Z_BACKGROUND},
};

/// 从 ChartSpec 构建图表（新 API 入口）
pub fn build_chart_from_spec(spec: &ChartSpec, theme: &Theme) -> Result<Vec<VisualElement>> {
    build_chart_internal(spec, theme)
}

/// 从 ChartOption 构建图表（旧 API 入口）
pub fn build_chart(
    option: &crate::option::ChartOption,
    width: u32,
    height: u32,
) -> Result<Vec<VisualElement>> {
    build_chart_with_theme(option, width, height, &Theme::echarts())
}

/// 从 ChartOption 构建图表（旧 API 入口，支持主题）
pub fn build_chart_with_theme(
    option: &crate::option::ChartOption,
    width: u32,
    height: u32,
    theme: &Theme,
) -> Result<Vec<VisualElement>> {
    // 将 ChartOption 转换为 ChartSpec，然后使用新管线
    let spec = crate::pipeline::compat::chart_option_to_chart_spec(option, width, height);
    build_chart_internal(&spec, theme)
}

/// 内部的 ChartSpec 管线（统一入口，仅依赖 ChartSpec + Theme）
fn build_chart_internal(spec: &ChartSpec, theme: &Theme) -> Result<Vec<VisualElement>> {
    let width = spec.width;
    let height = spec.height;

    // 0. 估计标题和图例的占用高度，用于 GridPlanner 计算 top margin
    let header_height = decorator::estimate_header_height(spec, theme);

    // 1. 布局规划
    let planner = GridPlanner::new(width, height, header_height, &spec.grids);
    let specs: Vec<SubplotSpec> = planner.plan(&spec.series, &spec.x_axes, &spec.y_axes);

    // 2. 解析轴范围
    let resolver = AxisBindingResolver::new(&spec.x_axes, &spec.y_axes, &spec.series);
    let axis_ranges: ResolvedAxisRanges = resolver.resolve(&specs);

    // 3. 分配颜色
    let series_count = spec.series.len();
    let assigner = ColorAssigner;
    let colors: ColorContext = assigner.assign_with_theme(series_count, theme);

    // 4. 数据转换（采样等）
    // 克隆 series 以便进行可变的数据处理，同时保持 spec 不变
    let mut processed_series = spec.series.clone();
    data_processor::process_series(&mut processed_series);
    // 创建包含处理后的数据的 ChartSpec（用于 Materialize 阶段）
    let data_spec = ChartSpec {
        series: processed_series,
        ..spec.clone()
    };

    // 5. 收集所有 VisualElement
    let mut all_elements: Vec<VisualElement> = Vec::new();

    // 添加背景
    all_elements.push(crate::pipeline::builder::rect(
        Rect::new(0.0, 0.0, width as f64, height as f64),
        FillStrokeStyle {
            fill: Some(Fill::Solid(colors.background)),
            stroke: None,
        },
        Z_BACKGROUND,
    ));

    // 5. 渲染轴（跳过纯表格子图，表格不需要坐标轴）
    let mut text_measurer = TextMeasurer::new();
    for subplot in &specs {
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
            &axis_ranges,
            &colors,
            &mut text_measurer,
        );
        all_elements.extend(axis_elements);
    }

    // 6. 渲染系列数据（新流程：Materialize + Build）
    for subplot in &specs {
        // 创建渲染上下文
        let ctx = RenderContext {
            colors: &colors,
            theme,
            bounds: subplot.bounds,
        };

        // Materialize 阶段：将 SeriesSpec 转换为 TypedSeries
        // 使用 data_spec（包含经过 DataProcessor 处理的系列数据）
        let typed_series_list = materialize_all(
            &subplot.series_indices,
            &data_spec,
            subplot.bounds,
            &axis_ranges,
            &colors,
        )?;

        // Build 阶段：将 TypedSeries 组装为 VisualElement
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

            let elements = build_typed_series(&typed_series, &ctx)?;
            all_elements.extend(elements);
        }
    }

    // 7. 渲染装饰元素（标题、图例、轴名称）
    let (decorator_elements, _title_height) =
        decorator::render_all_decorators(spec, width, height, &specs, &colors, theme);
    all_elements.extend(decorator_elements);

    // 10. 计算文本布局
    decorator::compute_text_layouts(&mut all_elements);

    Ok(all_elements)
}

#[cfg(test)]
mod tests {
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
                subtext: None,
                font_size: None,
                subfont_size: None,
                color: None,
                subcolor: None,
            }),
            legend: None,
            background: crate::visual::Color::rgb(255, 255, 255),
            palette: vec![],
            theme_name: None,
        };

        let elements = build_chart_from_spec(&spec, &Theme::echarts()).unwrap();

        assert!(
            !elements.is_empty(),
            "Pie chart should produce visual elements"
        );

        let sector_count = elements
            .iter()
            .filter(|e| matches!(&e.element, lievisual::scene::Element::Path { .. }))
            .count();
        assert!(sector_count >= 3, "Should have at least 3 pie sectors");
    }
}
