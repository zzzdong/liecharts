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
use crate::text::{create_text_layout, compute_text_offset};
use crate::theme::Theme;
use crate::visual::{Color, FillStrokeStyle, VisualElement, Z_BACKGROUND, Z_LABEL, Z_TITLE};
use vello_cpu::kurbo::{Point, Rect};

pub fn build_chart(
    option: &ChartOption,
    width: u32,
    height: u32,
) -> Result<Vec<VisualElement>> {
    build_chart_with_theme(option, width, height, &Theme::echarts())
}

pub fn build_chart_with_theme(
    option: &ChartOption,
    width: u32,
    height: u32,
    theme: &Theme,
) -> Result<Vec<VisualElement>> {
    let planner = GridPlanner::new(width, height, option);
    let specs: Vec<SubplotSpec> = planner.plan();

    let resolver = AxisBindingResolver::new(option);
    let axis_ranges: ResolvedAxisRanges = resolver.resolve(&specs);

    let series_count = option.series.len();
    let assigner = ColorAssigner;
    let colors: ColorContext = assigner.assign_with_theme(series_count, theme);

    let mut text_measurer = TextMeasurer::new();
    let mut axis_subplot_data: Vec<SubplotVisualData> = Vec::new();

    for spec in &specs {
        let axis_elements =
            AxisRenderer::render(spec, option, &axis_ranges, &colors, &mut text_measurer);
        axis_subplot_data.push(SubplotVisualData {
            series_elements: Vec::new(),
            axis_elements,
            grid_lines: Vec::new(),
        });
    }

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

    let builder = VisualElementBuilder::new();
    let mut all_data = axis_subplot_data;
    all_data.extend(series_subplot_data);
    let mut elements = vec![VisualElement::Rect {
        rect: Rect::new(0.0, 0.0, width as f64, height as f64),
        style: FillStrokeStyle {
            fill: Some(colors.background),
            stroke: None,
        },
        z_index: Z_BACKGROUND,
    }];
    elements.extend(builder.build(all_data));

    elements.extend(build_title_elements(option, width, theme));
    elements.extend(build_legend_elements(option, width, &colors, theme));
    elements.extend(build_axis_name_elements(option, &specs, &colors, theme));

    compute_text_layouts(&mut elements);

    Ok(elements)
}

fn compute_text_layouts(elements: &mut [VisualElement]) {
    for element in elements.iter_mut() {
        if let VisualElement::TextRun {
            text,
            style,
            max_width,
            layout,
            ..
        } = element
        {
            if layout.is_none() {
                let text_layout = create_text_layout(text, style, *max_width);
                let (x_off, y_off) = compute_text_offset(
                    &text_layout,
                    style.align,
                    style.vertical_align,
                );
                if let VisualElement::TextRun { position, layout, .. } = element {
                    position.x += x_off;
                    position.y += y_off;
                    *layout = Some(text_layout);
                }
            }
        }
    }
}

fn build_title_elements(option: &ChartOption, width: u32, theme: &Theme) -> Vec<VisualElement> {
    let mut elements = Vec::new();

    if let Some(title) = &option.title {
        let title_style = theme.get_title_text_style();
        let subtitle_style = theme.get_subtitle_text_style();

        let title_color = Color::from_hex(&title_style.color).unwrap_or(Color::new(60, 60, 65));
        let subtitle_color = Color::from_hex(&subtitle_style.color).unwrap_or(Color::new(84, 85, 90));

        let title_x = width as f64 / 2.0;
        let title_y = 16.0;

        if let Some(text) = &title.text {
            elements.push(VisualElement::TextRun {
                text: text.clone(),
                position: Point::new(title_x, title_y),
                style: crate::visual::TextStyle {
                    font_size: title_style.font_size,
                    color: title_color,
                    font_family: title_style.font_family.clone(),
                    align: crate::visual::TextAlign::Center,
                    vertical_align: crate::visual::TextBaseline::Top,
                    ..Default::default()
                },
                rotation: 0.0,
                max_width: None,
                layout: None,
                z_index: Z_TITLE,
            });
        }

        if let Some(subtext) = &title.subtext {
            let subtext_y = title_y + title_style.font_size + 2.0;
            elements.push(VisualElement::TextRun {
                text: subtext.clone(),
                position: Point::new(title_x, subtext_y),
                style: crate::visual::TextStyle {
                    font_size: subtitle_style.font_size,
                    color: subtitle_color,
                    font_family: subtitle_style.font_family.clone(),
                    align: crate::visual::TextAlign::Center,
                    vertical_align: crate::visual::TextBaseline::Top,
                    ..Default::default()
                },
                rotation: 0.0,
                max_width: None,
                layout: None,
                z_index: Z_TITLE,
            });
        }
    }

    elements
}

fn build_legend_elements(
    option: &ChartOption,
    width: u32,
    colors: &ColorContext,
    theme: &Theme,
) -> Vec<VisualElement> {
    let mut elements = Vec::new();

    if let Some(legend) = &option.legend {
        if legend.show != Some(true) {
            return elements;
        }

        let legend_style = theme.get_legend_text_style();
        let legend_color = Color::from_hex(&legend_style.color).unwrap_or(Color::new(60, 60, 65));

        let data = legend.data.as_ref().map(|d| d.as_slice()).unwrap_or(&[]);
        let item_width = legend.item_width.unwrap_or(80.0);
        let symbol_size = legend.symbol_size.unwrap_or(10.0);

        let total_width = data.len() as f64 * item_width;
        let start_x = (width as f64 - total_width) / 2.0;
        let y = 72.0;

        for (i, name) in data.iter().enumerate() {
            let x = start_x + i as f64 * item_width + item_width / 2.0;

            let color = colors
                .series_colors
                .get(i)
                .copied()
                .unwrap_or(Color::new(80, 112, 221));

            elements.push(VisualElement::Rect {
                rect: Rect::new(x - symbol_size - 5.0, y - symbol_size / 2.0, x - 5.0, y + symbol_size / 2.0),
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: None,
                },
                z_index: Z_TITLE,
            });

            elements.push(VisualElement::TextRun {
                text: name.clone(),
                position: Point::new(x, y),
                style: crate::visual::TextStyle {
                    font_size: legend_style.font_size,
                    color: legend_color,
                    font_family: legend_style.font_family.clone(),
                    align: crate::visual::TextAlign::Left,
                    vertical_align: crate::visual::TextBaseline::Middle,
                    ..Default::default()
                },
                rotation: 0.0,
                max_width: None,
                layout: None,
                z_index: Z_TITLE,
            });
        }
    }

    elements
}

fn build_axis_name_elements(
    option: &ChartOption,
    specs: &[SubplotSpec],
    _colors: &ColorContext,
    theme: &Theme,
) -> Vec<VisualElement> {
    let mut elements = Vec::new();

    let axis_label_style = theme.get_axis_label_style();
    let label_color = Color::from_hex(&axis_label_style.color).unwrap_or(Color::new(84, 85, 90));

    for spec in specs {
        let bounds = spec.bounds;

        for (i, &y_axis_idx) in spec.y_axis_indices.iter().enumerate() {
            if let Some(y_axis) = option.y_axis.get(y_axis_idx) {
                if let Some(name) = &y_axis.name {
                    let is_right = y_axis.position == Some(crate::option::AxisPosition::Right)
                        || (y_axis.position.is_none() && i > 0);
                    let (x, rotation) = if is_right {
                        (bounds.x1 + 40.0, std::f64::consts::FRAC_PI_2)
                    } else {
                        (bounds.x0 - 40.0, -std::f64::consts::FRAC_PI_2)
                    };
                    let y = bounds.y0 + bounds.height() / 2.0;

                    elements.push(VisualElement::TextRun {
                        text: name.clone(),
                        position: Point::new(x, y),
                        style: crate::visual::TextStyle {
                            font_size: axis_label_style.font_size,
                            color: label_color,
                            font_family: axis_label_style.font_family.clone(),
                            align: crate::visual::TextAlign::Center,
                            vertical_align: crate::visual::TextBaseline::Bottom,
                            ..Default::default()
                        },
                        rotation,
                        max_width: None,
                        layout: None,
                        z_index: Z_LABEL,
                    });
                }
            }
        }

        for &x_axis_idx in &spec.x_axis_indices {
            if let Some(x_axis) = option.x_axis.get(x_axis_idx) {
                if let Some(name) = &x_axis.name {
                    let x = bounds.x0 + bounds.width() / 2.0;
                    let y = bounds.y1 + 35.0;

                    elements.push(VisualElement::TextRun {
                        text: name.clone(),
                        position: Point::new(x, y),
                        style: crate::visual::TextStyle {
                            font_size: axis_label_style.font_size,
                            color: label_color,
                            font_family: axis_label_style.font_family.clone(),
                            align: crate::visual::TextAlign::Center,
                            vertical_align: crate::visual::TextBaseline::Top,
                            ..Default::default()
                        },
                        rotation: 0.0,
                        max_width: None,
                        layout: None,
                        z_index: Z_LABEL,
                    });
                }
            }
        }
    }

    elements
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
        assert!(label_count >= 3, "Should have at least 3 labels (pie + title)");
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
        let non_bg = elements.iter().filter(|e| !matches!(e, VisualElement::Rect { .. })).count();
        assert_eq!(non_bg, 0, "Empty pie should produce no elements besides background");
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
        assert!(rect_count >= 3, "Should have at least 3 bars (plus background)");

        let line_count = elements.iter().filter(|e| matches!(e, VisualElement::Line { .. })).count();
        assert!(line_count >= 2, "Should have at least 2 axis/grid lines");

        let label_count = elements.iter().filter(|e| matches!(e, VisualElement::TextRun { .. })).count();
        assert!(label_count >= 3, "Should have at least 3 text labels");
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
        assert!(!elements.is_empty());

        let rect_count = elements.iter().filter(|e| matches!(e, VisualElement::Rect { .. })).count();
        assert!(rect_count >= 3, "Should have at least 3 bars");
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
        assert!(!elements.is_empty());

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
        assert!(!elements.is_empty());

        let circle_count = elements.iter().filter(|e| matches!(e, VisualElement::Circle { .. })).count();
        assert_eq!(circle_count, 3, "Should have 3 scatter points");
    }
}