use crate::component::{
    AxisComponent, BarSeriesComponent, BubbleSeriesComponent, CandlestickSeriesComponent, ChartComponent, GaugeSeriesComponent, LegendComponent, LineSeriesComponent, PieSeriesComponent, PolarBarSeriesComponent, PolarScatterSeriesComponent, RadarSeriesComponent, ScatterSeriesComponent, TableSeriesComponent, TitleComponent,
};
use crate::error::{ChartError, Result};
use crate::layout::{
    AxisLayout, ChartLayout, DataCoordinateSystem, GridLayout, GridLayoutInfo, LayoutContext,
    LayoutEngine, LayoutOutput, Layoutable, LegendLayout, SubplotLayout, TitleLayout,
};
use crate::model::{Axis, AxisType, ResolvedOption, ResolvedSeries};
use crate::option::LieChartOption;
use crate::render::{PixmapRenderer, SvgRenderer};
use crate::theme::{Theme, ThemeRegistry};
use crate::visual::{FillStrokeStyle, VisualElement};
use vello_cpu::kurbo::Rect;
pub use vello_cpu::Pixmap;

pub struct LieChart {
    width: u32,
    height: u32,
    resolved: Option<ResolvedOption>,
    theme_registry: ThemeRegistry,
}

impl LieChart {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            resolved: None,
            theme_registry: ThemeRegistry::new(),
        }
    }

    pub fn set_option(&mut self, option: LieChartOption, theme: Option<&Theme>) -> Result<()> {
        let theme = theme.or_else(|| {
            option
                .theme
                .as_ref()
                .and_then(|name| self.theme_registry.get(name))
        });

        self.resolved = Some(ResolvedOption::merge(option, theme)?);
        Ok(())
    }

    pub fn set_option_json(&mut self, json: &str, theme: Option<&Theme>) -> Result<()> {
        let option: LieChartOption = serde_json::from_str(json)?;
        self.set_option(option, theme)
    }

    /// 构建图表的所有视觉元素
    fn build_visual_elements(&self, resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        // 背景
        elements.push(VisualElement::Rect {
            rect: Rect::new(0.0, 0.0, self.width as f64, self.height as f64),
            style: FillStrokeStyle {
                fill: Some(resolved.background),
                stroke: None,
            },
        });

        // 标题
        if let Some(title) = &resolved.title {
            let comp = TitleComponent::new(title);
            elements.extend(comp.build_visual_elements(resolved, layout));
        }

        // 图例
        if let Some(legend) = &resolved.legend {
            let comp = LegendComponent::new(legend);
            elements.extend(comp.build_visual_elements(resolved, layout));
        }

        // 构建子图上下文并渲染每个子图
        let subplots = build_subplot_contexts(resolved, layout);
        for subplot in &subplots {
            elements.extend(subplot.build_visual_elements(resolved, layout));
        }

        elements
    }

    fn compute_layout(&self, resolved: &ResolvedOption) -> LayoutOutput {
        let context = LayoutContext::new(self.width as f64, self.height as f64);
        let mut engine = LayoutEngine::new(context);

        let title = resolved.title.as_ref().map(|t| {
            Box::new(TitleLayout::new(
                t.text.clone(),
                t.subtext.clone(),
                t.text_style.clone(),
                t.subtext_style.clone(),
                t.left.clone(),
                t.top.clone(),
            )) as Box<dyn Layoutable>
        });

        let legend = resolved.legend.as_ref().and_then(|l| {
            if l.show {
                Some(Box::new(LegendLayout::new(
                    l.data.clone(),
                    l.orient,
                    l.left.clone(),
                    l.top.clone(),
                    l.text_style.clone(),
                )) as Box<dyn Layoutable>)
            } else {
                None
            }
        });

        // 为每个 grid 创建子图布局
        let mut subplots: Vec<SubplotLayout> = Vec::new();
        
        for (grid_index, grid) in resolved.grids.iter().enumerate() {
            // 收集属于当前 grid 的坐标轴
            let x_axes: Vec<Box<dyn Layoutable>> = resolved.x_axes.iter()
                .filter(|axis| axis.grid_index == grid_index)
                .map(|axis| {
                    Box::new(AxisLayout::new(axis.clone())) as Box<dyn Layoutable>
                }).collect();

            let y_axes: Vec<Box<dyn Layoutable>> = resolved.y_axes.iter()
                .filter(|axis| axis.grid_index == grid_index)
                .map(|axis| {
                    Box::new(AxisLayout::new(axis.clone())) as Box<dyn Layoutable>
                }).collect();

            subplots.push(SubplotLayout {
                grid_index,
                grid: Box::new(GridLayout::new()) as Box<dyn Layoutable>,
                x_axes,
                y_axes,
                left: grid.left.clone(),
                right: grid.right.clone(),
                top: grid.top.clone(),
                bottom: grid.bottom.clone(),
            });
        }

        let mut chart_layout = ChartLayout {
            title,
            legend,
            subplots,
        };

        let mut output = engine.layout(&mut chart_layout);
        
        // 为每个 grid 计算数据坐标系
        for grid_info in &mut output.grids {
            let grid_index = grid_info.grid_index;
            grid_info.data_coord = compute_data_coord_for_grid(resolved, grid_info, grid_index);
        }
        
        output
    }

    /// 构建并返回图表的所有视觉元素及尺寸
    ///
    /// 这是渲染管线的核心方法，将配置和布局转化为视觉元素列表，
    /// 开发者可以获取元素后自行传入任意 Renderer 实例进行渲染。
    pub fn collect_visual_elements(&self) -> Result<(Vec<VisualElement>, u32, u32)> {
        let resolved = self.resolved.as_ref().ok_or(ChartError::NoOption)?;
        let layout = self.compute_layout(resolved);
        let elements = self.build_visual_elements(resolved, &layout);
        Ok((elements, self.width, self.height))
    }

    /// 渲染图表到图片文件
    pub fn render_to_image(&self, path: &str) -> Result<()> {
        let (elements, width, height) = self.collect_visual_elements()?;
        let renderer = PixmapRenderer::new(width, height);
        let pixmap = renderer.render(&elements)?;

        // 将 pixmap 数据转换为 image 格式并保存
        let width = pixmap.width() as u32;
        let height = pixmap.height() as u32;
        let data: Vec<u8> = pixmap.data()
            .iter()
            .flat_map(|p| vec![p.r, p.g, p.b, p.a])
            .collect();
        let image = image::RgbaImage::from_raw(width, height, data)
            .ok_or_else(|| ChartError::RenderError("Failed to create image".to_string()))?;
        image.save(path)?;
        Ok(())
    }

    /// 渲染图表到 SVG 文件
    pub fn render_to_svg(&self, path: &str) -> Result<()> {
        let (elements, width, height) = self.collect_visual_elements()?;
        let renderer = SvgRenderer::new();
        let svg = renderer.render(&elements, width, height)?;
        std::fs::write(path, svg)?;
        Ok(())
    }

    pub fn register_theme(&mut self, theme: Theme) {
        self.theme_registry.register(theme);
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

/// 子图上下文 - 在布局阶段将 axes 和 series 按 grid_index 分配到各子图，
/// 消除后续渲染阶段重复的 grid_index 过滤逻辑。
struct SubplotContext {
    grid_index: usize,
    #[allow(dead_code)]
    grid_info: GridLayoutInfo,
    x_axes: Vec<Axis>,
    y_axes: Vec<Axis>,
    series: Vec<(usize, ResolvedSeries)>,
}

impl SubplotContext {
    fn build_visual_elements(&self, resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        for (local_idx, axis) in self.x_axes.iter().enumerate() {
            let comp = AxisComponent::new(axis, true, local_idx, self.grid_index);
            elements.extend(comp.build_visual_elements(resolved, layout));
        }

        for (local_idx, axis) in self.y_axes.iter().enumerate() {
            let comp = AxisComponent::new(axis, false, local_idx, self.grid_index);
            elements.extend(comp.build_visual_elements(resolved, layout));
        }

        for (global_idx, series) in &self.series {
            match series {
                ResolvedSeries::Bar(s) => {
                    let comp = BarSeriesComponent::new(s, *global_idx, self.grid_index);
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
                ResolvedSeries::Line(s) => {
                    let comp = LineSeriesComponent::new(s, *global_idx, self.grid_index);
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
                ResolvedSeries::Pie(s) => {
                    let comp = PieSeriesComponent::new(s, *global_idx);
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
                ResolvedSeries::Scatter(s) => {
                    let comp = ScatterSeriesComponent::new(s, *global_idx, self.grid_index);
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
                ResolvedSeries::Radar(s) => {
                    let comp = RadarSeriesComponent::new(s, *global_idx, resolved.radar.as_ref());
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
                ResolvedSeries::PolarBar(s) => {
                    let comp = PolarBarSeriesComponent::new(s, *global_idx);
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
                ResolvedSeries::PolarScatter(s) => {
                    let comp = PolarScatterSeriesComponent::new(s, *global_idx);
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
                ResolvedSeries::Bubble(s) => {
                    let comp = BubbleSeriesComponent::new(s, *global_idx, self.grid_index);
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
                ResolvedSeries::Gauge(s) => {
                    let comp = GaugeSeriesComponent::new(s, *global_idx);
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
                ResolvedSeries::Candlestick(s) => {
                    let comp = CandlestickSeriesComponent::new(s, *global_idx, self.grid_index);
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
                ResolvedSeries::Table(s) => {
                    let comp = TableSeriesComponent::new(s, *global_idx);
                    elements.extend(comp.build_visual_elements(resolved, layout));
                }
            }
        }
        
        elements
    }
}


fn series_grid_index(series: &ResolvedSeries) -> usize {
    match series {
        ResolvedSeries::Bar(s) => s.grid_index,
        ResolvedSeries::Line(s) => s.grid_index,
        ResolvedSeries::Scatter(s) => s.grid_index,
        ResolvedSeries::Pie(s) => s.grid_index,
        ResolvedSeries::Radar(_) => 0,
        ResolvedSeries::PolarBar(_) => 0,
        ResolvedSeries::PolarScatter(_) => 0,
        ResolvedSeries::Bubble(s) => s.grid_index,
        ResolvedSeries::Gauge(_) => 0,
        ResolvedSeries::Candlestick(s) => s.grid_index,
        ResolvedSeries::Table(s) => s.grid_index,
    }
}

/// 构建子图上下文列表 - 将 axes 和 series 按 grid_index 分配到各子图
fn build_subplot_contexts(resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<SubplotContext> {
    layout.grids.iter().map(|grid_info| {
        let grid_index = grid_info.grid_index;

        let x_axes = resolved.x_axes.iter()
            .filter(|axis| axis.grid_index == grid_index)
            .cloned()
            .collect();

        let y_axes = resolved.y_axes.iter()
            .filter(|axis| axis.grid_index == grid_index)
            .cloned()
            .collect();

        let series = resolved.series.iter()
            .enumerate()
            .filter(|(_, s)| series_grid_index(s) == grid_index)
            .map(|(i, s)| (i, s.clone()))
            .collect();

        SubplotContext {
            grid_index,
            grid_info: grid_info.clone(),
            x_axes,
            y_axes,
            series,
        }
    }).collect()
}

/// 为指定 grid 计算数据坐标系
fn compute_data_coord_for_grid(
    resolved: &ResolvedOption,
    grid_info: &GridLayoutInfo,
    grid_index: usize,
) -> DataCoordinateSystem {
    let plot_bounds = grid_info.grid_inner_bbox;

    // 获取属于当前 grid 的坐标轴
    let x_axes: Vec<_> = resolved.x_axes.iter()
        .filter(|axis| axis.grid_index == grid_index)
        .collect();
    let y_axes: Vec<_> = resolved.y_axes.iter()
        .filter(|axis| axis.grid_index == grid_index)
        .collect();

    // 构建全局Y轴索引到局部Y轴索引的映射
    // 系列中的 y_axis_index 是全局索引，需要转换为当前 grid 内的局部索引
    let global_to_local_y: std::collections::HashMap<usize, usize> = resolved.y_axes.iter()
        .enumerate()
        .filter(|(_, axis)| axis.grid_index == grid_index)
        .enumerate()
        .map(|(local, (global, _))| (global, local))
        .collect();

    // 为每个Y轴计算数据范围
    let mut y_axis_values: Vec<Vec<f64>> = vec![Vec::new(); y_axes.len().max(1)];
    let mut y_axis_stack_groups: Vec<std::collections::HashMap<Option<String>, Vec<Vec<f64>>>> = 
        vec![std::collections::HashMap::new(); y_axes.len().max(1)];
    // 标记每个Y轴是否包含需要从0开始的系列（柱状图、面积图）
    let mut y_axis_needs_zero_base: Vec<bool> = vec![false; y_axes.len().max(1)];
    
    // 收集X轴数据范围（用于数值型X轴）
    let mut x_axis_values: Vec<f64> = Vec::new();
    
    // 收集属于当前 grid 的系列数据到对应的Y轴
    for series in &resolved.series {
        // 只处理属于当前 grid 的系列
        let series_grid_index = series_grid_index(series);
        if series_grid_index != grid_index {
            continue;
        }
        
        // 提取数据信息，并判断是否需要从0开始
        let (values, stack, y_axis_index, x_values, needs_zero_base) = match series {
            ResolvedSeries::Bar(s) => {
                let vals: Vec<f64> = s.data.iter().map(|item| item.value).collect();
                (vals, s.stack.clone(), s.y_axis_index, None, true)
            }
            ResolvedSeries::Line(s) => {
                let vals: Vec<f64> = s.data.iter().map(|item| item.value).collect();
                // 有面积样式的折线图需要从0开始
                let has_area = s.area_style.is_some();
                (vals, s.stack.clone(), s.y_axis_index, None, has_area)
            }
            ResolvedSeries::Scatter(s) => {
                let vals: Vec<f64> = s.data.iter().map(|item| item.value).collect();
                (vals, None, s.y_axis_index, None, false)
            }
            ResolvedSeries::Bubble(s) => {
                let y_vals: Vec<f64> = s.data.iter().map(|b| b.y).collect();
                let x_vals: Vec<f64> = s.data.iter().map(|b| b.x).collect();
                (y_vals, None, s.y_axis_index, Some(x_vals), false)
            }
            ResolvedSeries::Candlestick(s) => {
                let vals: Vec<f64> = s.data.iter().flat_map(|c| vec![c.high, c.low]).collect();
                (vals, None, s.y_axis_index, None, true)
            }
            _ => continue,
        };
        
        // 收集X轴数值
        if let Some(x_vals) = x_values {
            x_axis_values.extend(x_vals);
        }
        
        // 将全局Y轴索引转换为当前 grid 内的局部索引
        let local_y_axis_index = global_to_local_y.get(&y_axis_index)
            .copied()
            .unwrap_or(0)
            .min(y_axis_values.len() - 1);
        
        // 标记是否需要从0开始
        if needs_zero_base {
            y_axis_needs_zero_base[local_y_axis_index] = true;
        }
        
        if let Some(ref stack_name) = stack {
            y_axis_stack_groups[local_y_axis_index]
                .entry(Some(stack_name.clone()))
                .or_default()
                .push(values.clone());
        }
        
        y_axis_values[local_y_axis_index].extend(values);
    }
    
    // 计算每个Y轴的数据范围
    let y_ranges: Vec<(f64, f64)> = y_axes.iter().enumerate().map(|(i, axis)| {
        let values = &y_axis_values[i];
        let needs_zero_base = y_axis_needs_zero_base[i];
        
        let mut max_stacked_value = 0.0f64;
        for group_values in y_axis_stack_groups[i].values() {
            let data_len = group_values.first().map(|v| v.len()).unwrap_or(0);
            for j in 0..data_len {
                let sum: f64 = group_values.iter().map(|v| v.get(j).copied().unwrap_or(0.0)).sum();
                max_stacked_value = max_stacked_value.max(sum);
            }
        }
        
        let (data_min, data_max) = if values.is_empty() {
            (0.0, 100.0)
        } else {
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let max = max.max(max_stacked_value);
            if min == max {
                (min - 10.0, max + 10.0)
            } else {
                (min, max)
            }
        };
        
        match axis.axis_type {
            AxisType::Category => {
                let count = axis.data.as_ref().map(|d| d.len()).unwrap_or(0);
                if axis.boundary_gap {
                    (0.0, count as f64)
                } else {
                    (0.0, (count - 1) as f64)
                }
            }
            _ => {
                // 对于柱状图和面积图，如果没有指定min，默认从0开始
                let min = axis.min.unwrap_or_else(|| {
                    if needs_zero_base && data_min >= 0.0 {
                        0.0
                    } else {
                        // 给 Y 轴添加 5% 的边距，防止数据点紧贴轴线
                        let range = data_max - data_min;
                        let margin_min = if range > 0.0 {
                            data_min - range * 0.05
                        } else {
                            data_min - 1.0
                        };
                        // 全正数据不穿入负半轴
                        if data_min > 0.0 && margin_min < 0.0 {
                            0.0
                        } else {
                            margin_min
                        }
                    }
                });
                let max = axis.max.unwrap_or_else(|| {
                    let range = data_max - data_min;
                    if range > 0.0 {
                        data_max + range * 0.05
                    } else {
                        data_max + 1.0
                    }
                });
                (min, max)
            }
        }
    }).collect();

    let (is_category_x, category_count, x_range) = x_axes.first()
        .map(|axis| {
            match axis.axis_type {
                AxisType::Category => {
                    let count = axis.data.as_ref().map(|d| d.len()).unwrap_or(0);
                    let range = if axis.boundary_gap {
                        (0.0, count as f64)
                    } else {
                        (0.0, (count - 1) as f64)
                    };
                    (true, count, range)
                }
                _ => {
                    // 使用收集到的X轴数据计算范围
                    let (data_min, data_max) = if x_axis_values.is_empty() {
                        // 如果没有X轴数据，使用Y轴数据作为回退（保持向后兼容）
                        let all_values: Vec<f64> = y_axis_values.iter().flatten().copied().collect();
                        let data_min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
                        let data_max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        (data_min, data_max)
                    } else {
                        let data_min = x_axis_values.iter().cloned().fold(f64::INFINITY, f64::min);
                        let data_max = x_axis_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        (data_min, data_max)
                    };
                    let min = axis.min.unwrap_or(data_min);
                    let max = axis.max.unwrap_or(data_max);
                    (false, 0, (min, max))
                }
            }
        })
        .unwrap_or((false, 0, (0.0, 100.0)));

    let is_category_y = y_axes.first()
        .map(|axis| matches!(axis.axis_type, AxisType::Category))
        .unwrap_or(false);

    DataCoordinateSystem::new(
        plot_bounds,
        x_range,
        y_ranges,
        is_category_x,
        is_category_y,
        category_count,
    )
}
