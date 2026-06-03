use vello_cpu::kurbo::{Point, Rect};

/// 基于 DataFrame 的新 Pipeline
///
/// 流程：
/// 1. GridPlanner: 计算每个 subplot 的布局边界
/// 2. AxisBindingResolver: 解析轴范围
/// 3. ColorAssigner: 分配颜色
/// 4. DataProcessorV2: 对每个 series:
///    a. to_dataframe(): 转换为 DataFrame
///    b. transform(): 转换 DataFrame（添加计算列）
///    c. to_visual_elements(): 生成 VisualElement
/// 5. 合并所有 VisualElement 并渲染
use crate::error::Result;
use crate::{
    option::ChartOption,
    pipeline::{
        axis_binding_resolver::AxisBindingResolver,
        axis_renderer::AxisRenderer,
        color_assigner::ColorAssigner,
        data_processor::{DataProcessorInput, create_processor},
        grid_planner::GridPlanner,
        group::{GroupAnalyzer, GroupType, dataframe_builder::GroupedBarProcessor},
        types::{ColorContext, ResolvedAxisRanges, SubplotSpec, TextMeasurer},
    },
    text::create_text_layout,
    theme::Theme,
    visual::{
        Color, FillStrokeStyle, TextAlign, TextBaseline, VisualElement, Z_BACKGROUND, Z_LABEL,
        Z_TITLE,
    },
};

pub fn build_chart(option: &ChartOption, width: u32, height: u32) -> Result<Vec<VisualElement>> {
    build_chart_with_theme(option, width, height, &Theme::echarts())
}

pub fn build_chart_with_theme(
    option: &ChartOption,
    width: u32,
    height: u32,
    theme: &Theme,
) -> Result<Vec<VisualElement>> {
    // 1. 布局规划
    let planner = GridPlanner::new(width, height, option);
    let specs: Vec<SubplotSpec> = planner.plan();

    // 2. 解析轴范围
    let resolver = AxisBindingResolver::new(option);
    let axis_ranges: ResolvedAxisRanges = resolver.resolve(&specs);

    // 3. 分配颜色
    let series_count = option.series.len();
    let assigner = ColorAssigner;
    let colors: ColorContext = assigner.assign_with_theme(series_count, theme);

    // 4. 收集所有 VisualElement
    let mut all_elements: Vec<VisualElement> = Vec::new();

    // 添加背景
    all_elements.push(VisualElement::Rect {
        rect: Rect::new(0.0, 0.0, width as f64, height as f64),
        style: FillStrokeStyle {
            fill: Some(colors.background),
            stroke: None,
        },
        z_index: Z_BACKGROUND,
    });

    // 5. 渲染轴
    let mut text_measurer = TextMeasurer::new();
    for spec in &specs {
        let axis_elements =
            AxisRenderer::render(spec, option, &axis_ranges, &colors, &mut text_measurer);
        all_elements.extend(axis_elements);
    }

    // 6. 渲染系列数据
    // 先通过 GroupAnalyzer 将 series 分组（SideBySide / Stacked / Single）
    // 分组模式：合并为一个 DataFrame，一次处理
    // 单 series 模式：保持原有流程
    for spec in &specs {
        let plans = GroupAnalyzer::analyze(&spec.series_indices, option);

        for plan in plans {
            match plan.group_type {
                GroupType::Single => {
                    let series_idx = plan.series_indices[0];
                    let series = &option.series[series_idx];
                    let processor = create_processor(series);

                    let input = DataProcessorInput {
                        spec,
                        option,
                        colors: &colors,
                        axis_ranges: &axis_ranges,
                        bounds: spec.bounds,
                        series_idx,
                    };

                    let elements = processor.process(series, &input)?;
                    all_elements.extend(elements);
                }
                GroupType::SideBySide | GroupType::Stacked => {
                    let first_idx = plan.series_indices[0];
                    let series = &option.series[first_idx];
                    let processor = create_processor(series);

                    let df = GroupedBarProcessor::combine_to_dataframe(&plan, option, &colors);

                    let input = DataProcessorInput {
                        spec,
                        option,
                        colors: &colors,
                        axis_ranges: &axis_ranges,
                        bounds: spec.bounds,
                        series_idx: first_idx,
                    };

                    let elements = processor.process_dataframe(df, &input)?;
                    all_elements.extend(elements);
                }
            }
        }
    }

    // 7. 渲染标题，并获取标题总高度
    let (title_elements, title_height) = build_title_elements(option, width, theme, &colors);
    all_elements.extend(title_elements);

    // 8. 渲染图例（根据标题高度动态计算位置）
    all_elements.extend(build_legend_elements_v2(
        option,
        width,
        &colors,
        theme,
        title_height,
    ));

    // 9. 渲染轴名称
    all_elements.extend(build_axis_name_elements(
        option, width, &specs, &colors, theme,
    ));

    // 10. 计算文本布局
    compute_text_layouts(&mut all_elements);

    Ok(all_elements)
}

/// 构建标题元素
///
/// 使用 layout_text 统一排版主标题和副标题，支持不同样式
fn build_title_elements(
    option: &ChartOption,
    width: u32,
    theme: &Theme,
    colors: &ColorContext,
) -> (Vec<VisualElement>, f64) {
    let mut elements = Vec::new();
    let mut title_height = 0.0;

    if let Some(title) = &option.title {
        let title_style = theme.get_title_text_style();
        let subtitle_style = theme.get_subtitle_text_style();

        // 从 ColorContext 获取颜色
        let title_color = Color::from_hex(&title_style.color).unwrap_or(colors.text_color);
        let subtitle_color =
            Color::from_hex(&subtitle_style.color).unwrap_or(colors.text_secondary_color);

        let mut y_offset = 24.0;

        if let Some(text) = &title.text {
            // 构建文本样式
            let main_text_style = crate::visual::TextStyle {
                font_size: title_style.font_size,
                color: title_color,
                font_family: title_style.font_family.clone(),
                font_weight: crate::option::FontWeight::Named(
                    crate::option::FontWeightNamed::Normal,
                ),
                ..Default::default()
            };

            let layout = create_text_layout(text, &main_text_style, None);
            // 文本块左上角为锚点，计算居中位置
            let position_x = (width as f64 - layout.width() as f64) / 2.0;

            let position_y = y_offset;

            y_offset += layout.height() as f64;
            title_height += layout.height() as f64;

            elements.push(VisualElement::TextRun {
                text: text.clone(),
                position: Point::new(position_x, position_y),
                style: main_text_style,
                rotation: 0.0,
                max_width: None,
                layout: Some(layout),
                z_index: Z_TITLE,
            });
        }

        if let Some(subtext) = &title.subtext {
            let sub_text_style = crate::visual::TextStyle {
                font_size: subtitle_style.font_size,
                color: subtitle_color,
                font_family: subtitle_style.font_family.clone(),
                font_weight: crate::option::FontWeight::Named(
                    crate::option::FontWeightNamed::Normal,
                ),
                ..Default::default()
            };

            let layout = create_text_layout(subtext, &sub_text_style, None);
            // 文本块左上角为锚点，计算居中位置
            let position_x = (width as f64 - layout.width() as f64) / 2.0;
            let position_y = y_offset + layout.height() as f64 * 0.1;
            title_height += layout.height() as f64 * 1.1; // 包含 0.1 倍间距
            elements.push(VisualElement::TextRun {
                text: subtext.clone(),
                position: Point::new(position_x, position_y),
                style: sub_text_style,
                rotation: 0.0,
                max_width: None,
                layout: Some(layout),
                z_index: Z_TITLE,
            });
        }
    }

    (elements, title_height)
}

/// 构建图例元素（V2 版本 - 支持饼图等从 palette 取色）
fn build_legend_elements_v2(
    option: &ChartOption,
    width: u32,
    colors: &ColorContext,
    theme: &Theme,
    title_height: f64,
) -> Vec<VisualElement> {
    use crate::{option::SeriesOption, text::create_text_layout};

    let mut elements = Vec::new();

    if let Some(legend) = &option.legend {
        if legend.show != Some(true) {
            return elements;
        }

        let legend_style = theme.get_legend_text_style();
        let legend_color = Color::from_hex(&legend_style.color).unwrap_or(colors.text_color);

        let data = legend.data.as_deref().unwrap_or(&[]);
        let symbol_size = legend.symbol_size.unwrap_or(10.0);
        let item_gap = 8.0; // symbol 和文本之间的间距
        let legend_padding = 16.0; // 每个 item 内部的 padding

        // 判断图表类型：饼图/环形图/极坐标柱状图使用 palette（按数据点着色），其他使用 series_colors（按系列着色）
        let use_palette = option
            .series
            .iter()
            .any(|s| matches!(s, SeriesOption::Pie(_) | SeriesOption::PolarBar(_)));

        // 第一步：计算每个 item 的实际宽度（symbol + gap + 文本宽度）
        let mut item_widths = Vec::new();
        let mut total_content_width = 0.0;

        for (_i, name) in data.iter().enumerate() {
            // 估算文本宽度（使用 create_text_layout）
            let text_style = crate::visual::TextStyle {
                font_size: legend_style.font_size,
                color: legend_color,
                font_family: legend_style.font_family.clone(),
                align: TextAlign::Left, // measure 时使用 Left
                vertical_align: TextBaseline::Middle,
                ..Default::default()
            };
            let text_layout = create_text_layout(name, &text_style, None);
            let text_width = text_layout.width() as f64;

            let item_width = symbol_size + item_gap + text_width + legend_padding * 2.0;
            item_widths.push(item_width);
            total_content_width += item_width;
        }

        // 第二步：计算整体起始位置（整体居中）
        let start_x = (width as f64 - total_content_width) / 2.0;
        // 根据标题高度动态计算图例位置：标题顶部边距(24) + 标题高度 + 额外间距(16)
        let y = 24.0 + title_height + 16.0;

        // 第三步：布局每个 item
        let mut current_x = start_x;

        for (i, name) in data.iter().enumerate() {
            let item_width = item_widths[i];
            let content_start_x = current_x + legend_padding;

            // 选择颜色源：按数据点着色的图表使用 palette，按系列着色的使用 series_colors
            let color = if use_palette {
                colors
                    .palette
                    .get(i)
                    .copied()
                    .unwrap_or_else(|| colors.get_series_color(i))
            } else {
                colors
                    .series_colors
                    .get(i)
                    .copied()
                    .unwrap_or_else(|| colors.get_series_color(i))
            };

            // 图例符号 - 以 y 为中心垂直对齐
            let symbol_x = content_start_x;
            elements.push(VisualElement::Rect {
                rect: Rect::new(
                    symbol_x,
                    y - symbol_size / 2.0,
                    symbol_x + symbol_size,
                    y + symbol_size / 2.0,
                ),
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: None,
                },
                z_index: Z_TITLE,
            });

            // 图例文字 - 使用 Left 对齐，位置在 symbol 右侧
            let text_x = symbol_x + symbol_size + item_gap;
            elements.push(VisualElement::TextRun {
                text: name.clone(),
                position: Point::new(text_x, y),
                style: crate::visual::TextStyle {
                    font_size: legend_style.font_size,
                    color: legend_color,
                    font_family: legend_style.font_family.clone(),
                    align: TextAlign::Left, // 使用 Left 对齐
                    vertical_align: TextBaseline::Middle,
                    ..Default::default()
                },
                rotation: 0.0,
                max_width: None,
                layout: None,
                z_index: Z_TITLE,
            });

            current_x += item_width;
        }
    }

    elements
}

/// 构建轴名称元素
fn build_axis_name_elements(
    option: &ChartOption,
    width: u32,
    specs: &[SubplotSpec],
    _colors: &ColorContext,
    theme: &Theme,
) -> Vec<VisualElement> {
    let mut elements = Vec::new();

    let axis_label_style = theme.get_axis_label_style();
    let label_color = Color::from_hex(&axis_label_style.color).unwrap_or(Color::new(84, 85, 90));

    for spec in specs {
        let bounds = spec.bounds;

        // 处理 Y 轴名称
        for (i, &y_axis_idx) in spec.y_axis_indices.iter().enumerate() {
            if let Some(y_axis) = option.y_axis.get(y_axis_idx)
                && let Some(name) = &y_axis.name
            {
                let is_right = y_axis.position == Some(crate::option::AxisPosition::Right)
                    || (y_axis.position.is_none() && i > 0);

                // 先测量文本尺寸，基于实际宽度和 grid 边界计算安全位置
                // 轴名称强制单行显示，避免旋转后布局异常
                // 使用左上角作为锚点，根据左右轴设置不同的对齐方式
                let (initial_align, initial_baseline) = (TextAlign::Left, TextBaseline::Top);
                let text_style = crate::visual::TextStyle {
                    font_size: axis_label_style.font_size,
                    color: label_color,
                    font_family: axis_label_style.font_family.clone(),
                    align: initial_align,
                    vertical_align: initial_baseline,
                    ..Default::default()
                };
                // 使用 None 让文本自然布局，不进行强制换行
                let text_layout = create_text_layout(name, &text_style, None);
                let _text_width = text_layout.width() as f64;
                let text_height = text_layout.height() as f64;

                // 计算轴名称位置
                // 旋转后文本呈竖直状态：
                // - 左轴（旋转-90°）：文本向上延伸，需保证 text_top >= margin
                // - 右轴（旋转+90°）：文本向下延伸，需保证 text_bottom <= width - margin
                let margin = 10.0; // 画布边缘留白
                let label_margin = 8.0; // 轴名称与刻度标签间距

                // 使用左上角作为锚点和旋转中心
                // 左轴：旋转-90°，文本向上延伸，锚点放在 grid 左侧
                // 右轴：旋转+90°，文本向下延伸，锚点放在 grid 右侧
                // 需要考虑刻度标签的空间，避免重叠
                let max_label_width = 35.0; // 预估最大刻度标签宽度（包括多位数）
                let (x, rotation, align, baseline) = if is_right {
                    // 右轴名称放在 grid 右侧
                    // 旋转+90°，文本从左上角向下延伸
                    // 刻度标签在 bounds.x1 + 8 处左对齐，最宽约 35px
                    // 轴名称应放在标签右侧：锚点.x >= bounds.x1 + 8 + max_label_width + label_margin
                    let min_anchor_x = bounds.x1 + 8.0 + max_label_width + label_margin;
                    let anchor_x = min_anchor_x
                        .max(bounds.x1 + label_margin)
                        .min(width as f64 - margin - text_height);
                    (
                        anchor_x,
                        std::f64::consts::FRAC_PI_2,
                        TextAlign::Left,
                        TextBaseline::Top,
                    )
                } else {
                    // 左轴名称放在 grid 左侧
                    // 旋转-90°，文本从左上角向上延伸
                    // 刻度标签在 bounds.x0 - 8 处右对齐，最宽约 35px，左边缘约在 bounds.x0 - 8 - 35 = bounds.x0 - 43
                    // 轴名称应放在标签左侧：锚点.x + text_height <= bounds.x0 - 43 - label_margin
                    let label_left_edge = bounds.x0 - 8.0 - max_label_width;
                    let max_anchor_x = label_left_edge - label_margin - text_height;
                    if max_anchor_x >= margin {
                        // 正常情况：放在标签左侧
                        (
                            max_anchor_x,
                            -std::f64::consts::FRAC_PI_2,
                            TextAlign::Left,
                            TextBaseline::Top,
                        )
                    } else {
                        // 空间不足：放在 grid 右侧，旋转+90°
                        let min_anchor_x = bounds.x1 + 8.0 + max_label_width + label_margin;
                        let anchor_x = min_anchor_x
                            .max(bounds.x1 + label_margin)
                            .min(width as f64 - margin - text_height);
                        (
                            anchor_x,
                            std::f64::consts::FRAC_PI_2,
                            TextAlign::Left,
                            TextBaseline::Top,
                        )
                    }
                };
                let y = bounds.y0 + bounds.height() / 2.0;

                elements.push(VisualElement::TextRun {
                    text: name.clone(),
                    position: Point::new(x, y),
                    style: crate::visual::TextStyle {
                        font_size: axis_label_style.font_size,
                        color: label_color,
                        font_family: axis_label_style.font_family.clone(),
                        align,
                        vertical_align: baseline,
                        ..Default::default()
                    },
                    rotation,
                    max_width: None,
                    layout: Some(text_layout),
                    z_index: Z_LABEL,
                });
            }
        }

        // 处理 X 轴名称
        for (i, &x_axis_idx) in spec.x_axis_indices.iter().enumerate() {
            if let Some(x_axis) = option.x_axis.get(x_axis_idx)
                && let Some(name) = &x_axis.name
            {
                let is_top = x_axis.position == Some(crate::option::AxisPosition::Top)
                    || (x_axis.position.is_none() && i > 0);

                let x = bounds.x0 + bounds.width() / 2.0;
                let y = if is_top {
                    bounds.y0 - 25.0 // 上方轴，名称在轴上方
                } else {
                    bounds.y1 + 35.0 // 下方轴，名称在轴下方
                };

                elements.push(VisualElement::TextRun {
                    text: name.clone(),
                    position: Point::new(x, y),
                    style: crate::visual::TextStyle {
                        font_size: axis_label_style.font_size,
                        color: label_color,
                        font_family: axis_label_style.font_family.clone(),
                        align: TextAlign::Center,
                        vertical_align: TextBaseline::Middle,
                        ..Default::default()
                    },
                    rotation: 0.0, // X轴名称不旋转
                    max_width: None,
                    layout: None,
                    z_index: Z_LABEL,
                });
            }
        }
    }

    elements
}

/// 计算文本布局
fn compute_text_layouts(elements: &mut [VisualElement]) {
    use crate::text::compute_text_offset;

    for element in elements.iter_mut() {
        if let VisualElement::TextRun {
            text,
            style,
            max_width,
            layout,
            ..
        } = element
            && layout.is_none()
        {
            let text_layout = create_text_layout(text, style, *max_width);
            let (x_off, y_off) =
                compute_text_offset(&text_layout, style.align, style.vertical_align);
            if let VisualElement::TextRun {
                position, layout, ..
            } = element
            {
                position.x += x_off;
                position.y += y_off;
                *layout = Some(text_layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::option::*;

    #[test]
    fn test_build_pie_chart_v2() {
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

        assert!(
            !elements.is_empty(),
            "Pie chart should produce visual elements"
        );

        let sector_count = elements
            .iter()
            .filter(|e| matches!(e, VisualElement::Path { .. }))
            .count();
        assert!(sector_count >= 3, "Should have at least 3 pie sectors");
    }
}
