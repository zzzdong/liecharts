use crate::component::{ChartComponent, SeriesComponent, SeriesContext};
use crate::layout::LayoutOutput;
use crate::model::{PolarBarSeries, ResolvedOption};
use crate::visual::{Color, FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement};
use crate::text::create_text_layout;
use vello_cpu::kurbo::{Arc, BezPath, PathSeg, Point, Shape, Vec2};
use std::f64::consts::PI;

pub struct PolarBarSeriesComponent {
    series: PolarBarSeries,
    series_index: usize,
    grid_index: usize,
}

impl PolarBarSeriesComponent {
    pub fn new(series: &PolarBarSeries, series_index: usize) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index: 0,
        }
    }

    fn get_center_and_radius(&self, ctx: &SeriesContext) -> (Point, f64) {
        let plot_bounds = ctx.plot_bounds();
        let chart_width = plot_bounds.width();
        let chart_height = plot_bounds.height();
        let chart_x = plot_bounds.x0;
        let chart_y = plot_bounds.y0;

        let cx = chart_x + chart_width * 0.5;
        let cy = chart_y + chart_height * 0.5;
        let max_radius = chart_width.min(chart_height) / 2.0;

        (Point::new(cx, cy), max_radius)
    }

    fn build_grid(&self, center: Point, max_radius: f64) -> Vec<VisualElement> {
        let mut elements = Vec::new();
        let grid_color = Color::new(200, 200, 200);
        let stroke = Stroke::new(grid_color, 1.0);
        let stroke_style = StrokeStyle::new(grid_color, 1.0);

        let max_value = self.series.data.iter().map(|d| d.value).fold(0.0, f64::max);

        let split_number = 5;
        for i in 1..=split_number {
            let r = max_radius * (i as f64 / split_number as f64);
            elements.push(VisualElement::Circle {
                center,
                radius: r,
                style: FillStrokeStyle {
                    fill: None,
                    stroke: Some(stroke.clone()),
                },
            });

            let label_value = max_value * (i as f64 / split_number as f64);
            let label_text = format!("{:.0}", label_value);
            let label_pos = Point::new(center.x, center.y - r);

            let label_font = crate::model::TextStyle {
                font_size: 10.0,
                font_family: "sans-serif".to_string(),
                color: Color::new(100, 100, 100),
                font_weight: crate::option::FontWeight::Named(crate::option::FontWeightNamed::Normal),
                ..Default::default()
            };

            let layout_obj = create_text_layout(&label_text, &label_font, None);
            let th = layout_obj.height() as f64;

            elements.push(VisualElement::TextRun {
                text: label_text,
                position: Point::new(label_pos.x + 3.0, label_pos.y - th / 2.0),
                style: crate::model::TextStyle {
                    color: label_font.color,
                    font_size: label_font.font_size,
                    font_family: label_font.font_family,
                    font_weight: label_font.font_weight,
                    font_style: label_font.font_style,
                    align: TextAlign::Left,
                    vertical_align: TextBaseline::Top,
                },
                rotation: 0.0,
                max_width: None,
                layout: Some(layout_obj),
            });
        }

        let n = self.series.data.len();
        if n > 0 {
            let total_angle = 2.0 * PI;
            let pad_angle = self.series.pad_angle;
            let available_angle = total_angle - pad_angle * n as f64;
            let bar_angle = available_angle / n as f64;
            let start_angle = self.series.start_angle;

            for i in 0..n {
                let angle = start_angle + i as f64 * (bar_angle + pad_angle) + bar_angle / 2.0;
                let end_x = center.x + angle.cos() * max_radius;
                let end_y = center.y + angle.sin() * max_radius;

                elements.push(VisualElement::Line {
                    start: center,
                    end: Point::new(end_x, end_y),
                    style: stroke_style.clone(),
                });
            }
        }

        elements
    }

    fn build_bars(&self, center: Point, max_radius: f64) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        if self.series.data.is_empty() {
            return elements;
        }

        let n = self.series.data.len();
        let total_angle = 2.0 * PI;
        let pad_angle = self.series.pad_angle;
        let available_angle = total_angle - pad_angle * n as f64;
        let bar_angle = available_angle / n as f64;
        let start_angle = self.series.start_angle;

        let max_value = self.series.data.iter().map(|d| d.value).fold(0.0, f64::max);
        let radius_scale = if max_value > 0.0 { 1.0 } else { 0.0 };

        for (i, data_item) in self.series.data.iter().enumerate() {
            let angle_start = start_angle + i as f64 * (bar_angle + pad_angle);
            let angle_end = angle_start + bar_angle;

            let radius_ratio = if max_value > 0.0 {
                (data_item.value / max_value).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let outer_r = max_radius * radius_ratio * radius_scale;

            if outer_r < 1.0 {
                continue;
            }

            let color = self.series.colors.get(i % self.series.colors.len())
                .copied()
                .unwrap_or(Color::new(100, 100, 100));

            let mut path = BezPath::new();
            path.move_to(center);

            let outer_start = Point::new(
                center.x + angle_start.cos() * outer_r,
                center.y + angle_start.sin() * outer_r,
            );
            path.line_to(outer_start);

            let sweep_angle = angle_end - angle_start;
            if sweep_angle.abs() > 1e-12 {
                let arc = Arc::new(
                    center,
                    Vec2::new(outer_r, outer_r),
                    angle_start,
                    sweep_angle,
                    0.0,
                );
                arc.to_path(0.5).segments().for_each(|seg| {
                    match seg {
                        PathSeg::Line(line) => path.line_to(line.p1),
                        PathSeg::Quad(quad) => path.quad_to(quad.p1, quad.p2),
                        PathSeg::Cubic(cubic) => path.curve_to(cubic.p1, cubic.p2, cubic.p3),
                    }
                });
            }

            path.line_to(center);
            path.close_path();

            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: Some(Stroke::new(Color::new(255, 255, 255), 1.0)),
                },
            });
        }

        elements
    }
}

impl SeriesComponent for PolarBarSeriesComponent {
    fn series_index(&self) -> usize {
        self.series_index
    }

    fn grid_index(&self) -> usize {
        self.grid_index
    }

    fn is_empty(&self) -> bool {
        self.series.data.is_empty()
    }
}

impl ChartComponent for PolarBarSeriesComponent {
    fn build_visual_elements(&self, resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<VisualElement> {
        let ctx = match self.create_context(resolved, layout) {
            Some(ctx) => ctx,
            None => return Vec::new(),
        };

        let (center, max_radius) = self.get_center_and_radius(&ctx);

        let mut elements = Vec::new();
        elements.extend(self.build_grid(center, max_radius));
        elements.extend(self.build_bars(center, max_radius));

        elements
    }
}