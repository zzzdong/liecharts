use crate::error::Result;
use crate::new_pipeline::axis_binding_resolver::AxisBindingResolver;
use crate::new_pipeline::axis_renderer::AxisRenderer;
use crate::new_pipeline::color_assigner::ColorAssigner;
use crate::new_pipeline::data_processor::create_processor;
use crate::new_pipeline::grid_planner::GridPlanner;
use crate::new_pipeline::types::{
    ColorContext, DataProcessorInput, ResolvedAxisRanges, SubplotSpec, SubplotVisualData,
    TextMeasurer,
};
use crate::new_pipeline::visual_element_builder::VisualElementBuilder;
use crate::option::ChartOption;
use crate::visual::VisualElement;

/// 新管线入口：执行完整的 GridPlanner → AxisBindingResolver → ColorAssigner → DataProcessor → AxisRenderer → VisualElementBuilder
pub fn build_chart(
    option: &ChartOption,
    width: u32,
    height: u32,
) -> Result<Vec<VisualElement>> {
    // Step 1: GridPlanner — 纯数学画布切分
    let planner = GridPlanner::new(width, height, option);
    let specs: Vec<SubplotSpec> = planner.plan();

    // Step 2: AxisBindingResolver — 轴范围协调
    let resolver = AxisBindingResolver::new(option);
    let axis_ranges: ResolvedAxisRanges = resolver.resolve(&specs);

    // Step 3: ColorAssigner — 颜色分配
    let series_count = option.series.len();
    let assigner = ColorAssigner;
    let colors: ColorContext = assigner.assign(series_count);

    // Step 4: AxisRenderer — 坐标轴视觉元素（网格线、轴线、刻度标签）
    let mut text_measurer = TextMeasurer::new();
    let mut axis_subplot_data: Vec<SubplotVisualData> = Vec::new();

    for spec in &specs {
        let axis_elements = AxisRenderer::render(spec, option, &axis_ranges, &colors, &mut text_measurer);
        axis_subplot_data.push(SubplotVisualData {
            series_elements: Vec::new(),
            axis_elements,
            grid_lines: Vec::new(),
        });
    }

    // Step 5: DataProcessor — 数据驱动的视觉元素生成
    let mut series_subplot_data: Vec<SubplotVisualData> = Vec::new();

    for spec in &specs {
        for &series_idx in &spec.series_indices {
            let series = &option.series[series_idx];
            let processor = create_processor(series, series_idx);

            let input = DataProcessorInput {
                spec,
                option,
                colors: &colors,
                axis_ranges: &axis_ranges,
                text_measurer: &mut text_measurer,
            };

            let subplot_data = processor.process(input)?;
            if !subplot_data.series_elements.is_empty() {
                series_subplot_data.push(subplot_data);
            }
        }
    }

    // Step 6: VisualElementBuilder — 合并与排序
    let builder = VisualElementBuilder::new();
    let mut all_data = axis_subplot_data;
    all_data.extend(series_subplot_data);
    let elements = builder.build(all_data);

    Ok(elements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::option::*;

    #[test]
    fn test_build_pie_chart() {
        let option = ChartOption {
            title: Some(TitleOption::new("Pie Test")),
            series: vec![SeriesOption::Pie(PieSeriesOption {
                name: Some("Sales".into()),
                data: vec![
                    DataPoint::Named("A".into(), 30.0),
                    DataPoint::Named("B".into(), 50.0),
                    DataPoint::Named("C".into(), 20.0),
                ],
                ..Default::default()
            })],
            ..Default::default()
        };

        let elements = build_chart(&option, 800, 600).unwrap();

        assert!(!elements.is_empty(), "Pie chart should produce visual elements");

        let sector_count = elements.iter().filter(|e| matches!(e, VisualElement::Path { .. })).count();
        assert_eq!(sector_count, 3, "Should have 3 pie sectors");

        let label_count = elements.iter().filter(|e| matches!(e, VisualElement::TextRun { .. })).count();
        assert_eq!(label_count, 3, "Should have 3 labels");
    }

    #[test]
    fn test_build_empty_pie_chart() {
        let option = ChartOption {
            series: vec![SeriesOption::Pie(PieSeriesOption {
                name: Some("Empty".into()),
                data: vec![],
                ..Default::default()
            })],
            ..Default::default()
        };

        let elements = build_chart(&option, 800, 600).unwrap();
        assert!(elements.is_empty(), "Empty pie should produce no elements");
    }

    #[test]
    fn test_build_single_value_pie() {
        let option = ChartOption {
            series: vec![SeriesOption::Pie(PieSeriesOption {
                name: Some("Single".into()),
                data: vec![DataPoint::Named("Only".into(), 100.0)],
                ..Default::default()
            })],
            ..Default::default()
        };

        let elements = build_chart(&option, 400, 400).unwrap();
        let sector_count = elements.iter().filter(|e| matches!(e, VisualElement::Path { .. })).count();
        assert_eq!(sector_count, 1, "Single value should produce 1 sector");
    }

    #[test]
    fn test_build_bar_chart() {
        let option = ChartOption {
            x_axis: vec![AxisOption {
                axis_type: Some(AxisType::Category),
                data: Some(vec!["A".into(), "B".into(), "C".into()]),
                ..Default::default()
            }],
            y_axis: vec![AxisOption {
                axis_type: Some(AxisType::Value),
                ..Default::default()
            }],
            series: vec![SeriesOption::Bar(BarSeriesOption {
                name: Some("Sales".into()),
                data: vec![
                    DataPoint::Named("A".into(), 30.0),
                    DataPoint::Named("B".into(), 50.0),
                    DataPoint::Named("C".into(), 20.0),
                ],
                ..Default::default()
            })],
            ..Default::default()
        };

        let elements = build_chart(&option, 800, 600).unwrap();
        assert!(!elements.is_empty(), "Bar chart should produce visual elements");

        let rect_count = elements.iter().filter(|e| matches!(e, VisualElement::Rect { .. })).count();
        assert_eq!(rect_count, 3, "Should have 3 bars");

        let axis_line_count = elements.iter().filter(|e| matches!(e, VisualElement::Line { .. })).count();
        assert!(axis_line_count >= 2, "Should have at least 2 axis/grid lines");

        let label_count = elements.iter().filter(|e| matches!(e, VisualElement::TextRun { .. })).count();
        assert!(label_count >= 3, "Should have at least 3 text labels (3 category labels)");
    }

    #[test]
    fn test_build_bar_chart_with_values_on_x() {
        let option = ChartOption {
            x_axis: vec![AxisOption {
                axis_type: Some(AxisType::Value),
                ..Default::default()
            }],
            y_axis: vec![AxisOption {
                axis_type: Some(AxisType::Value),
                ..Default::default()
            }],
            series: vec![SeriesOption::Bar(BarSeriesOption {
                name: Some("Data".into()),
                data: vec![
                    DataPoint::XY(1.0, 10.0),
                    DataPoint::XY(2.0, 30.0),
                    DataPoint::XY(3.0, 20.0),
                ],
                ..Default::default()
            })],
            ..Default::default()
        };

        let elements = build_chart(&option, 800, 600).unwrap();
        assert!(!elements.is_empty(), "Bar chart with value X should produce visual elements");

        let rect_count = elements.iter().filter(|e| matches!(e, VisualElement::Rect { .. })).count();
        assert_eq!(rect_count, 3, "Should have 3 bars");
    }

    #[test]
    fn test_build_line_chart() {
        let option = ChartOption {
            x_axis: vec![AxisOption {
                axis_type: Some(AxisType::Category),
                data: Some(vec!["A".into(), "B".into(), "C".into()]),
                ..Default::default()
            }],
            y_axis: vec![AxisOption {
                axis_type: Some(AxisType::Value),
                ..Default::default()
            }],
            series: vec![SeriesOption::Line(LineSeriesOption {
                name: Some("Trend".into()),
                data: vec![
                    DataPoint::Named("A".into(), 10.0),
                    DataPoint::Named("B".into(), 30.0),
                    DataPoint::Named("C".into(), 20.0),
                ],
                ..Default::default()
            })],
            ..Default::default()
        };

        let elements = build_chart(&option, 800, 600).unwrap();
        assert!(!elements.is_empty(), "Line chart should produce visual elements");

        let polyline_count = elements.iter().filter(|e| matches!(e, VisualElement::Polyline { .. })).count();
        assert_eq!(polyline_count, 1, "Should have 1 polyline");

        let circle_count = elements.iter().filter(|e| matches!(e, VisualElement::Circle { .. })).count();
        assert_eq!(circle_count, 3, "Should have 3 symbol circles");
    }

    #[test]
    fn test_build_scatter_chart() {
        let option = ChartOption {
            x_axis: vec![AxisOption {
                axis_type: Some(AxisType::Value),
                ..Default::default()
            }],
            y_axis: vec![AxisOption {
                axis_type: Some(AxisType::Value),
                ..Default::default()
            }],
            series: vec![SeriesOption::Scatter(ScatterSeriesOption {
                name: Some("Points".into()),
                data: vec![
                    DataPoint::XY(1.0, 10.0),
                    DataPoint::XY(2.0, 20.0),
                    DataPoint::XY(3.0, 15.0),
                ],
                ..Default::default()
            })],
            ..Default::default()
        };

        let elements = build_chart(&option, 800, 600).unwrap();
        assert!(!elements.is_empty(), "Scatter chart should produce visual elements");

        let circle_count = elements.iter().filter(|e| matches!(e, VisualElement::Circle { .. })).count();
        assert_eq!(circle_count, 3, "Should have 3 scatter points");
    }
}