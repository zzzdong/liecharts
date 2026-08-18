//! 图例渲染
//!
//! 支持饼图等从 palette 取色的图表类型。

use lievisual::scene::{Fill, SceneNode};
use lievisual::text::{TextAlign, TextBaseline, TextStyle};
use vello_cpu::kurbo::{Point, Rect};

use crate::{
    pipeline::{
        builder::{rect, text_el},
        types::{ChartSpec, ChartType, ColorContext, LegendSpec},
    },
    text::create_text_layout,
    theme::Theme,
};
use lievisual::Color;
use crate::pipeline::builder::{ColorExt, Z_TITLE};

/// 构建图例元素
///
/// 根据标题高度动态计算图例位置，支持 palette 取色和系列取色两种模式。
pub fn render_legend(
    spec: &ChartSpec,
    width: u32,
    colors: &ColorContext,
    theme: &Theme,
    title_height: f64,
) -> Vec<SceneNode> {
    let mut elements = Vec::new();

    // 图例来源：
    // - 用户显式配置了 legend → 遵循用户配置（show 决定显隐）
    // - 未配置但图表需要颜色区分 → 按需自动绘制图例
    let auto_legend = match &spec.legend {
        Some(legend) => {
            if !legend.show {
                return elements;
            }
            None
        }
        None if should_auto_legend(spec) => Some(LegendSpec {
            show: true,
            data: crate::pipeline::compat::collect_legend_names(&spec.series),
            symbol_size: 10.0,
            item_gap: 10.0,
            formatter: None,
        }),
        None => return elements,
    };

    let legend = match (&spec.legend, &auto_legend) {
        (Some(l), _) => l,
        (None, Some(a)) => a,
        _ => return elements,
    };

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
        let text_style = TextStyle::new(legend_color, legend_style.font_size, legend_style.font_family.clone());
        let text_layout = create_text_layout(name, &text_style, None);
        let text_width = text_layout.width;

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
        elements.push(rect(
            Rect::new(
                symbol_x,
                y - symbol_size / 2.0,
                symbol_x + symbol_size,
                y + symbol_size / 2.0,
            ),
            lievisual::scene::FillStrokeStyle {
                fill: Some(Fill::Solid(color)),
                stroke: None,
            },
            Z_TITLE,
        ));

        // 图例文字 - 使用 Left 对齐，位置在 symbol 右侧
        let text_x = symbol_x + symbol_size + item_gap;
        let mut style = TextStyle::new(legend_color, legend_style.font_size, legend_style.font_family.clone());
        style.align = TextAlign::Left;
        style.baseline = TextBaseline::Middle;
        elements.push(text_el(
            display_text.clone(),
            Point::new(text_x, y),
            style,
            Z_TITLE,
        ));

        current_x += item_width;
    }

    elements
}

/// 判断是否应为图表自动绘制图例（当用户未显式配置 legend 时）。
///
/// 规则：
/// - 按数据点着色的类型（饼图/环形图/极坐标柱状图）→ 需要（每个数据点一色）
/// - 多系列图表（line/bar/scatter 等，且非热力图/仪表盘/表格）→ 需要
/// - 其余（单系列、热力图、仪表盘、表格）→ 不需要
fn should_auto_legend(spec: &ChartSpec) -> bool {
    use crate::pipeline::types::SeriesConfig;

    // 饼图 / 环形图 / 极坐标柱状图：按数据点着色，必须有图例
    let has_palette_series = spec.series.iter().any(|s| {
        matches!(
            s.config,
            SeriesConfig::Pie(_) | SeriesConfig::PolarBar(_)
        )
    });
    if has_palette_series {
        return true;
    }

    // 多系列（>1）且非热力图/仪表盘/表格：需要颜色区分
    if spec.series.len() > 1 {
        let has_color_exempt = spec.series.iter().any(|s| {
            matches!(
                s.config,
                SeriesConfig::Heatmap(_) | SeriesConfig::Gauge(_) | SeriesConfig::Table(_)
            )
        });
        return !has_color_exempt;
    }

    false
}
