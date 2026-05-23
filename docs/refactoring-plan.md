# 重构方案与实施计划书

> 基于 [new-arch.md](./new-arch.md) 架构设计
> 目标: 将当前 8 层架构逐步重构为"GridPlanner → DataProcessor → VisualElementBuilder" 3 阶段管线
> 状态: 计划稿，待实现

---

## 目录

1. [现状分析](#1-现状分析)
2. [目标架构](#2-目标架构)
3. [模块结构设计](#3-模块结构设计)
4. [Phase 0: 骨架搭建](#4-phase-0-骨架搭建)
5. [Phase 1: GridPlanner](#5-phase-1-gridplanner)
6. [Phase 2: 饼图试点](#6-phase-2-饼图试点)
7. [Phase 3: 逐个系列迁移](#7-phase-3-逐个系列迁移)
8. [Phase 4: 旧代码清理](#8-phase-4-旧代码清理)
9. [并行运行与验证策略](#9-并行运行与验证策略)
10. [测试策略](#10-测试策略)
11. [工作量估算与里程碑](#11-工作量估算与里程碑)

---

## 1. 现状分析

### 1.1 当前架构

```
ChartOption → ChartModel (样式解析+颜色分配)
                  → Chart (绑定宽高)
                      → LayoutEngine (Measure-Arrange)
                          → compute_data_coord_for_grid() (数据范围收集)
                              → AxisComponent → build_visual()
                              → BarSeriesComponent → Pipeline (Transform→Map→Build)
                              → LineSeriesComponent → Pipeline ...
                              → PieSeriesComponent → Pipeline ...
                              → LegendComponent / TitleComponent
                          → SvgRenderer / PixmapRenderer
```

### 1.2 当前架构的问题

| 问题 | 表现 | 影响 |
|------|------|------|
| **DataCoordinateSystem 跨模块传递** | 从 LayoutEngine 产生，传递给所有 Component | 修改坐标系影响 5+ 模块，耦合度高 |
| **布局与数据混合** | LayoutEngine 既做 Grid 切分又做轴标签 Measure-Arrange | 布局逻辑纠缠数据依赖，难以独立测试 |
| **Pipeline 三阶段分散** | Transform→Mapper→Builder 分散在 pipeline/ 目录 | 添加新系列需要分别实现三个阶段 |
| **Component trait 膨胀** | ChartComponent::build_visual_elements 依赖 ChartModel + LayoutOutput | 函数签名耦合两个大结构体 |
| **SeriesContext 隐式依赖** | 各组件从 SeriesContext 中提取所需信息 | 隐式契约难以文档化 |

### 1.3 当前代码规模

| 模块 | 文件 | 估算行数 |
|------|------|---------|
| `model.rs` | 1 | ~500 |
| `chart.rs` | 1 | ~400 |
| `layout/` | 5 | ~600 |
| `component/` | 15 | ~2500 |
| `pipeline/` | 4 | ~400 |
| 合计 | ~26 | ~4400 |

---

## 2. 目标架构

```
ChartOption
    │
    ▼
GridPlanner (纯数学分配)
    │ → Vec<SubplotSpec>
    ▼
AxisBindingResolver (轴协调)
    │ → ResolvedAxisRanges
    ▼
ColorAssigner (颜色分配)
    │ → ColorContext
    ▼
DataProcessor[0..N] (每个 subplot 独立)
    │ ① data_transform()
    │ ② compute_axis_range()
    │ ③ generate_candidate_ticks()
    │ ④ compute_plot_area()
    │ ⑤ refine_ticks()
    │ ⑥ build_geometry()
    │ → SubplotVisualData
    ▼
VisualElementBuilder (合并+排序+浮层)
    │ → Vec<VisualElement>
    ▼
Renderer (SVG/PNG，不变)
```

### 2.1 新模块文件结构

```
src/
├── new_pipeline/               # 新管线根目录
│   ├── mod.rs                  # 模块声明 + 重新导出
│   ├── types.rs                # 核心类型定义 (SubplotSpec, ResolvedAxisRange 等)
│   ├── grid_planner.rs         # GridPlanner 实现
│   ├── axis_binding_resolver.rs # AxisBindingResolver 实现
│   ├── color_assigner.rs       # ColorAssigner 实现
│   ├── data_processor.rs       # DataProcessor trait + 基础结构
│   ├── text_measurer.rs        # TextMeasurer 缓存实现
│   ├── visual_element_builder.rs # VisualElementBuilder 实现
│   ├── processor/              # 各系列 DataProcessor 实现
│   │   ├── mod.rs
│   │   ├── pie.rs              # 饼图 DataProcessor
│   │   ├── bar.rs              # 柱状图 DataProcessor
│   │   ├── line.rs             # 折线图 DataProcessor
│   │   ├── scatter.rs          # 散点图 DataProcessor
│   │   └── ...                 # 其他系列
│   └── shared/                 # 共享工具
│       ├── mod.rs
│       ├── tick.rs             # 刻度计算工具
│       └── axis_range.rs       # 轴范围计算工具
```

### 2.2 核心类型定义

```rust
// === types.rs ===

use vello_cpu::kurbo::Rect;

/// GridPlanner 的输出：一个 subplot 的完整分配信息
pub struct SubplotSpec {
    pub id: usize,
    pub bounds: Rect,               // 像素区域 (x, y, width, height)
    pub series_indices: Vec<usize>,
    pub x_axis_indices: Vec<usize>,
    pub y_axis_indices: Vec<usize>,
}

/// AxisBindingResolver 的输出：单个轴实例的解析结果
pub struct ResolvedAxisRange {
    pub axis_index: usize,
    pub min: f64,
    pub max: f64,
    pub is_user_defined: bool,       // 用户是否显式指定了 min/max
    pub tick_count_hint: Option<usize>,
}

/// AxisBindingResolver 的输出：所有轴的解析结果集合
pub struct ResolvedAxisRanges {
    pub ranges: Vec<ResolvedAxisRange>,
}

/// ColorAssigner 的输出：颜色上下文
pub struct ColorContext {
    pub palette: Vec<Color>,
    pub background: Color,
    pub series_colors: Vec<Color>,    // 预分配给 series 的颜色
    pub axis_line_color: Color,
    pub axis_label_color: Color,
    pub grid_line_color: Color,
}

/// DataProcessor 的输入
pub struct DataProcessorInput<'a> {
    pub spec: &'a SubplotSpec,
    pub option: &'a ChartOption,
    pub colors: &'a ColorContext,
    pub axis_ranges: &'a ResolvedAxisRanges,
    pub external_data: Option<&'a DataFrame>,       // 可选 polars DataFrame
    pub text_measurer: &'a mut TextMeasurer,
}

/// DataProcessor 的输出
pub struct SubplotVisualData {
    pub series_elements: Vec<VisualElement>,
    pub axis_elements: Vec<VisualElement>,
    pub grid_lines: Vec<VisualElement>,
}

/// DataProcessor trait
pub trait DataProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData>;
}

/// TextMeasurer（文本测量缓存）
pub struct TextMeasurer {
    cache: HashMap<(String, f64, f64), (f64, f64)>,
}

impl TextMeasurer {
    pub fn new() -> Self;
    pub fn measure(&mut self, text: &str, style: &TextStyle) -> (f64, f64);
}
```

---

## 3. 模块结构设计

### 3.1 GridPlanner

```
src/new_pipeline/grid_planner.rs

职责:
  - 解析 option.grid 列表（若无则创建默认 grid）
  - 根据画布 width/height 将每个 grid 配置转换为绝对像素 Rect
  - 支持 Position::Value (px), Position::Percent, Position::Center, Position::Auto
  - 不接触任何数据、标签文本、刻度信息

关键实现:
  - 复用当前 GridManager 中的 Position 解析逻辑
  - 输入: &[GridOption], width: u32, height: u32
  - 输出: Vec<SubplotSpec> (含 series_indices / x_axis_indices / y_axis_indices 绑定)

依赖关系:
  - 当前 GridManager → 复用其纯数学算法
  - 不需要 LayoutContext / DataCoordinateSystem
```

### 3.2 AxisBindingResolver

```
src/new_pipeline/axis_binding_resolver.rs

职责:
  - 解析 xAxisIndex / yAxisIndex 为具体的轴配置（从 ChartOption 中查找）
  - 识别哪些 subplot 共用同一个轴实例
  - 对每个轴实例:
    · 收集所有关联 subplot 的系列数据范围（从 series.data 中计算 min/max）
    · 如果用户指定了 min/max，以用户配置优先
    · 取整后的轴范围
  - 输出每个轴实例的 ResolvedAxisRange

依赖关系:
  - 依赖 ChartOption（访问 xAxis, yAxis, series 配置）
  - 不依赖布局信息
```

### 3.3 ColorAssigner

```
src/new_pipeline/color_assigner.rs

职责:
  - 读取主题调色板或 option.color
  - 为每个 series 按索引分配固定颜色
  - 分面场景下同一系列跨分面保持颜色一致
  - 输出 ColorContext

实现策略:
  - 复用当前 ChartModel::new() 中的颜色分配逻辑
  - 当前逻辑: option.color → theme.color → 默认调色板
  - 新增: 基于 series 索引的色轮轮转
```

### 3.4 DataProcessor trait

```
src/new_pipeline/data_processor.rs

pub trait DataProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData>;
}

内部步骤（由具体实现调用）:
  ① data_transform()           — 数据变换
  ② compute_axis_range()       — 轴范围计算
  ③ generate_candidate_ticks() — 候选刻度生成
  ④ compute_plot_area()        — 绘图区域计算
  ⑤ refine_ticks()             — 刻度精确定位
  ⑥ build_geometry()           — 几何生成

这个 trait 的存在使得不同的图表类型可以有不同的内部逻辑，
同时保持一致的输入/输出接口。
```

### 3.5 系列 DataProcessor 实现

```
每个系列文件实现对应的 DataProcessor:

processor/pie.rs:    PieProcessor    — 饼图（无需轴，最简单）
processor/bar.rs:    BarProcessor    — 柱状图（需分类轴+数值轴）
processor/line.rs:   LineProcessor   — 折线图（类似柱状图但几何不同）
processor/scatter.rs: ScatterProcessor — 散点图（X/Y 双数值轴）
processor/radar.rs:  RadarProcessor  — 雷达图（多维度）
processor/...:       ...

每个 Processor 的结构:

pub struct BarProcessor {
    series_index: usize,
    series: BarSeriesOption,
}

impl DataProcessor for BarProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        // 1. 从 input.option.series[self.series_index] 读取数据
        // 2. 执行 data_transform (堆叠/聚合)
        // 3. 从 data 计算轴范围
        // 4. 生成候选刻度 → 测量标签 → 计算 plot_area
        // 5. 在 plot_area 内精确定位刻度
        // 6. 生成 Rect/Path/Text VisualElement
    }
}
```

---

## 4. Phase 0: 骨架搭建

### 4.1 目标

建立新管线的模块结构和核心类型定义，**不实现任何业务逻辑**。确保编译通过。

### 4.2 具体步骤

#### 步骤 0.1: 创建 `src/new_pipeline/` 目录和模块声明

```rust
// src/new_pipeline/mod.rs
pub mod types;
pub mod grid_planner;
pub mod axis_binding_resolver;
pub mod color_assigner;
pub mod data_processor;
pub mod text_measurer;
pub mod visual_element_builder;
pub mod processor;

pub use types::*;
pub use grid_planner::GridPlanner;
pub use axis_binding_resolver::AxisBindingResolver;
pub use color_assigner::ColorAssigner;
pub use data_processor::{DataProcessor, DataProcessorInput};
pub use text_measurer::TextMeasurer;
pub use visual_element_builder::VisualElementBuilder;
```

```rust
// src/new_pipeline/processor/mod.rs
// 后续逐步添加各系列 processor
// pub mod pie;
// pub mod bar;
// ...
```

#### 步骤 0.2: 创建 `types.rs`

将上述核心类型定义（SubplotSpec, ResolvedAxisRange, ResolvedAxisRanges, ColorContext, SubplotVisualData, DataProcessorInput）写入文件。

所有结构体标记 `#[derive(Debug, Clone)]`，需要的地方实现 `Default`。

#### 步骤 0.3: 创建空实现文件

每个模块文件创建骨架：

```rust
// grid_planner.rs
pub struct GridPlanner;

impl GridPlanner {
    pub fn new() -> Self { Self }
    pub fn plan(&self) -> Vec<SubplotSpec> { todo!() }
}

// axis_binding_resolver.rs, color_assigner.rs, text_measurer.rs, 
// visual_element_builder.rs — 类似骨架
```

#### 步骤 0.4: 在 `lib.rs` 中注册新模块

```rust
// src/lib.rs
pub mod new_pipeline;
```

#### 步骤 0.5: 验证编译

```bash
cargo build
```

### 4.3 交付物清单

| 文件 | 内容 | 状态 |
|------|------|------|
| `src/new_pipeline/mod.rs` | 模块声明 + 重新导出 | 新文件 |
| `src/new_pipeline/types.rs` | 6 个核心类型定义 | 新文件 |
| `src/new_pipeline/grid_planner.rs` | GridPlanner 骨架 | 新文件 |
| `src/new_pipeline/axis_binding_resolver.rs` | AxisBindingResolver 骨架 | 新文件 |
| `src/new_pipeline/color_assigner.rs` | ColorAssigner 骨架 | 新文件 |
| `src/new_pipeline/data_processor.rs` | DataProcessor trait + 骨架 | 新文件 |
| `src/new_pipeline/text_measurer.rs` | TextMeasurer 骨架 | 新文件 |
| `src/new_pipeline/visual_element_builder.rs` | VisualElementBuilder 骨架 | 新文件 |
| `src/new_pipeline/processor/mod.rs` | processor 子模块声明 | 新文件 |

---

## 5. Phase 1: GridPlanner

### 5.1 目标

实现完整功能的 GridPlanner，替换当前 `GridManager`。输出与现有 `GridRect` 兼容。

### 5.2 关键算法

```rust
impl GridPlanner {
    pub fn plan(&self) -> Vec<SubplotSpec> {
        if self.grid_configs.is_empty() {
            // 默认 grid: 填满画布（预留边距）
            return vec![SubplotSpec {
                id: 0,
                bounds: Rect::new(60.0, 60.0,
                    self.total_width as f64 - 120.0,
                    self.total_height as f64 - 120.0),
                series_indices: (0..self.total_series).collect(),
                x_axis_indices: (0..self.total_x_axes).collect(),
                y_axis_indices: (0..self.total_y_axes).collect(),
            }];
        }

        self.grid_configs.iter().enumerate().map(|(idx, grid)| {
            let bounds = self.resolve_position(grid, idx);
            SubplotSpec {
                id: idx,
                bounds,
                series_indices: self.find_series_for_grid(idx),
                x_axis_indices: self.find_x_axes_for_grid(idx),
                y_axis_indices: self.find_y_axes_for_grid(idx),
            }
        }).collect()
    }

    fn resolve_position(&self, grid: &GridOption, idx: usize) -> Rect {
        // 解析 left/right/top/bottom
        // 支持 Pixel 和 Percent
        let left = self.resolve(grid.left.as_ref(), self.total_width, 60.0);
        let right = self.resolve(grid.right.as_ref(), self.total_width, 60.0);
        let top = self.resolve(grid.top.as_ref(), self.total_height, 60.0);
        let bottom = self.resolve(grid.bottom.as_ref(), self.total_height, 60.0);

        // 支持 width/height 显式指定
        let width = match &grid.width {
            Some(w) => self.resolve_value(w, self.total_width),
            None => self.total_width as f64 - left - right,
        };
        let height = match &grid.height {
            Some(h) => self.resolve_value(h, self.total_height),
            None => self.total_height as f64 - top - bottom,
        };

        Rect::new(left, top, left + width, top + height)
    }
}
```

### 5.3 与现有 GridManager 的对接

当前 `GridManager` 的算法可以**直接复用**：

- `GridManager::calculate_grid_bounds()` → 对应 `resolve_position()`
- `GridManager::resolve_position()` → 可以直接提取为独立函数

**建议做法**：
1. 在新 `GridPlanner` 中重写 Position 解析逻辑（独立于 LayoutContext）
2. 不要直接依赖旧 GridManager，避免耦合
3. 复用旧 GridManager 的单元测试，验证输出一致

### 5.4 测试要点

| 测试场景 | 输入 | 期望输出 |
|---------|------|---------|
| 无 grid 配置 | `grids=[]`, `800x600` | 1 个 subplot，默认边距 60px |
| 单个 grid 百分比 | `left=10%, top=10%, width=80%, height=80%` | `bounds=(80,60,640,480)` |
| 多个 grid 并列 | 2 个 grid 各占 50% 宽度 | 2 个 subplot，左右并列 |
| grid 重叠 | 两个 grid 配置重叠 | 各自独立计算，允许重叠 |
| width/height 显式指定 | `width=300, height=200` | 精确像素尺寸 |

---

## 6. Phase 2: 饼图试点

### 6.1 选择饼图的原因

饼图是**最简单**的 DataProcessor 试点：
- 不需要轴（无轴范围、刻度、标签计算）
- 不需要 plot_area 扣除标签空间
- 只需要：根据 bounds 确定圆心和半径 → 根据数据计算百分比 → 生成扇区 Path
- 与现有 `PieSeriesComponent` 功能对应

### 6.2 饼图 DataProcessor 实现

```rust
// src/new_pipeline/processor/pie.rs

use vello_cpu::kurbo::{BezPath, Point, Rect};

pub struct PieProcessor {
    series_index: usize,
}

impl DataProcessor for PieProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let pie_series = match series {
            SeriesOption::Pie(p) => p,
            _ => return Err(ChartError::InvalidSeriesType),
        };

        // 步骤 ①: 数据变换
        let data = &pie_series.data;
        let total: f64 = data.iter().map(|d| d.value).sum();

        // 步骤 ②-⑤: 不需要（饼图没有轴）

        // 步骤 ④: 计算圆心和半径
        let bounds = spec.bounds;
        let cx = bounds.x0 + bounds.width() / 2.0;
        let cy = bounds.y0 + bounds.height() / 2.0;
        let radius = bounds.width().min(bounds.height()) / 2.0 * 0.8;

        // 分面颜色（由 ColorAssigner 预先分配）
        let series_color = input.colors.series_colors[self.series_index];

        // 步骤 ⑥: 生成几何
        let mut elements = Vec::new();
        let mut start_angle = -std::f64::consts::FRAC_PI_2;

        for (i, dp) in data.iter().enumerate() {
            let sweep = (dp.value / total) * 2.0 * std::f64::consts::PI;
            let end_angle = start_angle + sweep;

            // 生成扇区 Path
            let path = create_sector(cx, cy, radius, start_angle, end_angle);

            let color = dp.item_style
                .and_then(|s| s.color)
                .map(Color::from)
                .unwrap_or_else(|| {
                    // 从调色板轮转
                    input.colors.palette[i % input.colors.palette.len()]
                });

            elements.push(VisualElement::Path {
                path,
                style: FillStrokeStyle {
                    fill: Some(color),
                    stroke: None,
                },
            });

            start_angle = end_angle;
        }

        Ok(SubplotVisualData {
            series_elements: elements,
            axis_elements: Vec::new(),
            grid_lines: Vec::new(),
        })
    }
}

fn create_sector(cx: f64, cy: f64, r: f64, start: f64, end: f64) -> BezPath {
    let mut path = BezPath::new();
    path.move_to(Point::new(cx, cy));
    path.line_to(Point::new(
        cx + r * start.cos(),
        cy + r * start.sin(),
    ));
    // 弧线
    path.quad_to(/* ... */);
    path.close_path();
    path
}
```

### 6.3 并行运行方案

```rust
// src/chart.rs — 新增 method

impl Chart {
    /// 使用新管线收集视觉元素
    pub fn collect_visual_elements_new(&self) -> Result<(Vec<VisualElement>, u32, u32)> {
        // 1. GridPlanner
        let specs = GridPlanner::new(
            self.width, self.height,
            &self.model.grid_configs,
        ).plan();

        // 2. AxisBindingResolver
        let axis_ranges = AxisBindingResolver::new(&self.model).resolve(&specs);

        // 3. ColorAssigner
        let color_ctx = ColorAssigner::new(&self.model).assign();

        // 4. DataProcessor (per subplot)
        let mut text_measurer = TextMeasurer::new();
        let mut all_elements = Vec::new();

        for spec in &specs {
            let input = DataProcessorInput {
                spec,
                option: &self.model.raw_option,
                colors: &color_ctx,
                axis_ranges: &axis_ranges,
                external_data: None,
                text_measurer: &mut text_measurer,
            };

            // 根据系列类型选择对应的 DataProcessor
            for &series_idx in &spec.series_indices {
                let processor = create_processor(
                    &self.model.raw_option.series[series_idx],
                    series_idx,
                );
                let visual_data = processor.process(input)?;
                all_elements.extend(visual_data.series_elements);
                all_elements.extend(visual_data.axis_elements);
                all_elements.extend(visual_data.grid_lines);
            }
        }

        // 5. VisualElementBuilder
        let final_elements = VisualElementBuilder::new()
            .with_title(&self.model.title)
            .with_legend(&self.model.legend)
            .with_background(self.model.background)
            .build(all_elements);

        Ok((final_elements, self.width, self.height))
    }
}
```

```rust
// src/builder.rs — feature flag 控制

impl ChartBuilder {
    pub fn build(&self, width: u32, height: u32) -> Result<Chart> {
        let model = ChartModel::new(self.option.clone(), self.theme_registry.resolve_theme())?;
        Ok(Chart { model, width, height })
    }

    /// 使用新管线构建（Phase 2 阶段仅对部分系列类型生效）
    pub fn build_with_new_pipeline(&self, width: u32, height: u32) -> Result<Vec<VisualElement>> {
        let model = ChartModel::new(self.option.clone(), self.theme_registry.resolve_theme())?;
        let chart = Chart { model, width, height };
        chart.collect_visual_elements_new()
    }
}
```

### 6.4 验证方法

```rust
#[test]
fn test_pie_new_vs_old() {
    let option = ChartOption {
        series: vec![SeriesOption::Pie(PieSeriesOption::new(
            "Sales",
            vec![
                DataPoint::new("A", 30.0),
                DataPoint::new("B", 50.0),
                DataPoint::new("C", 20.0),
            ],
        ))],
        ..Default::default()
    };

    let builder = ChartBuilder::from_option(option);

    // 旧管线
    let chart = builder.build(800, 600).unwrap();
    let (old_elements, _, _) = chart.collect_visual_elements().unwrap();

    // 新管线
    let new_elements = builder.build_with_new_pipeline(800, 600).unwrap();

    // 验证元素数量一致（具体几何可以逐步对齐）
    assert_eq!(old_elements.len(), new_elements.len());
}
```

---

## 7. Phase 3: 逐个系列迁移

### 7.1 迁移顺序

| 顺序 | 系列 | 复杂度 | 依赖 | 说明 |
|:----:|------|:------:|------|------|
| 1 | 饼图 | ⭐ | 无轴 | 已完成 Phase 2 |
| 2 | 柱状图 | ⭐⭐⭐ | 分类轴+数值轴+堆叠 | 最常用的图表类型 |
| 3 | 折线图 | ⭐⭐⭐ | 分类轴+数值轴+面积 | 与柱状图共享轴逻辑 |
| 4 | 散点图 | ⭐⭐ | 双数值轴 | 不需要类目计算 |
| 5 | K线图 | ⭐⭐⭐ | OHLC 数据+数值轴 | 需要四分位数据转换 |
| 6 | 气泡图 | ⭐⭐ | 三数值维 | 类似散点图+半径 |
| 7 | 雷达图 | ⭐⭐⭐⭐ | 多维度+角度轴 | 需要极坐标映射 |
| 8 | 仪表盘 | ⭐⭐ | 角度轴+指针 | 特殊类型 |

### 7.2 柱状图 DataProcessor 实现要点

```rust
// src/new_pipeline/processor/bar.rs

pub struct BarProcessor {
    series_index: usize,
}

impl DataProcessor for BarProcessor {
    fn process(&self, input: DataProcessorInput) -> Result<SubplotVisualData> {
        let spec = input.spec;
        let series = &input.option.series[self.series_index];
        let bar_series = match series {
            SeriesOption::Bar(b) => b,
            _ => return Err(ChartError::InvalidSeriesType),
        };

        // 步骤 ①: 数据变换
        let data = transform_data(bar_series, input.option)?;

        // 步骤 ②: 计算轴范围
        let y_range = compute_y_range(&data, bar_series);
        let x_range = compute_x_range(&data, bar_series);

        // 步骤 ③: 生成候选刻度
        let y_candidate_ticks = generate_candidate_ticks(y_range, AxisType::Value);

        // 步骤 ④: 计算 plot_area
        let (plot_area, _) = compute_plot_area(
            spec.bounds,
            &y_candidate_ticks,
            input.text_measurer,
            &input.option.text_style,
        );

        // 步骤 ⑤: 精确定位刻度
        let y_ticks = refine_ticks(&y_candidate_ticks, plot_area, AxisType::Value);

        // 步骤 ⑥: 生成几何
        let color = input.colors.series_colors[self.series_index];
        let bars = build_bars(&data, &x_range, &y_range, plot_area, color);
        let axes = build_axes(plot_area, &y_ticks, &data.categories, input.colors);

        Ok(SubplotVisualData {
            series_elements: bars,
            axis_elements: axes,
            grid_lines: build_grid_lines(plot_area, &y_ticks, input.colors),
        })
    }
}
```

### 7.3 每个系列迁移的工作项

对一个系列（如 Bar），迁移工作分 4 步：

| 步骤 | 内容 | 验证 |
|------|------|------|
| 1 | 实现 DataProcessor trait（processor/bar.rs） | 单元测试 |
| 2 | 在 `create_processor()` 中添加分发 | 集成测试（新旧对比） |
| 3 | 并行运行验证输出一致性 | SVG diff 对比 |
| 4 | 删除旧的 Component 文件 | 确保旧管线仍可用 |

---

## 8. Phase 4: 旧代码清理

### 8.1 可删除的旧模块

当所有系列迁移完成后：

| 旧文件 | 替代 |
|--------|------|
| `src/layout/grid_manager.rs` | `new_pipeline/grid_planner.rs` |
| `src/layout/engine.rs` | 不再需要 LayoutEngine |
| `src/layout/elements.rs` (AxisLayout, GridLayout 等) | 不再需要 |
| `src/layout/mod.rs` (部分) | 仅保留 table_layout |
| `src/component/*.rs` (除 title, legend) | 被 processor/ 替代 |
| `src/pipeline/` (全部) | 被 DataProcessor 内部逻辑替代 |
| `src/chart.rs` (部分) | 只保留 render_* 方法 |
| `src/model.rs` (部分) | 简化成只保留样式解析 |

### 8.2 保留的旧模块

| 旧文件 | 保留理由 |
|--------|---------|
| `src/option.rs` | 用户 API 不变，仍使用 ChartOption |
| `src/visual.rs` | VisualElement 类型不变 |
| `src/render/` | 渲染后端不变 |
| `src/theme.rs` | 主题系统不变 |
| `src/text.rs` | 文本布局不变 |
| `src/component/title.rs` | 全局标题（VisualElementBuilder 可复用或重写） |
| `src/component/legend.rs` | 全局图例（同上） |
| `src/model.rs` (样式解析部分) | 可简化但核心样式解析逻辑可复用 |

### 8.3 最终模块结构

```
src/
├── option.rs              ← 不变
├── visual.rs              ← 不变
├── render/                ← 不变
├── theme.rs               ← 不变
├── text.rs                ← 不变
├── builder.rs             ← 简化，直接使用新管线
├── chart.rs               ← 简化，只做渲染调度
├── model.rs               ← 简化，只保留样式/主题解析
├── error.rs               ← 不变
├── new_pipeline/          ← 核心逻辑全部集中于此
│   ├── grid_planner.rs
│   ├── axis_binding_resolver.rs
│   ├── color_assigner.rs
│   ├── data_processor.rs
│   ├── text_measurer.rs
│   ├── visual_element_builder.rs
│   ├── types.rs
│   ├── shared/
│   │   ├── tick.rs
│   │   └── axis_range.rs
│   └── processor/
│       ├── pie.rs
│       ├── bar.rs
│       ├── line.rs
│       ├── scatter.rs
│       └── ...
├── layout/                ← 精简，只保留table_layout
│   └── table_layout.rs
├── component/             ← 精简，只保留 title, legend
│   ├── title.rs
│   └── legend.rs
└── pipeline/              ← 删除
```

---

## 9. 并行运行与验证策略

### 9.1 并行运行方案

在迁移过程中，新旧两条管线同时存在。通过 `ChartBuilder` 的方法选择：

```rust
impl ChartBuilder {
    /// 旧管线（默认）
    pub fn build(&self, width: u32, height: u32) -> Result<Chart>;

    /// 新管线（仅对已迁移的系列生效）
    pub fn build_new(&self, width: u32, height: u32) -> Result<Vec<VisualElement>>;

    /// 双管线对比（仅用于验证）
    pub fn build_both(&self, width: u32, height: u32) -> Result<(Vec<VisualElement>, Vec<VisualElement>)> {
        let old = self.build(width, height)?.collect_visual_elements()?;
        let new = self.build_new(width, height)?;
        Ok((old.0, new))
    }
}
```

### 9.2 环境变量切换

```rust
// .env 或命令行
// LIECHARTS_USE_NEW_PIPELINE=1 — 全局启用新管线
// LIECHARTS_COMPARE_PIPELINES=1 — 双管线对比输出比较日志

impl ChartBuilder {
    pub fn build(&self, width: u32, height: u32) -> Result<Chart> {
        if std::env::var("LIECHARTS_USE_NEW_PIPELINE").is_ok() {
            // 使用新管线
            let elements = self.build_new(width, height)?;
            // 包装为 Chart 兼容返回
            // ...
        }
        // 旧管线
        let model = ChartModel::new(self.option.clone(), ...)?;
        Ok(Chart { model, width, height })
    }
}
```

### 9.3 输出对比验证

```rust
/// 比较新旧管线的 VisualElement 列表
fn compare_elements(old: &[VisualElement], new: &[VisualElement]) -> bool {
    if old.len() != new.len() {
        eprintln!("元素数量不同: old={}, new={}", old.len(), new.len());
        return false;
    }

    for (i, (o, n)) in old.iter().zip(new.iter()).enumerate() {
        match (o, n) {
            (VisualElement::Rect { rect: r1, style: s1 },
             VisualElement::Rect { rect: r2, style: s2 }) => {
                let pos_ok = (r1.x0 - r2.x0).abs() < 1.0
                          && (r1.y0 - r2.y0).abs() < 1.0;
                let color_ok = s1.fill == s2.fill;
                if !pos_ok || !color_ok {
                    eprintln!("元素 {} 不匹配: {:?} vs {:?}", i, o, n);
                    return false;
                }
            }
            // 其他变体...
            _ => {
                eprintln!("元素 {} 类型不同: {:?} vs {:?}", i, o, n);
                return false;
            }
        }
    }
    true
}
```

### 9.4 SVG 像素级对比

```rust
#[test]
fn test_pie_svg_identical() {
    let option = get_test_option();
    let builder = ChartBuilder::from_option(option);

    let chart = builder.build(800, 600).unwrap();
    let old_svg = chart.render_svg().unwrap();

    // 生成新管线 SVG（需要新管线支持渲染）
    std::fs::write("/tmp/old.svg", &old_svg).unwrap();

    // 手动检查或使用 svg_diff 工具
    // assert_svg_identical("old.svg", "new.svg");
}
```

---

## 10. 测试策略

### 10.1 单元测试

| 模块 | 测试内容 | 测试类型 |
|------|---------|---------|
| GridPlanner | 位置解析、百分比转像素、多 grid 分配 | 纯数学，无依赖 |
| AxisBindingResolver | 轴索引解析、共用轴范围协调、用户 min/max 优先级 | 纯数据 |
| ColorAssigner | 色轮轮转、分面颜色一致性 | 纯算法 |
| TextMeasurer | 缓存命中、字体尺寸测量 | 需文本引擎 |
| DataProcessor (各系列) | 给定固定 SubplotSpec + 数据，验证 VisualElement 输出 | 算法验证 |
| VisualElementBuilder | 合并顺序、z 索引排序、浮层位置 | 排序验证 |

### 10.2 集成测试

```rust
// tests/new_pipeline_test.rs

mod grid_planner_tests {
    #[test]
    fn test_default_grid() { /* ... */ }
    #[test]
    fn test_multi_grid() { /* ... */ }
    #[test]
    fn test_percentage_positions() { /* ... */ }
}

mod data_processor_tests {
    #[test]
    fn test_pie_processor() { /* ... */ }
    #[test]
    fn test_bar_processor_basic() { /* ... */ }
    #[test]
    fn test_bar_processor_stacked() { /* ... */ }
}

mod pipeline_comparison_tests {
    #[test]
    fn test_pie_new_vs_old() { /* ... */ }
    #[test]
    fn test_bar_new_vs_old() { /* ... */ }
}
```

### 10.3 回归测试

每个迁移完成后的回归测试步骤：

1. 运行旧管线的所有现有测试 → 确认通过
2. 运行新管线的单元测试 → 确认通过
3. 运行新旧对比测试 → 确认输出一致（或差异在可接受范围）
4. 运行所有 examples/ 目录下的示例 → 确认不崩溃

### 10.4 DataProcessor 的"固件测试"

```rust
/// 给定固定的 SubplotSpec 和数据，验证 VisualElement 输出是否符合预期
/// 不依赖渲染器，可以在 CI 中运行
#[test]
fn test_bar_processor_fixed_output() {
    let spec = SubplotSpec {
        id: 0,
        bounds: Rect::new(50.0, 50.0, 400.0, 300.0),
        series_indices: vec![0],
        x_axis_indices: vec![0],
        y_axis_indices: vec![0],
    };

    let option = create_test_option();
    let color_ctx = ColorContext::default();
    let axis_ranges = ResolvedAxisRanges { ranges: vec![
        ResolvedAxisRange { axis_index: 0, min: 0.0, max: 250.0, .. },
    ]};
    let mut text_measurer = TextMeasurer::new();

    let input = DataProcessorInput {
        spec: &spec,
        option: &option,
        colors: &color_ctx,
        axis_ranges: &axis_ranges,
        external_data: None,
        text_measurer: &mut text_measurer,
    };

    let processor = BarProcessor { series_index: 0 };
    let result = processor.process(input).unwrap();

    // 验证柱子数量
    assert_eq!(result.series_elements.len(), 3);

    // 验证第一根柱子的位置和颜色
    if let VisualElement::Rect { rect, style } = &result.series_elements[0] {
        assert!((rect.x0 - 100.0).abs() < 2.0);
        assert!((rect.y0 - 150.0).abs() < 2.0);
        assert!(style.fill.is_some());
    } else {
        panic!("期望 Rect，得到其他类型");
    }
}
```

---

## 11. 工作量估算与里程碑

### 11.1 工作量估算

| Phase | 内容 | 估算人天 | 文件数 | 新增行数 | 删除行数 |
|:-----:|------|:--------:|:------:|:--------:|:--------:|
| 0 | 骨架搭建 | 0.5 | 9 | ~200 | 0 |
| 1 | GridPlanner 实现 | 1 | 2 | ~150 | 0 |
| 2 | 饼图试点 | 2 | 2 | ~200 | 0 |
| 3 | 柱状图迁移 | 3 | 2 | ~400 | ~200 |
| 3 | 折线图迁移 | 2 | 1 | ~250 | ~150 |
| 3 | 散点图迁移 | 1 | 1 | ~150 | ~100 |
| 3 | 其他系列 | 5 | 5 | ~500 | ~300 |
| 4 | 旧代码清理 | 2 | -10 | 0 | ~1000 |
| 合计 | | ~16.5 | | ~1850 | ~1750 |

### 11.2 里程碑

| 里程碑 | 时间 | 交付物 | 风险 |
|--------|:----:|--------|:----:|
| M0: 骨架完成 | Day 1 | 新模块目录结构，编译通过 | 低 |
| M1: GridPlanner | Day 2 | GridPlanner 替代 GridManager | 低 |
| M2: 饼图试点 | Day 4 | 饼图新旧管线并行，输出一致 | 中 |
| M3: 50% 系列迁移 | Day 8 | Pie + Bar + Line 完成迁移 | 中 |
| M4: 100% 系列迁移 | Day 12 | 所有系列完成迁移 | 中 |
| M5: 旧代码清理 | Day 14 | 删除旧 Component/Pipeline/Layout | 高（需确保无遗漏） |
| M6: 稳定化 | Day 16 | 全量测试通过，文档更新 | 低 |

### 11.3 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|:----:|:----:|---------|
| 新旧管线输出不一致 | 高 | 中 | 逐个元素对比，容忍微小浮点差异 |
| 文本测量结果不同 | 中 | 中 | 统一使用 TextMeasurer，确保相同字体渲染 |
| 旧代码清理漏掉引用 | 中 | 高 | 每次删除前 `cargo check`，逐步删除 |
| 并行 feature flag 维护成本 | 低 | 低 | 迁移完成后立即清理 flag |
| 极坐标系列（雷达/仪表盘）复杂度 | 中 | 中 | 放在迁移计划最后，有足够时间学习 |

---

## 附录

### A. 现有 Component → DataProcessor 对照表

| 旧 Component | 新 Processor | 迁移阶段 | 关键差异 |
|-------------|-------------|:--------:|---------|
| PieSeriesComponent | PieProcessor | Phase 2 | 不再需要 PolarPieMapper + IdentityTransformer |
| BarSeriesComponent | BarProcessor | Phase 3 | 不再需要 CartesianBarMapper + SeriesContext |
| LineSeriesComponent | LineProcessor | Phase 3 | 类似 Bar，几何输出 Path 而非 Rect |
| ScatterSeriesComponent | ScatterProcessor | Phase 3 | 双数值轴，Circle 输出 |
| CandlestickSeriesComponent | CandlestickProcessor | Phase 3 | OHLC 数据转换 |
| BubbleSeriesComponent | BubbleProcessor | Phase 3 | 三数值维 + 半径映射 |
| RadarSeriesComponent | RadarProcessor | Phase 3 | 极坐标映射 |
| GaugeSeriesComponent | GaugeProcessor | Phase 3 | 角度轴 + 指针 |
| PolarBarSeriesComponent | PolarBarProcessor | Phase 3 | 极坐标柱状图 |
| PolarScatterSeriesComponent | PolarScatterProcessor | Phase 3 | 极坐标散点图 |
| TableSeriesComponent | (保留旧实现) | — | 表格不参与视觉重构 |

### B. 依赖关系图

```
new_pipeline/
├── types.rs                  ← 无依赖（基础类型）
├── text_measurer.rs          ← 依赖 text.rs
├── grid_planner.rs            ← 依赖 types.rs + option.rs (GridOption)
├── axis_binding_resolver.rs   ← 依赖 types.rs + option.rs
├── color_assigner.rs          ← 依赖 types.rs + theme.rs + option.rs
├── data_processor.rs          ← 依赖 types.rs + visual.rs + error.rs
├── visual_element_builder.rs  ← 依赖 types.rs + visual.rs + option.rs
├── processor/
│   ├── pie.rs                ← 依赖 data_processor.rs + option.rs (PieSeriesOption)
│   ├── bar.rs                ← 依赖 data_processor.rs + shared/tick.rs + shared/axis_range.rs
│   ├── line.rs               ← 同上
│   └── ...
└── shared/
    ├── tick.rs               ← 依赖 types.rs
    └── axis_range.rs         ← 依赖 types.rs
```

### C. 编译命令

```bash
# 阶段性编译验证
cargo build                                          # Phase 0: 骨架
cargo build && cargo test --test grid_planner_tests  # Phase 1: GridPlanner
cargo build && cargo test --test pie_processor_tests # Phase 2: 饼图
cargo check --all-features                           # 全 feature 验证
cargo clippy                                         # 代码风格检查
```