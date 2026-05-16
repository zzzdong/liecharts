//! 数据管线 - 图表渲染的数据变换、坐标映射、视觉构建

use crate::visual::{Color, Stroke};

pub mod builder;
pub mod mapper;
pub mod transform;

pub use builder::{BarVisualBuilder, LineVisualBuilder, PieVisualBuilder, ScatterVisualBuilder, VisualBuilder};
pub use mapper::{CartesianBarMapper, CartesianLineMapper, CartesianScatterMapper, CoordinateMapper, MappedGeometry, PolarPieMapper};
pub use transform::{DataTransformer, IdentityTransformer, StackedTransformer, TransformedItem, TransformedSeries};

/// 系列样式
#[derive(Debug, Clone)]
pub struct SeriesStyle {
    pub color: Color,
    pub stroke: Option<Stroke>,
    pub fill: Option<Color>,
}

impl Default for SeriesStyle {
    fn default() -> Self {
        Self {
            color: Color::new(0, 0, 0),
            stroke: None,
            fill: None,
        }
    }
}

/// 标签配置
#[derive(Debug, Clone)]
pub struct LabelConfig {
    pub show: bool,
    pub position: LabelPosition,
    pub color: Color,
    pub font_size: f64,
    pub font_family: String,
    pub formatter: Option<fn(&str, f64) -> String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelPosition {
    Top,
    Left,
    Right,
    Bottom,
    Inside,
    Outside,
    Center,
}

impl Default for LabelConfig {
    fn default() -> Self {
        Self {
            show: false,
            position: LabelPosition::Top,
            color: Color::new(0, 0, 0),
            font_size: 12.0,
            font_family: "sans-serif".to_string(),
            formatter: None,
        }
    }
}
