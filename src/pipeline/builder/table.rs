//! Table Builder: 将 TableSeries 组装为 VisualElement

use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, fill_style},
        typed_series::{RenderContext, TableSeries},
    },
    visual::{TextStyle, VisualElement},
};

pub struct TableBuilder;

impl SeriesBuilder<TableSeries> for TableBuilder {
    fn build(series: &TableSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::new();

        // 表格布局：使用子图 bounds 定位，动态计算单元格尺寸
        let start_x = ctx.bounds.x0;
        let start_y = ctx.bounds.y0;
        let total_width = ctx.bounds.width();
        let total_height = ctx.bounds.height();

        let col_count = series.headers.len();
        let row_count = series.rows.len();
        let cell_width = total_width / col_count as f64;
        let cell_height = total_height / (row_count + 1) as f64;

        // 1. 绘制表头背景
        let header_rect = Rect::new(
            start_x,
            start_y,
            start_x + cell_width * col_count as f64,
            start_y + cell_height,
        );
        elements.push(VisualElement::Rect {
            rect: header_rect,
            style: fill_style(series.header_bg),
            z_index: Z_SERIES_FILL,
        });

        // 2. 绘制表头文本（居中对齐）
        for (col_idx, header) in series.headers.iter().enumerate() {
            let x = start_x + col_idx as f64 * cell_width + cell_width / 2.0;
            let y = start_y + cell_height / 2.0;

            elements.push(VisualElement::TextRun {
                text: header.clone(),
                position: Point::new(x, y),
                style: TextStyle {
                    font_size: 12.0,
                    color: ctx.colors.text_color,
                    font_family: "Arial".to_string(),
                    font_weight: crate::option::FontWeight::Named(
                        crate::option::FontWeightNamed::Bold,
                    ),
                    font_style: crate::visual::FontStyle::Normal,
                    align: crate::visual::TextAlign::Center,
                    vertical_align: crate::visual::TextBaseline::Middle,
                },
                rotation: 0.0,
                max_width: Some(cell_width - 10.0),
                layout: None,
                z_index: Z_SERIES_FILL + 1,
            });
        }

        // 3. 绘制表头顶部和底部分隔线
        let header_bottom_y = start_y + cell_height;

        // 顶部边框
        elements.push(VisualElement::Line {
            start: Point::new(start_x, start_y),
            end: Point::new(start_x + cell_width * col_count as f64, start_y),
            style: crate::pipeline::builder::stroke_style(series.header_border_color, 1.0),
            z_index: Z_SERIES_FILL + 2,
        });

        // 底部边框
        elements.push(VisualElement::Line {
            start: Point::new(start_x, header_bottom_y),
            end: Point::new(start_x + cell_width * col_count as f64, header_bottom_y),
            style: crate::pipeline::builder::stroke_style(series.header_border_color, 1.0),
            z_index: Z_SERIES_FILL + 2,
        });

        // 4. 绘制表头区域纵向分隔线（使用可见颜色，与灰色背景区分）
        for col_idx in 0..=col_count {
            let x = start_x + col_idx as f64 * cell_width;
            elements.push(VisualElement::Line {
                start: Point::new(x, start_y),
                end: Point::new(x, header_bottom_y),
                style: crate::pipeline::builder::stroke_style(series.header_border_color, 1.0),
                z_index: Z_SERIES_FILL + 2,
            });
        }

        // 5. 绘制行
        for (row_idx, row) in series.rows.iter().enumerate() {
            let y = start_y + (row_idx + 1) as f64 * cell_height;
            let bg_color = if row_idx % 2 == 0 {
                series.row_even_bg
            } else {
                series.row_odd_bg
            };

            // 行背景
            let row_rect = Rect::new(
                start_x,
                y,
                start_x + cell_width * col_count as f64,
                y + cell_height,
            );
            elements.push(VisualElement::Rect {
                rect: row_rect,
                style: fill_style(bg_color),
                z_index: Z_SERIES_FILL,
            });

            // 行文本（居中对齐）
            for (col_idx, cell) in row.iter().enumerate() {
                let x = start_x + col_idx as f64 * cell_width + cell_width / 2.0;
                let text_y = y + cell_height / 2.0;

                elements.push(VisualElement::TextRun {
                    text: cell.clone(),
                    position: Point::new(x, text_y),
                    style: TextStyle {
                        font_size: 11.0,
                        color: ctx.colors.text_color,
                        font_family: "Arial".to_string(),
                        font_weight: crate::option::FontWeight::Named(
                            crate::option::FontWeightNamed::Normal,
                        ),
                        font_style: crate::visual::FontStyle::Normal,
                        align: crate::visual::TextAlign::Center,
                        vertical_align: crate::visual::TextBaseline::Middle,
                    },
                    rotation: 0.0,
                    max_width: Some(cell_width - 10.0),
                    layout: None,
                    z_index: Z_SERIES_FILL + 1,
                });
            }
        }

        // 6. 绘制网格线（仅数据区域，表头区域已在步骤4绘制）
        let grid_color = ctx.colors.grid_line_color;

        // 垂直线（从表头底部开始）
        for col_idx in 0..=col_count {
            let x = start_x + col_idx as f64 * cell_width;
            elements.push(VisualElement::Line {
                start: Point::new(x, header_bottom_y),
                end: Point::new(x, start_y + (row_count + 1) as f64 * cell_height),
                style: crate::pipeline::builder::stroke_style(grid_color, 1.0),
                z_index: Z_SERIES_FILL + 2,
            });
        }

        // 7. 水平线（从第三行开始，顶部和表头底部边框已在步骤3绘制）
        for row_idx in 2..=row_count + 1 {
            let y = start_y + row_idx as f64 * cell_height;
            elements.push(VisualElement::Line {
                start: Point::new(start_x, y),
                end: Point::new(start_x + cell_width * col_count as f64, y),
                style: crate::pipeline::builder::stroke_style(grid_color, 1.0),
                z_index: Z_SERIES_FILL + 2,
            });
        }

        Ok(elements)
    }
}
