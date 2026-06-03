use crate::{pipeline::types::ColorContext, theme::Theme, visual::Color};

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
            Color::new(99, 132, 255),  // #6384FF
            Color::new(255, 159, 67),  // #FF9F43
            Color::new(46, 203, 113),  // #2ECB71
            Color::new(255, 99, 132),  // #FF6384
            Color::new(153, 102, 255), // #9966FF
            Color::new(255, 205, 86),  // #FFCD56
            Color::new(75, 192, 192),  // #4BC0C0
            Color::new(255, 159, 127), // #FF9F7F
        ];

        let series_colors: Vec<Color> = (0..series_count)
            .map(|i| default_palette[i % default_palette.len()])
            .collect();

        ColorContext {
            palette: default_palette.clone(),
            background: Color::new(255, 255, 255),
            series_colors,
            axis_line_color: Color::new(200, 200, 200),
            axis_label_color: Color::new(50, 50, 50),
            grid_line_color: Color::new(230, 230, 230),
            border_color: Color::new(255, 255, 255),
            text_color: Color::new(51, 51, 51),
            text_secondary_color: Color::new(102, 102, 102),
            up_color: Color::new(234, 85, 67),
            down_color: Color::new(80, 170, 94),
            table_header_bg: Color::new(220, 220, 220),
            table_row_even_bg: Color::new(248, 248, 248),
            table_row_odd_bg: Color::new(255, 255, 255),
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
                Color::new(99, 132, 255),  // #6384FF
                Color::new(255, 159, 67),  // #FF9F43
                Color::new(46, 203, 113),  // #2ECB71
                Color::new(255, 99, 132),  // #FF6384
                Color::new(153, 102, 255), // #9966FF
                Color::new(255, 205, 86),  // #FFCD56
                Color::new(75, 192, 192),  // #4BC0C0
                Color::new(255, 159, 127), // #FF9F7F
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
            Color::from_hex(&theme.background_color).unwrap_or(Color::new(255, 255, 255));

        // 从主题读取轴相关颜色
        let axis_line_color =
            Color::from_hex(&theme.axis.axis_line.color).unwrap_or(Color::new(200, 200, 200));

        let axis_label_color =
            Color::from_hex(&theme.axis.axis_label.color).unwrap_or(Color::new(50, 50, 50));

        let grid_line_color =
            Color::from_hex(&theme.axis.split_line.color).unwrap_or(Color::new(230, 230, 230));

        ColorContext {
            palette: palette.clone(),
            background,
            series_colors,
            axis_line_color,
            axis_label_color,
            grid_line_color,
            border_color: Color::new(255, 255, 255),
            text_color: Color::new(51, 51, 51),
            text_secondary_color: Color::new(102, 102, 102),
            up_color: Color::new(234, 85, 67),
            down_color: Color::new(80, 170, 94),
            table_header_bg: Color::new(220, 220, 220),
            table_row_even_bg: Color::new(248, 248, 248),
            table_row_odd_bg: Color::new(255, 255, 255),
        }
    }
}
