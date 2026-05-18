use crate::component::{ChartComponent, SeriesComponent, SeriesContext};
use crate::layout::LayoutOutput;
use crate::model::{RadarConfig, RadarSeries, ResolvedOption};
use crate::visual::{Color, FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement};
use crate::text::create_text_layout;
use vello_cpu::kurbo::{BezPath, Point, Vec2};
use std::f64::consts::PI;

pub struct RadarSeriesComponent {
    series: RadarSeries,
    series_index: usize,
    grid_index: usize,
    radar_config: Option<RadarConfig>,
}

impl RadarSeriesComponent {
    pub fn new(series: &RadarSeries, series_index: usize, radar_config: Option<&RadarConfig>) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index: 0,
            radar_config: radar_config.cloned(),
        }
    }

    fn get_center_and_radius(&self, ctx: &SeriesContext) -> (Point, f64, f64) {
        let plot_bounds = ctx.plot_bounds();
        let chart_width = plot_bounds.width();
        let chart_height = plot_bounds.height();
        let chart_x = plot_bounds.x0;
        let chart_y = plot_bounds.y0;

        let config = self.radar_config.as_ref();

        let center_x = config.map(|c| c.center.0 / 100.0).unwrap_or(0.5);
        let center_y = config.map(|c| c.center.1 / 100.0).unwrap_or(0.5);
        let radius_inner = config.map(|c| c.radius.0 / 100.0).unwrap_or(0.0);
        let radius_outer = config.map(|c| c.radius.1 / 100.0).unwrap_or(0.75);

        let cx = chart_x + chart_width * center_x;
        let cy = chart_y + chart_height * center_y;
        let max_radius = chart_width.min(chart_height) / 2.0;
        let outer_r = max_radius * radius_outer;
        let inner_r = max_radius * radius_inner;

        (Point::new(cx, cy), inner_r, outer_r)
    }

    fn build_radar_grid(&self, center: Point, outer_r: f64, inner_r: f64) -> Vec<VisualElement> {
        let mut elements = Vec::new();
        let config = match &self.radar_config {
            Some(c) => c,
            None => return elements,
        };
        let n = config.indicator.len();
        if n == 0 {
            return elements;
        }

        let split_number = config.split_number;
        let angle_step = 2.0 * PI / n as f64;
        let start_angle = -PI / 2.0;

        let directions: Vec<Vec2> = (0..n)
            .map(|i| {
                let angle = start_angle + i as f64 * angle_step;
                Vec2::new(angle.cos(), angle.sin())
            })
            .collect();

        let grid_color = Color::new(200, 200, 200);

        for level in 1..=split_number {
            let r = inner_r + (outer_r - inner_r) * (level as f64 / split_number as f64);
            let mut points: Vec<Point> = directions
                .iter()
                .map(|dir| Point::new(center.x + dir.x * r, center.y + dir.y * r))
                .collect();
            points.push(points[0]);

            elements.push(VisualElement::Polyline {
                points,
                style: StrokeStyle {
                    color: grid_color,
                    width: 0.5,
                },
            });
        }

        for dir in &directions {
            let end = Point::new(center.x + dir.x * outer_r, center.y + dir.y * outer_r);
            elements.push(VisualElement::Line {
                start: center,
                end,
                style: StrokeStyle {
                    color: grid_color,
                    width: 0.5,
                },
            });
        }

        if config.name.show {
            for (i, indicator) in config.indicator.iter().enumerate() {
                let dir = directions[i];
                let label_pos = Point::new(
                    center.x + dir.x * (outer_r + 15.0),
                    center.y + dir.y * (outer_r + 15.0),
                );

                let align = if dir.x.abs() < 0.01 {
                    TextAlign::Center
                } else if dir.x > 0.0 {
                    TextAlign::Left
                } else {
                    TextAlign::Right
                };

                let baseline = if dir.y.abs() < 0.01 {
                    TextBaseline::Middle
                } else if dir.y > 0.0 {
                    TextBaseline::Top
                } else {
                    TextBaseline::Bottom
                };

                let name_font = &config.name.text_style;
                let layout_obj = create_text_layout(&indicator.name, name_font, None);
                let text_width = layout_obj.width() as f64;
                let text_height = layout_obj.height() as f64;

                let dx = match align {
                    TextAlign::Left => 0.0,
                    TextAlign::Center => -text_width / 2.0,
                    TextAlign::Right => -text_width,
                };
                let dy_adj = match baseline {
                    TextBaseline::Top => 0.0,
                    TextBaseline::Middle => -text_height / 2.0,
                    TextBaseline::Bottom => -text_height,
                    _ => 0.0,
                };

                elements.push(VisualElement::TextRun {
                    text: indicator.name.clone(),
                    position: Point::new(label_pos.x + dx, label_pos.y + dy_adj),
                    style: crate::model::TextStyle {
                        color: name_font.color,
                        font_size: name_font.font_size,
                        font_family: name_font.font_family.clone(),
                        font_weight: name_font.font_weight,
                        font_style: name_font.font_style,
                        align,
                        vertical_align: baseline,
                    },
                    rotation: 0.0,
                    max_width: None,
                    layout: Some(layout_obj),
                });
            }
        }

        elements
    }

    fn build_data_polygon(&self, center: Point, outer_r: f64, inner_r: f64) -> Vec<VisualElement> {
        let mut elements = Vec::new();
        let config = match &self.radar_config {
            Some(c) => c,
            None => return elements,
        };
        let n = config.indicator.len();
        if n == 0 || self.series.data.is_empty() {
            return elements;
        }

        let angle_step = 2.0 * PI / n as f64;
        let start_angle = -PI / 2.0;

        let directions: Vec<Vec2> = (0..n)
            .map(|i| {
                let angle = start_angle + i as f64 * angle_step;
                Vec2::new(angle.cos(), angle.sin())
            })
            .collect();

        for data_item in &self.series.data {
            let values = &data_item.value;
            if values.len() < n {
                continue;
            }

            let mut points: Vec<Point> = Vec::new();
            for (i, &val) in values.iter().enumerate().take(n) {
                let max_val = config.indicator[i].max;
                let ratio = if max_val > 0.0 { (val / max_val).clamp(0.0, 1.0) } else { 0.0 };
                let r = inner_r + (outer_r - inner_r) * ratio;
                let dir = directions[i];
                points.push(Point::new(center.x + dir.x * r, center.y + dir.y * r));
            }

            if let Some(area_style) = &self.series.area_style
                && points.len() >= 2 {
                    let mut path = BezPath::new();
                    path.move_to(points[0]);
                    for pt in points.iter().skip(1) {
                        path.line_to(*pt);
                    }
                    path.close_path();

                    elements.push(VisualElement::Path {
                        path,
                        style: FillStrokeStyle {
                            fill: Some(area_style.color.unwrap_or(self.series.color).set_alpha(area_style.opacity)),
                            stroke: None,
                        },
                    });
                }

            let mut line_points = points.clone();
            line_points.push(points[0]);
            elements.push(VisualElement::Polyline {
                points: line_points,
                style: StrokeStyle {
                    color: self.series.line_style.color,
                    width: self.series.line_style.width,
                },
            });

            for pt in &points {
                elements.push(VisualElement::Circle {
                    center: *pt,
                    radius: self.series.symbol_size,
                    style: FillStrokeStyle {
                        fill: Some(self.series.color),
                        stroke: Some(Stroke::new(Color::new(255, 255, 255), 1.0)),
                    },
                });
            }
        }

        elements
    }
}

impl SeriesComponent for RadarSeriesComponent {
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

impl ChartComponent for RadarSeriesComponent {
    fn build_visual_elements(&self, resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<VisualElement> {
        let ctx = match self.create_context(resolved, layout) {
            Some(ctx) => ctx,
            None => return Vec::new(),
        };

        let (center, inner_r, outer_r) = self.get_center_and_radius(&ctx);

        let mut elements = Vec::new();
        elements.extend(self.build_radar_grid(center, outer_r, inner_r));
        elements.extend(self.build_data_polygon(center, outer_r, inner_r));

        elements
    }
}