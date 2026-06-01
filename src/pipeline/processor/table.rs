use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    option::SeriesOption,
    pipeline::{
        data_processor::{DataProcessor, DataProcessorInput},
        dataframe::{DataFrame, DataValue, Series},
    },
    visual::{
        Color, FillStrokeStyle, StrokeStyle, TextAlign, TextBaseline, VisualElement, Z_LABEL,
        Z_SERIES_FILL, Z_SERIES_LINE,
    },
};

pub struct TableProcessor;

impl TableProcessor {
    pub fn new() -> Self {
        Self
    }

    fn cell_text(v: &serde_json::Value) -> String {
        match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => v.to_string(),
        }
    }
}

impl DataProcessor for TableProcessor {
    fn to_dataframe(
        &self,
        series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        let table = match series {
            SeriesOption::Table(t) => t,
            _ => {
                return Err(crate::error::ChartError::DataError(
                    "Expected Table series".into(),
                ));
            }
        };

        let columns = table.columns.as_deref().unwrap_or(&[]);
        let data = table.data.as_deref().unwrap_or(&[]);
        let num_cols = columns
            .len()
            .max(data.first().map(|r| r.len()).unwrap_or(0));
        if num_cols == 0 {
            return Ok(DataFrame::new());
        }

        let mut row_indices: Vec<DataValue> = Vec::new();
        let mut col_indices: Vec<DataValue> = Vec::new();
        let mut texts: Vec<DataValue> = Vec::new();
        let mut is_headers: Vec<DataValue> = Vec::new();

        for (col_idx, col_name) in columns.iter().enumerate() {
            row_indices.push(DataValue::Integer(0));
            col_indices.push(DataValue::Integer(col_idx as i64));
            texts.push(DataValue::String(col_name.clone()));
            is_headers.push(DataValue::Bool(true));
        }

        for (row_idx, row) in data.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate().take(num_cols) {
                row_indices.push(DataValue::Integer((row_idx + 1) as i64));
                col_indices.push(DataValue::Integer(col_idx as i64));
                texts.push(DataValue::String(Self::cell_text(cell)));
                is_headers.push(DataValue::Bool(false));
            }
        }

        let mut df = DataFrame::new();
        df.add_column(Series::new("row_idx", row_indices));
        df.add_column(Series::new("col_idx", col_indices));
        df.add_column(Series::new("text", texts));
        df.add_column(Series::new("is_header", is_headers));

        Ok(df)
    }

    fn transform(&self, mut df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame> {
        let series = &input.option.series[input.series_idx];
        let table = match series {
            SeriesOption::Table(t) => t,
            _ => return Ok(df),
        };

        let bounds = input.bounds;
        let header_opt = table.header.as_ref();
        let body_opt = table.body.as_ref();
        let row_style = table.row_style.as_ref();

        let header_height = header_opt.and_then(|h| h.height).unwrap_or(40.0);
        let row_height = body_opt.and_then(|b| b.row_height).unwrap_or(32.0);

        let header_bg = header_opt
            .and_then(|h| h.background_color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(input.colors.table_row_even_bg);
        let header_text_color = header_opt
            .and_then(|h| h.style.as_ref())
            .and_then(|s| s.color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(input.colors.text_color);
        let header_font_size = header_opt
            .and_then(|h| h.style.as_ref())
            .and_then(|s| s.font_size)
            .unwrap_or(14.0);

        let body_text_color = body_opt
            .and_then(|b| b.style.as_ref())
            .and_then(|s| s.color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(input.colors.text_color);
        let body_font_size = body_opt
            .and_then(|b| b.style.as_ref())
            .and_then(|s| s.font_size)
            .unwrap_or(12.0);
        let even_bg = body_opt
            .and_then(|b| b.even_row_background_color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(input.colors.table_row_odd_bg);
        let odd_bg = body_opt
            .and_then(|b| b.odd_row_background_color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(input.colors.table_row_even_bg);

        let border_color = row_style
            .and_then(|r| r.border_color.as_ref())
            .map(|c| Color::new(c.r, c.g, c.b))
            .unwrap_or(input.colors.table_header_bg);
        let border_width = row_style.and_then(|r| r.border_width).unwrap_or(1.0);

        let columns = table.columns.as_deref().unwrap_or(&[]);
        let data = table.data.as_deref().unwrap_or(&[]);
        let num_cols = columns
            .len()
            .max(data.first().map(|r| r.len()).unwrap_or(0))
            .max(1);
        let num_rows = data.len();
        let col_width = bounds.width() / num_cols as f64;

        df.compute_column("x", |_i, df| {
            let row = df.get_column("row_idx");
            let col = df.get_column("col_idx");
            if let (Some(r), Some(c)) = (row, col) {
                if let (Some(cv), _) = (c.as_f64(_i), r.as_f64(_i)) {
                    DataValue::Float(bounds.x0 + cv * col_width + col_width / 2.0)
                } else {
                    DataValue::Null
                }
            } else {
                DataValue::Null
            }
        });

        df.compute_column("y", |_i, df| {
            let row = df.get_column("row_idx");
            if let Some(r) = row {
                if let Some(rv) = r.as_f64(_i) {
                    if rv < 1.0 {
                        DataValue::Float(bounds.y0 + header_height / 2.0)
                    } else {
                        let data_y =
                            bounds.y0 + header_height + (rv - 1.0) * row_height + row_height / 2.0;
                        DataValue::Float(data_y)
                    }
                } else {
                    DataValue::Null
                }
            } else {
                DataValue::Null
            }
        });

        df.compute_column("cell_height", |_i, df| {
            let row = df.get_column("row_idx");
            if let Some(r) = row {
                if let Some(rv) = r.as_f64(_i) {
                    DataValue::Float(if rv < 1.0 { header_height } else { row_height })
                } else {
                    DataValue::Null
                }
            } else {
                DataValue::Null
            }
        });

        df.compute_column("bg_color", |_i, df| {
            let row = df.get_column("row_idx");
            if let Some(r) = row {
                if let Some(rv) = r.as_f64(_i) {
                    let c = if rv < 1.0 {
                        header_bg
                    } else if (rv as usize) % 2 == 0 {
                        even_bg
                    } else {
                        odd_bg
                    };
                    DataValue::Color(c)
                } else {
                    DataValue::Null
                }
            } else {
                DataValue::Null
            }
        });

        df.compute_column("font_size", |_i, df| {
            let row = df.get_column("row_idx");
            if let Some(r) = row {
                if let Some(rv) = r.as_f64(_i) {
                    DataValue::Float(if rv < 1.0 {
                        header_font_size
                    } else {
                        body_font_size
                    })
                } else {
                    DataValue::Null
                }
            } else {
                DataValue::Null
            }
        });

        df.compute_column("text_color", |_i, df| {
            let row = df.get_column("row_idx");
            if let Some(r) = row {
                if let Some(rv) = r.as_f64(_i) {
                    let c = if rv < 1.0 {
                        header_text_color
                    } else {
                        body_text_color
                    };
                    DataValue::Color(c)
                } else {
                    DataValue::Null
                }
            } else {
                DataValue::Null
            }
        });

        df.add_column(Series::new_constant(
            "col_width",
            DataValue::Float(col_width),
            df.row_count(),
        ));
        df.add_column(Series::new_constant(
            "header_height",
            DataValue::Float(header_height),
            df.row_count(),
        ));
        df.add_column(Series::new_constant(
            "row_height",
            DataValue::Float(row_height),
            df.row_count(),
        ));
        df.add_column(Series::new_constant(
            "border_color",
            DataValue::Color(border_color),
            df.row_count(),
        ));
        df.add_column(Series::new_constant(
            "border_width",
            DataValue::Float(border_width),
            df.row_count(),
        ));
        df.add_column(Series::new_constant(
            "num_cols",
            DataValue::Integer(num_cols as i64),
            df.row_count(),
        ));
        df.add_column(Series::new_constant(
            "num_rows",
            DataValue::Integer(num_rows as i64),
            df.row_count(),
        ));
        df.add_column(Series::new_constant(
            "bounds_x0",
            DataValue::Float(bounds.x0),
            df.row_count(),
        ));
        df.add_column(Series::new_constant(
            "bounds_y0",
            DataValue::Float(bounds.y0),
            df.row_count(),
        ));

        Ok(df)
    }

    fn to_visual_elements(
        &self,
        df: &DataFrame,
        _input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let num_cols = df
            .get_column("num_cols")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(0.0) as usize;
        let num_rows = df
            .get_column("num_rows")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(0.0) as usize;
        let col_width = df
            .get_column("col_width")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(100.0);
        let header_height = df
            .get_column("header_height")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(40.0);
        let row_height_val = df
            .get_column("row_height")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(32.0);
        let border_color = df
            .get_column("border_color")
            .and_then(|c| c.as_color(0))
            .unwrap_or(Color::new(200, 200, 200));
        let border_width = df
            .get_column("border_width")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(1.0);
        let bounds_x0 = df
            .get_column("bounds_x0")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(0.0);
        let bounds_y0 = df
            .get_column("bounds_y0")
            .and_then(|c| c.as_f64(0))
            .unwrap_or(0.0);

        let x_col = df.get_column("x");
        let y_col = df.get_column("y");
        let text_col = df.get_column("text");
        let bg_col = df.get_column("bg_color");
        let cell_h_col = df.get_column("cell_height");
        let font_sz_col = df.get_column("font_size");
        let text_c_col = df.get_column("text_color");

        let mut elements = Vec::new();
        let mut last_row: i64 = -1;

        for i in 0..df.row_count() {
            let x = x_col.and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let y = y_col.and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let bg = bg_col
                .and_then(|c| c.as_color(i))
                .unwrap_or(Color::new(255, 255, 255));
            let cell_h = cell_h_col.and_then(|c| c.as_f64(i)).unwrap_or(32.0);
            let font_sz = font_sz_col.and_then(|c| c.as_f64(i)).unwrap_or(12.0);
            let text_c = text_c_col
                .and_then(|c| c.as_color(i))
                .unwrap_or(Color::new(0, 0, 0));
            let current_row = df
                .get_column("row_idx")
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0) as i64;

            if current_row != last_row {
                let row_top = bounds_y0
                    + (if current_row == 0 {
                        0.0
                    } else {
                        y_col.and_then(|c| c.as_f64(i)).unwrap_or(0.0) - cell_h / 2.0
                    });
                elements.push(VisualElement::Rect {
                    rect: Rect::new(
                        bounds_x0,
                        row_top,
                        bounds_x0 + num_cols as f64 * col_width,
                        row_top + cell_h,
                    ),
                    style: FillStrokeStyle {
                        fill: Some(bg),
                        stroke: None,
                    },
                    z_index: Z_SERIES_FILL,
                });
                last_row = current_row;
            }

            let cell_text = text_col.and_then(|c| c.as_string(i)).unwrap_or_default();
            elements.push(VisualElement::TextRun {
                text: cell_text,
                position: Point::new(x, y),
                style: crate::visual::TextStyle {
                    font_size: font_sz,
                    color: text_c,
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

        let total_height = header_height + num_rows as f64 * row_height_val;

        // horizontal border lines
        elements.push(VisualElement::Line {
            start: Point::new(bounds_x0, bounds_y0),
            end: Point::new(bounds_x0 + num_cols as f64 * col_width, bounds_y0),
            style: StrokeStyle {
                color: border_color,
                width: border_width,
            },
            z_index: Z_SERIES_LINE,
        });
        for r in 0..=num_rows {
            let line_y = bounds_y0 + header_height + r as f64 * row_height_val;
            elements.push(VisualElement::Line {
                start: Point::new(bounds_x0, line_y),
                end: Point::new(bounds_x0 + num_cols as f64 * col_width, line_y),
                style: StrokeStyle {
                    color: border_color,
                    width: border_width,
                },
                z_index: Z_SERIES_LINE,
            });
        }

        // vertical border lines
        for c in 0..=num_cols {
            let line_x = bounds_x0 + c as f64 * col_width;
            elements.push(VisualElement::Line {
                start: Point::new(line_x, bounds_y0),
                end: Point::new(line_x, bounds_y0 + total_height),
                style: StrokeStyle {
                    color: border_color,
                    width: border_width,
                },
                z_index: Z_SERIES_LINE,
            });
        }

        Ok(elements)
    }
}
