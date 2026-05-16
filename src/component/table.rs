use crate::component::ChartComponent;
use crate::layout::{LayoutOutput, TableLayout};
use crate::model::{ResolvedOption, TableSeries};
use crate::visual::{Color, FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement};
use crate::text::create_text_layout;
use vello_cpu::kurbo::{Point, Rect};

pub struct TableSeriesComponent {
    series: TableSeries,
    #[allow(dead_code)]
    global_index: usize,
}

impl TableSeriesComponent {
    pub fn new(series: &TableSeries, global_index: usize) -> Self {
        Self {
            series: series.clone(),
            global_index,
        }
    }

    /// 获取表格所属的 grid 边界
    fn get_grid_bounds(&self, layout: &LayoutOutput) -> Option<Rect> {
        layout.grids.iter()
            .find(|g| g.grid_index == self.series.grid_index)
            .map(|g| g.grid_inner_bbox)
    }
}

impl ChartComponent for TableSeriesComponent {
    fn build_visual_elements(
        &self,
        _resolved: &ResolvedOption,
        layout: &LayoutOutput,
    ) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        // 使用 grid_index 找到正确的 grid
        let plot_bounds = self.get_grid_bounds(layout)
            .unwrap_or_else(|| Rect::new(0.0, 0.0, 800.0, 600.0));

        let padding = 8.0;
        let table_left = plot_bounds.x0 + padding;
        let table_top = plot_bounds.y0 + padding;
        let table_width = plot_bounds.width() - padding * 2.0;

        let header = &self.series.header_config;
        let body = &self.series.body_config;
        let columns = &self.series.columns;
        let data = &self.series.data;

        if columns.is_empty() {
            return elements;
        }

        // 使用 TableLayout 计算列宽
        let table_layout = TableLayout::from_series(&self.series);
        let col_widths = table_layout.calc_column_widths(table_width);
        let cell_max_w = col_widths.iter().copied().fold(0.0f64, f64::max) - padding * 2.0;

        // ── 表头 ──
        if header.show {
            elements.push(VisualElement::Rect {
                rect: Rect::new(table_left, table_top, table_left + table_width, table_top + header.height),
                style: FillStrokeStyle {
                    fill: Some(header.background_color),
                    stroke: None,
                },
            });

            for (i, col) in columns.iter().enumerate() {
                let col_width = col_widths.get(i).copied().unwrap_or(100.0);
                let cx = table_left + col_widths.iter().take(i).sum::<f64>() + col_width / 2.0;
                let cy = table_top + header.height / 2.0;

                let layout_obj = create_text_layout(col, &header.style, Some(cell_max_w));
                let tw = layout_obj.width() as f64;
                let th = layout_obj.height() as f64;

                elements.push(VisualElement::TextRun {
                    text: col.clone(),
                    position: Point::new(cx - tw / 2.0, cy - th / 2.0),
                    font_size: header.style.font_size,
                    font_family: header.style.font_family.clone(),
                    color: header.style.color,
                    font_weight: header.style.font_weight,
                    font_style: header.style.font_style,
                    align: TextAlign::Left,
                    baseline: TextBaseline::Top,
                    rotation: 0.0,
                    max_width: Some(cell_max_w),
                    layout: Some(layout_obj),
                });
            }
        }

        // ── 数据行 ──
        for (row_idx, row) in data.iter().enumerate() {
            let row_y = table_top + header.height + row_idx as f64 * body.row_height;
            let bg_color = if row_idx % 2 == 0 {
                body.even_row_background_color
            } else {
                body.odd_row_background_color
            };

            // 行背景
            elements.push(VisualElement::Rect {
                rect: Rect::new(table_left, row_y, table_left + table_width, row_y + body.row_height),
                style: FillStrokeStyle {
                    fill: Some(bg_color),
                    stroke: None,
                },
            });

            for (col_idx, cell) in row.iter().enumerate() {
                let cell_text = match cell {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => cell.to_string(),
                };

                let col_width = col_widths.get(col_idx).copied().unwrap_or(100.0);
                let cx = table_left + col_widths.iter().take(col_idx).sum::<f64>() + col_width / 2.0;
                let cy = row_y + body.row_height / 2.0;

                let layout_obj = create_text_layout(&cell_text, &body.style, Some(cell_max_w));
                let tw = layout_obj.width() as f64;
                let th = layout_obj.height() as f64;

                elements.push(VisualElement::TextRun {
                    text: cell_text,
                    position: Point::new(cx - tw / 2.0, cy - th / 2.0),
                    font_size: body.style.font_size,
                    font_family: body.style.font_family.clone(),
                    color: body.style.color,
                    font_weight: body.style.font_weight,
                    font_style: body.style.font_style,
                    align: TextAlign::Left,
                    baseline: TextBaseline::Top,
                    rotation: 0.0,
                    max_width: Some(cell_max_w),
                    layout: Some(layout_obj),
                });
            }
        }

        // ── 网格线 ──
        let total_height = header.height + data.len() as f64 * body.row_height;
        let grid_color = Color::new(200, 200, 200);
        let grid_style = StrokeStyle::new(grid_color, 1.0);

        // 列分隔线（垂直线）
        let mut current_x = table_left;
        for i in 0..=columns.len() {
            let x = if i < columns.len() {
                let w = col_widths.get(i).copied().unwrap_or(0.0);
                let pos = current_x;
                current_x += w;
                pos
            } else {
                current_x
            };
            elements.push(VisualElement::Line {
                start: Point::new(x, table_top),
                end: Point::new(x, table_top + total_height),
                style: grid_style.clone(),
            });
        }

        // 行分隔线（水平线）
        for i in 0..=(data.len()) {
            let y = table_top + header.height + i as f64 * body.row_height;
            elements.push(VisualElement::Line {
                start: Point::new(table_left, y),
                end: Point::new(table_left + table_width, y),
                style: grid_style.clone(),
            });
        }

        // ── 外边框（加粗） ──
        elements.push(VisualElement::Rect {
            rect: Rect::new(table_left, table_top, table_left + table_width, table_top + total_height),
            style: FillStrokeStyle {
                fill: None,
                stroke: Some(Stroke {
                    color: Color::new(160, 160, 160),
                    width: 2.0,
                }),
            },
        });

        elements
    }
}
