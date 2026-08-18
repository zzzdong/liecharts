//! Table Builder: 将 TableSeries 组装为 lievisual `SceneNode`

use lievisual::scene::{Element, SceneNode};
use lievisual::text::{RichSpan, TextAlign, TextBaseline, TextStyle};
use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        builder::{SeriesBuilder, Z_SERIES_FILL, fill_style, line, rect, stroke_style},
        typed_series::{RenderContext, TableSeries},
    },
    option::{FontWeight, FontWeightNamed},
};

fn weight_to_f32(w: &FontWeight) -> f32 {
    match w {
        FontWeight::Named(n) => match n {
            FontWeightNamed::Bold => 700.0,
            FontWeightNamed::Bolder => 700.0,
            FontWeightNamed::Lighter => 300.0,
            FontWeightNamed::Normal => 400.0,
        },
        FontWeight::Numeric(n) => *n as f32,
    }
}

pub struct TableBuilder;

impl SeriesBuilder<TableSeries> for TableBuilder {
    fn build(series: &TableSeries, ctx: &RenderContext) -> Result<Vec<SceneNode>> {
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
        elements.push(rect(header_rect, fill_style(series.header_bg), Z_SERIES_FILL));

        // 2. 绘制表头文本（居中对齐）
        for (col_idx, header) in series.headers.iter().enumerate() {
            let x = start_x + col_idx as f64 * cell_width + cell_width / 2.0;
            let y = start_y + cell_height / 2.0;

            let mut style = TextStyle::new(ctx.colors.text_color, 12.0, "Arial");
            style.font_weight = weight_to_f32(&FontWeight::Named(FontWeightNamed::Bold));
            style.align = TextAlign::Center;
            style.baseline = TextBaseline::Middle;
            style.max_width = Some(cell_width - 10.0);
            elements.push(
                SceneNode::new(Element::Text {
                    spans: vec![RichSpan::new(header.clone(), style.clone())],
                    position: Point::new(x, y),
                    style,
                    layout: None,
                })
                .with_z(Z_SERIES_FILL + 1),
            );
        }

        // 3. 绘制表头顶部和底部分隔线
        let header_bottom_y = start_y + cell_height;

        // 顶部边框
        elements.push(line(
            Point::new(start_x, start_y),
            Point::new(start_x + cell_width * col_count as f64, start_y),
            stroke_style(series.header_border_color, 1.0),
            Z_SERIES_FILL + 2,
        ));

        // 底部边框
        elements.push(line(
            Point::new(start_x, header_bottom_y),
            Point::new(start_x + cell_width * col_count as f64, header_bottom_y),
            stroke_style(series.header_border_color, 1.0),
            Z_SERIES_FILL + 2,
        ));

        // 4. 绘制表头区域纵向分隔线（使用可见颜色，与灰色背景区分）
        for col_idx in 0..=col_count {
            let x = start_x + col_idx as f64 * cell_width;
            elements.push(line(
                Point::new(x, start_y),
                Point::new(x, header_bottom_y),
                stroke_style(series.header_border_color, 1.0),
                Z_SERIES_FILL + 2,
            ));
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
            elements.push(rect(row_rect, fill_style(bg_color), Z_SERIES_FILL));

            // 行文本（居中对齐）
            for (col_idx, cell) in row.iter().enumerate() {
                let x = start_x + col_idx as f64 * cell_width + cell_width / 2.0;
                let text_y = y + cell_height / 2.0;

                let mut style = TextStyle::new(ctx.colors.text_color, 11.0, "Arial");
                style.font_weight = weight_to_f32(&FontWeight::Named(FontWeightNamed::Normal));
                style.align = TextAlign::Center;
                style.baseline = TextBaseline::Middle;
                style.max_width = Some(cell_width - 10.0);
                elements.push(
                    SceneNode::new(Element::Text {
                        spans: vec![RichSpan::new(cell.clone(), style.clone())],
                        position: Point::new(x, text_y),
                        style,
                        layout: None,
                    })
                    .with_z(Z_SERIES_FILL + 1),
                );
            }
        }

        // 6. 绘制网格线（仅数据区域，表头区域已在步骤4绘制）
        let grid_color = ctx.colors.grid_line_color;

        // 垂直线（从表头底部开始）
        for col_idx in 0..=col_count {
            let x = start_x + col_idx as f64 * cell_width;
            elements.push(line(
                Point::new(x, header_bottom_y),
                Point::new(x, start_y + (row_count + 1) as f64 * cell_height),
                stroke_style(grid_color, 1.0),
                Z_SERIES_FILL + 2,
            ));
        }

        // 7. 水平线（从第三行开始，顶部和表头底部边框已在步骤3绘制）
        for row_idx in 2..=row_count + 1 {
            let y = start_y + row_idx as f64 * cell_height;
            elements.push(line(
                Point::new(start_x, y),
                Point::new(start_x + cell_width * col_count as f64, y),
                stroke_style(grid_color, 1.0),
                Z_SERIES_FILL + 2,
            ));
        }

        Ok(elements)
    }
}
