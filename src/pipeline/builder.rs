//! 视觉构建器 - 将映射后的几何描述转换为 VisualElement 列表

use crate::component::label::PieLeaderLineLabel;
use crate::layout::DataCoordinateSystem;
use crate::pipeline::mapper::MappedGeometry;
use crate::pipeline::transform::TransformedSeries;
use crate::visual::{Color, FillStrokeStyle, Stroke, StrokeStyle, TextAlign, TextBaseline, VisualElement};
use crate::text::{compute_text_offset, create_text_layout};
use vello_cpu::kurbo::{BezPath, Point, Rect};
use vello_cpu::kurbo::Shape as KurboShape;

/// ECharts 风格默认调色板
const DEFAULT_PIE_COLORS: [Color; 10] = [
    Color::new(0x54, 0x7E, 0xC5),
    Color::new(0x91, 0xCC, 0x75),
    Color::new(0xEE, 0xE5, 0x5F),
    Color::new(0xFA, 0x8C, 0x35),
    Color::new(0x6E, 0x70, 0x7E),
    Color::new(0xBF, 0x5C, 0x5C),
    Color::new(0x8D, 0x6A, 0xBD),
    Color::new(0x5B, 0xA9, 0xAE),
    Color::new(0xE5, 0x7E, 0x9D),
    Color::new(0x7B, 0xCE, 0xA4),
];

/// 视觉构建器 trait
pub trait VisualBuilder {
    /// 构建系列的所有视觉元素
    fn build(&self, transformed: &TransformedSeries, mapped: &[MappedGeometry], coord: &DataCoordinateSystem) -> Vec<VisualElement>;
}

/// 柱状图视觉构建器
pub struct BarVisualBuilder {
    pub bar_width_ratio: f64,
    pub label_config: super::LabelConfig,
    pub series_style: super::SeriesStyle,
}

impl Default for BarVisualBuilder {
    fn default() -> Self {
        Self {
            bar_width_ratio: 0.6,
            label_config: super::LabelConfig::default(),
            series_style: super::SeriesStyle::default(),
        }
    }
}

impl BarVisualBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bar_width_ratio(mut self, ratio: f64) -> Self {
        self.bar_width_ratio = ratio;
        self
    }

    pub fn with_label_config(mut self, config: super::LabelConfig) -> Self {
        self.label_config = config;
        self
    }

    pub fn with_series_style(mut self, style: super::SeriesStyle) -> Self {
        self.series_style = style;
        self
    }
}

impl VisualBuilder for BarVisualBuilder {
    fn build(&self, transformed: &TransformedSeries, mapped: &[MappedGeometry], _coord: &DataCoordinateSystem) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        for (item, geom) in transformed.items.iter().zip(mapped.iter()) {
            let MappedGeometry::CartesianBar { center_x, bottom_y, top_y, width } = geom else {
                continue;
            };

            let x = center_x - width / 2.0;
            let bar_top = top_y.min(*bottom_y);
            let bar_height = (top_y - bottom_y).abs();

            let fill_color = self.series_style.fill.unwrap_or(self.series_style.color);

            elements.push(VisualElement::Rect {
                rect: Rect::new(x, bar_top, x + width, bar_top + bar_height),
                style: FillStrokeStyle {
                    fill: Some(fill_color),
                    stroke: self.series_style.stroke.clone(),
                },
            });

            // 标签
            if self.label_config.show {
                let label_y = match self.label_config.position {
                    super::LabelPosition::Top => bar_top - 5.0,
                    super::LabelPosition::Inside => bar_top + bar_height / 2.0,
                    _ => bar_top - 5.0,
                };

                let label_text = format!("{:.0}", item.original.value);
                let label_font = crate::model::TextStyle {
                    font_size: self.label_config.font_size,
                    font_family: "sans-serif".to_string(),
                    color: self.label_config.color,
                    font_weight: crate::option::FontWeight::Named(crate::option::FontWeightNamed::Normal),
                    ..Default::default()
                };

                let layout = create_text_layout(&label_text, &label_font, None);
                let (x_offset, y_offset) = compute_text_offset(&layout, TextAlign::Center, TextBaseline::Bottom);
                let final_position = Point::new(center_x + x_offset, label_y + y_offset);

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

        elements
    }
}

/// 折线图视觉构建器
pub struct LineVisualBuilder {
    pub smooth: bool,
    pub show_symbol: bool,
    pub symbol_size: f64,
    pub line_style: Stroke,
    pub area_color: Option<Color>,
    pub area_opacity: f64,
    pub label_config: super::LabelConfig,
    pub series_style: super::SeriesStyle,
}

impl Default for LineVisualBuilder {
    fn default() -> Self {
        Self {
            smooth: false,
            show_symbol: true,
            symbol_size: 6.0,
            line_style: Stroke {
                color: Color::new(0, 0, 0),
                width: 2.0,
            },
            area_color: None,
            area_opacity: 0.7,
            label_config: super::LabelConfig::default(),
            series_style: super::SeriesStyle::default(),
        }
    }
}

impl LineVisualBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_smooth(mut self, smooth: bool) -> Self {
        self.smooth = smooth;
        self
    }

    pub fn with_show_symbol(mut self, show: bool) -> Self {
        self.show_symbol = show;
        self
    }

    pub fn with_symbol_size(mut self, size: f64) -> Self {
        self.symbol_size = size;
        self
    }

    pub fn with_line_style(mut self, style: Stroke) -> Self {
        self.line_style = style;
        self
    }

    pub fn with_area(mut self, color: Option<Color>, opacity: f64) -> Self {
        self.area_color = color;
        self.area_opacity = opacity;
        self
    }

    pub fn with_series_style(mut self, style: super::SeriesStyle) -> Self {
        self.series_style = style;
        self
    }
}

impl VisualBuilder for LineVisualBuilder {
    fn build(&self, _transformed: &TransformedSeries, mapped: &[MappedGeometry], coord: &DataCoordinateSystem) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        let MappedGeometry::CartesianLine { points, area_baseline } = &mapped[0] else {
            return elements;
        };

        // 获取图表边界用于裁剪
        let bounds = coord.plot_bounds;

        // 面积填充
        if let Some(baseline) = area_baseline
            && let Some(mut area_color) = self.area_color {
                let alpha = (area_color.a as f64 * self.area_opacity).clamp(0.0, 255.0) as u8;
                area_color.a = alpha;

                let fill_path = build_area_fill_path(points, baseline, self.smooth, &bounds);
                elements.push(VisualElement::Path {
                    path: fill_path,
                    style: FillStrokeStyle {
                        fill: Some(area_color),
                        stroke: None,
                    },
                });
            }

        // 折线
        if points.len() >= 2 {
            if self.smooth {
                let path = catmull_rom_spline(points, 1.0, &bounds);
                elements.push(VisualElement::Path {
                    path,
                    style: FillStrokeStyle {
                        fill: None,
                        stroke: Some(self.line_style.clone()),
                    },
                });
            } else {
                // 对于非平滑折线，也需要裁剪到边界内
                let clipped_points: Vec<Point> = points.iter()
                    .map(|p| Point::new(
                        p.x.clamp(bounds.x0, bounds.x1),
                        p.y.clamp(bounds.y0, bounds.y1)
                    ))
                    .collect();
                elements.push(VisualElement::Polyline {
                    points: clipped_points,
                    style: StrokeStyle {
                        color: self.line_style.color,
                        width: self.line_style.width,
                    },
                });
            }
        }

        // 数据点符号
        if self.show_symbol {
            for point in points {
                elements.push(VisualElement::Circle {
                    center: *point,
                    radius: self.symbol_size / 2.0,
                    style: FillStrokeStyle {
                        fill: Some(self.series_style.color),
                        stroke: Some(Stroke {
                            color: Color::new(255, 255, 255),
                            width: 1.0,
                        }),
                    },
                });
            }
        }

        elements
    }
}

/// 构建面积填充的闭合路径
fn build_area_fill_path(top: &[Point], bottom: &[Point], smooth: bool, bounds: &Rect) -> BezPath {
    let mut path = BezPath::new();
    if top.is_empty() {
        return path;
    }

    // 上边线（正向）
    if smooth && top.len() >= 2 {
        let spline = catmull_rom_spline(top, 1.0, bounds);
        for (i, seg) in spline.segments().enumerate() {
            if i == 0 {
                match seg {
                    vello_cpu::kurbo::PathSeg::Line(l) => path.move_to(l.p0),
                    vello_cpu::kurbo::PathSeg::Quad(q) => path.move_to(q.p0),
                    vello_cpu::kurbo::PathSeg::Cubic(c) => path.move_to(c.p0),
                }
            }
            match seg {
                vello_cpu::kurbo::PathSeg::Line(l) => path.line_to(l.p1),
                vello_cpu::kurbo::PathSeg::Quad(q) => path.quad_to(q.p1, q.p2),
                vello_cpu::kurbo::PathSeg::Cubic(c) => path.curve_to(c.p1, c.p2, c.p3),
            }
        }
    } else {
        path.move_to(top[0]);
        for p in &top[1..] {
            path.line_to(*p);
        }
    }

    // 下边线（反向）
    if smooth && bottom.len() >= 2 {
        if let Some(last) = bottom.last() {
            path.line_to(*last);
        }
        let spline = catmull_rom_spline(bottom, 1.0, bounds);
        let segments: Vec<_> = spline.segments().collect();
        for seg in segments.iter().rev() {
            match seg {
                vello_cpu::kurbo::PathSeg::Line(l) => path.line_to(l.p0),
                vello_cpu::kurbo::PathSeg::Cubic(c) => path.curve_to(c.p2, c.p1, c.p0),
                _ => {}
            }
        }
    } else {
        for p in bottom.iter().rev() {
            path.line_to(*p);
        }
    }

    path.close_path();
    path
}

/// Catmull-Rom 样条
fn catmull_rom_spline(points: &[Point], tension: f64, bounds: &Rect) -> BezPath {
    let n = points.len();
    if n < 2 {
        return BezPath::new();
    }
    if n == 2 || tension <= 0.0 {
        let mut path = BezPath::new();
        path.move_to(points[0]);
        path.line_to(points[1]);
        return path;
    }

    let mut path = BezPath::new();
    path.move_to(points[0]);

    for i in 0..n - 1 {
        let p0 = if i == 0 {
            Point::new(2.0 * points[0].x - points[1].x, 2.0 * points[0].y - points[1].y)
        } else {
            points[i - 1]
        };

        let p1 = points[i];
        let p2 = points[i + 1];

        let p3 = if i + 2 >= n {
            Point::new(
                2.0 * points[n - 1].x - points[n - 2].x,
                2.0 * points[n - 1].y - points[n - 2].y,
            )
        } else {
            points[i + 2]
        };

        let mut cp1_x = p1.x + (p2.x - p0.x) / (6.0 * tension);
        let mut cp1_y = p1.y + (p2.y - p0.y) / (6.0 * tension);

        let mut cp2_x = p2.x - (p3.x - p1.x) / (6.0 * tension);
        let mut cp2_y = p2.y - (p3.y - p1.y) / (6.0 * tension);

        // 将控制点限制在边界内
        cp1_x = cp1_x.clamp(bounds.x0, bounds.x1);
        cp1_y = cp1_y.clamp(bounds.y0, bounds.y1);
        cp2_x = cp2_x.clamp(bounds.x0, bounds.x1);
        cp2_y = cp2_y.clamp(bounds.y0, bounds.y1);

        path.curve_to((cp1_x, cp1_y), (cp2_x, cp2_y), (p2.x, p2.y));
    }

    path
}

/// 饼图视觉构建器
pub struct PieVisualBuilder {
    pub label_config: super::LabelConfig,
    pub border_color: Color,
    pub border_width: f64,
    pub colors: Vec<Color>,
}

impl Default for PieVisualBuilder {
    fn default() -> Self {
        Self {
            label_config: super::LabelConfig::default(),
            border_color: Color::new(255, 255, 255),
            border_width: 1.0,
            colors: Vec::new(),
        }
    }
}

impl PieVisualBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_label_config(mut self, config: super::LabelConfig) -> Self {
        self.label_config = config;
        self
    }

    pub fn with_border(mut self, color: Color, width: f64) -> Self {
        self.border_color = color;
        self.border_width = width;
        self
    }

    pub fn with_colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self
    }
}

impl VisualBuilder for PieVisualBuilder {
    fn build(&self, transformed: &TransformedSeries, mapped: &[MappedGeometry], _coord: &DataCoordinateSystem) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        // 第一遍：预计算所有标签信息
        let mut label_infos: Vec<(usize, String, f64, f64, bool, f64)> = Vec::new();
        let mut max_text_width: f64 = 0.0;

        if self.label_config.show {
            for (i, (item, geom)) in transformed.items.iter().zip(mapped.iter()).enumerate() {
                let MappedGeometry::PolarSector { outer_radius, start_angle, sweep_angle, .. } = geom else {
                    continue;
                };

                let end_angle = start_angle + sweep_angle;
                let mid_angle = (start_angle + end_angle) / 2.0;
                let is_right_side = mid_angle.cos() >= 0.0;

                let label_text = if let Some(formatter) = self.label_config.formatter {
                    formatter(item.original.name.as_deref().unwrap_or(""), item.original.value)
                } else {
                    item.original.name.as_ref()
                        .map(|n| format!("{}: {:.0}", n, item.original.value))
                        .unwrap_or_else(|| format!("{:.0}", item.original.value))
                };

                let label_font = crate::model::TextStyle {
                    font_size: self.label_config.font_size,
                    font_family: "sans-serif".to_string(),
                    color: self.label_config.color,
                    font_weight: crate::option::FontWeight::Named(crate::option::FontWeightNamed::Normal),
                    ..Default::default()
                };

                let layout = create_text_layout(&label_text, &label_font, None);
                let text_width = layout.width() as f64;

                if text_width > max_text_width {
                    max_text_width = text_width;
                }

                label_infos.push((i, label_text, mid_angle, text_width, is_right_side, *outer_radius));
            }
        }

        // 第二遍：绘制扇形和标签
        for (i, (_item, geom)) in transformed.items.iter().zip(mapped.iter()).enumerate() {
            let MappedGeometry::PolarSector { center, inner_radius, outer_radius, start_angle, sweep_angle } = geom else {
                continue;
            };

            let end_angle = start_angle + sweep_angle;

            // 绘制扇形
            let mut path = BezPath::new();
            let x1 = center.x + outer_radius * start_angle.cos();
            let y1 = center.y + outer_radius * start_angle.sin();
            path.move_to(Point::new(x1, y1));

            let arc = vello_cpu::kurbo::Arc {
                center: *center,
                radii: (*outer_radius, *outer_radius).into(),
                start_angle: *start_angle,
                sweep_angle: *sweep_angle,
                x_rotation: 0.0,
            };
            arc.to_path(0.1).segments().for_each(|seg| {
                use vello_cpu::kurbo::PathSeg;
                match seg {
                    PathSeg::Line(line) => path.line_to(line.p1),
                    PathSeg::Quad(quad) => path.quad_to(quad.p1, quad.p2),
                    PathSeg::Cubic(cubic) => path.curve_to(cubic.p1, cubic.p2, cubic.p3),
                }
            });

            if *inner_radius > 0.0 {
                let x3 = center.x + inner_radius * end_angle.cos();
                let y3 = center.y + inner_radius * end_angle.sin();
                path.line_to(Point::new(x3, y3));

                let inner_arc = vello_cpu::kurbo::Arc {
                    center: *center,
                    radii: (*inner_radius, *inner_radius).into(),
                    start_angle: end_angle,
                    sweep_angle: -sweep_angle,
                    x_rotation: 0.0,
                };
                inner_arc.to_path(0.1).segments().for_each(|seg| {
                    use vello_cpu::kurbo::PathSeg;
                    match seg {
                        PathSeg::Line(line) => path.line_to(line.p1),
                        PathSeg::Quad(quad) => path.quad_to(quad.p1, quad.p2),
                        PathSeg::Cubic(cubic) => path.curve_to(cubic.p1, cubic.p2, cubic.p3),
                    }
                });
                path.line_to(Point::new(x1, y1));
            } else {
                path.line_to(*center);
                path.line_to(Point::new(x1, y1));
            }
            path.close_path();

            let color = self.colors.get(i).copied().unwrap_or_else(|| DEFAULT_PIE_COLORS[i % DEFAULT_PIE_COLORS.len()]);
            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: Some(Stroke {
                        color: self.border_color,
                        width: self.border_width,
                    }),
                },
            });

            // 绘制标签和引导线
            if self.label_config.show
                && let Some(label_info) = label_infos.iter().find(|(idx, _, _, _, _, _)| *idx == i) {
                    let (_, label_text, mid_angle, text_width, is_right_side, _) = label_info;
                    let mid_angle = *mid_angle;
                    let is_right_side = *is_right_side;
                    let text_width = *text_width;

                    // 使用 PieLeaderLineLabel 组件绘制标签和引导线
                    let label = PieLeaderLineLabel::new(
                        label_text.clone(),
                        *center,
                        *outer_radius,
                        mid_angle,
                        is_right_side,
                        text_width,
                    )
                    .with_label_config(&self.label_config);

                    elements.extend(label.build());
                }
        }

        elements
    }
}

/// 散点图视觉构建器
pub struct ScatterVisualBuilder {
    pub symbol_size: f64,
    pub series_style: super::SeriesStyle,
    pub label_config: super::LabelConfig,
}

impl Default for ScatterVisualBuilder {
    fn default() -> Self {
        Self {
            symbol_size: 8.0,
            series_style: super::SeriesStyle::default(),
            label_config: super::LabelConfig::default(),
        }
    }
}

impl ScatterVisualBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_symbol_size(mut self, size: f64) -> Self {
        self.symbol_size = size;
        self
    }

    pub fn with_series_style(mut self, style: super::SeriesStyle) -> Self {
        self.series_style = style;
        self
    }

    pub fn with_label_config(mut self, config: super::LabelConfig) -> Self {
        self.label_config = config;
        self
    }
}

impl VisualBuilder for ScatterVisualBuilder {
    fn build(&self, transformed: &TransformedSeries, mapped: &[MappedGeometry], _coord: &DataCoordinateSystem) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        for (item, geom) in transformed.items.iter().zip(mapped.iter()) {
            let MappedGeometry::CartesianPoint { x, y } = geom else {
                continue;
            };

            let fill_color = self.series_style.fill.unwrap_or(self.series_style.color);

            elements.push(VisualElement::Circle {
                center: Point::new(*x, *y),
                radius: self.symbol_size / 2.0,
                style: FillStrokeStyle {
                    fill: Some(fill_color),
                    stroke: self.series_style.stroke.clone(),
                },
            });

            // 标签
            if self.label_config.show {
                let label_text = format!("{:.0}", item.original.value);
                let label_font = crate::model::TextStyle {
                    font_size: self.label_config.font_size,
                    font_family: "sans-serif".to_string(),
                    color: self.label_config.color,
                    font_weight: crate::option::FontWeight::Named(crate::option::FontWeightNamed::Normal),
                    ..Default::default()
                };

                let layout = create_text_layout(&label_text, &label_font, None);
                let (x_offset, y_offset) = compute_text_offset(&layout, TextAlign::Center, TextBaseline::Bottom);
                let final_position = Point::new(x + x_offset, y - self.symbol_size / 2.0 - 3.0 + y_offset);

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

        elements
    }
}
