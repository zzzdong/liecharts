//! 图例渲染
//!
//! 支持饼图等从 palette 取色的图表类型。

use vello_cpu::kurbo::{Point, Rect};

use crate::{
    pipeline::types::{ChartSpec, ChartType, ColorContext},
    text::create_text_layout,
    theme::Theme,
    visual::{Color, FillStrokeStyle, TextAlign, TextBaseline, TextStyle, VisualElement, Z_TITLE},
};

/// 构建图例元素
///
/// 根据标题高度动态计算图例位置，支持 palette 取色和系列取色两种模式。
pub fn render_legend(
    spec: &ChartSpec,
    width: u32,
    colors: &ColorContext,
    theme: &Theme,
    title_height: f64,
) -> Vec<VisualElement> {
    let mut elements = Vec::new();

    if let Some(legend) = &spec.legend {
        if !legend.show {
            return elements;
        }

        let legend_style = theme.get_legend_text_style();
        let legend_color = Color::from_hex(&legend_style.color).unwrap_or(colors.text_color);

        let data = &legend.data;
        let symbol_size = legend.symbol_size;
        let item_gap = 8.0; // symbol 和文本之间的间距
        let legend_padding = 16.0; // 每个 item 内部的 padding

        // 判断图表类型：饼图/环形图/极坐标柱状图使用 palette（按数据点着色），其他使用 series_colors（按系列着色）
        let use_palette = spec
            .series
            .iter()
            .any(|s| matches!(s.config.chart_type(), ChartType::Pie | ChartType::PolarBar));

        // 应用图例 formatter 模板，得到每个 item 的展示文本
        let display_texts: Vec<String> = data
            .iter()
            .map(|name| {
                // 图例项既是数据项名也是系列名，`{a}`/`{b}`/`{name}` 都指向它
                let ctx = crate::pipeline::template::TemplateContext {
                    series_name: Some(name),
                    name: Some(name),
                    value: None,
                    percent: None,
                };
                crate::pipeline::template::render_template(legend.formatter.as_deref(), &ctx, name)
            })
            .collect();

        // 第一步：计算每个 item 的实际宽度（symbol + gap + 文本宽度）
        let mut item_widths = Vec::new();
        let mut total_content_width = 0.0;

        for name in &display_texts {
            let text_style = TextStyle {
                font_size: legend_style.font_size,
                color: legend_color,
                font_family: legend_style.font_family.clone(),
                align: TextAlign::Left,
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
        let y = 24.0 + title_height + 16.0;

        // 第三步：布局每个 item
        let mut current_x = start_x;

        for i in 0..data.len() {
            let item_width = item_widths[i];
            let content_start_x = current_x + legend_padding;
            let display_text = &display_texts[i];

            let color = if spec
                .series
                .get(i)
                .is_some_and(|s| s.config.chart_type() == ChartType::Candlestick)
            {
                // K 线图用 up_color（红色）可同时代表涨/跌，比 palette 颜色更贴切
                colors.up_color
            } else if use_palette {
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
                text: display_text.clone(),
                position: Point::new(text_x, y),
                style: TextStyle {
                    font_size: legend_style.font_size,
                    color: legend_color,
                    font_family: legend_style.font_family.clone(),
                    align: TextAlign::Left,
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
