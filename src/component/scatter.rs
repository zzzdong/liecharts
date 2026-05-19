use crate::{
    component::{ChartComponent, SeriesComponent, SeriesContext},
    layout::LayoutOutput,
    model::{ChartModel, ScatterSeries},
    pipeline::{builder::VisualBuilder, mapper::MappedGeometry},
    visual::{Stroke, VisualElement},
};

pub struct ScatterSeriesComponent {
    series: ScatterSeries,
    series_index: usize,
    grid_index: usize,
}

impl ScatterSeriesComponent {
    pub fn new(series: &ScatterSeries, series_index: usize, grid_index: usize) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index,
        }
    }

    fn build_with_context(&self, ctx: &SeriesContext) -> Vec<VisualElement> {
        let coord = ctx.coord;

        // 散点图直接使用 X-Y 坐标，不需要复杂的变换流程
        // 直接将数据点映射到像素坐标
        let mapped: Vec<MappedGeometry> = self
            .series
            .data
            .iter()
            .map(|item| {
                let x = coord.x_value_to_pixel(item.x);
                let y = coord.y_to_pixel(item.y, self.series.y_axis_index);
                MappedGeometry::CartesianPoint { x, y }
            })
            .collect();

        // 构建视觉元素
        let color = ctx.get_series_color(self.series.item_style.color);

        let border_stroke = self.series.item_style.border_color.map(|c| Stroke {
            color: c,
            width: self.series.item_style.border_width,
        });

        let series_style = crate::pipeline::SeriesStyle {
            color,
            stroke: border_stroke,
            fill: Some(color),
        };

        // 创建模拟的 TransformedSeries 供 builder 使用
        let transformed = crate::pipeline::transform::TransformedSeries {
            series_index: self.series_index,
            items: self
                .series
                .data
                .iter()
                .enumerate()
                .map(|(i, item)| crate::pipeline::transform::TransformedItem {
                    original: crate::model::DataItem {
                        name: item.name.clone(),
                        value: item.y,
                        x_value: None,
                    },
                    display_value: item.y,
                    baseline: 0.0,
                    data_index: i,
                })
                .collect(),
        };

        let builder = crate::pipeline::builder::ScatterVisualBuilder::new()
            .with_symbol_size(self.series.symbol_size)
            .with_series_style(series_style);

        builder.build(&transformed, &mapped, coord)
    }
}

impl SeriesComponent for ScatterSeriesComponent {
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

impl ChartComponent for ScatterSeriesComponent {
    fn build_visual_elements(
        &self,
        resolved: &ChartModel,
        layout: &LayoutOutput,
    ) -> Vec<VisualElement> {
        let ctx = match self.create_context(resolved, layout) {
            Some(ctx) => ctx,
            None => return Vec::new(),
        };
        self.build_with_context(&ctx)
    }
}
