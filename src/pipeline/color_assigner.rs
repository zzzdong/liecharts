use lievisual::Color;

use crate::{
    pipeline::{builder::ColorExt, types::ColorContext},
    theme::Theme,
};

/// 颜色分配器
///
/// 职责：
/// - 读取主题调色板或 option.color
/// - 为每个 series 按索引分配固定颜色
/// - 分面场景下同一系列跨分面保持颜色一致
pub struct ColorAssigner;

impl Default for ColorAssigner {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorAssigner {
    pub fn new() -> Self {
        Self
    }

    /// 执行颜色分配，输出 ColorContext
    pub fn assign(&self, series_count: usize) -> ColorContext {
        // 使用默认调色板，为每个 series 分配颜色
        let default_palette = vec![
            Color::rgb(99, 132, 255),  // #6384FF
            Color::rgb(255, 159, 67),  // #FF9F43
            Color::rgb(46, 203, 113),  // #2ECB71
            Color::rgb(255, 99, 132),  // #FF6384
            Color::rgb(153, 102, 255), // #9966FF
            Color::rgb(255, 205, 86),  // #FFCD56
            Color::rgb(75, 192, 192),  // #4BC0C0
            Color::rgb(255, 159, 127), // #FF9F7F
        ];

        let series_colors: Vec<Color> = (0..series_count)
            .map(|i| default_palette[i % default_palette.len()])
            .collect();

        ColorContext {
            palette: default_palette.clone(),
            series_colors,
            ..ColorContext::default()
        }
    }

    /// 使用主题执行颜色分配
    pub fn assign_with_theme(&self, series_count: usize, theme: &Theme) -> ColorContext {
        // 从主题读取调色板
        let palette: Vec<Color> = theme
            .color
            .iter()
            .filter_map(|hex| Color::from_hex(hex))
            .collect();

        // 如果主题调色板为空，使用默认调色板
        let palette = if palette.is_empty() {
            vec![
                Color::rgb(99, 132, 255),  // #6384FF
                Color::rgb(255, 159, 67),  // #FF9F43
                Color::rgb(46, 203, 113),  // #2ECB71
                Color::rgb(255, 99, 132),  // #FF6384
                Color::rgb(153, 102, 255), // #9966FF
                Color::rgb(255, 205, 86),  // #FFCD56
                Color::rgb(75, 192, 192),  // #4BC0C0
                Color::rgb(255, 159, 127), // #FF9F7F
            ]
        } else {
            palette
        };

        // 为每个 series 分配颜色
        let series_colors: Vec<Color> = (0..series_count)
            .map(|i| palette[i % palette.len()])
            .collect();

        // 从主题读取背景色
        let background =
            Color::from_hex(&theme.background_color).unwrap_or(Color::rgb(255, 255, 255));

        // 从主题读取轴相关颜色
        let axis_line_color =
            Color::from_hex(&theme.axis.axis_line.color).unwrap_or(Color::rgb(200, 200, 200));

        let axis_label_color =
            Color::from_hex(&theme.axis.axis_label.color).unwrap_or(Color::rgb(50, 50, 50));

        let grid_line_color =
            Color::from_hex(&theme.axis.split_line.color).unwrap_or(Color::rgb(230, 230, 230));

        ColorContext {
            palette: palette.clone(),
            background,
            series_colors,
            axis_line_color,
            axis_label_color,
            grid_line_color,
            // 基准字号（1rem）：主题 rem 根。用户显式根已在
            // `build_chart_with_theme` 折算进主题令牌，此处直接读取即可。
            body_font_size: theme.tokens().text.body_size,
            // 辅助文字（0.833rem）：标注线/点等辅助文本兜底
            caption_font_size: theme.tokens().text.caption_size,
            // 其余固定色（文字/表格/K 线涨跌）继承 `ColorContext::default()`
            ..ColorContext::default()
        }
    }
}
