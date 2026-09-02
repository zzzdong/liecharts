//! 图例渲染
//!
//! 支持饼图等从 palette 取色的图表类型。

use lievisual::{
    Color,
    scene::{Element, Fill, SceneNode},
    text::{RichSpan, TextAlign, TextBaseline, TextStyle, measure_text},
};
use vello_cpu::kurbo::{Point, Rect};

use crate::{
    pipeline::{
        builder::{ColorExt, Z_TITLE, rect},
        types::{ChartSpec, ChartType, ColorContext, LegendSpec},
    },
    theme::{DEFAULT_FONT_STACK, Theme},
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
    // 相邻图例项的间隔：用户字段 `legend.item_gap`（ECharts 语义）。
    // symbol 与文本之间的间距是独立常量 `LEGEND_SYMBOL_TEXT_GAP`——历史
    // bug 中局部 `item_gap = 8.0` 与字段同名混用，导致字段完全未生效
    //（项间距只靠两侧 legend_padding 隐式形成）。
    let item_gap = legend.item_gap.max(0.0);
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

    for name in &display_texts {
        let text_style = TextStyle::new(
            legend_color,
            legend_style.font_size,
            legend_style.font_family.clone(),
        );
        let mut lv_style = text_style.clone();
        if lv_style.font_family.trim().is_empty()
            || lv_style
                .font_family
                .trim()
                .eq_ignore_ascii_case("sans-serif")
        {
            lv_style.font_family = DEFAULT_FONT_STACK.to_string();
        }
        let text_layout =
            (*measure_text(&[RichSpan::new(name.clone(), lv_style)], None).layout).clone();
        let text_width = text_layout.width;

        let item_width = symbol_size + LEGEND_SYMBOL_TEXT_GAP + text_width + legend_padding * 2.0;
        item_widths.push(item_width);
    }

    // 第二步：按可用宽度分行（超宽溢出裁剪 → 换行，信息零丢失）
    let rows = wrap_legend_rows(&item_widths, width, item_gap);

    // 第三步：逐行整体居中并布局 item
    let row_height = legend_style.font_size * 1.4 + 16.0;
    let y0 = 24.0 + title_height + 16.0;

    for (row_idx, row) in rows.iter().enumerate() {
        // 行总宽计入项间距：k 项有 (k-1) 个 item_gap，用于整行居中
        let row_total: f64 = row.iter().map(|&i| item_widths[i]).sum::<f64>()
            + row.len().saturating_sub(1) as f64 * item_gap;
        let y = y0 + row_idx as f64 * (row_height + LEGEND_ROW_GAP);
        // 整行居中；单个 item 就超出可用宽度时（换行也放不下）退化为左对齐到
        // 安全边距，避免 `start_x < 0` 导致图例被画布左缘裁掉（旧行为）。
        let mut current_x = ((width as f64 - row_total) / 2.0).max(LEGEND_EDGE_MARGIN);

        for &i in row {
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

            // 图例文字 - 使用 Left 对齐，位置在 symbol 右侧。
            // 垂直对齐：显式计算文本 ink_bounds 的视觉中心，将其对齐到 symbol 矩形中心，
            // 而不是依赖 baseline=Middle 的隐式居中（浏览器 SVG 渲染时基线换算
            // 与 parley 度量不一致会产生 1-2px 偏差）。
            let text_x = symbol_x + symbol_size + LEGEND_SYMBOL_TEXT_GAP;
            let mut style = TextStyle::new(
                legend_color,
                legend_style.font_size,
                legend_style.font_family.clone(),
            );
            style.align = TextAlign::Left;
            style.baseline = TextBaseline::Top; // 布局原点语义：ink_bounds 相对此原点
            let mut lv_style = style.clone();
            if lv_style.font_family.trim().is_empty()
                || lv_style
                    .font_family
                    .trim()
                    .eq_ignore_ascii_case("sans-serif")
            {
                lv_style.font_family = DEFAULT_FONT_STACK.to_string();
            }
            let text_layout =
                (*measure_text(&[RichSpan::new(display_text.clone(), lv_style)], None).layout)
                    .clone();
            let ink = text_layout.ink_bounds();
            let ink_center_y = ink.min_y() + (ink.max_y() - ink.min_y()).max(0.0) / 2.0;
            elements.push(
                SceneNode::new(Element::Text {
                    spans: vec![RichSpan::new(display_text.clone(), style.clone())],
                    position: Point::new(text_x, y - ink_center_y),
                    style,
                    layout: Some(std::sync::Arc::new(text_layout)),
                })
                .with_z(Z_TITLE),
            );

            current_x += item_width + item_gap;
        }
    }

    elements
}

/// 图例布局常量（`measure_legend_layout` 与 `render_legend` 共用）
///
/// 行高 = `font_size * 1.4 + 16`（与旧单行估算一致）；行间距 6px；
/// 两侧各留 8px 安全边距。单行且放得下时行为与旧版逐字节一致。
const LEGEND_ROW_GAP: f64 = 6.0;
const LEGEND_EDGE_MARGIN: f64 = 8.0;
/// symbol 与图例文本之间的固定间距（与用户可配的 `LegendSpec::item_gap`
/// ——相邻图例项的间隔——是两个不同语义，勿混用）
const LEGEND_SYMBOL_TEXT_GAP: f64 = 8.0;

/// 按可用宽度把图例项分行（贪心装填：放不下即换行）
///
/// `item_gap` 为相邻项间隔：行内第 2 项起的装入代价是 `item_gap + 宽度`，
/// 与 `render_legend` 的行总宽（Σ宽 + (k-1)×gap）口径一致，保证换行
/// 行数与实际绘制不脱节。
///
/// 返回每行包含的 item 下标。空输入返回单个空行。
fn wrap_legend_rows(item_widths: &[f64], width: u32, item_gap: f64) -> Vec<Vec<usize>> {
    let avail = (width as f64 - 2.0 * LEGEND_EDGE_MARGIN).max(0.0);
    let mut rows: Vec<Vec<usize>> = vec![Vec::new()];
    let mut row_w = 0.0;

    for (i, w) in item_widths.iter().enumerate() {
        let cur = rows.last_mut().expect("rows 非空");
        if cur.is_empty() {
            cur.push(i);
            row_w = *w;
        } else if row_w + item_gap + *w > avail {
            rows.push(vec![i]);
            row_w = *w;
        } else {
            cur.push(i);
            row_w += item_gap + *w;
        }
    }
    rows
}

/// 图例布局度量：行数与总占用高度（供 `estimate_header_height` 预留顶部空间）
pub struct LegendLayout {
    pub rows: usize,
    pub row_height: f64,
    /// rows × row_height + (rows-1) × 行距
    pub total_height: f64,
}

/// 度量图例换行后的实际占用（不产生元素）
///
/// 图例来源与 `render_legend` 一致：显式配置（`show` 决定显隐）或按需自动生成。
pub fn measure_legend_layout(spec: &ChartSpec, width: u32, theme: &Theme) -> Option<LegendLayout> {
    let auto;
    let legend = match &spec.legend {
        Some(l) => {
            if !l.show {
                return None;
            }
            l
        }
        None if should_auto_legend(spec) => {
            auto = LegendSpec {
                show: true,
                data: crate::pipeline::compat::collect_legend_names(&spec.series),
                symbol_size: 10.0,
                item_gap: 10.0,
                formatter: None,
            };
            &auto
        }
        None => return None,
    };

    let legend_style = theme.get_legend_text_style();
    let symbol_size = legend.symbol_size;
    // 与 `render_legend` 同口径：item_gap 为相邻项间隔（用户字段），
    // 符号↔文本间距用 LEGEND_SYMBOL_TEXT_GAP 常量
    let item_gap = legend.item_gap.max(0.0);
    let legend_padding = 16.0;
    let legend_color = Color::from_hex(&legend_style.color).unwrap_or(Color::rgb(50, 50, 50));

    let mut item_widths = Vec::with_capacity(legend.data.len());
    for name in &legend.data {
        let mut lv_style = TextStyle::new(
            legend_color,
            legend_style.font_size,
            legend_style.font_family.clone(),
        );
        if lv_style.font_family.trim().is_empty()
            || lv_style
                .font_family
                .trim()
                .eq_ignore_ascii_case("sans-serif")
        {
            lv_style.font_family = DEFAULT_FONT_STACK.to_string();
        }
        let layout = (*measure_text(&[RichSpan::new(name.clone(), lv_style)], None).layout).clone();
        let w = symbol_size + LEGEND_SYMBOL_TEXT_GAP + layout.width + legend_padding * 2.0;
        item_widths.push(w);
    }
    if item_widths.is_empty() {
        return None;
    }

    let row_height = legend_style.font_size * 1.4 + 16.0;
    let rows = wrap_legend_rows(&item_widths, width, item_gap).len();
    Some(LegendLayout {
        rows,
        row_height,
        total_height: rows as f64 * row_height + (rows.saturating_sub(1)) as f64 * LEGEND_ROW_GAP,
    })
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
    let has_palette_series = spec
        .series
        .iter()
        .any(|s| matches!(s.config, SeriesConfig::Pie(_) | SeriesConfig::PolarBar(_)));
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
