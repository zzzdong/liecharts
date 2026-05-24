# liecharts 架构设计文档

> 版本: v0.1.0-beta.1
> 最后更新: 2026-05-21

## 目录

1. [项目概述](#1-项目概述)
2. [整体架构](#2-整体架构)
3. [模块详解](#3-模块详解)
4. [核心数据流](#4-核心数据流)
5. [设计模式](#5-设计模式)
6. [与 ECharts 的关系](#6-与-echarts-的关系)
7. [关键文件清单](#7-关键文件清单)

---

## 1. 项目概述

liecharts 是一个使用 Rust 编写的图表库，受 Apache ECharts 启发，支持 11+ 种图表类型，提供 PNG/JPEG 位图和 SVG 矢量图双渲染引擎。

### 1.1 核心特性

- **丰富的图表类型**: 折线图、柱状图、饼图、面积图、散点图、雷达图、仪表盘、K 线图、极坐标柱状图、极坐标散点图、气泡图、表格
- **双渲染引擎**: PNG/JPEG（基于 vello_cpu）和 SVG（原生 XML 生成）
- **主题系统**: 基于 ECharts 6 Design Tokens 的完整主题体系
- **JSON 配置**: 支持通过 JSON 直接配置图表
- **复杂布局**: 多 grid 混合图表、多 Y 轴支持
- **Fluent Builder API**: Rust 链式调用风格

### 1.2 技术栈

| 分类 | 依赖 | 用途 |
|------|------|------|
| 渲染引擎 | `vello_cpu` | 位图渲染（Compute CID） |
| 文字排版 | `parley` | 文本布局与测量 |
| 序列化 | `serde` / `serde_json` | JSON 配置解析 |
| 图像编码 | `image` | PNG/JPEG 编码输出 |
| 错误处理 | `thiserror` | 错误类型定义 |

---

## 2. 整体架构

liecharts 采用**分层架构**，自底向上分为 8 层：

```
┌──────────────────────────────────────────────────────────────────┐
│                         用户 API 层                               │
│   ChartBuilder (Fluent API)  /  serde_json 直接反序列化           │
├──────────────────────────────────────────────────────────────────┤
│   Option 层 (option.rs)                                          │
│   ChartOption — 原始用户配置，mirror ECharts JSON schema         │
│   { title, legend, grid[], xAxis[], yAxis[], series[], color }   │
├──────────────────────────────────────────────────────────────────┤
│   Model 层 (model.rs)                                            │
│   ChartModel::new(option, theme) → 解析为具体类型                 │
│   解决 Theme 默认值、颜色分配、数据类型转换、堆叠分组索引          │
├──────────────────────────────────────────────────────────────────┤
│   Chart 层 (chart.rs)                                            │
│   Chart { model, width, height } — 布局编排 + 渲染入口            │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  Layout 层 (layout/)                                     │   │
│   │  LayoutEngine — Measure-Arrange 两阶段布局                │   │
│   │  GridManager — 多 grid 统一管理，百分比/像素定位          │   │
│   │  输出: LayoutOutput { title_bbox, legend_bbox, grids[] } │   │
│   └─────────────────────────────────────────────────────────┘   │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  Component 层 (component/)                               │   │
│   │  ChartComponent trait — 每个组件将配置+布局→VisualElement│   │
│   │  ├─ 布局组件: Axis / Title / Legend                       │   │
│   │  └─ 系列组件: Bar / Line / Pie / Scatter / Radar / ...   │   │
│   │     ┌──────────────────────────────────────────────┐     │   │
│   │     │  Pipeline 层 (pipeline/) — 系列渲染三阶段管线  │     │   │
│   │     │  Transform → Map → Build                      │     │   │
│   │     │  ① DataTransformer: 堆叠/百分比/恒等变换       │     │   │
│   │     │  ② CoordinateMapper: 数据→几何坐标(像素)      │     │   │
│   │     │  ③ VisualBuilder: 几何→VisualElement 列表    │     │   │
│   │     └──────────────────────────────────────────────┘     │   │
│   └─────────────────────────────────────────────────────────┘   │
├──────────────────────────────────────────────────────────────────┤
│   Visual 层 (visual.rs)                                         │
│   VisualElement enum — 纯数据图元，与渲染后端完全解耦             │
│   { Rect, Circle, Line, Polyline, Path, TextRun, Group }       │
├──────────────────────────────────────────────────────────────────┤
│   Render 层 (render/)                                           │
│   Renderer trait — 统一渲染接口                                  │
│   ├─ PixmapRenderer (vello_cpu → 位图 → PNG/JPEG)              │
│   └─ SvgRenderer (XML 构建 → SVG 字符串 → .svg 文件)            │
└──────────────────────────────────────────────────────────────────┘
```

### 2.1 依赖方向

每层只依赖正下方的层，不存在跨层依赖：

```
API → Option → Model → Chart → Layout → Component → Visual → Render
                                ↕
                            Pipeline
```

---

## 3. 模块详解

### 3.1 Option 层 — [src/option.rs](../src/option.rs)

**职责**: 定义用户可见的配置结构体，全部支持 `Serialize` / `Deserialize`。

核心类型：

```rust
pub struct ChartOption {
    pub title: Option<TitleOption>,
    pub legend: Option<LegendOption>,
    pub grid: Vec<GridOption>,
    pub x_axis: Vec<AxisOption>,
    pub y_axis: Vec<AxisOption>,
    pub series: Vec<SeriesOption>,
    pub color: Option<Vec<ColorOption>>,
    pub background_color: Option<ColorOption>,
    pub theme: Option<String>,
    pub text_style: Option<TextStyleOption>,
    pub radar: Option<RadarOption>,
}

pub enum SeriesOption {
    Line(LineSeriesOption),
    Bar(BarSeriesOption),
    Pie(PieSeriesOption),
    // ... 共 11 种
}
```

**设计要点**:
- 所有字段为 `Option`，支持 JSON 部分配置
- serde tag 枚举 `SeriesOption` 通过 `"type": "bar"`/`"line"` 自动反序列化
- 每个系列选项提供 builder 方法链（`.smooth(true)`, `.stack("name")`）

### 3.2 Model 层 — [src/model.rs](../src/model.rs)

**职责**: 将 `ChartOption` + `Theme` 解析为渲染可用的具体类型。

核心结构：

```rust
pub struct ChartModel {
    pub grids: Vec<Grid>,
    pub title: Option<Title>,
    pub legend: Option<Legend>,
    pub x_axes: Vec<Axis>,
    pub y_axes: Vec<Axis>,
    pub series: Vec<ResolvedSeries>,
    pub colors: Vec<Color>,
    pub background: Color,
    pub text_style: Option<TextStyle>,
}

pub enum ResolvedSeries {
    Line(LineSeries),
    Bar(BarSeries),
    Pie(PieSeries),
    Candlestick(CandlestickSeries),
    Scatter(ScatterSeries),
    Radar(RadarSeries),
    PolarBar(PolarBarSeries),
    PolarScatter(PolarScatterSeries),
    Bubble(BubbleSeries),
    Gauge(GaugeSeries),
    Table(TableSeries),
}
```

**关键逻辑**:
- **颜色解析**: 优先使用用户配置的颜色，回退到 Theme 调色板
- **主题合并**: 每个视觉属性都走 `option → theme → default` 的回退链
- **自动分组**: `assign_bar_group_indices()` — 根据 stack 和 grid_index 自动为柱状图分配分组索引
- **数据采样**: 支持大数据集的降采样

### 3.3 Builder 层 — [src/builder.rs](../src/builder.rs)

**职责**: 提供 Fluent API 构建入口。

```rust
pub struct ChartBuilder {
    theme_registry: ThemeRegistry,
    option: ChartOption,
}
```

提供三种构建途径：
1. **Fluent API**: `ChartBuilder::new().with_title(...).with_series(...).build(800, 600)`
2. **直接 JSON**: `ChartBuilder::from_option_json(json_string)?.build(800, 600)`
3. **提前获取 Model**: `.build_model()` 用于需要复用 Model 的场景

### 3.4 Chart 层 — [src/chart.rs](../src/chart.rs)

**职责**: 编排布局计算和视觉元素生成。

```rust
pub struct Chart {
    model: ChartModel,
    width: u32,
    height: u32,
}
```

**核心方法 `collect_visual_elements()`**:
1. `compute_layout()` → 运行 LayoutEngine，得到 LayoutOutput
2. `build_visual_elements()` → 遍历所有组件，收集 VisualElement
3. 返回 `(Vec<VisualElement>, width, height)` 供 Renderer 消费

### 3.5 Layout 层 — [src/layout/](../src/layout/)

**职责**: 计算图表各元素的位置和尺寸。

采用 **Measure-Arrange 两阶段布局**（V2 版本）：

| 阶段 | 操作 | 说明 |
|------|------|------|
| **Measure** | `layoutable.measure(constraint)` | 各组件在约束下报告期望尺寸 |
| **Arrange** | `layoutable.arrange(bounds)` | 父组件分配最终位置 |

关键类型：

```rust
// 布局引擎
pub struct LayoutEngine { context, grid_manager }
impl LayoutEngine {
    pub fn layout(&mut self, chart_layout: &mut ChartLayout) -> LayoutOutput
}

// 布局输出
pub struct LayoutOutput {
    pub title_bbox: Option<Rect>,
    pub legend_bbox: Option<Rect>,
    pub grids: Vec<GridLayoutInfo>,
}

// 每个子图的布局信息
pub struct GridLayoutInfo {
    pub grid_index: usize,
    pub grid_bbox: Rect,          // grid 外框（含坐标轴）
    pub grid_inner_bbox: Rect,    // grid 内框（仅绘图区）
    pub x_axis_areas: Vec<AxisArea>,
    pub y_axis_areas: Vec<AxisArea>,
    pub data_coord: DataCoordinateSystem,  // 数据↔像素 坐标系
}
```

**DataCoordinateSystem** — 数据↔像素坐标映射：
- `x_to_pixel(data_x)` / `y_to_pixel(data_y, y_axis_index)`
- 支持多 Y 轴
- 支持类目轴和数值轴
- 支持横向柱状图（Y 轴为类目）

### 3.6 Component 层 — [src/component/](../src/component/)

**职责**: 将配置+布局转换为具体图元。

核心 trait：

```rust
/// 所有图表组件的统一接口
pub trait ChartComponent {
    fn build_visual_elements(&self, resolved: &ChartModel, layout: &LayoutOutput) -> Vec<VisualElement>;
}

/// 系列组件的扩展接口
pub trait SeriesComponent: ChartComponent {
    fn series_index(&self) -> usize;
    fn grid_index(&self) -> usize;
    fn is_empty(&self) -> bool;
}
```

组件分类：

| 类别 | 组件 | 文件 |
|------|------|------|
| 布局组件 | `TitleComponent` | [title.rs](../src/component/title.rs) |
| 布局组件 | `LegendComponent` | [legend.rs](../src/component/legend.rs) |
| 布局组件 | `AxisComponent` | [axis.rs](../src/component/axis.rs) |
| 系列组件 | `BarSeriesComponent` | [bar.rs](../src/component/bar.rs) |
| 系列组件 | `LineSeriesComponent` | [line.rs](../src/component/line.rs) |
| 系列组件 | `PieSeriesComponent` | [pie.rs](../src/component/pie.rs) |
| 系列组件 | `ScatterSeriesComponent` | [scatter.rs](../src/component/scatter.rs) |
| ... | ... | ... |

**SeriesContext** — 为系列渲染提供统一上下文：

```rust
pub struct SeriesContext<'a> {
    pub series_index: usize,
    pub grid_index: usize,
    pub resolved: &'a ChartModel,
    pub layout: &'a LayoutOutput,
    pub grid_info: &'a GridLayoutInfo,
    pub coord: &'a DataCoordinateSystem,
}
```

### 3.7 Pipeline 层 — [src/pipeline/](../src/pipeline/)

**职责**: 实现系列渲染的 Transform → Map → Build 三阶段管线。

这是系列渲染的核心，每个阶段都是 trait 抽象的：

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ ① Transform  │ →  │ ② Map       │ →  │ ③ Build     │
│ DataTransformer│   │ CoordinateMapper│  │ VisualBuilder│
│ 堆叠/百分比/   │   │ 笛卡尔/极坐标  │   │ 几何→图元    │
│ 恒等变换      │   │ 数据→几何坐标  │   │ 带上色/标签  │
└──────────────┘    └──────────────┘    └──────────────┘
```

#### Transform — 数据变换

```rust
pub trait DataTransformer {
    fn transform(&self, all_series: &[ResolvedSeries]) -> Vec<TransformedSeries>;
}
```

| 实现 | 说明 |
|------|------|
| `IdentityTransformer` | 恒等变换，不处理 |
| `StackedTransformer` | 堆叠变换，按 stack 分组累积 |
| `PercentStackedTransformer` | 百分比堆叠，归一化到 100% |

#### Map — 坐标映射

```rust
pub trait CoordinateMapper {
    fn map(&self, transformed: &TransformedSeries, coord: &DataCoordinateSystem, y_axis_index: usize) -> Vec<MappedGeometry>;
}

pub enum MappedGeometry {
    CartesianBar { center_x, bottom_y, top_y, width },
    HorizontalBar { center_y, left_x, right_x, height },
    CartesianPoint { x, y },
    CartesianLine { points, area_baseline },
    PolarSector { center, inner_radius, outer_radius, start_angle, sweep_angle },
}
```

| 实现 | 用途 |
|------|------|
| `CartesianBarMapper` | 柱状图（含横向），支持分组并排 |
| `CartesianLineMapper` | 折线图/面积图 |
| `CartesianScatterMapper` | 散点图 |
| `PolarPieMapper` | 饼图 |

#### Build — 视觉构建

```rust
pub trait VisualBuilder {
    fn build(&self, transformed: &TransformedSeries, mapped: &[MappedGeometry], coord: &DataCoordinateSystem) -> Vec<VisualElement>;
}
```

| 实现 | 用途 |
|------|------|
| `BarVisualBuilder` | 生成带填充色的矩形 + 标签 |
| `LineVisualBuilder` | 生成折线 + 符号 + 面积填充 |
| `PieVisualBuilder` | 生成扇形 + 引导线 + 标签 |
| `ScatterVisualBuilder` | 生成圆形符号 |

### 3.8 Visual 层 — [src/visual.rs](../src/visual.rs)

**职责**: 定义与渲染后端无关的纯数据图元。

```rust
pub enum VisualElement {
    Rect { rect: Rect, style: FillStrokeStyle },
    Circle { center: Point, radius: f64, style: FillStrokeStyle },
    Line { start: Point, end: Point, style: StrokeStyle },
    Polyline { points: Vec<Point>, style: StrokeStyle },
    Path { path: BezPath, style: FillStrokeStyle },
    GradientPath { path: BezPath, gradient: GradientDef, stroke: Option<Stroke> },
    TextRun { text, position, style, rotation, max_width, layout },
    Group { children: Vec<VisualElement>, transform: Option<Transform> },
}
```

### 3.9 Render 层 — [src/render/](../src/render/)

**职责**: 将 VisualElement 转换为最终输出格式。

```rust
pub trait Renderer {
    fn draw_rect(&mut self, rect: Rect, style: &FillStrokeStyle);
    fn draw_circle(&mut self, center: Point, radius: f64, style: &FillStrokeStyle);
    fn draw_line(&mut self, start: Point, end: Point, style: &StrokeStyle);
    fn draw_polyline(&mut self, points: &[Point], style: &StrokeStyle);
    fn draw_path(&mut self, path: &BezPath, style: &FillStrokeStyle);
    fn draw_gradient_path(&mut self, path: &BezPath, gradient: &GradientDef, stroke: Option<&Stroke>);
    fn draw_text(&mut self, text: &str, position: Point, color: Color, font_size: f64, font_family: &str, rotation: f64, layout: Option<&TextLayout>);
    fn begin_group(&mut self, transform: Option<&Transform>);
    fn end_group(&mut self);
}
```

| 渲染器 | 后端 | 输出格式 |
|--------|------|---------|
| `PixmapRenderer` | vello_cpu (Compute CID) | PNG / JPEG |
| `SvgRenderer` | 原生 XML 构建 | SVG |

### 3.10 Theme 层 — [src/theme.rs](../src/theme.rs)

**职责**: 提供完整的 Design Tokens 体系。

```rust
pub struct Theme {
    pub name: String,
    pub color: Vec<String>,          // 主色调色板
    pub design_tokens: DesignTokens, // ECharts 6 完整设计令牌
}

pub struct DesignTokens {
    pub color: ColorTokens,    // 21阶中性色 + 21阶强调色 + 主题色板
    pub text: TextTokens,      // 标题/副标题/图例/坐标轴样式
    pub spacing: SpacingTokens,// 间距系统
    pub border: BorderTokens,  // 边框与分割线
    pub effect: EffectTokens,  // 阴影与效果
}
```

**回退链**: 每个视觉属性解析走 `user option → theme value → global default`。

---

## 4. 核心数据流

### 4.1 从配置到渲染的完整流程

```
ChartBuilder::new()
    .with_title(TitleOption::new("标题"))
    .with_series(SeriesOption::Bar(...))
    .build(800, 600)              ← 构建 Chart
        │
        ├── ChartBuilder::build_model()
        │     └── ChartModel::new(option, theme)
        │           ├── 解析颜色调色板
        │           ├── 解析标题/图例/坐标轴
        │           ├── 为每个系列调用 resolve_series()
        │           │     └── 系列选项 → LineSeries / BarSeries / ...
        │           └── assign_bar_group_indices()
        │                 └── 自动分配分组索引
        │
        └── Chart::new(model, 800, 600)

chart.render_to_svg("output.svg")  ← 渲染入口
    │
    └── Chart::collect_visual_elements()
          ├── compute_layout()
          │     └── LayoutEngine::layout()
          │           ├── measure() — 标题/图例/坐标轴期望尺寸
          │           ├── GridManager::compute_layout() — grid 定位
          │           └── arrange() — 布局最终分配
          │
          └── build_visual_elements()
                ├── 背景矩形
                ├── TitleComponent::build_visual_elements()
                ├── LegendComponent::build_visual_elements()
                └── for each grid:
                      ├── AxisComponent (X)
                      ├── AxisComponent (Y)
                      └── for each series in grid:
                            ├── 创建 SeriesContext
                            ├── DataTransformer::transform()
                            ├── CoordinateMapper::map()
                            └── VisualBuilder::build()
                                  └── Vec<VisualElement>

SvgRenderer::render(elements, width, height)
    └── SVG XML 字符串 → 写入文件
```

### 4.2 多 grid 布局流程

```
3 Grid 配置
     │
     ▼
GridManager::compute_layout()
     │
     ├── 计算每个 grid 的绝对位置（考虑标题/图例占用的空间）
     ├── 处理 grid 间重叠/间距
     │
     ▼
每个 Grid 独立:
  ├── grid_bbox:      grid 外框（含坐标轴）
  ├── grid_inner_bbox:grid 内框（仅绘图区）
  ├── AxisArea:       每个坐标轴的位置 + 标签盒
  └── DataCoordinateSystem: 当前 grid 的数据↔像素映射
```

---

## 5. 设计模式

### 5.1 策略模式 — Pipeline 三阶段

每个阶段使用 trait 抽象，具体策略可替换：

```rust
// 管线组合示例：堆叠柱状图
let transformer = StackedTransformer::new(Some("direct".into()));
let mapper = CartesianBarMapper::new().with_group(0, 2);
let builder = BarVisualBuilder::new()
    .with_series_style(SeriesStyle { color, stroke, fill })
    .with_label_config(label_config);

// 执行管线
let transformed = transformer.transform(all_series);
let mapped = mapper.map(transformed, coord, y_axis_index);
let elements = builder.build(transformed, mapped, coord);
```

### 5.2 模板方法模式 — 系列渲染流程

[renderers.rs](../src/component/renderers.rs) 中的 `render_cartesian_pipeline` 和 `render_polar_pipeline` 定义了固定的渲染流程骨架，具体实现通过 trait 方法定制：

```rust
pub fn render_cartesian_pipeline<R: CartesianRenderer>(renderer: &R, ctx: &SeriesContext) -> Vec<VisualElement> {
    if renderer.is_data_empty() { return Vec::new(); }
    renderer.render_cartesian(ctx)  // 调用具体实现
}
```

### 5.3 组合模式 — 布局树

`ChartLayout` 是一个组合结构：

```
ChartLayout
├── TitleLayout (Option)
├── LegendLayout (Option)
└── SubplotLayout[] (复合)
      ├── GridLayout
      ├── AxisLayout[] (X)
      └── AxisLayout[] (Y)
```

所有元素实现 `Layoutable` trait，统一通过 measure/arrange 处理。

### 5.4 建造者模式 — ChartBuilder

`ChartBuilder` 提供 Fluent API，将复杂对象的构建步骤封装为链式调用：

```rust
ChartBuilder::new()
    .with_title(...)
    .with_x_axis(...)
    .with_y_axis(...)
    .with_series(...)
    .build(800, 600)
```

### 5.5 桥接模式 — 渲染后端

`Renderer` trait 是抽象接口，`PixmapRenderer` 和 `SvgRenderer` 是具体实现。VisualElement 是抽象图元，两个渲染器各自实现绘制逻辑。

---

## 6. 与 ECharts 的关系

### 6.1 相同/相似点

| ECharts 概念 | liecharts 对应 |
|-------------|---------------|
| `option.title` | `TitleOption` |
| `option.legend` | `LegendOption` |
| `option.grid[]` | `Vec<GridOption>` |
| `option.xAxis[]` | `Vec<AxisOption>` |
| `option.series[].type: 'bar'` | `SeriesOption::Bar(...)` |
| `option.series[].stack` | `.stack("name")` |
| `option.color` | `Vec<ColorOption>` |
| `option.series[].label` | `LabelOption` |

### 6.2 JSON 互通

liecharts 可以直接解析 ECharts 风格的 JSON 配置：

```json
{
    "title": { "text": "Sales" },
    "xAxis": [{ "type": "category", "data": ["A", "B"] }],
    "yAxis": [{ "type": "value" }],
    "series": [{ "type": "bar", "name": "Sales", "data": [100, 200] }]
}
```

```rust
ChartBuilder::from_option_json(json_string)?.build(800, 600)?;
```

### 6.3 差异点

| 方面 | ECharts | liecharts |
|------|---------|-----------|
| 语言 | JavaScript | Rust (编译型) |
| 渲染 | Canvas/SVG (浏览器) | vello_cpu/SVG (服务端) |
| 交互 | 丰富 (事件/动画/dataZoom) | 无交互 (静态输出) |
| 布局 | 自动计算 | Measure-Arrange 引擎 |
| 扩展 | 插件机制 | Trait 多态 |

---

## 7. 关键文件清单

| 文件 | 行数 | 复杂度 | 说明 |
|------|------|--------|------|
| [src/option.rs](../src/option.rs) | ~700 | 中等 | 所有用户配置结构体，serde 支持 |
| [src/model.rs](../src/model.rs) | ~950 | 高 | ChartModel 解析 + 11种 ResolvedSeries |
| [src/chart.rs](../src/chart.rs) | ~350 | 高 | 布局编排 + 视觉元素生成入口 |
| [src/builder.rs](../src/builder.rs) | ~150 | 低 | Fluent API 构建器 |
| [src/visual.rs](../src/visual.rs) | ~250 | 低 | 平台无关图元定义 |
| [src/theme.rs](../src/theme.rs) | ~300 | 低 | Design Tokens 定义 |
| [src/layout/engine.rs](../src/layout/engine.rs) | ~300 | 高 | 布局引擎 + GridManager |
| [src/layout/elements.rs](../src/layout/elements.rs) | ~400 | 中 | 各布局元素实现 |
| [src/layout/grid_manager.rs](../src/layout/grid_manager.rs) | ~100 | 中 | Grid 管理器 |
| [src/pipeline/transform.rs](../src/pipeline/transform.rs) | ~200 | 中 | 数据变换器 |
| [src/pipeline/mapper.rs](../src/pipeline/mapper.rs) | ~300 | 中 | 坐标映射器 |
| [src/pipeline/builder.rs](../src/pipeline/builder.rs) | ~250 | 中 | 视觉构建器 |
| [src/component/mod.rs](../src/component/mod.rs) | ~100 | 低 | 组件 trait 定义 + 导出 |
| [src/component/base.rs](../src/component/base.rs) | ~100 | 低 | 系列组件基类 |
| [src/component/context.rs](../src/component/context.rs) | ~180 | 低 | 系列渲染上下文 |
| [src/component/renderers.rs](../src/component/renderers.rs) | ~200 | 低 | 渲染流程函数 |
| [src/component/bar.rs](../src/component/bar.rs) | ~150 | 中 | 柱状图组件 |
| [src/component/line.rs](../src/component/line.rs) | ~150 | 中 | 折线图组件 |
| [src/component/pie.rs](../src/component/pie.rs) | ~150 | 中 | 饼图组件 |
| [src/component/axis.rs](../src/component/axis.rs) | ~300 | 高 | 坐标轴组件（含刻度计算） |
| [src/render/mod.rs](../src/render/mod.rs) | ~100 | 低 | Renderer trait 定义 |
| [src/render/svg.rs](../src/render/svg.rs) | ~300 | 中 | SVG 渲染实现 |
| [src/render/pixmap.rs](../src/render/pixmap.rs) | ~200 | 中 | 位图渲染实现 |