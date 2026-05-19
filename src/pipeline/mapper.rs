//! 坐标映射器 - 将变换后的数据映射到几何坐标

use vello_cpu::kurbo::Point;

use crate::{layout::DataCoordinateSystem, pipeline::transform::TransformedSeries};

/// 映射后的几何描述
#[derive(Debug, Clone)]
pub enum MappedGeometry {
    /// 笛卡尔坐标柱状图
    CartesianBar {
        center_x: f64,
        bottom_y: f64,
        top_y: f64,
        width: f64,
    },
    /// 笛卡尔坐标点
    CartesianPoint { x: f64, y: f64 },
    /// 笛卡尔坐标折线
    CartesianLine {
        points: Vec<Point>,
        /// 面积填充的基线（用于面积图）
        area_baseline: Option<Vec<Point>>,
    },
    /// 极坐标扇形（饼图）
    PolarSector {
        center: Point,
        inner_radius: f64,
        outer_radius: f64,
        start_angle: f64,
        sweep_angle: f64,
    },
}

/// 坐标映射器 trait
pub trait CoordinateMapper {
    fn map(
        &self,
        transformed: &TransformedSeries,
        coord: &DataCoordinateSystem,
        y_axis_index: usize,
    ) -> Vec<MappedGeometry>;
}

/// 笛卡尔坐标柱状图映射器
pub struct CartesianBarMapper {
    /// 柱子宽度（相对于类目宽度的比例）
    pub bar_width_ratio: f64,
}

impl Default for CartesianBarMapper {
    fn default() -> Self {
        Self {
            bar_width_ratio: 0.6,
        }
    }
}

impl CartesianBarMapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bar_width_ratio(mut self, ratio: f64) -> Self {
        self.bar_width_ratio = ratio;
        self
    }
}

impl CoordinateMapper for CartesianBarMapper {
    fn map(
        &self,
        transformed: &TransformedSeries,
        coord: &DataCoordinateSystem,
        y_axis_index: usize,
    ) -> Vec<MappedGeometry> {
        let cat_width = coord.category_width();
        let bar_width = cat_width * self.bar_width_ratio;
        let y_range = coord.get_y_range(y_axis_index);

        transformed
            .items
            .iter()
            .map(|item| {
                let center_x = coord.x_to_pixel(item.data_index as f64 + 0.5);
                let top_y = coord.y_to_pixel(item.display_value, y_axis_index);
                // 基线应该是 y_range 的最小值（如果 baseline 为 0 且不在范围内）
                // 或者是 baseline 本身（如果在范围内）
                let baseline_value = if item.baseline < y_range.0 {
                    y_range.0
                } else {
                    item.baseline
                };
                let bottom_y = coord.y_to_pixel(baseline_value, y_axis_index);

                MappedGeometry::CartesianBar {
                    center_x,
                    bottom_y,
                    top_y,
                    width: bar_width,
                }
            })
            .collect()
    }
}

/// 笛卡尔坐标折线图映射器
pub struct CartesianLineMapper {
    /// 是否平滑曲线
    pub smooth: bool,
    /// 是否生成面积基线
    pub has_area: bool,
}

impl CartesianLineMapper {
    pub fn new(smooth: bool) -> Self {
        Self {
            smooth,
            has_area: false,
        }
    }

    pub fn with_area(mut self, has_area: bool) -> Self {
        self.has_area = has_area;
        self
    }
}

impl CoordinateMapper for CartesianLineMapper {
    fn map(
        &self,
        transformed: &TransformedSeries,
        coord: &DataCoordinateSystem,
        y_axis_index: usize,
    ) -> Vec<MappedGeometry> {
        let points: Vec<Point> = transformed
            .items
            .iter()
            .map(|item| {
                let x = match item.original.x_value {
                    Some(xv) => coord.x_value_to_pixel(xv),
                    None => coord.x_to_pixel(item.data_index as f64 + 0.5),
                };
                Point::new(x, coord.y_to_pixel(item.display_value, y_axis_index))
            })
            .collect();

        // 面积填充的基线
        let area_baseline = if self.has_area {
            // 获取Y轴范围，用于确定基线位置
            let y_range = coord.get_y_range(y_axis_index);
            // 对于非堆叠面积图，基线应该是Y轴的最小值（而不是固定的0）
            // 这样可以确保面积填充不会延伸到图表区域之外
            let baseline_value = if transformed.items.iter().any(|item| item.baseline != 0.0) {
                // 堆叠图：使用每个点的baseline
                None
            } else {
                // 非堆叠图：使用Y轴最小值作为基线
                Some(y_range.0)
            };

            Some(
                transformed
                    .items
                    .iter()
                    .map(|item| {
                        let y = match baseline_value {
                            Some(b) => coord.y_to_pixel(b, y_axis_index),
                            None => coord.y_to_pixel(item.baseline, y_axis_index),
                        };
                        let x = match item.original.x_value {
                            Some(xv) => coord.x_value_to_pixel(xv),
                            None => coord.x_to_pixel(item.data_index as f64 + 0.5),
                        };
                        Point::new(x, y)
                    })
                    .collect(),
            )
        } else {
            None
        };

        vec![MappedGeometry::CartesianLine {
            points,
            area_baseline,
        }]
    }
}

/// 极坐标饼图映射器
pub struct PolarPieMapper {
    /// 中心点（相对于绘图区的百分比）
    pub center: (f64, f64),
    /// 内外半径（相对于绘图区最小边的百分比）
    pub radius: (f64, f64),
}

impl Default for PolarPieMapper {
    fn default() -> Self {
        Self {
            center: (50.0, 50.0),
            radius: (0.0, 75.0),
        }
    }
}

impl PolarPieMapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_center(mut self, x: f64, y: f64) -> Self {
        self.center = (x, y);
        self
    }

    pub fn with_radius(mut self, inner: f64, outer: f64) -> Self {
        self.radius = (inner, outer);
        self
    }
}

impl CoordinateMapper for PolarPieMapper {
    fn map(
        &self,
        transformed: &TransformedSeries,
        coord: &DataCoordinateSystem,
        _y_axis_index: usize,
    ) -> Vec<MappedGeometry> {
        let total: f64 = transformed
            .items
            .iter()
            .map(|item| item.original.value)
            .sum();

        if total <= 0.0 {
            return Vec::new();
        }

        let plot_bounds = coord.plot_bounds;
        let center_x = plot_bounds.x0 + plot_bounds.width() * self.center.0 / 100.0;
        let center_y = plot_bounds.y0 + plot_bounds.height() * self.center.1 / 100.0;
        let center = Point::new(center_x, center_y);

        let min_dim = plot_bounds.width().min(plot_bounds.height());
        let inner_radius = min_dim / 2.0 * self.radius.0 / 100.0;
        let outer_radius = min_dim / 2.0 * self.radius.1 / 100.0;

        let mut start_angle = -std::f64::consts::PI / 2.0;
        let mut geometries = Vec::new();

        for item in &transformed.items {
            let value = item.original.value;
            let angle = (value / total) * 2.0 * std::f64::consts::PI;

            geometries.push(MappedGeometry::PolarSector {
                center,
                inner_radius,
                outer_radius,
                start_angle,
                sweep_angle: angle,
            });

            start_angle += angle;
        }

        geometries
    }
}

/// 笛卡尔坐标散点图映射器
pub struct CartesianScatterMapper;

impl Default for CartesianScatterMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl CartesianScatterMapper {
    pub fn new() -> Self {
        Self
    }
}

impl CoordinateMapper for CartesianScatterMapper {
    fn map(
        &self,
        transformed: &TransformedSeries,
        coord: &DataCoordinateSystem,
        y_axis_index: usize,
    ) -> Vec<MappedGeometry> {
        transformed
            .items
            .iter()
            .map(|item| {
                let x = coord.x_to_pixel(item.data_index as f64 + 0.5);
                let y = coord.y_to_pixel(item.display_value, y_axis_index);

                MappedGeometry::CartesianPoint { x, y }
            })
            .collect()
    }
}
