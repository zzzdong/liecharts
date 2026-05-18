//! 渲染模块 - 提供多种渲染后端

pub use pixmap::PixmapRenderer;
pub use svg::SvgRenderer;
use vello_cpu::kurbo::{BezPath, Point, Rect};

use crate::VisualElement;
use crate::text::TextLayout;
use crate::visual::{FillStrokeStyle, GradientDef, Stroke, StrokeStyle, Transform};

mod pixmap;
mod svg;

/// 渲染器 trait - 通用渲染后端需要实现的基本操作
///
/// 这是渲染后端的核心接口，每个具体后端（Pixmap、SVG等）都需要实现这些原子操作
pub trait Renderer {
    /// 绘制矩形
    fn draw_rect(&mut self, rect: Rect, style: &FillStrokeStyle);

    /// 绘制圆形
    fn draw_circle(&mut self, center: Point, radius: f64, style: &FillStrokeStyle);

    /// 绘制线段
    fn draw_line(&mut self, start: Point, end: Point, style: &StrokeStyle);

    /// 绘制折线
    fn draw_polyline(&mut self, points: &[Point], style: &StrokeStyle);

    /// 绘制路径
    fn draw_path(&mut self, path: &BezPath, style: &FillStrokeStyle);

    /// 绘制渐变填充路径
    fn draw_gradient_path(&mut self, path: &BezPath, gradient: &GradientDef, stroke: Option<&Stroke>);

    /// 绘制文本
    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        color: crate::visual::Color,
        font_size: f64,
        font_family: &str,
        rotation: f64,
        layout: Option<&TextLayout>,
    );

    /// 开始一个变换组
    fn begin_group(&mut self, transform: Option<&Transform>);

    /// 结束一个变换组
    fn end_group(&mut self);

    /// 渲染视觉元素序列的默认实现
    ///
    /// 遍历元素树并调用对应的原子操作
    fn render_elements(&mut self, elements: &[VisualElement])
    where
        Self: Sized,
    {
        for element in elements {
            self.render_element(element);
        }
    }

    /// 渲染单个视觉元素
    fn render_element(&mut self, element: &VisualElement)
    where
        Self: Sized,
    {
        match element {
            VisualElement::Rect { rect, style } => {
                self.draw_rect(*rect, style);
            }
            VisualElement::Circle {
                center,
                radius,
                style,
            } => {
                self.draw_circle(*center, *radius, style);
            }
            VisualElement::Line { start, end, style } => {
                self.draw_line(*start, *end, style);
            }
            VisualElement::Polyline { points, style } => {
                self.draw_polyline(points, style);
            }
            VisualElement::Path { path, style } => {
                self.draw_path(path, style);
            }
            VisualElement::GradientPath { path, gradient, stroke } => {
                self.draw_gradient_path(path, gradient, stroke.as_ref());
            }
            VisualElement::TextRun {
                text,
                position,
                style,
                rotation,
                layout,
                ..
            } => {
                self.draw_text(text, *position, style.color, style.font_size, &style.font_family, *rotation, layout.as_ref());
            }
            VisualElement::Group {
                children,
                transform,
            } => {
                self.begin_group(transform.as_ref());
                self.render_elements(children);
                self.end_group();
            }
        }
    }
}
