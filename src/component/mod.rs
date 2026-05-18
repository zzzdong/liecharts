use crate::layout::LayoutOutput;
use crate::model::ChartModel;
use crate::visual::VisualElement;

// 基础模块
pub mod base;
pub mod context;
pub mod renderers;

// 组件模块
pub mod axis;
pub mod bar;
pub mod bubble;
pub mod candlestick;
pub mod gauge;
pub mod label;
pub mod legend;
pub mod line;
pub mod pie;
pub mod polar_bar;
pub mod polar_scatter;
pub mod radar;
pub mod scatter;
pub mod table;
pub mod title;

// 基础模块导出
pub use base::{SeriesComponentBase, SeriesComponentExt};
pub use context::{CartesianRenderer, PolarRenderer, SeriesContext, SeriesRenderer};
pub use renderers::{
    LabelConfig, LabelPosition, SeriesStyle, color_utils, create_label_config, grid_utils,
    render_cartesian_pipeline, render_polar_pipeline,
};

// 组件导出
pub use axis::AxisComponent;
pub use bar::BarSeriesComponent;
pub use bubble::BubbleSeriesComponent;
pub use candlestick::CandlestickSeriesComponent;
pub use gauge::GaugeSeriesComponent;
pub use label::{LabelComponent, PieLeaderLineLabel, PieLeaderLineLabelBuilder};
pub use legend::LegendComponent;
pub use line::LineSeriesComponent;
pub use pie::PieSeriesComponent;
pub use polar_bar::PolarBarSeriesComponent;
pub use polar_scatter::PolarScatterSeriesComponent;
pub use radar::RadarSeriesComponent;
pub use scatter::ScatterSeriesComponent;
pub use table::TableSeriesComponent;
pub use title::TitleComponent;

/// 图表组件 trait
///
/// 所有图表组件（标题、图例、坐标轴、数据系列等）都实现此 trait，
/// 将配置和布局信息转换为视觉元素列表。
pub trait ChartComponent {
    fn build_visual_elements(
        &self,
        resolved: &ChartModel,
        layout: &LayoutOutput,
    ) -> Vec<VisualElement>;
}

/// 系列组件 trait - 所有数据系列组件的基础接口
///
/// 提供了系列组件的通用功能，包括 grid 信息获取、颜色管理等。
pub trait SeriesComponent: ChartComponent {
    /// 获取系列索引
    fn series_index(&self) -> usize;

    /// 获取 grid 索引
    fn grid_index(&self) -> usize;

    /// 检查数据是否为空
    fn is_empty(&self) -> bool;

    /// 获取 grid 信息
    fn get_grid_info<'a>(
        &self,
        layout: &'a LayoutOutput,
    ) -> Option<&'a crate::layout::GridLayoutInfo> {
        layout
            .grids
            .iter()
            .find(|g| g.grid_index == self.grid_index())
    }

    /// 创建渲染上下文
    fn create_context<'a>(
        &self,
        resolved: &'a ChartModel,
        layout: &'a LayoutOutput,
    ) -> Option<SeriesContext<'a>> {
        let grid_info = self.get_grid_info(layout)?;
        Some(SeriesContext::new(
            self.series_index(),
            self.grid_index(),
            resolved,
            layout,
            grid_info,
        ))
    }
}
