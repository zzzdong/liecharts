use crate::component::{ChartComponent, SeriesComponent, SeriesContext};
use crate::layout::LayoutOutput;
use crate::model::{ChartModel, PolarScatterSeries};
use crate::text::create_text_layout;
use crate::visual::{
    Color, FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement,
};
use std::f64::consts::PI;
use vello_cpu::kurbo::Point;

pub struct PolarScatterSeriesComponent {
    series: PolarScatterSeries,
    series_index: usize,
    grid_index: usize,
}

impl PolarScatterSeriesComponent {
    pub fn new(series: &PolarScatterSeries, series_index: usize) -> Self {
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

        let max_data_radius = self
            .series
            .data
            .iter()
            .map(|d| d.radius)
            .fold(0.0, f64::max)
            .max(1.0);

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

            let label_value = max_data_radius * (i as f64 / split_number as f64);
            let label_text = format!("{:.0}", label_value);
            let label_pos = Point::new(center.x, center.y - r);

            let label_font = crate::model::TextStyle {
                font_size: 10.0,
                font_family: "sans-serif".to_string(),
                color: Color::new(100, 100, 100),
                font_weight: crate::option::FontWeight::Named(
                    crate::option::FontWeightNamed::Normal,
                ),
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

        let directions = 8;
        for i in 0..directions {
            let angle = -PI / 2.0 + (i as f64 * 2.0 * PI / directions as f64);
            let end_x = center.x + angle.cos() * max_radius;
            let end_y = center.y + angle.sin() * max_radius;

            elements.push(VisualElement::Line {
                start: center,
                end: Point::new(end_x, end_y),
                style: stroke_style.clone(),
            });
        }

        elements
    }

    fn build_scatter_points(&self, center: Point, max_radius: f64) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        let max_data_radius = self
            .series
            .data
            .iter()
            .map(|d| d.radius)
            .fold(0.0, f64::max)
            .max(1.0);

        let color = self
            .series
            .item_style
            .color
            .unwrap_or(Color::new(84, 112, 198));

        for data_item in &self.series.data {
            let r = (data_item.radius / max_data_radius).clamp(0.0, 1.0) * max_radius;
            let angle = data_item.angle;

            let x = center.x + angle.cos() * r;
            let y = center.y + angle.sin() * r;

            let point_center = Point::new(x, y);
            let symbol_size = data_item.symbol_size;

            match self.series.symbol {
                crate::model::Symbol::Circle => {
                    elements.push(VisualElement::Circle {
                        center: point_center,
                        radius: symbol_size,
                        style: FillStrokeStyle {
                            fill: Some(color),
                            stroke: Some(Stroke::new(Color::new(255, 255, 255), 1.0)),
                        },
                    });
                }
                crate::model::Symbol::Rect => {
                    let half_size = symbol_size;
                    let rect = vello_cpu::kurbo::Rect::new(
                        point_center.x - half_size,
                        point_center.y - half_size,
                        point_center.x + half_size,
                        point_center.y + half_size,
                    );
                    elements.push(VisualElement::Rect {
                        rect,
                        style: FillStrokeStyle {
                            fill: Some(color),
                            stroke: Some(Stroke::new(Color::new(255, 255, 255), 1.0)),
                        },
                    });
                }
                crate::model::Symbol::Diamond => {
                    let mut path = vello_cpu::kurbo::BezPath::new();
                    path.move_to(Point::new(point_center.x, point_center.y - symbol_size));
                    path.line_to(Point::new(point_center.x + symbol_size, point_center.y));
                    path.line_to(Point::new(point_center.x, point_center.y + symbol_size));
                    path.line_to(Point::new(point_center.x - symbol_size, point_center.y));
                    path.close_path();
                    elements.push(VisualElement::Path {
                        path,
                        style: FillStrokeStyle {
                            fill: Some(color),
                            stroke: Some(Stroke::new(Color::new(255, 255, 255), 1.0)),
                        },
                    });
                }
                crate::model::Symbol::Triangle => {
                    let mut path = vello_cpu::kurbo::BezPath::new();
                    let sqrt3_2 = 0.86602540378;
                    path.move_to(Point::new(point_center.x, point_center.y - symbol_size));
                    path.line_to(Point::new(
                        point_center.x + symbol_size * sqrt3_2,
                        point_center.y + symbol_size * 0.5,
                    ));
                    path.line_to(Point::new(
                        point_center.x - symbol_size * sqrt3_2,
                        point_center.y + symbol_size * 0.5,
                    ));
                    path.close_path();
                    elements.push(VisualElement::Path {
                        path,
                        style: FillStrokeStyle {
                            fill: Some(color),
                            stroke: Some(Stroke::new(Color::new(255, 255, 255), 1.0)),
                        },
                    });
                }
                _ => {
                    elements.push(VisualElement::Circle {
                        center: point_center,
                        radius: symbol_size,
                        style: FillStrokeStyle {
                            fill: Some(color),
                            stroke: Some(Stroke::new(Color::new(255, 255, 255), 1.0)),
                        },
                    });
                }
            }
        }

        elements
    }
}

impl SeriesComponent for PolarScatterSeriesComponent {
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

impl ChartComponent for PolarScatterSeriesComponent {
    fn build_visual_elements(
        &self,
        resolved: &ChartModel,
        layout: &LayoutOutput,
    ) -> Vec<VisualElement> {
        let ctx = match self.create_context(resolved, layout) {
            Some(ctx) => ctx,
            None => return Vec::new(),
        };

        let (center, max_radius) = self.get_center_and_radius(&ctx);

        let mut elements = Vec::new();
        elements.extend(self.build_grid(center, max_radius));
        elements.extend(self.build_scatter_points(center, max_radius));

        elements
    }
}
