use vello_cpu::kurbo::{Point, Rect};

use crate::error::Result;
use crate::new_pipeline::data_processor::DataProcessor;
use crate::new_pipeline::types::{DataProcessorInput, SubplotVisualData};
use crate::option::{AxisType, DataPoint, SeriesOption};
use crate::visual::{Color, FillStrokeStyle, Stroke, TextAlign, TextBaseline, VisualElement};

pub struct BarProcessor {
    series_index: usize,
}

impl BarProcessor {
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
}

impl DataProcessor for BarProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let bar = match series {
            SeriesOption::Bar(b) => b,
            _ => return Err(crate::error::ChartError::DataError("Expected Bar series".into())),
        };

        let bounds = spec.bounds;

        // 获取 X 轴和 Y 轴的配置与范围
        let x_axis_idx = spec.x_axis_indices.first().copied().unwrap_or(0);
        let y_axis_idx = spec.y_axis_indices.first().copied().unwrap_or(0);

        let x_axis_config = input.option.x_axis.get(x_axis_idx);
        let y_axis_config = input.option.y_axis.get(y_axis_idx);

        let x_range = input.axis_ranges.ranges.iter()
            .find(|r| r.axis_index == x_axis_idx);
        let y_range = input.axis_ranges.ranges.iter()
            .find(|r| r.axis_index == y_axis_idx);

        let (x_min, x_max) = x_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 1.0));
        let (y_min, y_max) = y_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 100.0));

        // 判断是否为类目轴
        let _is_cat_x = x_axis_config
            .and_then(|a| a.axis_type)
            .map(|t| t == AxisType::Category)
            .unwrap_or(false);
        let is_cat_y = y_axis_config
            .and_then(|a| a.axis_type)
            .map(|t| t == AxisType::Category)
            .unwrap_or(false);

        // 确定方向：垂直柱状图 (category X) / 横向柱状图 (category Y)
        let is_horizontal = is_cat_y;

        // 收集该 grid 中所有 bar series 的分组信息
        let grid_idx = spec.id;
        let group_total: usize = input.option.series.iter()
            .filter_map(|s| match s {
                SeriesOption::Bar(b) if b.grid_index.unwrap_or(0) == grid_idx => {
                    Some(b.group_index.unwrap_or(0))
                }
                _ => None,
            })
            .max()
            .map(|m| m + 1)
            .unwrap_or(1);

        let group_index = bar.group_index.unwrap_or(0);

        // 使用默认或用户指定的 bar width
        let default_bar_width_ratio = 0.6;
        let bar_width_ratio = if let Some(ref bw) = bar.bar_width {
            if let Some(pct) = bw.strip_suffix('%') {
                pct.parse::<f64>().unwrap_or(default_bar_width_ratio * 100.0) / 100.0
            } else {
                default_bar_width_ratio
            }
        } else {
            default_bar_width_ratio
        };

        // 颜色
        let colors = &input.colors;

        let mut elements = Vec::new();
        let mut label_elements = Vec::new();

        for (i, item) in bar.data.iter().enumerate() {
            let value = Self::extract_value(item);

            let color = colors
                .series_colors
                .get(self.series_index)
                .copied()
                .unwrap_or(Color::new(100, 149, 237));

            if is_horizontal {
                // 横向柱状图：Y 为类目，X 为数值
                let cat_count = (y_max - y_min).max(1.0);
                let cat_height = bounds.height() / cat_count;
                let group_height = cat_height * bar_width_ratio;
                let bar_height = group_height / group_total as f64;

                let category_center = bounds.y0 + (i as f64 + 0.5) * cat_height;
                let group_offset = (group_index as f64 - (group_total as f64 - 1.0) / 2.0) * bar_height;
                let center_y = category_center + group_offset;

                let right_x = bounds.x0 + (value - x_min) / (x_max - x_min) * bounds.width();
                let left_x = bounds.x0;

                let bar_left = left_x.min(right_x);
                let bar_w = (right_x - left_x).abs();
                let y = center_y - bar_height / 2.0;

                elements.push(VisualElement::Rect {
                    rect: Rect::new(bar_left, y, bar_left + bar_w, y + bar_height),
                    style: FillStrokeStyle {
                        fill: Some(color),
                        stroke: Some(Stroke {
                            color: Color::new(255, 255, 255),
                            width: 1.0,
                        }),
                    },
                });

                // 标签
                let label_text = format!("{:.0}", value);
                let label_x = right_x + 5.0;
                label_elements.push(VisualElement::TextRun {
                    text: label_text,
                    position: Point::new(label_x, center_y),
                    style: crate::model::TextStyle {
                        font_size: 11.0,
                        color,
                        align: TextAlign::Left,
                        vertical_align: TextBaseline::Middle,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                });
            } else {
                // 垂直柱状图：X 为类目，Y 为数值
                let cat_count = (x_max - x_min).max(1.0);
                let cat_width = bounds.width() / cat_count;
                let group_width = cat_width * bar_width_ratio;
                let bar_width = group_width / group_total as f64;

                let category_center = bounds.x0 + (i as f64 + 0.5) * cat_width;
                let group_offset = (group_index as f64 - (group_total as f64 - 1.0) / 2.0) * bar_width;
                let center_x = category_center + group_offset;

                let top_y = bounds.y1 - (value - y_min) / (y_max - y_min) * bounds.height();
                let bottom_y = bounds.y1;

                let bar_top = top_y.min(bottom_y);
                let bar_h = (top_y - bottom_y).abs();
                let x = center_x - bar_width / 2.0;

                elements.push(VisualElement::Rect {
                    rect: Rect::new(x, bar_top, x + bar_width, bar_top + bar_h),
                    style: FillStrokeStyle {
                        fill: Some(color),
                        stroke: Some(Stroke {
                            color: Color::new(255, 255, 255),
                            width: 1.0,
                        }),
                    },
                });

                // 标签
                let label_text = format!("{:.0}", value);
                let label_y = bar_top - 5.0;
                label_elements.push(VisualElement::TextRun {
                    text: label_text,
                    position: Point::new(center_x, label_y),
                    style: crate::model::TextStyle {
                        font_size: 11.0,
                        color,
                        align: TextAlign::Center,
                        vertical_align: TextBaseline::Bottom,
                        ..Default::default()
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: None,
                });
            }
        }

        elements.extend(label_elements);

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: Vec::new(),
            grid_lines: Vec::new(),
        })
    }
}