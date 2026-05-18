use crate::component::ChartComponent;
use crate::layout::{AxisPosition, LayoutOutput};
use crate::model::{Axis, AxisNameSide, AxisType, NameLocation, ResolvedOption};
use crate::option::{LabelAlign, LabelVerticalAlign};
use crate::visual::{VisualElement, StrokeStyle, TextAlign, TextBaseline};
use crate::text::{create_text_layout, compute_text_offset};
use vello_cpu::kurbo::Point;

pub struct AxisComponent {
    axis: Axis,
    is_x: bool,
    area_index: usize,
    grid_index: usize,
}

impl AxisComponent {
    pub fn new(axis: &Axis, is_x: bool, area_index: usize, grid_index: usize) -> Self {
        Self {
            axis: axis.clone(),
            is_x,
            area_index,
            grid_index,
        }
    }

    fn get_grid_info<'a>(&self, layout: &'a LayoutOutput) -> Option<&'a crate::layout::GridLayoutInfo> {
        layout.grids.iter().find(|g| g.grid_index == self.grid_index)
    }

    fn category_label_ts(&self) -> Vec<f64> {
        let data = match &self.axis.data {
            Some(d) => d,
            None => return Vec::new(),
        };
        let n = data.len();
        if n == 0 {
            return Vec::new();
        }
        if self.axis.boundary_gap {
            (0..n).map(|i| (i as f64 + 0.5) / n as f64).collect()
        } else {
            if n > 1 {
                (0..n).map(|i| i as f64 / (n - 1) as f64).collect()
            } else {
                vec![0.5]
            }
        }
    }

    fn category_tick_ts(&self) -> Vec<f64> {
        let data = match &self.axis.data {
            Some(d) => d,
            None => return Vec::new(),
        };
        let n = data.len();
        if n == 0 {
            return Vec::new();
        }
        if self.axis.axis_tick.align_with_label {
            self.category_label_ts()
        } else if self.axis.boundary_gap {
            (0..=n).map(|i| i as f64 / n as f64).collect()
        } else {
            if n > 1 {
                (0..n).map(|i| i as f64 / (n - 1) as f64).collect()
            } else {
                vec![0.5]
            }
        }
    }

    fn category_split_ts(&self) -> Vec<f64> {
        let data = match &self.axis.data {
            Some(d) => d,
            None => return Vec::new(),
        };
        let n = data.len();
        if n == 0 {
            return Vec::new();
        }
        if self.axis.boundary_gap {
            (0..=n).map(|i| i as f64 / n as f64).collect()
        } else {
            if n > 1 {
                (0..n).map(|i| i as f64 / (n - 1) as f64).collect()
            } else {
                vec![0.5]
            }
        }
    }

    fn value_tick_labels(&self, layout: &LayoutOutput) -> Vec<(f64, String)> {
        let grid_info = match self.get_grid_info(layout) {
            Some(g) => g,
            None => return Vec::new(),
        };

        let (data_min, data_max) = if self.is_x {
            grid_info.data_coord.x_range
        } else {
            // 对于Y轴，使用对应索引的Y轴范围
            grid_info.data_coord.y_ranges.get(self.area_index).copied()
                .or_else(|| grid_info.data_coord.y_ranges.first().copied())
                .unwrap_or((0.0, 100.0))
        };
        compute_nice_ticks(data_min, data_max, 5)
            .into_iter()
            .map(|v| {
                let t = if data_max != data_min {
                    if self.is_x {
                        (v - data_min) / (data_max - data_min)
                    } else {
                        (data_max - v) / (data_max - data_min)
                    }
                } else {
                    0.5
                };
                let label = if v.fract() == 0.0 {
                    format!("{:.0}", v)
                } else {
                    format!("{}", v)
                };
                (t, label)
            })
            .collect()
    }
}

impl ChartComponent for AxisComponent {
    fn build_visual_elements(&self, _resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        let grid_info = match self.get_grid_info(layout) {
            Some(g) => g,
            None => return elements,
        };

        let axis_areas = if self.is_x {
            &grid_info.x_axis_areas
        } else {
            &grid_info.y_axis_areas
        };

        let Some(area) = axis_areas.get(self.area_index) else {
            return elements;
        };

        // ── 轴线 ──
        if self.axis.axis_line.show {
            let (start, end) = match area.position {
                AxisPosition::Bottom => {
                    (Point::new(area.axis_bbox.x0, area.axis_bbox.y0), Point::new(area.axis_bbox.x1, area.axis_bbox.y0))
                }
                AxisPosition::Top => {
                    (Point::new(area.axis_bbox.x0, area.axis_bbox.y1), Point::new(area.axis_bbox.x1, area.axis_bbox.y1))
                }
                AxisPosition::Left => {
                    (Point::new(area.axis_bbox.x1, area.axis_bbox.y0), Point::new(area.axis_bbox.x1, area.axis_bbox.y1))
                }
                AxisPosition::Right => {
                    (Point::new(area.axis_bbox.x0, area.axis_bbox.y0), Point::new(area.axis_bbox.x0, area.axis_bbox.y1))
                }
            };

            elements.push(VisualElement::Line {
                start,
                end,
                style: StrokeStyle {
                    color: self.axis.axis_line.line_style.color,
                    width: self.axis.axis_line.line_style.width,
                },
            });
        }

        // ── 刻度标签数据 ──
        let tick_labels: Vec<(f64, String)> = if self.axis.axis_type == AxisType::Category {
            let ts = self.category_label_ts();
            let data = self.axis.data.as_deref().unwrap_or(&[]);
            ts.into_iter().enumerate().map(|(i, t)| {
                let label = data.get(i).cloned().unwrap_or_default();
                (t, label)
            }).collect()
        } else {
            self.value_tick_labels(layout)
        };

        let tick_base_rect = if self.axis.axis_type == AxisType::Category {
            area.axis_bbox
        } else {
            area.grid_bbox
        };

        // ── 刻度线 ──
        if self.axis.axis_tick.show {
            let tick_ts: Vec<f64> = if self.axis.axis_type == AxisType::Category {
                self.category_tick_ts()
            } else {
                tick_labels.iter().map(|(t, _)| *t).collect()
            };

            for t in &tick_ts {
                let tick_len = self.axis.tick_length;
                let (tick_start, tick_end) = match area.position {
                    AxisPosition::Bottom => {
                        let x = tick_base_rect.x0 + t * tick_base_rect.width();
                        (
                            Point::new(x, area.axis_bbox.y0),
                            Point::new(x, area.axis_bbox.y0 + tick_len),
                        )
                    }
                    AxisPosition::Top => {
                        let x = tick_base_rect.x0 + t * tick_base_rect.width();
                        (
                            Point::new(x, area.axis_bbox.y1),
                            Point::new(x, area.axis_bbox.y1 - tick_len),
                        )
                    }
                    AxisPosition::Left => {
                        let y = tick_base_rect.y0 + t * tick_base_rect.height();
                        (
                            Point::new(area.axis_bbox.x1, y),
                            Point::new(area.axis_bbox.x1 - tick_len, y),
                        )
                    }
                    AxisPosition::Right => {
                        let y = tick_base_rect.y0 + t * tick_base_rect.height();
                        (
                            Point::new(area.axis_bbox.x0, y),
                            Point::new(area.axis_bbox.x0 + tick_len, y),
                        )
                    }
                };

                elements.push(VisualElement::Line {
                    start: tick_start,
                    end: tick_end,
                    style: StrokeStyle {
                        color: self.axis.axis_tick.line_style.color,
                        width: self.axis.axis_tick.line_style.width,
                    },
                });
            }
        }

        // ── 刻度标签 ──
        if self.axis.axis_label.show {
            // 预测量所有标签宽度，用于密度检测
            let label_font = crate::model::TextStyle {
                font_size: self.axis.axis_label.font_size,
                font_family: self.axis.axis_label.font_family.clone(),
                color: self.axis.axis_label.color,
                font_weight: self.axis.axis_label.font_weight,
                ..Default::default()
            };

            let mut label_widths: Vec<f64> = Vec::with_capacity(tick_labels.len());
            for (_, label) in &tick_labels {
                let layout = create_text_layout(label, &label_font, None);
                label_widths.push(layout.width() as f64);
            }

            // 自动旋转检测：Bottom/Top 轴标签密集时自动倾斜 45°
            let auto_rotate = {
                let n = tick_labels.len();
                let is_horizontal = matches!(area.position, AxisPosition::Bottom | AxisPosition::Top);
                let user_rotation = self.axis.axis_label.rotate == 0.0;
                let spacing = if n > 1 {
                    (tick_labels[1].0 - tick_labels[0].0) * tick_base_rect.width()
                } else {
                    f64::MAX
                };
                let max_width = label_widths.iter().copied().fold(0.0, f64::max);
                is_horizontal && user_rotation && spacing > 0.0 && max_width > spacing * 1.2
            };

            let auto_angle = -std::f64::consts::PI / 4.0; // -45°（顺时针，字向右上倾斜）

            for ((t, label), text_width) in tick_labels.iter().zip(label_widths.iter()) {
                let tick_x = tick_base_rect.x0 + t * tick_base_rect.width();
                let tick_y = tick_base_rect.y0 + t * tick_base_rect.height();

                let layout_obj = create_text_layout(label, &label_font, None);
                let text_height = layout_obj.height() as f64;

                let (pos_x, pos_y) = match area.position {
                    AxisPosition::Bottom => {
                        let x = match self.axis.axis_label.align {
                            LabelAlign::Left => tick_x,
                            LabelAlign::Center => tick_x - text_width / 2.0,
                            LabelAlign::Right => tick_x - text_width,
                        };
                        let y = area.axis_bbox.y0 + self.axis.axis_label.margin;
                        (x, y)
                    }
                    AxisPosition::Top => {
                        let x = match self.axis.axis_label.align {
                            LabelAlign::Left => tick_x,
                            LabelAlign::Center => tick_x - text_width / 2.0,
                            LabelAlign::Right => tick_x - text_width,
                        };
                        let y = area.axis_bbox.y1 - self.axis.axis_label.margin - text_height;
                        (x, y)
                    }
                    AxisPosition::Left => {
                        let x = area.axis_bbox.x1 - self.axis.axis_label.margin - text_width;
                        let y = match self.axis.axis_label.vertical_align {
                            LabelVerticalAlign::Top => tick_y,
                            LabelVerticalAlign::Middle => tick_y - text_height / 2.0,
                            LabelVerticalAlign::Bottom => tick_y - text_height,
                        };
                        (x, y)
                    }
                    AxisPosition::Right => {
                        let x = area.axis_bbox.x0 + self.axis.axis_label.margin;
                        let y = match self.axis.axis_label.vertical_align {
                            LabelVerticalAlign::Top => tick_y,
                            LabelVerticalAlign::Middle => tick_y - text_height / 2.0,
                            LabelVerticalAlign::Bottom => tick_y - text_height,
                        };
                        (x, y)
                    }
                };

                let (final_align, final_baseline, final_rotation) = if auto_rotate {
                    match area.position {
                        // Bottom: 锚点在刻度处，文字左上对齐，向右上倾斜 45°
                        AxisPosition::Bottom => (TextAlign::Left, TextBaseline::Top, auto_angle),
                        // Top: 锚点在刻度处，文字左下对齐，向右上倾斜 45°
                        AxisPosition::Top => (TextAlign::Left, TextBaseline::Bottom, auto_angle),
                        _ => (TextAlign::Left, TextBaseline::Top, self.axis.axis_label.rotate),
                    }
                } else {
                    (TextAlign::Left, TextBaseline::Top, self.axis.axis_label.rotate)
                };

                elements.push(VisualElement::TextRun {
                    text: label.clone(),
                    position: Point::new(pos_x, pos_y),
                    style: crate::model::TextStyle {
                        color: self.axis.axis_label.color,
                        font_size: self.axis.axis_label.font_size,
                        font_family: self.axis.axis_label.font_family.clone(),
                        font_weight: self.axis.axis_label.font_weight,
                        font_style: crate::model::FontStyle::Normal,
                        align: final_align,
                        vertical_align: final_baseline,
                    },
                    rotation: final_rotation,
                    max_width: None,
                    layout: Some(layout_obj),
                });
            }
        }

        // ── 轴名称 ──
        if let Some(name) = &self.axis.name {
            let name_font = &self.axis.name_text_style;
            let name_layout = create_text_layout(name, name_font, None);
            let name_gap = self.axis.name_gap;

            // ECharts 行为：
            // 垂直轴（Left/Right）：
            //   - nameLocation: 'end'   → 水平文字在轴顶部，居中于轴线
            //   - nameLocation: 'start' → 水平文字在轴底部，居中于轴线
            //   - nameLocation: 'middle' → 竖向文字（旋转90°）在轴外侧
            // 水平轴（Bottom/Top）：
            //   - nameLocation: 'end'   → 水平文字在轴右端，start 对齐
            //   - nameLocation: 'middle' → 水平文字在轴中间，居中
            //   - nameLocation: 'start' → 水平文字在轴左端，end 对齐
            //
            // SVG 渲染器将 position 视为文本块左上角，忽略 align/baseline。
            // 因此先计算锚点，再用 compute_text_offset 转换为左上角坐标。
            let is_vertical_middle = matches!(area.position, AxisPosition::Left | AxisPosition::Right)
                && self.axis.name_location == NameLocation::Middle;

            let (anchor, align, baseline, rotation) = if is_vertical_middle {
                let anchor = match area.position {
                    AxisPosition::Left => {
                        let x = match self.axis.name_side {
                            AxisNameSide::Outside => area.axis_bbox.x0 - name_gap,
                            AxisNameSide::Inside => area.axis_bbox.x1 + name_gap,
                        };
                        Point::new(x, area.axis_bbox.center().y)
                    }
                    AxisPosition::Right => {
                        let x = match self.axis.name_side {
                            AxisNameSide::Outside => area.axis_bbox.x1 + name_gap,
                            AxisNameSide::Inside => area.axis_bbox.x0 - name_gap,
                        };
                        Point::new(x, area.axis_bbox.center().y)
                    }
                    _ => unreachable!(),
                };
                (anchor, TextAlign::Center, TextBaseline::Middle, std::f64::consts::PI / 2.0)
            } else {
                let (anchor, align, baseline) = match area.position {
                    AxisPosition::Bottom => {
                        // 底轴：轴线在 axis_bbox.y0（上边缘），名称在轴线下方
                        let (x, align) = match self.axis.name_location {
                            NameLocation::Start => (area.axis_bbox.x0 - name_gap, TextAlign::Right),
                            NameLocation::Middle => (area.axis_bbox.center().x, TextAlign::Center),
                            NameLocation::End => (area.axis_bbox.x1 + name_gap, TextAlign::Left),
                        };
                        let y = area.axis_bbox.y0 + name_gap;
                        (Point::new(x, y), align, TextBaseline::Top)
                    }
                    AxisPosition::Top => {
                        // 顶轴：轴线在 axis_bbox.y1（下边缘），名称在轴线上方
                        let (x, align) = match self.axis.name_location {
                            NameLocation::Start => (area.axis_bbox.x0 - name_gap, TextAlign::Right),
                            NameLocation::Middle => (area.axis_bbox.center().x, TextAlign::Center),
                            NameLocation::End => (area.axis_bbox.x1 + name_gap, TextAlign::Left),
                        };
                        let y = area.axis_bbox.y1 - name_gap;
                        (Point::new(x, y), align, TextBaseline::Bottom)
                    }
                    AxisPosition::Left => {
                        // 左轴：轴线在 axis_bbox.x1（右边缘），名称居中于轴线
                        let anchor = Point::new(area.axis_bbox.x1, match self.axis.name_location {
                            NameLocation::Start => area.axis_bbox.y1 + name_gap,
                            NameLocation::End => area.axis_bbox.y0 - name_gap,
                            _ => unreachable!(),
                        });
                        (anchor, TextAlign::Center, match self.axis.name_location {
                            NameLocation::Start => TextBaseline::Top,
                            NameLocation::End => TextBaseline::Bottom,
                            _ => unreachable!(),
                        })
                    }
                    AxisPosition::Right => {
                        // 右轴：轴线在 axis_bbox.x0（左边缘），名称居中于轴线
                        let anchor = Point::new(area.axis_bbox.x0, match self.axis.name_location {
                            NameLocation::Start => area.axis_bbox.y1 + name_gap,
                            NameLocation::End => area.axis_bbox.y0 - name_gap,
                            _ => unreachable!(),
                        });
                        (anchor, TextAlign::Center, match self.axis.name_location {
                            NameLocation::Start => TextBaseline::Top,
                            NameLocation::End => TextBaseline::Bottom,
                            _ => unreachable!(),
                        })
                    }
                };
                (anchor, align, baseline, 0.0)
            };

            // 使用 name_text_style 中的 align/vertical_align 覆盖位置默认值
            let final_align = if align != TextAlign::Left { align } else { name_font.align };
            let final_baseline = if baseline != TextBaseline::Top { baseline } else { name_font.vertical_align };

            // 将锚点转换为文本块左上角（SVG 渲染器要求）
            let (x_off, y_off) = compute_text_offset(&name_layout, final_align, final_baseline);
            let top_left = Point::new(anchor.x + x_off, anchor.y + y_off);

            elements.push(VisualElement::TextRun {
                text: name.clone(),
                position: top_left,
                style: crate::model::TextStyle {
                    color: name_font.color,
                    font_size: name_font.font_size,
                    font_family: name_font.font_family.clone(),
                    font_weight: name_font.font_weight,
                    font_style: name_font.font_style,
                    align: TextAlign::Left,
                    vertical_align: TextBaseline::Top,
                },
                rotation,
                max_width: None,
                layout: Some(name_layout),
            });
        }

        // ── 分割线 ──
        if self.axis.split_line.show {
            let split_ts: Vec<f64> = if self.axis.axis_type == AxisType::Category {
                self.category_split_ts()
            } else {
                tick_labels.iter().map(|(t, _)| *t).collect()
            };

            let grid_inner_bbox = grid_info.grid_inner_bbox;

            for t in &split_ts {
                if self.is_x {
                    let x = grid_inner_bbox.x0 + t * grid_inner_bbox.width();
                    elements.push(VisualElement::Line {
                        start: Point::new(x, grid_inner_bbox.y0),
                        end: Point::new(x, grid_inner_bbox.y1),
                        style: StrokeStyle {
                            color: self.axis.split_line.line_style.color,
                            width: self.axis.split_line.line_style.width,
                        },
                    });
                } else {
                    let y = grid_inner_bbox.y0 + t * grid_inner_bbox.height();
                    elements.push(VisualElement::Line {
                        start: Point::new(grid_inner_bbox.x0, y),
                        end: Point::new(grid_inner_bbox.x1, y),
                        style: StrokeStyle {
                            color: self.axis.split_line.line_style.color,
                            width: self.axis.split_line.line_style.width,
                        },
                    });
                }
            }
        }

        elements
    }
}

/// 计算美观的刻度值
fn compute_nice_ticks(min: f64, max: f64, max_ticks: usize) -> Vec<f64> {
    if min == max {
        return vec![min];
    }

    let range = max - min;
    let rough_step = range / max_ticks as f64;

    // 计算数量级
    let magnitude = 10f64.powf(rough_step.log10().floor());

    // 选择美观的步长
    let nice_step = if rough_step / magnitude < 1.5 {
        magnitude
    } else if rough_step / magnitude < 3.0 {
        2.0 * magnitude
    } else if rough_step / magnitude < 7.0 {
        5.0 * magnitude
    } else {
        10.0 * magnitude
    };

    // 计算起始和结束值
    let nice_min = (min / nice_step).floor() * nice_step;
    let nice_max = (max / nice_step).ceil() * nice_step;

    let mut ticks = Vec::new();
    let mut current = nice_min;
    while current <= nice_max {
        if current >= min && current <= max {
            ticks.push(current);
        }
        current += nice_step;
    }

    ticks
}
