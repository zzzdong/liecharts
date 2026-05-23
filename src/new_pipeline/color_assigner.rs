use crate::new_pipeline::types::ColorContext;
use crate::visual::Color;

/// 颜色分配器
///
/// 职责：
/// - 读取主题调色板或 option.color
/// - 为每个 series 按索引分配固定颜色
/// - 分面场景下同一系列跨分面保持颜色一致
pub struct ColorAssigner;

impl ColorAssigner {
    pub fn new() -> Self {
        Self
    }

    /// 执行颜色分配，输出 ColorContext
    pub fn assign(&self, series_count: usize) -> ColorContext {
        // 使用默认调色板，为每个 series 分配颜色
        let default_palette = vec![
            Color::new(99, 132, 255),   // #6384FF
            Color::new(255, 159, 67),   // #FF9F43
            Color::new(46, 203, 113),   // #2ECB71
            Color::new(255, 99, 132),   // #FF6384
            Color::new(153, 102, 255),  // #9966FF
            Color::new(255, 205, 86),   // #FFCD56
            Color::new(75, 192, 192),   // #4BC0C0
            Color::new(255, 159, 127),  // #FF9F7F
        ];

        let series_colors: Vec<Color> = (0..series_count)
            .map(|i| default_palette[i % default_palette.len()])
            .collect();

        ColorContext {
            palette: default_palette,
            background: Color::new(255, 255, 255),
            series_colors,
            axis_line_color: Color::new(200, 200, 200),
            axis_label_color: Color::new(50, 50, 50),
            grid_line_color: Color::new(230, 230, 230),
        }
    }
}