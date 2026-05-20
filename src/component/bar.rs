use crate::{
    component::{ChartComponent, SeriesComponent, SeriesContext},
    layout::LayoutOutput,
    model::{BarSeries, ChartModel},
    pipeline::{
        builder::{BarVisualBuilder, VisualBuilder},
        mapper::{CartesianBarMapper, CoordinateMapper},
        transform::IdentityTransformer,
    },
    visual::{Stroke, VisualElement},
};

pub struct BarSeriesComponent {
    series: BarSeries,
    series_index: usize,
    grid_index: usize,
}

impl BarSeriesComponent {
    pub fn new(series: &BarSeries, series_index: usize, grid_index: usize) -> Self {
        Self {
            series: series.clone(),
            series_index,
            grid_index,
        }
    }

    fn build_with_context(&self, ctx: &SeriesContext) -> Vec<VisualElement> {
        let coord = ctx.coord;

        // 1. Transform
        // 如果设置了 stack，使用 StackedTransformer 进行堆叠计算
        let transformer: Box<dyn crate::pipeline::transform::DataTransformer> =
            if self.series.stack.is_some() {
                Box::new(crate::pipeline::transform::StackedTransformer::new(
                    self.series.stack.clone(),
                ))
            } else {
                Box::new(IdentityTransformer)
            };

        let all_series = &ctx.resolved.series;
        let transformed_list = transformer.transform(all_series);
        let transformed = match transformed_list
            .iter()
            .find(|t| t.series_index == self.series_index)
        {
            Some(t) => t,
            None => return Vec::new(),
        };

        // 2. Map
        // 根据坐标轴类型选择类目尺寸（横向时用高度，纵向时用宽度）
        let bar_size_ratio = if coord.is_category_y {
            // 横向柱状图：Y轴为类目
            let cat_height = coord.category_height();
            let bar_height = self.series.bar_width.unwrap_or(cat_height * 0.6);
            if cat_height > 0.0 {
                bar_height / cat_height
            } else {
                0.6
            }
        } else {
            // 垂直柱状图：X轴为类目
            let cat_width = coord.category_width();
            let bar_width = self.series.bar_width.unwrap_or(cat_width * 0.6);
            if cat_width > 0.0 {
                bar_width / cat_width
            } else {
                0.6
            }
        };

        // 计算当前 grid 中柱状图系列的分组总数（用于分组并排）
        let group_count = ctx
            .resolved
            .series
            .iter()
            .filter(|s| match s {
                crate::model::ResolvedSeries::Bar(b) => b.grid_index == self.grid_index,
                _ => false,
            })
            .map(|s| match s {
                crate::model::ResolvedSeries::Bar(b) => b.group_index.unwrap_or(0),
                _ => 0,
            })
            .max()
            .unwrap_or(0)
            + 1;

        let group_index = self.series.group_index.unwrap_or(0);

        let mapper = CartesianBarMapper::new()
            .with_bar_width_ratio(bar_size_ratio)
            .with_group(group_index, group_count);
        let mapped = mapper.map(transformed, coord, self.series.y_axis_index);

        // 3. Build
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

        let label_config = if let Some(label) = &self.series.label {
            if label.show {
                use crate::pipeline::LabelPosition as PL;
                let pos = match label.position {
                    crate::model::LabelPosition::Top => PL::Top,
                    crate::model::LabelPosition::Inside => PL::Inside,
                    _ => PL::Top,
                };
                crate::pipeline::LabelConfig {
                    show: true,
                    position: pos,
                    color: label.color,
                    font_size: label.font_size,
                    font_family: label.font_family.clone(),
                    formatter: None,
                }
            } else {
                crate::pipeline::LabelConfig::default()
            }
        } else {
            crate::pipeline::LabelConfig::default()
        };

        let builder = BarVisualBuilder::new()
            .with_series_style(series_style)
            .with_label_config(label_config);

        builder.build(transformed, &mapped, coord)
    }
}

impl SeriesComponent for BarSeriesComponent {
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

impl ChartComponent for BarSeriesComponent {
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
