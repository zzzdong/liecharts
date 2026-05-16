use crate::component::{ChartComponent, SeriesComponent, SeriesContext};
use crate::layout::LayoutOutput;
use crate::model::{GaugeSeries, ResolvedOption};
use crate::visual::{FillStrokeStyle, GradientDef, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement};
use crate::text::{compute_text_offset, create_text_layout};
use vello_cpu::kurbo::{BezPath, Point};
use vello_cpu::kurbo::Shape;
use std::f64::consts::PI;

pub struct GaugeSeriesComponent {
    series: GaugeSeries,
    series_index: usize,
    grid_index: usize,
}

impl GaugeSeriesComponent {
    pub fn new(series: &GaugeSeries, series_index: usize) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index: 0,
        }
    }

    fn deg_to_rad(deg: f64) -> f64 {
        deg * PI / 180.0
    }

    fn point_at_angle(center: Point, radius: f64, angle_deg: f64) -> Point {
        let angle_rad = Self::deg_to_rad(angle_deg);
        Point::new(
            center.x + radius * angle_rad.cos(),
            center.y + radius * angle_rad.sin(),
        )
    }

    fn build_with_context(&self, ctx: &SeriesContext) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        let chart_bounds = ctx.grid_info.grid_bbox;
        let chart_width = chart_bounds.width();
        let chart_height = chart_bounds.height();

        let center_x = chart_bounds.x0 + chart_width * self.series.center.0 / 100.0;
        let center_y = chart_bounds.y0 + chart_height * self.series.center.1 / 100.0;
        let center = Point::new(center_x, center_y);

        let radius = chart_width.min(chart_height) * self.series.radius / 100.0 * 0.5;

        let start_angle = self.series.start_angle;
        let end_angle = self.series.end_angle;
        let angle_range = end_angle - start_angle;

        let bg_width = 20.0;
        let axis_line_width = self.series.axis_line_style.width;
        let bg_inner_radius = radius - bg_width;
        let bg_outer_radius = radius + axis_line_width / 2.0;

        // 先渲染轴线弧（灰色底圈），再渲染彩色背景色带覆盖其上
        if self.series.axis_line_show {
            let mut axis_path = BezPath::new();
            let axis_start = Self::point_at_angle(center, radius, start_angle);
            axis_path.move_to(axis_start);

            let axis_segments = 100;
            for j in 1..=axis_segments {
                let t = j as f64 / axis_segments as f64;
                let angle = start_angle + angle_range * t;
                let p = Self::point_at_angle(center, radius, angle);
                axis_path.line_to(p);
            }

            elements.push(VisualElement::Path {
                path: axis_path,
                style: FillStrokeStyle {
                    fill: None,
                    stroke: Some(Stroke {
                        color: self.series.axis_line_style.color,
                        width: self.series.axis_line_style.width,
                    }),
                },
            });
        }

        let gradient_stops = &self.series.gradient_colors;

        // 使用单个渐变路径替代多段色块
        let bg_segments = 100;
        let mut bg_path = BezPath::new();
        let outer_start = Self::point_at_angle(center, bg_outer_radius, start_angle);
        bg_path.move_to(outer_start);

        for j in 1..=bg_segments {
            let t = j as f64 / bg_segments as f64;
            let angle = start_angle + angle_range * t;
            let p = Self::point_at_angle(center, bg_outer_radius, angle);
            bg_path.line_to(p);
        }

        for j in (0..=bg_segments).rev() {
            let t = j as f64 / bg_segments as f64;
            let angle = start_angle + angle_range * t;
            let p = Self::point_at_angle(center, bg_inner_radius, angle);
            bg_path.line_to(p);
        }

        bg_path.close_path();

        elements.push(VisualElement::GradientPath {
            path: bg_path,
            gradient: GradientDef::new(gradient_stops.clone()),
            stroke: None,
        });

        let split_count = self.series.split_number;
        if split_count > 0 {
            for i in 0..=split_count {
                let ratio = i as f64 / split_count as f64;
                let angle = start_angle + angle_range * ratio;

                if self.series.axis_tick_show {
                    let tick_start = Self::point_at_angle(center, bg_outer_radius - 5.0, angle);
                    let tick_end = Self::point_at_angle(center, bg_outer_radius - 5.0 - self.series.axis_tick_length, angle);

                    elements.push(VisualElement::Line {
                        start: tick_start,
                        end: tick_end,
                        style: StrokeStyle {
                            color: self.series.axis_tick_style.color,
                            width: self.series.axis_tick_style.width,
                        },
                    });
                }

                if self.series.split_line_show && i % (split_count / 5 + 1) == 0 {
                    let split_start = Self::point_at_angle(center, bg_outer_radius, angle);
                    let split_end = Self::point_at_angle(center, bg_outer_radius - self.series.split_line_length, angle);

                    elements.push(VisualElement::Line {
                        start: split_start,
                        end: split_end,
                        style: StrokeStyle {
                            color: self.series.split_line_style.color,
                            width: self.series.split_line_style.width,
                        },
                    });
                }

                if self.series.axis_label_show && i % (split_count / 5 + 1) == 0 {
                    let value = self.series.min + (self.series.max - self.series.min) * ratio;
                    let label_text = format!("{:.0}", value);

                    let label_pos = Self::point_at_angle(center, bg_outer_radius + self.series.axis_label_distance, angle);

                    let label_font = crate::model::TextStyle {
                        font_size: self.series.axis_label_font_size,
                        font_family: self.series.axis_label_font_family.clone(),
                        color: self.series.axis_label_color,
                        font_weight: self.series.axis_label_font_weight,
                        ..Default::default()
                    };

                    let layout = create_text_layout(&label_text, &label_font, None);
                    let (x_offset, y_offset) = compute_text_offset(&layout, TextAlign::Center, TextBaseline::Middle);
                    let final_position = Point::new(label_pos.x + x_offset, label_pos.y + y_offset);

                    elements.push(VisualElement::TextRun {
                        text: label_text,
                        position: final_position,
                        font_size: label_font.font_size,
                        font_family: label_font.font_family,
                        color: label_font.color,
                        font_weight: label_font.font_weight,
                        font_style: label_font.font_style,
                        align: TextAlign::Left,
                        baseline: TextBaseline::Top,
                        rotation: 0.0,
                        max_width: None,
                        layout: Some(layout),
                    });
                }
            }
        }

        if self.series.pointer_show {
            let value_ratio = ((self.series.value - self.series.min) / (self.series.max - self.series.min))
                .clamp(0.0, 1.0);
            let pointer_angle = start_angle + angle_range * value_ratio;

            let pointer_length = radius * self.series.pointer_length / 100.0;
            let pointer_width = self.series.pointer_width;

            let pointer_end = Self::point_at_angle(center, pointer_length, pointer_angle);

            let perp_angle1 = pointer_angle - 90.0;
            let perp_angle2 = pointer_angle + 90.0;

            let p1 = Self::point_at_angle(center, pointer_width * 0.5, perp_angle1);
            let p2 = Self::point_at_angle(center, pointer_width * 0.5, perp_angle2);

            let mut path = BezPath::new();
            path.move_to(p1);
            path.line_to(pointer_end);
            path.line_to(p2);
            path.close_path();

            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(self.series.pointer_color),
                    stroke: None,
                },
            });

            let center_circle = vello_cpu::kurbo::Circle::new(center, pointer_width * 0.8);
            let center_path = center_circle.into_path(0.1);

            elements.push(VisualElement::Path {
                path: center_path,
                style: FillStrokeStyle {
                    fill: Some(self.series.pointer_color),
                    stroke: None,
                },
            });
        }

        if self.series.title_show {
            let title_text = if self.series.name.is_empty() {
                ""
            } else {
                &self.series.name
            };

            if !title_text.is_empty() {
                let title_x = center_x + chart_width * self.series.title_offset.0 / 100.0;
                let title_y = center_y + chart_height * self.series.title_offset.1 / 100.0;

                let title_font = crate::model::TextStyle {
                    font_size: self.series.title_font_size,
                    font_family: self.series.title_font_family.clone(),
                    color: self.series.title_color,
                    font_weight: self.series.title_font_weight,
                    ..Default::default()
                };

                let layout = create_text_layout(title_text, &title_font, None);
                let (x_offset, y_offset) = compute_text_offset(&layout, TextAlign::Center, TextBaseline::Middle);
                let final_position = Point::new(title_x + x_offset, title_y + y_offset);

                elements.push(VisualElement::TextRun {
                    text: title_text.to_string(),
                    position: final_position,
                    font_size: title_font.font_size,
                    font_family: title_font.font_family,
                    color: title_font.color,
                    font_weight: title_font.font_weight,
                    font_style: title_font.font_style,
                    align: TextAlign::Left,
                    baseline: TextBaseline::Top,
                    rotation: 0.0,
                    max_width: None,
                    layout: Some(layout),
                });
            }
        }

        if self.series.detail_show {
            let detail_text = format!("{:.1}", self.series.value);
            let detail_x = center_x + chart_width * self.series.detail_offset.0 / 100.0;
            let detail_y = center_y + chart_height * self.series.detail_offset.1 / 100.0;

            let detail_font = crate::model::TextStyle {
                font_size: self.series.detail_font_size,
                font_family: self.series.detail_font_family.clone(),
                color: self.series.detail_color,
                font_weight: self.series.detail_font_weight,
                ..Default::default()
            };

            let layout = create_text_layout(&detail_text, &detail_font, None);
            let (x_offset, y_offset) = compute_text_offset(&layout, TextAlign::Center, TextBaseline::Middle);
            let final_position = Point::new(detail_x + x_offset, detail_y + y_offset);

            elements.push(VisualElement::TextRun {
                text: detail_text,
                position: final_position,
                font_size: detail_font.font_size,
                font_family: detail_font.font_family,
                color: detail_font.color,
                font_weight: detail_font.font_weight,
                font_style: detail_font.font_style,
                align: TextAlign::Left,
                baseline: TextBaseline::Top,
                rotation: 0.0,
                max_width: None,
                layout: Some(layout),
            });
        }

        elements
    }
}

impl SeriesComponent for GaugeSeriesComponent {
    fn series_index(&self) -> usize {
        self.series_index
    }

    fn grid_index(&self) -> usize {
        self.grid_index
    }

    fn is_empty(&self) -> bool {
        false
    }
}

impl ChartComponent for GaugeSeriesComponent {
    fn build_visual_elements(&self, resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<VisualElement> {
        let ctx = match self.create_context(resolved, layout) {
            Some(ctx) => ctx,
            None => return Vec::new(),
        };
        self.build_with_context(&ctx)
    }
}