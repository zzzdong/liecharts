use vello_cpu::kurbo::Point;

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::{DataPoint, SeriesOption};
use crate::visual::{Color, FillStrokeStyle, Stroke, StrokeStyle, VisualElement, Z_SERIES_FILL, Z_SERIES_LINE, Z_SERIES_POINT};

pub struct LineProcessor {
    series_index: usize,
}

impl LineProcessor {
    pub fn new(series_index: usize) -> Self {
        Self { series_index }
    }

    fn extract_value(dp: &DataPoint) -> f64 {
        match dp {
            DataPoint::Value(v) => *v,
            DataPoint::Named(_, v) => *v,
            DataPoint::XY(_, y) => *y,
        }
    }

    fn extract_x_value(dp: &DataPoint) -> Option<f64> {
        match dp {
            DataPoint::XY(x, _) => Some(*x),
            _ => None,
        }
    }
}

impl DataProcessor for LineProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let line = match series {
            SeriesOption::Line(l) => l,
            _ => return Err(crate::error::ChartError::DataError("Expected Line series".into())),
        };

        let bounds = spec.bounds;

        let x_axis_idx = spec.x_axis_indices.first().copied().unwrap_or(0);
        let y_axis_idx = line.y_axis_index
            .or_else(|| spec.y_axis_indices.first().copied())
            .unwrap_or(0);

        let x_range = input.axis_ranges.get_x_range(x_axis_idx);
        let y_range = input.axis_ranges.get_y_range(y_axis_idx);

        let (x_min, x_max) = x_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 1.0));
        let (y_min, y_max) = y_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 100.0));

        // 检查是否为数值 X 轴（通过是否有 XY 数据点推断）
        let has_numeric_x = line.data.iter().any(|d| matches!(d, DataPoint::XY(_, _)));

        // 将数据点映射为像素坐标
        let mut points: Vec<Point> = Vec::new();
        for (i, item) in line.data.iter().enumerate() {
            let value = Self::extract_value(item);

            let px = if has_numeric_x {
                if let Some(xv) = Self::extract_x_value(item) {
                    bounds.x0 + (xv - x_min) / (x_max - x_min) * bounds.width()
                } else {
                    bounds.x0 + (i as f64 + 0.5) / line.data.len().max(1) as f64 * bounds.width()
                }
            } else {
                let cat_count = (x_max - x_min).max(1.0);
                bounds.x0 + (i as f64 + 0.5) / cat_count * bounds.width()
            };

            let py = bounds.y1 - (value - y_min) / (y_max - y_min) * bounds.height();
            points.push(Point::new(px, py));
        }

        let colors = &input.colors;
        let series_color = colors
            .series_colors
            .get(self.series_index)
            .copied()
            .unwrap_or(Color::new(100, 149, 237));

        // 线宽
        let line_width = line.line_style
            .as_ref()
            .and_then(|ls| ls.width)
            .unwrap_or(2.0);

        // 平滑
        let smooth = line.smooth.unwrap_or(false);

        // 面积填充
        let area_color: Option<Color> = line.area_style
            .as_ref()
            .and_then(|a| a.color)
            .map(|c| Color::new(c.r, c.g, c.b));

        let mut elements = Vec::new();

        // 面积填充
        if points.len() >= 2 && area_color.is_some() {
            if let Some(ac) = area_color {
                let alpha = (ac.a as f64 * 0.3).clamp(0.0, 255.0) as u8;
                let mut fill_color = ac;
                fill_color.a = alpha;

                let mut path = vello_cpu::kurbo::BezPath::new();
                path.move_to(points[0]);
                for p in &points[1..] {
                    path.line_to(*p);
                }
                // 回到基线
                let baseline_y = bounds.y1;
                path.line_to(Point::new(points.last().unwrap().x, baseline_y));
                path.line_to(Point::new(points[0].x, baseline_y));
                path.close_path();

                elements.push(VisualElement::Path {
                    path,
                    style: FillStrokeStyle {
                        fill: Some(fill_color),
                        stroke: None,
                    },
                    z_index: Z_SERIES_FILL,
                });
            }
        }

        // 折线
        if points.len() >= 2 {
            if smooth {
                let n = points.len();
                let tension = 0.3;
                let mut path = vello_cpu::kurbo::BezPath::new();
                path.move_to(points[0]);
                for i in 0..n - 1 {
                    let p0 = if i == 0 { points[0] } else { points[i - 1] };
                    let p1 = points[i];
                    let p2 = points[i + 1];
                    let p3 = if i + 2 < n { points[i + 2] } else { points[n - 1] };
                    let ctrl1 = Point::new(
                        p1.x + (p2.x - p0.x) * tension,
                        p1.y + (p2.y - p0.y) * tension,
                    );
                    let ctrl2 = Point::new(
                        p2.x - (p3.x - p1.x) * tension,
                        p2.y - (p3.y - p1.y) * tension,
                    );
                    path.curve_to(ctrl1, ctrl2, p2);
                }
                elements.push(VisualElement::Path {
                    path,
                    style: FillStrokeStyle {
                        fill: None,
                        stroke: Some(Stroke {
                            color: series_color,
                            width: line_width,
                        }),
                    },
                    z_index: Z_SERIES_LINE,
                });
            } else {
                elements.push(VisualElement::Polyline {
                    points: points.clone(),
                    style: StrokeStyle {
                        color: series_color,
                        width: line_width,
                    },
                    z_index: Z_SERIES_LINE,
                });
            }
        }

        // 数据点符号
        let show_symbol = line.symbol.as_ref()
            .map(|s| !matches!(s, crate::option::SymbolType::None))
            .unwrap_or(true);
        let symbol_size = line.symbol_size.unwrap_or(8.0);

        if show_symbol {
            for pt in &points {
                elements.push(VisualElement::Circle {
                    center: *pt,
                    radius: symbol_size / 2.0,
                    style: FillStrokeStyle {
                        fill: Some(Color::new(255, 255, 255)),
                        stroke: Some(Stroke {
                            color: series_color,
                            width: 2.0,
                        }),
                    },
                    z_index: Z_SERIES_POINT,
                });
            }
        }

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: Vec::new(),
            grid_lines: Vec::new(),
        })
    }
}