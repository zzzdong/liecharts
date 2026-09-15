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
        builder::{ColorExt, Z_TITLE, circle, rect},
        types::{ChartSpec, ChartType, ColorContext, LegendOrient, LegendSpec},
    },
    theme::{DEFAULT_FONT_STACK, Theme},
};

/// 构建图例元素
///
/// 图例默认位于**画布底部居中**（ECharts v6 `legend` 默认：`left:'center'`、
/// `bottom: tokens.size.m`），支持 palette 取色和系列取色两种模式。
pub fn render_legend(
    spec: &ChartSpec,
    width: u32,
    height: u32,
    colors: &ColorContext,
    theme: &Theme,
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
            item_width: DEFAULT_LEGEND_ITEM_WIDTH,
            item_height: DEFAULT_LEGEND_ITEM_HEIGHT,
            symbol_size: None,
            item_gap: DEFAULT_LEGEND_ITEM_GAP,
            formatter: None,
            orient: LegendOrient::Horizontal,
            left: None,
            right: None,
            top: None,
            bottom: None,
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
    // 符号框：`itemWidth` / `itemHeight`（ECharts v6 默认 25×14）；
    // 显式 `symbolSize` 时覆盖为正方形（兼容旧字段语义）
    let (icon_w, icon_h) = match legend.symbol_size {
        Some(s) => (s.max(1.0), s.max(1.0)),
        None => (legend.item_width.max(1.0), legend.item_height.max(1.0)),
    };
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

    // 图例项「名称 → 颜色 / 符号形状」对照表（与系列/数据点实际取色同源）
    let entries = legend_entries(spec, colors);

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

        let item_width = icon_w + LEGEND_SYMBOL_TEXT_GAP + text_width + legend_padding * 2.0;
        item_widths.push(item_width);
    }

    // 第二步：分行。`orient: vertical` 每项独占一行（单列）；水平方向上按可用宽度
    // 贪心换行（超宽溢出裁剪 → 换行，信息零丢失）。
    let vertical = legend.orient == LegendOrient::Vertical;
    let rows: Vec<Vec<usize>> = if vertical {
        (0..item_widths.len()).map(|i| vec![i]).collect()
    } else {
        wrap_legend_rows(&item_widths, width, item_gap)
    };
    // 垂直排列时整块宽度取最长一项，用于水平锚点（保证各项左缘对齐）
    let block_width = rows
        .iter()
        .map(|row| row_total_width(row, &item_widths, item_gap))
        .fold(0.0_f64, f64::max);

    // 第三步：逐行布局 item。整块位置：水平默认居中（v6 `left:'center'`）、
    // 垂直默认贴画布底部（v6 `bottom: tokens.size.m`），`left/right/top/bottom`
    // 显式配置时按配置定位。
    let row_height = legend_style.font_size * 1.4 + 16.0;
    let total_height =
        rows.len() as f64 * row_height + rows.len().saturating_sub(1) as f64 * LEGEND_ROW_GAP;
    let block_top = resolve_block_top(
        legend.top.as_deref(),
        legend.bottom.as_deref(),
        total_height,
        height as f64,
    );
    let h_anchor = resolve_h_block_anchor(
        legend.left.as_deref(),
        legend.right.as_deref(),
        width as f64,
    );

    for (row_idx, row) in rows.iter().enumerate() {
        // 行总宽计入项间距：k 项有 (k-1) 个 item_gap，用于整行定位
        let row_total = row_total_width(row, &item_widths, item_gap);
        let y = block_top + row_height / 2.0 + row_idx as f64 * (row_height + LEGEND_ROW_GAP);
        // 整行按锚点定位（默认居中）；垂直排列时各行按整块宽度对齐（左缘一致）。
        // 单个 item 就超出可用宽度时（换行也放不下）退化为左对齐到安全边距，
        // 避免 `start_x < 0` 导致图例被画布左缘裁掉。
        let anchor_total = if vertical { block_width } else { row_total };
        let mut current_x = h_anchor
            .row_start(anchor_total, width as f64)
            .max(LEGEND_EDGE_MARGIN);

        for &i in row {
            let item_width = item_widths[i];
            let content_start_x = current_x + legend_padding;
            let display_text = &display_texts[i];

            // 取色与形状按**名称**回查（`legend.data` 重排/取子集后仍与系列一致）；
            // 名称匹配不到任何系列时（例如图例名写错）退回按下标取色的旧口径。
            let entry = entries.iter().find(|e| e.name == data[i]);
            let color = entry.map(|e| e.color).unwrap_or_else(|| {
                if spec
                    .series
                    .get(i)
                    .is_some_and(|s| s.config.chart_type() == ChartType::Candlestick)
                {
                    // K 线图用 up_color（红色）可同时代表涨/跌，比 palette 颜色更贴切
                    colors.up_color
                } else if use_palette {
                    colors.get_data_color(i)
                } else {
                    colors.get_series_color(i)
                }
            });
            let kind = entry.map(|e| e.kind).unwrap_or(LegendSymbolKind::Rect);

            // 图例符号：以 y 为中心绘制 `itemWidth × itemHeight` 的符号框
            let symbol_x = content_start_x;
            let fill_color = lievisual::scene::FillStrokeStyle {
                fill: Some(Fill::Solid(color)),
                stroke: None,
            };
            match kind {
                LegendSymbolKind::Rect => elements.push(rect(
                    Rect::new(
                        symbol_x,
                        y - icon_h / 2.0,
                        symbol_x + icon_w,
                        y + icon_h / 2.0,
                    ),
                    fill_color,
                    Z_TITLE,
                )),
                LegendSymbolKind::Circle => {
                    // 饼图/散点：圆（直径取符号框短边）
                    let d = icon_w.min(icon_h);
                    elements.push(circle(
                        Point::new(symbol_x + icon_w / 2.0, y),
                        d / 2.0,
                        fill_color,
                        Z_TITLE,
                    ));
                }
                LegendSymbolKind::Line => {
                    // 折线：横线段 + 中点标记（ECharts 线图图例的默认形态）
                    let lw = 2.5_f64.min(icon_h);
                    elements.push(rect(
                        Rect::new(symbol_x, y - lw / 2.0, symbol_x + icon_w, y + lw / 2.0),
                        fill_color.clone(),
                        Z_TITLE,
                    ));
                    let r = (icon_h.min(8.0) / 2.0).max(1.0);
                    elements.push(circle(
                        Point::new(symbol_x + icon_w / 2.0, y),
                        r,
                        fill_color,
                        Z_TITLE,
                    ));
                }
                LegendSymbolKind::Candle => {
                    // K 线：竖线 + 中部实体
                    let lw = 1.5_f64.min(icon_w);
                    let cx = symbol_x + icon_w / 2.0;
                    elements.push(rect(
                        Rect::new(
                            cx - lw / 2.0,
                            y - icon_h / 2.0,
                            cx + lw / 2.0,
                            y + icon_h / 2.0,
                        ),
                        fill_color.clone(),
                        Z_TITLE,
                    ));
                    let body_w = (icon_w * 0.5).max(lw);
                    let body_h = (icon_h * 0.5).max(2.0);
                    elements.push(rect(
                        Rect::new(
                            cx - body_w / 2.0,
                            y - body_h / 2.0,
                            cx + body_w / 2.0,
                            y + body_h / 2.0,
                        ),
                        fill_color,
                        Z_TITLE,
                    ));
                }
            }

            // 图例文字 - 使用 Left 对齐，位置在 symbol 右侧。
            // 垂直对齐：显式计算文本 ink_bounds 的视觉中心，将其对齐到 symbol 矩形中心，
            // 而不是依赖 baseline=Middle 的隐式居中（浏览器 SVG 渲染时基线换算
            // 与 parley 度量不一致会产生 1-2px 偏差）。
            let text_x = symbol_x + icon_w + LEGEND_SYMBOL_TEXT_GAP;
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

/// 一行的总宽（Σ 项宽 + (k-1) × 项间距）
fn row_total_width(row: &[usize], item_widths: &[f64], item_gap: f64) -> f64 {
    row.iter().map(|&i| item_widths[i]).sum::<f64>() + row.len().saturating_sub(1) as f64 * item_gap
}

/// 图例符号形状（按所标注的系列类型决定，ECharts 同源行为）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LegendSymbolKind {
    /// 折线/面积：横线段 + 中点标记
    Line,
    /// 柱状/热力/其它：矩形
    Rect,
    /// 饼图/散点/极坐标散点：圆
    Circle,
    /// K 线：竖线 + 实体
    Candle,
}

/// 图例项：名称 → 取色与符号形状
struct LegendEntry {
    name: String,
    color: Color,
    kind: LegendSymbolKind,
}

/// 图例项对照表（名称 → 颜色 / 符号形状），与系列/数据点的**实际**取色口径一致。
///
/// 历史 bug：图例按**项下标**取色，`legend.data` 重排或只列子集（例如只显示
/// 其中两个系列）后，色块与它标注的系列/数据点错位。这里按名称回查：
/// - 按数据点着色的系列（饼图/极坐标柱）：数据点名 → `get_data_color(行号)`
/// - 其余系列：系列名 → `series_colors[系列下标]`（K 线用涨色）
fn legend_entries(spec: &ChartSpec, colors: &ColorContext) -> Vec<LegendEntry> {
    use crate::pipeline::types::SeriesConfig;

    let mut table: Vec<LegendEntry> = Vec::new();

    for (si, series) in spec.series.iter().enumerate() {
        let kind = symbol_kind_of(series.config.chart_type());
        match &series.config {
            SeriesConfig::Pie(cfg) => {
                push_data_point_entries(&mut table, series, &cfg.category_col, colors, kind);
            }
            SeriesConfig::PolarBar(cfg) => {
                push_data_point_entries(&mut table, series, &cfg.angle_col, colors, kind);
            }
            _ => {
                let color = if kind == LegendSymbolKind::Candle {
                    colors.up_color
                } else {
                    colors
                        .series_colors
                        .get(si)
                        .copied()
                        .unwrap_or_else(|| colors.get_series_color(si))
                };
                push_unique_entry(&mut table, series.name.clone(), color, kind);
            }
        }
    }

    table
}

/// 系列类型 → 图例符号形状
fn symbol_kind_of(chart_type: ChartType) -> LegendSymbolKind {
    match chart_type {
        ChartType::Line => LegendSymbolKind::Line,
        ChartType::Pie | ChartType::Scatter | ChartType::Bubble | ChartType::PolarScatter => {
            LegendSymbolKind::Circle
        }
        ChartType::PolarBar => LegendSymbolKind::Circle,
        ChartType::Candlestick => LegendSymbolKind::Candle,
        _ => LegendSymbolKind::Rect,
    }
}

/// 逐个数据点登记「名称 → `palette[行号]`」（与 [`ColorContext::get_data_color`] 同源）
fn push_data_point_entries(
    table: &mut Vec<LegendEntry>,
    series: &crate::pipeline::types::SeriesSpec,
    name_col: &str,
    colors: &ColorContext,
    kind: LegendSymbolKind,
) {
    let Some(col) = series.data.get_column(name_col) else {
        return;
    };
    for i in 0..series.data.row_count() {
        if let Some(name) = col.as_string(i) {
            push_unique_entry(table, name, colors.get_data_color(i), kind);
        }
    }
}

/// 首次出现的名称生效（与 ECharts 一致：图例名匹配到第一个同名系列）
fn push_unique_entry(
    table: &mut Vec<LegendEntry>,
    name: String,
    color: Color,
    kind: LegendSymbolKind,
) {
    if !name.is_empty() && !table.iter().any(|e| e.name == name) {
        table.push(LegendEntry { name, color, kind });
    }
}

/// 解析 ECharts 位置字面量：`"20%"` 相对 `total`，`"20"` / `"20px"` / `20` 为像素。
fn parse_len(s: &str, total: f64) -> Option<f64> {
    let s = s.trim();
    if let Some(pct) = s.strip_suffix('%') {
        pct.trim().parse::<f64>().ok().map(|p| total * p / 100.0)
    } else {
        // `"50px"` 也接受（ECharts 宽松数值写法）
        s.strip_suffix("px").unwrap_or(s).trim().parse::<f64>().ok()
    }
}

/// 图例块的水平锚点
enum HBlockAnchor {
    /// 距画布左侧 `d` 像素
    Left(f64),
    /// 水平居中（ECharts 默认 `left:'center'`）
    Center,
    /// 距画布右侧 `d` 像素
    Right(f64),
}

impl HBlockAnchor {
    /// 一行的起始 x
    fn row_start(&self, row_total: f64, width: f64) -> f64 {
        match *self {
            HBlockAnchor::Left(d) => d,
            HBlockAnchor::Center => (width - row_total) / 2.0,
            HBlockAnchor::Right(d) => width - d - row_total,
        }
    }
}

/// 位置字面量中 `"auto"` / 空串视为"未指定"。
fn specified_pos(v: Option<&str>) -> Option<&str> {
    v.map(str::trim)
        .filter(|s| !s.eq_ignore_ascii_case("auto") && !s.is_empty())
}

/// 解析 `legend.left` / `legend.right`（`auto` 视为未指定）。
fn resolve_h_block_anchor(left: Option<&str>, right: Option<&str>, width: f64) -> HBlockAnchor {
    if let Some(l) = specified_pos(left) {
        let key = l.to_ascii_lowercase();
        return match key.as_str() {
            "center" | "middle" => HBlockAnchor::Center,
            "left" => HBlockAnchor::Left(0.0),
            "right" => HBlockAnchor::Right(0.0),
            other => parse_len(other, width)
                .map(HBlockAnchor::Left)
                .unwrap_or(HBlockAnchor::Center),
        };
    }
    if let Some(r) = specified_pos(right) {
        let key = r.to_ascii_lowercase();
        return match key.as_str() {
            "center" | "middle" => HBlockAnchor::Center,
            "right" => HBlockAnchor::Right(0.0),
            "left" => HBlockAnchor::Left(0.0),
            other => parse_len(other, width)
                .map(HBlockAnchor::Right)
                .unwrap_or(HBlockAnchor::Center),
        };
    }
    HBlockAnchor::Center
}

/// 图例块的垂直落点区域（决定它占用头部还是底部预留空间）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendVerticalZone {
    /// 顶部：占用头部空间（`grid.top` 需为之让位）
    Top,
    /// 中部：显式配置在画布中部，允许与绘图区重叠，不做预留
    Middle,
    /// 底部（ECharts v6 默认）：占用底部空间
    Bottom,
}

/// 由 `legend.top` / `legend.bottom` 解析图例块的垂直落点区域。
pub fn legend_vertical_zone(legend: &LegendSpec) -> LegendVerticalZone {
    if let Some(t) = specified_pos(legend.top.as_deref()) {
        return match t.to_ascii_lowercase().as_str() {
            "middle" | "center" => LegendVerticalZone::Middle,
            "bottom" => LegendVerticalZone::Bottom,
            // `top` 预设 / 像素 / 百分比：都从顶部起算
            _ => LegendVerticalZone::Top,
        };
    }
    if let Some(b) = specified_pos(legend.bottom.as_deref()) {
        return match b.to_ascii_lowercase().as_str() {
            "top" => LegendVerticalZone::Top,
            "middle" | "center" => LegendVerticalZone::Middle,
            _ => LegendVerticalZone::Bottom,
        };
    }
    LegendVerticalZone::Bottom
}

/// 图例块顶部 y（与 [`render_legend`] 同源，供布局阶段预留空间）。
pub fn legend_block_top(legend: &LegendSpec, block_height: f64, canvas_height: f64) -> f64 {
    resolve_block_top(
        legend.top.as_deref(),
        legend.bottom.as_deref(),
        block_height,
        canvas_height,
    )
}

/// 解析图例块顶部 y。
///
/// 未指定 `top` / `bottom` 时贴画布底部（ECharts v6 默认 `bottom: tokens.size.m`）。
fn resolve_block_top(top: Option<&str>, bottom: Option<&str>, block_h: f64, height: f64) -> f64 {
    let len = |v: &str| parse_len(v, height).unwrap_or(LEGEND_BOTTOM_MARGIN);

    if let Some(t) = specified_pos(top) {
        return match t.to_ascii_lowercase().as_str() {
            "top" => 0.0,
            "middle" | "center" => (height - block_h) / 2.0,
            "bottom" => (height - block_h).max(0.0),
            other => len(other),
        };
    }
    if let Some(b) = specified_pos(bottom) {
        return match b.to_ascii_lowercase().as_str() {
            "bottom" => (height - block_h).max(0.0),
            "middle" | "center" => (height - block_h) / 2.0,
            "top" => 0.0,
            other => (height - len(other) - block_h).max(0.0),
        };
    }
    (height - LEGEND_BOTTOM_MARGIN - block_h).max(0.0)
}

/// 图例布局常量（`measure_legend_layout` 与 `render_legend` 共用）
///
/// 行高 = `font_size * 1.4 + 16`（与旧单行估算一致）；行间距 6px；
/// 两侧各留 8px 安全边距。单行且放得下时行为与旧版逐字节一致。
const LEGEND_ROW_GAP: f64 = 6.0;
const LEGEND_EDGE_MARGIN: f64 = 8.0;
/// 图例整体底边距画布底部的距离（ECharts v6 `legend.bottom` = `tokens.size.m`）
pub const LEGEND_BOTTOM_MARGIN: f64 = 15.0;
/// 相邻图例项默认间隔（ECharts v6 `legend.itemGap` 默认 8）
pub const DEFAULT_LEGEND_ITEM_GAP: f64 = 8.0;
/// 图例符号框默认宽高（ECharts v6 `legend.itemWidth` / `itemHeight` 默认 25 × 14）
pub const DEFAULT_LEGEND_ITEM_WIDTH: f64 = 25.0;
pub const DEFAULT_LEGEND_ITEM_HEIGHT: f64 = 14.0;
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
                item_width: DEFAULT_LEGEND_ITEM_WIDTH,
                item_height: DEFAULT_LEGEND_ITEM_HEIGHT,
                symbol_size: None,
                item_gap: DEFAULT_LEGEND_ITEM_GAP,
                formatter: None,
                orient: LegendOrient::Horizontal,
                left: None,
                right: None,
                top: None,
                bottom: None,
            };
            &auto
        }
        None => return None,
    };

    let legend_style = theme.get_legend_text_style();
    // 与 `render_legend` 同口径：符号框 = itemWidth/itemHeight（或 symbolSize 覆盖），
    // item_gap 为相邻项间隔（用户字段），符号↔文本间距用 LEGEND_SYMBOL_TEXT_GAP 常量
    let icon_w = match legend.symbol_size {
        Some(s) => s.max(1.0),
        None => legend.item_width.max(1.0),
    };
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
        let w = icon_w + LEGEND_SYMBOL_TEXT_GAP + layout.width + legend_padding * 2.0;
        item_widths.push(w);
    }
    if item_widths.is_empty() {
        return None;
    }

    let row_height = legend_style.font_size * 1.4 + 16.0;
    let rows = if legend.orient == LegendOrient::Vertical {
        // 垂直：每项独占一行（单列），高度随项数线性增长
        item_widths.len()
    } else {
        wrap_legend_rows(&item_widths, width, item_gap).len()
    };
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
