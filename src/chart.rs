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
    theme_registry: ThemeRegistry,
}

impl LieChart {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            theme_registry: ThemeRegistry::new(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme_registry.register(theme);
        self
    }

    fn resolve_option(&self, option: LieChartOption) -> Result<ResolvedOption> {
        let theme = option
            .theme
            .as_ref()
            .and_then(|name| self.theme_registry.get(name))
            .cloned();
        ResolvedOption::merge(option, theme)
    }

    fn build_visual_elements(&self, resolved: &ResolvedOption, layout: &LayoutOutput) -> Vec<VisualElement> {
        let mut elements = Vec::new();

        elements.push(VisualElement::Rect {
            rect: Rect::new(0.0, 0.0, self.width as f64, self.height as f64),
            style: FillStrokeStyle {
                fill: Some(resolved.background),
                stroke: None,
            },
        });

        if let Some(title) = &resolved.title {
            let comp = TitleComponent::new(title);
            elements.extend(comp.build_visual_elements(resolved, layout));
        }

        if let Some(legend) = &resolved.legend {
            let comp = LegendComponent::new(legend);
            elements.extend(comp.build_visual_elements(resolved, layout));
        }

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

        let mut subplots: Vec<SubplotLayout> = Vec::new();

        for (grid_index, grid) in resolved.grids.iter().enumerate() {
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

        for grid_info in &mut output.grids {
            let grid_index = grid_info.grid_index;
            grid_info.data_coord = compute_data_coord_for_grid(resolved, grid_info, grid_index);
        }

        output
    }

    pub fn collect_visual_elements(&self, option: LieChartOption)
        -> Result<(Vec<VisualElement>, u32, u32)>
    {
        let resolved = self.resolve_option(option)?;
        let layout = self.compute_layout(&resolved);
        let elements = self.build_visual_elements(&resolved, &layout);
        Ok((elements, self.width, self.height))
    }

    pub fn render_to_image(&self, option: LieChartOption, path: &str) -> Result<()> {
        let (elements, width, height) = self.collect_visual_elements(option)?;
        let renderer = PixmapRenderer::new(width, height);
        let pixmap = renderer.render(&elements)?;

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

    pub fn render_to_svg(&self, option: LieChartOption, path: &str) -> Result<()> {
        let (elements, width, height) = self.collect_visual_elements(option)?;
        let renderer = SvgRenderer::new();
        let svg = renderer.render(&elements, width, height)?;
        std::fs::write(path, svg)?;
        Ok(())
    }

    pub fn render_png(&self, option: LieChartOption) -> Result<Vec<u8>> {
        let (elements, width, height) = self.collect_visual_elements(option)?;
        let renderer = PixmapRenderer::new(width, height);
        let pixmap = renderer.render(&elements)?;

        let data: Vec<u8> = pixmap.data()
            .iter()
            .flat_map(|p| vec![p.r, p.g, p.b, p.a])
            .collect();
        let image = image::RgbaImage::from_raw(pixmap.width() as u32, pixmap.height() as u32, data)
            .ok_or_else(|| ChartError::RenderError("Failed to create PNG image".to_string()))?;

        let mut buf = Vec::new();
        image.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
        Ok(buf)
    }

    pub fn render_svg(&self, option: LieChartOption) -> Result<String> {
        let (elements, width, height) = self.collect_visual_elements(option)?;
        let renderer = SvgRenderer::new();
        let svg = renderer.render(&elements, width, height)?;
        Ok(svg)
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

fn compute_data_coord_for_grid(
    resolved: &ResolvedOption,
    grid_info: &GridLayoutInfo,
    grid_index: usize,
) -> DataCoordinateSystem {
    let _plot_bounds = grid_info.grid_inner_bbox;

    let x_axes: Vec<_> = resolved.x_axes.iter()
        .filter(|axis| axis.grid_index == grid_index)
        .collect();
    let y_axes: Vec<_> = resolved.y_axes.iter()
        .filter(|axis| axis.grid_index == grid_index)
        .collect();

    let global_to_local_y: std::collections::HashMap<usize, usize> = resolved.y_axes.iter()
        .enumerate()
        .filter(|(_, axis)| axis.grid_index == grid_index)
        .enumerate()
        .map(|(local, (global, _))| (global, local))
        .collect();

    let mut y_axis_values: Vec<Vec<f64>> = vec![Vec::new(); y_axes.len().max(1)];
    let mut y_axis_stack_groups: Vec<std::collections::HashMap<Option<String>, Vec<Vec<f64>>>> =
        vec![std::collections::HashMap::new(); y_axes.len().max(1)];
    let mut y_axis_needs_zero_base: Vec<bool> = vec![false; y_axes.len().max(1)];

    let mut x_axis_values: Vec<f64> = Vec::new();

    for series in &resolved.series {
        let series_grid_index = series_grid_index(series);
        if series_grid_index != grid_index {
            continue;
        }

        let (values, stack, y_axis_index, x_values, needs_zero_base) = match series {
            ResolvedSeries::Bar(s) => {
                let vals: Vec<f64> = s.data.iter().map(|item| item.value).collect();
                (vals, s.stack.clone(), s.y_axis_index, None, true)
            }
            ResolvedSeries::Line(s) => {
                let vals: Vec<f64> = s.data.iter().map(|item| item.value).collect();
                let has_area = s.area_style.is_some();
                (vals, s.stack.clone(), s.y_axis_index, None, has_area)
            }
            ResolvedSeries::Scatter(s) => {
                let y_vals: Vec<f64> = s.data.iter().map(|item| item.y).collect();
                let x_vals: Vec<f64> = s.data.iter().map(|item| item.x).collect();
                (y_vals, None, s.y_axis_index, Some(x_vals), false)
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

        if let Some(x_vals) = x_values {
            x_axis_values.extend(x_vals);
        }

        let local_y_axis_index = global_to_local_y.get(&y_axis_index)
            .copied()
            .unwrap_or(0)
            .min(y_axis_values.len() - 1);

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

        let compute_value_range = |data_min: f64, data_max: f64, needs_zero_base: bool| {
            let min = axis.min.unwrap_or_else(|| {
                if needs_zero_base && data_min >= 0.0 {
                    0.0
                } else {
                    let range = data_max - data_min;
                    if range > 0.0 {
                        data_min - range * 0.05
                    } else {
                        data_min - 1.0
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
        };

        match axis.axis_type {
            AxisType::Category => {
                let count = axis.data.as_ref().map(|d| d.len()).unwrap_or(0);
                if count > 0 {
                    if axis.boundary_gap {
                        (0.0, count as f64)
                    } else {
                        (0.0, (count - 1) as f64)
                    }
                } else {
                    // Category axis without data (e.g., y-axis defaulting to Category type):
                    // fall back to value-based range from series data
                    compute_value_range(data_min, data_max, needs_zero_base)
                }
            }
            _ => compute_value_range(data_min, data_max, needs_zero_base),
        }
    }).collect();

    let x_range = if x_axes.is_empty() {
        (0.0, 1.0)
    } else {
        let axis = x_axes[0];
        match axis.axis_type {
            AxisType::Category => {
                let count = axis.data.as_ref().map(|d| d.len()).unwrap_or(0);
                if axis.boundary_gap {
                    (0.0, count as f64)
                } else {
                    (0.0, (count - 1) as f64)
                }
            }
            AxisType::Value => {
                if x_axis_values.is_empty() {
                    (0.0, 100.0)
                } else {
                    let min = axis.min.unwrap_or_else(|| {
                        let m = x_axis_values.iter().cloned().fold(f64::INFINITY, f64::min);
                        let range = x_axis_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                            - m;
                        if range > 0.0 { m - range * 0.05 } else { m - 1.0 }
                    });
                    let max = axis.max.unwrap_or_else(|| {
                        let m = x_axis_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        let range = m - x_axis_values.iter().cloned().fold(f64::INFINITY, f64::min);
                        if range > 0.0 { m + range * 0.05 } else { m + 1.0 }
                    });
                    (min, max)
                }
            }
            _ => (0.0, 1.0),
        }
    };

    DataCoordinateSystem {
        x_range,
        y_ranges,
        plot_bounds: grid_info.grid_inner_bbox,
        is_category_x: x_axes.first().map(|a| matches!(a.axis_type, AxisType::Category)).unwrap_or(false),
        category_count: x_axes.first()
            .and_then(|a| a.data.as_ref().map(|d| d.len()))
            .unwrap_or(0),
    }
}