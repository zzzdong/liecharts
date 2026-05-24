use vello_cpu::kurbo::{Point, Rect};

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::SeriesOption;
use crate::visual::{
    Color, FillStrokeStyle, StrokeStyle, TextAlign, TextBaseline, VisualElement, Z_LABEL, Z_SERIES_FILL, Z_SERIES_LINE
};

pub struct TableProcessor {
    series_index: usize,
}

impl TableProcessor {
    pub fn new(series_index: usize) -> Self {
        Self { series_index }
    }
}

impl DataProcessor for TableProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let table = match series {
            SeriesOption::Table(t) => t,
            _ => return Err(crate::error::ChartError::DataError("Expected Table series".into())),
        };

        let bounds = spec.bounds;
        let header_opt = table.header.as_ref();
        let body_opt = table.body.as_ref();
        let row_style = table.row_style.as_ref();
        let _cell_style = table.cell_style.as_ref();

        let header_height = header_opt
            .and_then(|h| h.height)
            .unwrap_or(40.0);
        let row_height = body_opt
            .and_then(|b| b.row_height)
            .unwrap_or(32.0);

        let columns = table.columns.as_deref().unwrap_or(&[]);
        let data = table.data.as_deref().unwrap_or(&[]);
        let num_cols = columns.len().max(data.first().map(|r| r.len()).unwrap_or(0));
        if num_cols == 0 {
            return Ok(SubplotVisualData {
                series_elements: Vec::new(),
                axis_elements: Vec::new(),
                grid_lines: Vec::new(),
            });
        }

        let col_width = bounds.width() / num_cols as f64;
        let border_color = row_style
            .and_then(|r| r.border_color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(Color::new(220, 220, 220));
        let border_width = row_style
            .and_then(|r| r.border_width)
            .unwrap_or(1.0);

        let header_bg = header_opt
            .and_then(|h| h.background_color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(Color::new(248, 248, 248));
        let header_text_color = header_opt
            .and_then(|h| h.style.as_ref())
            .and_then(|s| s.color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(Color::new(51, 51, 51));
        let header_font_size = header_opt
            .and_then(|h| h.style.as_ref())
            .and_then(|s| s.font_size)
            .unwrap_or(14.0);

        let body_text_color = body_opt
            .and_then(|b| b.style.as_ref())
            .and_then(|s| s.color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(Color::new(51, 51, 51));
        let body_font_size = body_opt
            .and_then(|b| b.style.as_ref())
            .and_then(|s| s.font_size)
            .unwrap_or(12.0);
        let even_bg = body_opt
            .and_then(|b| b.even_row_background_color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(Color::new(255, 255, 255));
        let odd_bg = body_opt
            .and_then(|b| b.odd_row_background_color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(Color::new(250, 250, 250));

        let mut elements = Vec::new();
        let mut y = bounds.y0;

        elements.push(VisualElement::Rect {
            rect: Rect::new(bounds.x0, y, bounds.x1, y + header_height),
            style: FillStrokeStyle {
                fill: Some(header_bg),
                stroke: None,
            },
            z_index: Z_SERIES_FILL,
        });

        for (col_idx, col_name) in columns.iter().enumerate() {
            let x = bounds.x0 + col_idx as f64 * col_width + col_width / 2.0;
            elements.push(VisualElement::TextRun {
                text: col_name.clone(),
                position: Point::new(x, y + header_height / 2.0),
                style: crate::visual::TextStyle {
                    font_size: header_font_size,
                    color: header_text_color,
                    align: TextAlign::Center,
                    vertical_align: TextBaseline::Middle,
                    ..Default::default()
                },
                rotation: 0.0,
                max_width: Some(col_width - 8.0),
                layout: None,
                z_index: Z_LABEL,
            });
        }

        y += header_height;

        elements.push(VisualElement::Line {
            start: Point::new(bounds.x0, y),
            end: Point::new(bounds.x1, y),
            style: StrokeStyle {
                color: border_color,
                width: border_width,
            },
            z_index: Z_SERIES_LINE,
        });

        for (row_idx, row) in data.iter().enumerate() {
            let bg = if row_idx % 2 == 0 { even_bg } else { odd_bg };
            elements.push(VisualElement::Rect {
                rect: Rect::new(bounds.x0, y, bounds.x1, y + row_height),
                style: FillStrokeStyle {
                    fill: Some(bg),
                    stroke: None,
                },
                z_index: Z_SERIES_FILL,
            });

            for (col_idx, cell) in row.iter().enumerate() {
                let x = bounds.x0 + col_idx as f64 * col_width + col_width / 2.0;
                let text = match cell {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => cell.to_string(),
                };
                elements.push(VisualElement::TextRun {
                    text,
                    position: Point::new(x, y + row_height / 2.0),
                    style: crate::visual::TextStyle {
                        font_size: body_font_size,
                        color: body_text_color,
                        align: TextAlign::Center,
                        vertical_align: TextBaseline::Middle,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: Some(col_width - 8.0),
                    layout: None,
                    z_index: Z_LABEL,
                });
            }

            y += row_height;
            elements.push(VisualElement::Line {
                start: Point::new(bounds.x0, y),
                end: Point::new(bounds.x1, y),
                style: StrokeStyle {
                    color: border_color,
                    width: border_width,
                },
                z_index: Z_SERIES_LINE,
            });
        }

        for col_idx in 0..=num_cols {
            let x = bounds.x0 + col_idx as f64 * col_width;
            elements.push(VisualElement::Line {
                start: Point::new(x, bounds.y0),
                end: Point::new(x, y),
                style: StrokeStyle {
                    color: border_color,
                    width: border_width,
                },
                z_index: Z_SERIES_LINE,
            });
        }

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: Vec::new(),
            grid_lines: Vec::new(),
        })
    }
}
