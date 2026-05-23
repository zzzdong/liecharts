use vello_cpu::kurbo::Point;

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::{DataPoint, SeriesOption};
use crate::visual::{Color, FillStrokeStyle, Stroke, StrokeStyle, VisualElement};

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
        let y_axis_idx = spec.y_axis_indices.first().copied().unwrap_or(0);

        let x_range = input.axis_ranges.ranges.iter()
            .find(|r| r.axis_index == x_axis_idx);
        let y_range = input.axis_ranges.ranges.iter()
            .find(|r| r.axis_index == y_axis_idx);

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
                });
            }
        }

        // 折线
        if points.len() >= 2 {
            if smooth {
                // 使用 Catmull-Rom 样条 — 简化版: 使用基础曲线
                let mut path = vello_cpu::kurbo::BezPath::new();
                path.move_to(points[0]);
                for i in 1..points.len() {
                    let prev = points[i - 1];
                    let curr = points[i];
                    let ctrl1 = Point::new((prev.x + curr.x) / 2.0, prev.y);
                    let ctrl2 = Point::new((prev.x + curr.x) / 2.0, curr.y);
                    path.curve_to(ctrl1, ctrl2, curr);
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
                });
            } else {
                elements.push(VisualElement::Polyline {
                    points: points.clone(),
                    style: StrokeStyle {
                        color: series_color,
                        width: line_width,
                    },
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