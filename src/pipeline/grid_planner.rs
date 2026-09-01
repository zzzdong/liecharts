use vello_cpu::kurbo::Rect;

use crate::pipeline::{
    axis_label::{AxisLabelSet, auto_rotate, measure_labels, rotated_bounds},
    types::{
        AxisPosition, AxisSpec, AxisType, ColorContext, FitMode, GridEdge, GridSpec, SeriesSpec,
        SubplotSpec, TextMeasurer,
    },
};

/// 一个 subplot 的**纯绑定关系**（不含任何像素信息）
///
/// 由 [`GridPlanner::bind`] 在像素布局之前产出。轴范围解析
/// （`AxisBindingResolver`）只关心"哪些系列/轴属于哪个 subplot"，
/// 与 subplot 最终的像素边界无关，因此这一步可以提前，
/// 使"刻度标签文本 → 文本测量 → 像素布局"形成单向依赖链。
#[derive(Debug, Clone, Default)]
pub struct SubplotBinding {
    pub id: usize,
    pub series_indices: Vec<usize>,
    pub x_axis_indices: Vec<usize>,
    pub y_axis_indices: Vec<usize>,
}

/// 像素布局阶段的输入（画布尺寸由 `GridPlanner` 自身持有）
pub struct LayoutInput<'a> {
    pub bindings: Vec<SubplotBinding>,
    pub x_axes: &'a [AxisSpec],
    pub y_axes: &'a [AxisSpec],
    /// 预先生成的轴标签文本（measure 阶段产物）
    pub labels: &'a AxisLabelSet,
    pub colors: &'a ColorContext,
    /// 画布尺寸语义：Fixed = 只能向内收缩；Hug = 空间不足时上报需求、由
    /// 调用方扩画布后重跑（见 [`GridPlanner::plan`]）
    pub fit_mode: FitMode,
}

/// 单个 subplot 的空间需求（P1 需求上报回路的载体）
///
/// 仅 [`FitMode::Hug`] 会产生非默认值，[`FitMode::Fixed`] 恒为默认。
///
/// `grid_*` 是需要写回 `GridSpec` 的**目标边距**（P2b 起直接为绝对像素
/// `GridEdge::Px`，不再折算 `LABEL_*_PADDING`——resolve_position 已无叠加）。
/// `grow_*` 是画布需向该侧扩大的像素量（= 目标边距 − 当前边距）。
#[derive(Debug, Clone, Copy, Default)]
pub struct SubplotDemand {
    pub grid_left: Option<GridEdge>,
    pub grid_right: Option<GridEdge>,
    pub grid_top: Option<GridEdge>,
    pub grid_bottom: Option<GridEdge>,
    pub grow_left: f64,
    pub grow_right: f64,
    pub grow_top: f64,
    pub grow_bottom: f64,
}

impl SubplotDemand {
    pub fn has_shortfall(&self) -> bool {
        self.grid_left.is_some()
            || self.grid_right.is_some()
            || self.grid_top.is_some()
            || self.grid_bottom.is_some()
            || self.grow_left > 0.0
            || self.grow_right > 0.0
            || self.grow_top > 0.0
            || self.grow_bottom > 0.0
    }

    /// 记录目标边距（多轴同侧时取最大，绝对值）
    fn set_left(&mut self, margin_value: f64, grow: f64) {
        let best = match self.grid_left {
            Some(GridEdge::Px(v)) => v.max(margin_value),
            _ => margin_value,
        };
        self.grid_left = Some(GridEdge::Px(best));
        self.grow_left = self.grow_left.max(grow);
    }

    fn set_right(&mut self, margin_value: f64, grow: f64) {
        let best = match self.grid_right {
            Some(GridEdge::Px(v)) => v.max(margin_value),
            _ => margin_value,
        };
        self.grid_right = Some(GridEdge::Px(best));
        self.grow_right = self.grow_right.max(grow);
    }

    fn set_top(&mut self, margin_value: f64, grow: f64) {
        let best = match self.grid_top {
            Some(GridEdge::Px(v)) => v.max(margin_value),
            _ => margin_value,
        };
        self.grid_top = Some(GridEdge::Px(best));
        self.grow_top = self.grow_top.max(grow);
    }

    fn set_bottom(&mut self, margin_value: f64, grow: f64) {
        let best = match self.grid_bottom {
            Some(GridEdge::Px(v)) => v.max(margin_value),
            _ => margin_value,
        };
        self.grid_bottom = Some(GridEdge::Px(best));
        self.grow_bottom = self.grow_bottom.max(grow);
    }
}

/// [`GridPlanner::plan`] 的输出
pub struct PlanOutput {
    pub specs: Vec<SubplotSpec>,
    /// 与 `specs` 下标对齐的空间需求
    pub demands: Vec<SubplotDemand>,
}

/// 纯数学画布切分器
///
/// 职责分两阶段：
/// 1. [`Self::bind`]：把 series/axis 按 grid_index 绑定到 subplot（**无像素**）
/// 2. [`Self::plan`]：根据 grid 配置和画布尺寸，计算每个 subplot 的像素边界
///
/// 阶段 2 只依赖：画布尺寸、grid 配置、轴标签的实测尺寸。
/// **不接触系列数据、刻度计算**。
///
/// header_height 参数告诉 planner 顶部有多少空间被标题/图例占据，
/// 确保 subplot 不会与这些装饰元素重叠。
pub struct GridPlanner<'a> {
    total_width: u32,
    total_height: u32,
    header_height: f64,
    grids: &'a [GridSpec],
}

impl<'a> GridPlanner<'a> {
    pub fn new(width: u32, height: u32, header_height: f64, grids: &'a [GridSpec]) -> Self {
        Self {
            total_width: width,
            total_height: height,
            header_height: header_height.max(0.0),
            grids,
        }
    }

    /// 绑定阶段：把 series / x_axes / y_axes 按 grid_index 归到各 subplot
    ///
    /// 结果不含像素信息，可安全地在轴范围解析之前调用。
    pub fn bind(
        &self,
        series: &[SeriesSpec],
        x_axes: &[AxisSpec],
        y_axes: &[AxisSpec],
    ) -> Vec<SubplotBinding> {
        let n = self.grids.len().max(1);
        let mut bindings: Vec<SubplotBinding> = (0..n)
            .map(|idx| SubplotBinding {
                id: idx,
                ..Default::default()
            })
            .collect();

        for (series_idx, series) in series.iter().enumerate() {
            if series.grid_index < bindings.len() {
                bindings[series.grid_index]
                    .series_indices
                    .push(series_idx);
            }
        }
        for (axis_idx, axis) in x_axes.iter().enumerate() {
            if axis.grid_index < bindings.len() {
                bindings[axis.grid_index].x_axis_indices.push(axis_idx);
            }
        }
        for (axis_idx, axis) in y_axes.iter().enumerate() {
            if axis.grid_index < bindings.len() {
                bindings[axis.grid_index].y_axis_indices.push(axis_idx);
            }
        }

        bindings
    }

    /// 像素布局：计算每个 subplot 的边界，并按实测标签尺寸自适应调整边距
    ///
    /// 两种模式（`input.fit_mode`）：
    /// - **Fixed**：空间不足只能收缩绘图区（历史行为），收缩到 `MIN_PLOT_*`
    ///   仍不够时放弃（标签重叠），需求恒为零。
    /// - **Hug**：空间不足时**不收缩**，把目标边距与缺口写入 `demands`，
    ///   由调用方扩画布 + 扩边距后重跑（通常 1~2 轮收敛：画布变大后
    ///   `slot_w` 变宽，旋转/抽稀决策可能随之放松，需求单调递减）。
    pub fn plan(&self, input: &LayoutInput<'_>, measurer: &mut TextMeasurer) -> PlanOutput {
        let mut specs: Vec<SubplotSpec> = if self.grids.is_empty() {
            vec![SubplotSpec {
                id: 0,
                bounds: self.default_bounds(),
                series_indices: Vec::new(),
                x_axis_indices: Vec::new(),
                y_axis_indices: Vec::new(),
            }]
        } else {
            self.grids
                .iter()
                .enumerate()
                .map(|(idx, grid)| SubplotSpec {
                    id: idx,
                    bounds: self.resolve_position(grid),
                    series_indices: Vec::new(),
                    x_axis_indices: Vec::new(),
                    y_axis_indices: Vec::new(),
                })
                .collect()
        };

        // 套用绑定阶段的结果（两者数量一致，均由 grids 决定）
        for (spec, binding) in specs.iter_mut().zip(input.bindings.iter()) {
            spec.series_indices = binding.series_indices.clone();
            spec.x_axis_indices = binding.x_axis_indices.clone();
            spec.y_axis_indices = binding.y_axis_indices.clone();
        }

        // 根据坐标轴标签的**实测**占用空间自适应调整 subplot 边界，
        // 避免密集/长文本标签（尤其旋转后）超出画布或被截断。
        let mut demands = self.adjust_label_margins(
            &mut specs,
            input.x_axes,
            input.y_axes,
            input.labels,
            input.colors,
            input.fit_mode,
            measurer,
        );

        // Fixed 语义：画布不可变，需求恒零（调用方不消费）
        if input.fit_mode == FitMode::Fixed {
            demands = vec![SubplotDemand::default(); demands.len()];
        }

        PlanOutput { specs, demands }
    }

    /// 根据坐标轴标签尺寸自适应调整边距。
    ///
    /// 与 `CartesianAxisRenderer` 共用同一套旋转决策：
    /// - X 轴标签横向放不下时自动旋转（45°/90°），按旋转后的投影高度预留底部空间
    /// - Y 轴标签按宽度预留左侧/右侧空间
    ///
    /// 文本尺寸由 [`measure_labels`] 实测得到（parley），与渲染阶段同源；
    /// 标签文本来自 [`AxisLabelSet`]，与 `CartesianAxisRenderer` 逐字一致。
    ///
    /// 早期版本此处使用 `estimate_text_size` 启发式估算、数值轴用 `"1234.5"`
    /// 占位串顶替，导致预留边距与实绘结果错位（见 docs/布局自适应改造计划.md P0）。
    fn adjust_label_margins(
        &self,
        specs: &mut [SubplotSpec],
        x_axes: &[AxisSpec],
        y_axes: &[AxisSpec],
        labels: &AxisLabelSet,
        colors: &ColorContext,
        fit_mode: FitMode,
        measurer: &mut TextMeasurer,
    ) -> Vec<SubplotDemand> {
        const X_LABEL_GAP: f64 = 14.0; // 锚点距坐标轴的距离
        const Y_LABEL_GAP: f64 = 8.0; // 锚点距坐标轴的距离
        const LABEL_PAD: f64 = 4.0; // 额外安全边距
        const MIN_PLOT_W: f64 = 50.0;
        const MIN_PLOT_H: f64 = 40.0;
        let hug = matches!(fit_mode, FitMode::Hug | FitMode::HugMax);

        let total_w = self.total_width as f64;
        let total_h = self.total_height as f64;

        let mut demands: Vec<SubplotDemand> = specs
            .iter()
            .map(|_| SubplotDemand::default())
            .collect();

        for (si, spec) in specs.iter_mut().enumerate() {
            let demand = &mut demands[si];
            let mut grow_bottom: f64 = 0.0;
            let mut grow_top: f64 = 0.0;
            let mut grow_left: f64 = 0.0;
            let mut grow_right: f64 = 0.0;

            // ── X 轴：按旋转后的投影高度预留顶部/底部空间 ──
            for &axis_idx in &spec.x_axis_indices {
                let Some(axis) = x_axes.get(axis_idx) else {
                    continue;
                };
                if !axis.label_show {
                    continue;
                }
                let axis_labels = labels.x_labels(axis_idx);
                if axis_labels.is_empty() {
                    continue;
                }
                let n = axis_labels.len();
                let slot_w = spec.bounds.width() / n as f64;
                let (max_w, max_h) = measure_labels(axis_labels, measurer, colors);
                let rotation = axis
                    .label_rotate
                    .map(|deg| deg.to_radians())
                    .unwrap_or_else(|| auto_rotate(max_w, max_h, slot_w));
                let (_, rotated_h) = rotated_bounds(max_w, max_h, rotation);
                let needed = X_LABEL_GAP + rotated_h + LABEL_PAD;

                // P3：Hug 下自动旋转意味着标签横排放不下——优先扩宽画布
                // 让标签保持水平（信息零损失），而非旋转/抽稀。用户显式
                // 配置的 label_rotate 不干预。
                if hug && axis.label_rotate.is_none() && rotation != 0.0 {
                    let needed_width = n as f64 * max_w;
                    let current_width = spec.bounds.width();
                    if needed_width > current_width {
                        let grow = needed_width - current_width;
                        // 只加宽画布、保持边距不变（grid_* 不写回）
                        demand.grow_left += grow * 0.5;
                        demand.grow_right += grow * 0.5;
                    }
                }

                if axis.position == AxisPosition::Top {
                    // 顶部 X 轴：标签在绘图区上方，且不能侵入标题/图例占用的头部空间，
                    // 可用空间 = 绘图区上缘到画布顶部的距离减去 header_height
                    let current = (spec.bounds.y0 - self.header_height).max(0.0);
                    if needed > current {
                        grow_top = grow_top.max(needed - current);
                        if hug {
                            // 目标 y0 = needed + header_height（resolve_position 无 padding）
                            demand.set_top(needed + self.header_height, needed - current);
                        }
                    }
                } else {
                    let current = total_h - spec.bounds.y1;
                    if needed > current {
                        grow_bottom = grow_bottom.max(needed - current);
                        if hug {
                            demand.set_bottom(needed, needed - current);
                        }
                    }
                }
            }

            // ── Y 轴：按标签宽度预留左侧/右侧空间 ──
            for &axis_idx in &spec.y_axis_indices {
                let Some(axis) = y_axes.get(axis_idx) else {
                    continue;
                };
                if !axis.label_show {
                    continue;
                }
                let axis_labels = labels.y_labels(axis_idx);
                if axis_labels.is_empty() {
                    continue;
                }
                let (max_w, max_h) = measure_labels(axis_labels, measurer, colors);
                // Y 轴不自动旋转，仅尊重用户配置
                let rotation = axis.label_rotate.map(|deg| deg.to_radians()).unwrap_or(0.0);
                let (rotated_w, _) = rotated_bounds(max_w, max_h, rotation);
                // 预留 Y 轴名称空间：名称（旋转后厚度 ≈ 15px）+ 与刻度标签 8px 间隙
                // + 名称锚点定位余量（AxisRenderer 侧标签锚点同步右移 30px，
                // 见 `CartesianAxisRenderer::draw_y_tick_labels_side`），
                // 避免轴名称与标签重叠。
                let name_extra: f64 = if axis.name.is_some() { 34.0 } else { 0.0 };
                let needed = Y_LABEL_GAP + rotated_w + LABEL_PAD + name_extra;

                // P3：Hug 下 Y 轴 category 标签过多导致抽稀（label_step > 1）
                // 时，加高画布保持全部标签可见（信息零损失）。用户显式
                // label_rotate 时可能是有意纵向排布，不干预。
                if hug
                    && axis.label_rotate.is_none()
                    && axis.axis_type == AxisType::Category
                {
                    let n = axis_labels.len();
                    if n > 0 {
                        let needed_height = n as f64 * max_h;
                        let current_height = spec.bounds.height();
                        if needed_height > current_height {
                            let grow = needed_height - current_height;
                            demand.grow_top += grow * 0.5;
                            demand.grow_bottom += grow * 0.5;
                        }
                    }
                }

                let is_right = axis.position == AxisPosition::Right;
                if is_right {
                    let current = total_w - spec.bounds.x1;
                    if needed > current {
                        grow_right = grow_right.max(needed - current);
                        if hug {
                            demand.set_right(needed, needed - current);
                        }
                    }
                } else {
                    let current = spec.bounds.x0;
                    if needed > current {
                        grow_left = grow_left.max(needed - current);
                        if hug {
                            demand.set_left(needed, needed - current);
                        }
                    }
                }
            }

            if hug {
                // Hug：不收缩绘图区（保持理想形状），缺口由调用方扩画布 +
                // 扩边距后重跑消化。收敛后边界自然放得下标签。
                continue;
            }

            // Fixed：应用收缩（保留最小绘图尺寸；放不下则放弃，即历史行为）
            if grow_left > 0.0 || grow_right > 0.0 {
                let new_x0 = spec.bounds.x0 + grow_left;
                let new_x1 = spec.bounds.x1 - grow_right;
                if new_x1 - new_x0 >= MIN_PLOT_W {
                    spec.bounds.x0 = new_x0;
                    spec.bounds.x1 = new_x1;
                }
            }
            if grow_bottom > 0.0 {
                let new_y1 = spec.bounds.y1 - grow_bottom;
                if new_y1 - spec.bounds.y0 >= MIN_PLOT_H {
                    spec.bounds.y1 = new_y1;
                }
            }
            if grow_top > 0.0 {
                let new_y0 = spec.bounds.y0 + grow_top;
                if spec.bounds.y1 - new_y0 >= MIN_PLOT_H {
                    spec.bounds.y0 = new_y0;
                }
            }
        }

        demands
    }

    /// 无 grid 配置时的默认区域
    ///
    /// left/right/bottom: 留 60px 边距供轴标签和名称使用
    /// top: 使用 header_height（若无标题/图例则回退 60px）
    fn default_bounds(&self) -> Rect {
        let total_w = self.total_width as f64;
        let total_h = self.total_height as f64;
        let margin = 60.0;
        let top = self.header_height.max(margin);
        Rect::new(margin, top, total_w - margin, total_h - margin)
    }

    /// 根据 GridSpec 计算 subplot 像素边界
    ///
    /// - 当用户显式指定 left/right/top/bottom 时，直接使用
    /// - 当值为 None（auto）时：
    ///   - top 使用 header_height（标题/图例空间）
    ///   - 若 contain_label=true，left/bottom 使用更大默认值以容纳轴标签
    ///   - 否则使用标准边距
    ///
    /// 注意：用户指定的 bottom 值被视为子图整体（含坐标轴标签）的底部边距，
    /// 因此计算图表区域时会额外减去标签占用的空间（约 28px），
    /// 确保坐标轴标签不会被画布底部截断。
    ///
    /// P2b：边距以 [`GridEdge`] 延迟解析（`Pct` 相对画布，随画布缩放跟随）；
    /// 移除了对显式边距叠加的 `LABEL_*_PADDING` —— 标签留白统一由
    /// [`Self::adjust_label_margins`] 按实测需求预留（Fixed 收缩绘图区，
    /// Hug 上报需求），避免"用户 10% 边距被悄悄扩成 10%+50px"的不一致。
    fn resolve_position(&self, grid: &GridSpec) -> Rect {
        let total_w = self.total_width as f64;
        let total_h = self.total_height as f64;

        // 根据 contain_label 决定默认边距
        // contain_label=true 时，边距需要足够容纳轴刻度标签
        let default_left = if grid.contain_label { 70.0 } else { 60.0 };
        let default_right = if grid.contain_label { 50.0 } else { 60.0 };
        let default_bottom = 60.0;

        let left = resolve_edge(grid.left, default_left, total_w);
        let right = resolve_edge(grid.right, default_right, total_w);
        let top = resolve_edge(grid.top, self.header_height.max(40.0), total_h);
        let bottom = resolve_edge(grid.bottom, default_bottom, total_h);

        let width = (total_w - left - right).max(0.0);
        let height = (total_h - top - bottom).max(0.0);

        Rect::new(left, top, left + width, top + height)
    }
}

/// 解析 [`GridEdge`]：`Px` 原样，`Pct` 相对 `total`，`None` 用默认值
fn resolve_edge(edge: Option<GridEdge>, default: f64, total: f64) -> f64 {
    match edge {
        None => default,
        Some(GridEdge::Px(v)) => v,
        Some(GridEdge::Pct(p)) => total * p / 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::types::*;

    fn make_series(name: &str, grid_index: usize) -> SeriesSpec {
        use crate::pipeline::dataframe::{DataFrame, DataValue, Series};
        let mut df = DataFrame::new();
        df.add_column(Series::new("x", vec![DataValue::Float(0.0)]));
        df.add_column(Series::new("y", vec![DataValue::Float(0.0)]));
        SeriesSpec {
            name: name.to_string(),
            data: df,
            grid_index,
            x_axis_index: 0,
            y_axis_index: 0,
            stack: None,
            group_index: 0,
            sampling: None,
            item_style: ItemStyleSpec::default(),
            config: SeriesConfig::Line(LineConfig::default()),
        }
    }

    fn make_grids(count: usize) -> Vec<GridSpec> {
        (0..count)
            .map(|_| GridSpec {
                left: None,
                right: None,
                top: None,
                bottom: None,
                contain_label: false,
            })
            .collect()
    }

    /// 测试辅助：走完整的 `bind → plan` 流程。
    ///
    /// 测试用的轴都是 category 轴，直接以 `categories` 作为标签文本，
    /// 与 `collect_axis_labels` 对 category 轴的处理一致。
    fn plan(
        planner: &GridPlanner<'_>,
        series: &[SeriesSpec],
        x_axes: &[AxisSpec],
        y_axes: &[AxisSpec],
    ) -> Vec<SubplotSpec> {
        plan_with_mode(planner, series, x_axes, y_axes, FitMode::Fixed).specs
    }

    /// 同 [`plan`]，但可指定 `FitMode`，返回完整 `PlanOutput`（含需求）
    fn plan_with_mode(
        planner: &GridPlanner<'_>,
        series: &[SeriesSpec],
        x_axes: &[AxisSpec],
        y_axes: &[AxisSpec],
        fit_mode: FitMode,
    ) -> PlanOutput {
        let bindings = planner.bind(series, x_axes, y_axes);
        let labels = AxisLabelSet {
            x: x_axes.iter().map(|a| a.categories.clone()).collect(),
            y: y_axes.iter().map(|a| a.categories.clone()).collect(),
        };
        let colors = ColorContext::default();
        let input = LayoutInput {
            bindings,
            x_axes,
            y_axes,
            labels: &labels,
            colors: &colors,
            fit_mode,
        };
        planner.plan(&input, &mut TextMeasurer::new())
    }

    #[test]
    fn test_fixed_mode_demand_is_zero() {
        // Fixed 模式即使空间不足，需求也恒为零（shrink-only 语义）
        let grids = make_grids(1);
        let y_axes = vec![AxisSpec {
            axis_type: AxisType::Category,
            position: AxisPosition::Left,
            grid_index: 0,
            categories: vec!["很长的类别标签一号".into(), "很长的类别标签二号".into()],
            label_show: true,
            ..make_axis_spec_base()
        }];
        let planner = GridPlanner::new(300, 200, 40.0, &grids);
        let out = plan_with_mode(&planner, &[], &[], &y_axes, FitMode::Fixed);
        assert!(!out.demands[0].has_shortfall(), "Fixed 需求应恒为零");
        assert!(out.specs[0].bounds.x0 > 60.0, "Fixed 应通过收缩绘图区腾位");
    }

    #[test]
    fn test_hug_mode_reports_demand_without_shrink() {
        // Hug 模式：绘图区不被收缩，缺口写入需求（目标边距 + 画布增量）
        let grids = make_grids(1);
        let y_axes = vec![AxisSpec {
            axis_type: AxisType::Category,
            position: AxisPosition::Left,
            grid_index: 0,
            categories: vec!["很长的类别标签一号".into(), "很长的类别标签二号".into()],
            label_show: true,
            ..make_axis_spec_base()
        }];
        let planner = GridPlanner::new(300, 200, 40.0, &grids);
        let out = plan_with_mode(&planner, &[], &[], &y_axes, FitMode::Hug);

        let d = &out.demands[0];
        assert!(d.has_shortfall(), "Hug 应上报缺口");
        let GridEdge::Px(target_x0) = d.grid_left.expect("左侧应有目标边距") else {
            panic!("Hug 目标边距应为绝对像素");
        };
        assert!(target_x0 > 60.0, "目标左边距应大于默认 60，实际 {target_x0}");
        assert!(
            (d.grow_left - (target_x0 - 60.0)).abs() < 1e-6,
            "grow_left 应等于目标边距与默认边距之差"
        );
        // Hug 不收缩绘图区：x0 保持默认边距
        assert!((out.specs[0].bounds.x0 - 60.0).abs() < 1e-6);
    }

    #[test]
    fn test_hug_grows_width_to_keep_x_labels_horizontal() {
        // P3：12 个长日期标签在 300px 画布下必须旋转（slot 20 < 标签 60），
        // Hug 应上报"绘图区宽度需求"以保持标签水平（信息零损失）
        let grids = make_grids(1);
        let cats: Vec<String> = (0..12).map(|i| format!("2024-01-{:02}", i + 1)).collect();
        let x_axes = vec![AxisSpec {
            axis_type: AxisType::Category,
            position: AxisPosition::Bottom,
            grid_index: 0,
            categories: cats,
            label_show: true,
            ..make_axis_spec_base()
        }];
        let planner = GridPlanner::new(300, 200, 40.0, &grids);
        let out = plan_with_mode(&planner, &[], &x_axes, &[], FitMode::Hug);

        let d = &out.demands[0];
        assert!(
            d.grow_left > 0.0 && d.grow_right > 0.0,
            "Hug 应上报绘图区宽度需求以保持标签水平"
        );
        // 需求只含画布增量（不写回边距），仍应被识别为缺口
        assert!(d.has_shortfall(), "宽度需求应触发画布扩容");
    }

    /// 构造AxisSpec 的公共字段默认值（测试辅助）
    fn make_axis_spec_base() -> AxisSpec {
        AxisSpec {
            axis_type: AxisType::Category,
            position: AxisPosition::Bottom,
            grid_index: 0,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories: vec![],
            boundary_gap: true,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate: None,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }
    }

    #[test]
    fn test_single_grid_default() {
        let grids = vec![];
        let planner = GridPlanner::new(800, 600, 100.0, &grids);
        let specs = plan(&planner, &[], &[], &[]);

        assert_eq!(specs.len(), 1);
        let s = &specs[0];
        assert_eq!(s.id, 0);
        assert!(s.bounds.width() > 0.0);
        assert!(s.bounds.height() > 0.0);
        assert!(s.bounds.x0 >= 60.0);
        assert!(s.bounds.y0 >= 100.0); // header_height
    }

    #[test]
    fn test_single_grid_no_header() {
        let grids = vec![];
        let planner = GridPlanner::new(800, 600, 0.0, &grids);
        let specs = plan(&planner, &[], &[], &[]);

        let s = &specs[0];
        // top 回退到 60.0（因为 0.0 < margin 60.0）
        assert!(s.bounds.y0 >= 60.0);
    }

    #[test]
    fn test_dense_long_category_labels_grow_bottom_margin() {
        let grids = make_grids(1);
        // 30 个长日期标签，slot 宽度远小于标签宽度 → 自动旋转 90°，底部边距增大
        let x_axes = vec![AxisSpec {
            axis_type: AxisType::Category,
            position: AxisPosition::Bottom,
            grid_index: 0,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories: (0..30).map(|i| format!("2024-01-{:02}", i + 1)).collect(),
            boundary_gap: true,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate: None,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }];
        let planner = GridPlanner::new(800, 600, 60.0, &grids);
        let specs = plan(&planner, &[], &x_axes, &[]);

        // 默认 bounds.y1 = 600 - 60 = 540；密集长标签应使绘图区向上收缩
        assert!(
            specs[0].bounds.y1 < 540.0,
            "旋转标签后底部边距应增大，实际 y1={}",
            specs[0].bounds.y1
        );
    }

    #[test]
    fn test_top_axis_dense_labels_grow_top_margin() {
        let grids = make_grids(1);
        // 顶部 X 轴 + 密集长标签 → 上边距增大（绘图区下移）
        let x_axes = vec![AxisSpec {
            axis_type: AxisType::Category,
            position: AxisPosition::Top,
            grid_index: 0,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories: (0..30).map(|i| format!("2024-01-{:02}", i + 1)).collect(),
            boundary_gap: true,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate: None,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }];
        let planner = GridPlanner::new(800, 600, 60.0, &grids);
        let specs = plan(&planner, &[], &x_axes, &[]);

        // 默认 bounds.y0 = 60（header_height=60）；
        // 顶部密集长标签需要 14 + 投影高 + 4 ≈ 78px 空间，
        // 且不能侵入头部（header_height）区域 → 绘图区应明显下移
        assert!(
            specs[0].bounds.y0 > 60.0 + 60.0,
            "顶部旋转标签后上边距应增大，实际 y0={}",
            specs[0].bounds.y0
        );
    }

    #[test]
    fn test_short_labels_keep_default_margins() {
        let grids = make_grids(1);
        // 3 个短标签，横向能放下 → 不旋转、不额外预留
        let x_axes = vec![AxisSpec {
            axis_type: AxisType::Category,
            position: AxisPosition::Bottom,
            grid_index: 0,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories: vec!["一".into(), "二".into(), "三".into()],
            boundary_gap: true,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate: None,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }];
        let planner = GridPlanner::new(800, 600, 60.0, &grids);
        let specs = plan(&planner, &[], &x_axes, &[]);

        assert!((specs[0].bounds.y1 - 540.0).abs() < 1e-6);
    }

    #[test]
    fn test_two_grids_horizontal() {
        let grids = vec![
            GridSpec {
                left: Some(GridEdge::Px(0.0)),
                top: Some(GridEdge::Px(0.0)),
                right: Some(GridEdge::Px(400.0)), // width = 800 - 400 - 0 = 400
                bottom: Some(GridEdge::Px(0.0)),
                contain_label: false,
            },
            GridSpec {
                left: Some(GridEdge::Px(400.0)),
                top: Some(GridEdge::Px(0.0)),
                right: Some(GridEdge::Px(0.0)),
                bottom: Some(GridEdge::Px(0.0)),
                contain_label: false,
            },
        ];
        let series = vec![make_series("S1", 0), make_series("S2", 1)];
        let planner = GridPlanner::new(800, 600, 100.0, &grids);
        let specs = plan(&planner, &series, &[], &[]);

        assert_eq!(specs.len(), 2);
        // P2b：显式边距不再叠加 LABEL_*_PADDING
        // Grid 0: left=0 → x0=0, right=400 → x1=800-400=400
        assert!((specs[0].bounds.x1 - 400.0).abs() < 1.0);
        // Grid 1: left=400 → x0=400, right=0 → x1=800
        assert!((specs[1].bounds.x0 - 400.0).abs() < 1.0);
        assert!((specs[1].bounds.x1 - 800.0).abs() < 1.0);

        assert_eq!(specs[0].series_indices, vec![0]);
        assert_eq!(specs[1].series_indices, vec![1]);
    }

    #[test]
    fn test_series_binding_to_grid() {
        let grids = make_grids(2);
        let series = vec![
            make_series("Line1", 0),
            make_series("Pie1", 1),
            make_series("Bar1", 0),
        ];
        let planner = GridPlanner::new(800, 600, 100.0, &grids);
        let specs = plan(&planner, &series, &[], &[]);

        assert_eq!(specs[0].series_indices, vec![0, 2]);
        assert_eq!(specs[1].series_indices, vec![1]);
    }

    #[test]
    fn test_axis_binding_to_grid() {
        let grids = make_grids(2);
        let x_axes = vec![
            AxisSpec {
                axis_type: AxisType::Category,
                position: AxisPosition::Bottom,
                grid_index: 0,
                min: None,
                max: None,
                name: None,
                name_location: None,
                categories: vec![],
                boundary_gap: true,
                inverse: false,
                split_number: None,
                label_show: true,
                label_formatter: None,
                label_rotate: None,
                axis_line_show: true,
                split_line_show: true,
                z: None,
            },
            AxisSpec {
                axis_type: AxisType::Value,
                position: AxisPosition::Bottom,
                grid_index: 1,
                min: None,
                max: None,
                name: None,
                name_location: None,
                categories: vec![],
                boundary_gap: true,
                inverse: false,
                split_number: None,
                label_show: true,
                label_formatter: None,
                label_rotate: None,
                axis_line_show: true,
                split_line_show: true,
                z: None,
            },
        ];
        let y_axes = vec![AxisSpec {
            axis_type: AxisType::Value,
            position: AxisPosition::Left,
            grid_index: 0,
            min: None,
            max: None,
            name: None,
            name_location: None,
            categories: vec![],
            boundary_gap: true,
            inverse: false,
            split_number: None,
            label_show: true,
            label_formatter: None,
            label_rotate: None,
            axis_line_show: true,
            split_line_show: true,
            z: None,
        }];
        let planner = GridPlanner::new(800, 600, 100.0, &grids);
        let specs = plan(&planner, &[], &x_axes, &y_axes);

        assert_eq!(specs[0].x_axis_indices, vec![0]);
        assert_eq!(specs[1].x_axis_indices, vec![1]);
        assert_eq!(specs[0].y_axis_indices, vec![0]);
        assert!(specs[1].y_axis_indices.is_empty());
    }

    #[test]
    fn test_contain_label_increases_margins() {
        let grids = vec![GridSpec {
            left: None,
            right: None,
            top: None,
            bottom: None,
            contain_label: true,
        }];
        let planner = GridPlanner::new(800, 600, 100.0, &grids);
        let specs = plan(&planner, &[], &[], &[]);
        let s = &specs[0];

        // contain_label=true 时 left 默认 70，right 默认 50，bottom 默认 60
        assert!((s.bounds.x0 - 70.0).abs() < 1.0);
        assert!((s.bounds.x1 - 750.0).abs() < 1.0); // 800 - 50
        assert!((s.bounds.y0 - 100.0).abs() < 1.0); // header_height
        assert!((s.bounds.y1 - 540.0).abs() < 1.0); // 600 - 60
    }
}
