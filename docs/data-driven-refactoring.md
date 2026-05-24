# 数据驱动重构方案

> 评估日期: 2026-05-21
> 对应版本: v0.1.0-beta.1

## 目录

1. [现状分析](#1-现状分析)
2. [数据驱动架构的目标](#2-数据驱动架构的目标)
3. [可选方案](#3-可选方案)
4. [推荐方案：渐进式混合架构](#4-推荐方案渐进式混合架构)
5. [Phase 1 详细设计：DataSpec 层](#5-phase-1-详细设计dataspec-层)
6. [Phase 2 详细设计：统一管线重构](#6-phase-2-详细设计统一管线重构)
7. [Phase 3 详细设计：数据驱动的系列渲染](#7-phase-3-详细设计数据驱动的系列渲染)
8. [Phase 4 详细设计：Layer/分面/统计变换](#8-phase-4-详细设计layer分面统计变换)
9. [风险与收益分析](#9-风险与收益分析)
10. [附录：与 Vega-Lite/ggplot2 的对比](#10-附录与vega-liteggplot2-的对比)

---

## 1. 现状分析

### 1.1 当前架构特点

liecharts 当前设计紧密参照 Apache ECharts，采用"配置驱动"模式：

- **用户提供完整的视觉配置**：指定图表类型、坐标轴类型、颜色、标签位置等
- **选项模型直接映射 JSON**：`ChartOption` 结构体 mirror ECharts JSON schema
- **系列类型为 enum 变体**：每个图表类型有独立的选项结构体和组件实现
- **细粒度的视觉控制**：用户可以控制每一个像素级细节

### 1.2 当前架构的优势

| 优势 | 说明 |
|------|------|
| **ECharts 生态兼容** | 用户可复用 ECharts 配置心智模型，前后端共享同一套配置语言 |
| **高可控性** | 每个视觉细节都可手动配置 |
| **复杂布局能力** | 多 grid 混合图表、多 Y 轴、分组堆叠组合等 |
| **JSON 互通** | serde 直接反序列化 ECharts 风格 JSON，利于前后端分离 |

### 1.3 当前架构的局限

| 局限 | 具体问题 | 影响 |
|------|---------|------|
| **配置冗长** | 每个系列需单独指定类型、名称、数据、样式 | 简单图表也需多行配置 |
| **数据抽象层级低** | 数据只是 `Vec<f64>` / `Vec<DataPoint>`，无列式/表格数据支持 | 用户需手动处理数据聚合和变换 |
| **系列扩展成本高** | 每新增一种系列需在 7+ 处添加代码 | 维护负担重 |
| **无自动推断** | 需手动指定轴类型、图表类型、刻度范围 | 学习曲线陡峭 |
| **resolve 逻辑集中** | `ChartModel::new()` ~650 行巨型函数，`resolve_series` 11 分支 | 难以维护和测试 |
| **重复的管线实现** | 每种系列有自己的 Pipeline + Component，代码高度相似 | 重复劳动 |

### 1.4 当前架构的代码分布

当前每种系列类型的代码分布在以下 7 个位置：

```
option.rs           → 系列选项结构体 (BarSeriesOption, LineSeriesOption, ...)
model.rs            → ResolvedSeries enum 变体 + resolve_series() 分支
component/xxx.rs    → XxxSeriesComponent 组件实现
pipeline/mapper.rs  → XxxMapper 坐标映射
pipeline/builder.rs → XxxVisualBuilder 视觉构建
pipeline/mod.rs     → SeriesStyle / LabelConfig 扩展
chart.rs            → SubplotContext::build_visual_elements() 匹配分支
```

新增一种系列类型需要同时修改以上所有文件。

---

## 2. 数据驱动架构的目标

### 2.1 核心原则

1. **数据优先**：用户描述"有什么数据"和"想看什么"，而非"怎么画"
2. **声明式**：声明"what"，而非"how"
3. **自动推断**：从数据特性推断合适的视觉编码
4. **可组合**：支持图层的叠加、分面的组合
5. **渐进增强**：简单场景一行代码出图，复杂场景仍可精细控制

### 2.2 理想的使用方式

```rust
// 当前方式：ECharts 风格 - 配置每个视觉细节
ChartBuilder::new()
    .with_title(TitleOption::new("月度销售"))
    .with_x_axis(AxisOption::category().data(["1月", "2月", "3月"]))
    .with_y_axis(AxisOption::value())
    .with_series(SeriesOption::Bar(
        BarSeriesOption::new("销售额", vec![120.0, 200.0, 150.0])
    ))
    .build(800, 600)?

// 目标方式：数据驱动 - 声明数据 + 映射
ChartBuilder::new()
    .with_data(DataFrame::new(data))        // 提供表格数据
    .mark(MarkType::Bar)                      // 声明图表类型
    .encode(Encoding::new()                  // 声明视觉映射
        .x(Column("month"))
        .y(Column("sales"))
        .color(Column("region"))
        .stack(Column("channel"))
    )
    .build(800, 600)?

// 高级方式：叠加 + 分面
ChartBuilder::new()
    .with_data(DataFrame::new(data))
    .layer(Layer::new()
        .mark(MarkType::Bar)
        .encode(Encoding::new()
            .x(Column("month")).y(Aggregate::Sum("sales"))
        )
    )
    .layer(Layer::new()
        .mark(MarkType::Line)
        .encode(Encoding::new()
            .x(Column("month")).y(Aggregate::Avg("growth"))
        )
    )
    .facet(Facet::wrap(Column("year"), 2))  // 自动分面
    .build(1200, 800)?
```

---

## 3. 可选方案

### 3.1 方案 A：轻量级 — 在现有 API 上叠加 DataSpec 层

**核心思路**：在 `ChartBuilder` 上增加一个 `from_data_spec()` 方法，将高层次的数据规范编译为现有的 `ChartOption`。

```
用户 API (DataSpec)
     ↓ GoG Compiler
ChartOption (现有)
     ↓ 现有管线
渲染输出
```

**优点**：
- 非侵入式，新老 API 共存
- 不修改现有代码，零风险
- 快速交付

**缺点**：
- 底层仍是 ECharts 模式，双重维护
- 无法消除系列扩展的高成本

**工作量估计**：2-3 周

### 3.2 方案 B：中量级 — 引入 Grammar of Graphics 中间层

**核心思路**：在 Option 层之上增加一个完整的 GoG（Grammar of Graphics）编译层，包含数据类型推断、自动轴推断、自动缩放等。

```
用户 API (DataSpec)
     ↓ GoG Compiler (类型推断 + 编码映射 + 缩放)
ChartOption (现有)
     ↓ 现有管线
渲染输出
```

**关键组件**：
- `DataFrame` — 列式数据容器
- `Mark` — 可视化标记类型
- `Encoding` — 数据列→视觉通道映射
- `Scale` — 数据空间→视觉空间缩放
- `DataTransform` — 过滤/聚合/排序/窗口函数

**优点**：
- 真正解耦"数据描述"和"视觉配置"
- 可支持高级可视化语法
- 编译到现有 Option 层，兼容 JSON 导出

**缺点**：
- 需要实现非平凡的编译器
- 部分 ECharts 精细控制需要"逃逸口"

**工作量估计**：6-8 周

### 3.3 方案 C：重量级 — 完全重构为数据驱动架构

**核心思路**：抛弃 ECharts 选项模型，从头设计类似 ggplot2 或 Vega-Lite 的架构。

```
DataSpec + Encoding + Mark
     ↓
统一中间表示 (IR)
     ↓
通用渲染管线 (不再分系列类型)
     ↓
VisualElement → Renderer
```

**优点**：
- 最自然、最强大的表达力
- 从根本上消除系列扩展成本
- 类型安全的编译时检查

**缺点**：
- 完全重写，破坏性变更
- 开发周期长（3-6个月）
- 高风险，可能丢失现有 JSON 兼容优势

**工作量估计**：12-24 周

### 3.4 方案对比

| 维度 | 方案 A (轻量) | 方案 B (中量) | 方案 C (重量) |
|------|:------------:|:------------:|:------------:|
| 交付速度 | 快 (2-3周) | 中 (6-8周) | 慢 (12-24周) |
| 侵入性 | 无 | 低 | 高 (破坏性) |
| 数据抽象 | 低 | 中 | 高 |
| 自动推断 | 无 | 有 | 强 |
| 扩展成本 | 不变 | 降低 | 消除 |
| JSON 兼容 | 保持 | 保持 | 需额外层 |
| API 一致性 | 双 API | 双 API | 统一新 API |
| 风险 | 低 | 中 | 高 |

---

## 4. 推荐方案：渐进式混合架构

### 4.1 总体策略

采用 **方案 A → 方案 B 分步演进**，共 4 个阶段：

```
Phase 1 (2-3周)          Phase 2 (2周)           Phase 3 (3周)           Phase 4 (3周+)
┌──────────────┐      ┌──────────────┐       ┌──────────────┐        ┌──────────────┐
│ 引入 DataSpec │  →   │ 提炼公共管线  │   →   │ 数据驱动渲染  │   →   │ Layer/分面    │
│ 零破坏性叠加  │      │ 重构 resolve  │       │ 统一管线      │       │ 统计变换      │
└──────────────┘      └──────────────┘       └──────────────┘        └──────────────┘
    非侵入              低风险                中等风险                长期目标
```

### 4.2 各阶段依赖关系

```
Phase 1 (DataSpec)
     │
     ▼
Phase 2 (公共管线) ─── 可独立交付 ───→ 降低当前维护成本
     │
     ▼
Phase 3 (数据驱动渲染) ─── 依赖 Phase 1 + 2 ───→ 减少系列代码量
     │
     ▼
Phase 4 (高级特性) ─── 依赖 Phase 3 ───→ 完整 GoG 实现
```

每个阶段都是可独立交付的，可以根据优先级跳过或延迟后续阶段。

---

## 5. Phase 1 详细设计：DataSpec 层

### 5.1 目标

在现有 API 上叠加一个更高层次的数据规范层，使用户可以用更少的代码描述图表。

### 5.2 新增文件

```
src/spec/
├── mod.rs          # 模块导出
├── data_frame.rs   # DataFrame 列式数据容器
├── encoding.rs     # Encoding / Channel 定义
├── mark.rs         # MarkType 定义
├── compiler.rs     # DataSpec → ChartOption 编译器
└── inference.rs    # 数据类型/轴类型推断
```

### 5.3 核心类型设计

```rust
// === data_frame.rs ===

/// 列式数据容器
pub struct DataFrame {
    columns: Vec<Column>,
    row_count: usize,
}

pub struct Column {
    pub name: String,
    pub data: ColumnData,
}

pub enum ColumnData {
    Text(Vec<String>),
    Number(Vec<f64>),
    // 后续可扩展: Date, Category, Boolean
}

impl DataFrame {
    pub fn new(data: impl Into<DataFrame>) -> Self;
    pub fn from_csv(csv: &str) -> Result<Self>;
    pub fn from_records(records: &[impl Serialize]) -> Result<Self>;
    pub fn column(&self, name: &str) -> Option<&Column>;
    pub fn infer_type(&self, name: &str) -> ColumnType;
}

pub enum ColumnType {
    Text,
    Number,
    Category,  // 低基数的 Text 自动推断为 Category
    Temporal,  // 日期时间 (后续支持)
}

// === encoding.rs ===

/// 视觉编码
pub struct Encoding {
    pub x: Option<Channel>,
    pub y: Option<Channel>,
    pub color: Option<Channel>,
    pub size: Option<Channel>,
    pub shape: Option<Channel>,
    pub stack: Option<Channel>,
    pub facet: Option<Channel>,
}

pub struct Channel {
    pub field: String,           // 数据列名
    pub aggregate: Option<AggregateOp>,  // 聚合操作
    pub axis_type: Option<AxisType>,     // 可选：强制指定轴类型
    pub scale: Option<ScaleConfig>,      // 可选：自定义缩放
}

pub enum AggregateOp {
    Sum, Mean, Count, Min, Max, Median,
}

/// 标记类型
pub enum MarkType {
    Bar,
    Line,
    Point,
    Area,
    Pie,
    Scatter,
    Radar,
    // ...
}

// === compiler.rs ===

/// 高层数据规范
pub struct DataSpec {
    pub data: DataFrame,
    pub mark: MarkType,
    pub encoding: Encoding,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl DataSpec {
    /// 编译为 ChartOption
    pub fn compile(self) -> Result<ChartOption>;
}
```

### 5.4 Compiler 编译规则

| 输入 | 编译输出 | 推断逻辑 |
|------|---------|---------|
| mark=Bar, x="month", y="sales" | `SeriesOption::Bar` + category X axis + value Y axis | 文本列→Category 轴，数字列→Value 轴 |
| mark=Line, x="date", y="price" | `SeriesOption::Line` + category X axis + value Y axis | 自动设置 `smooth: true` 如果数据点密集 |
| color="region" | 自动按 region 分组，每个组生成一个 series | 不同值生成不同颜色 |
| stack="channel" | 为每个 series 设置 `.stack("channel")` | 按 stack 字段分组 |
| aggregate=Sum | 自动对数据按 x+color 分组求和 | 数据预处理后再生成 series |

### 5.5 使用方式

```rust
// 方式一：从 DataFrame 构建
let df = DataFrame::from_records(sales_data)?;
ChartBuilder::new()
    .from_data_spec(DataSpec {
        data: df,
        mark: MarkType::Bar,
        encoding: Encoding::new()
            .x("month")
            .y("sales")
            .color("region")
            .stack("channel"),
        title: Some("月度销售".into()),
        width: Some(800),
        height: Some(600),
    })
    .build_model()?;
// 注意: 此方法返回 ChartModel，可以使用现有的渲染接口

// 方式二：更简洁的链式语法
ChartBuilder::from_data(DataFrame::from_csv(csv_data)?)
    .mark(MarkType::Bar)
    .encode(Encoding::new().x("month").y("sales"))
    .build(800, 600)?
    .render_to_svg("chart.svg")?;
```

### 5.6 向后兼容

- `ChartBuilder` 保持现有方法不变
- 新增 `.from_data_spec()` 和 `.encode()` 方法
- `DataSpec::compile()` 内部生成 `ChartOption`，走现有管线
- 用户仍可继续使用旧的 Builder API 和 JSON 配置

---

## 6. Phase 2 详细设计：统一管线重构

### 6.1 目标

提炼公共代码，减少系列扩展的重复劳动，降低 `model.rs` 的复杂度。

### 6.2 重构方案

#### 6.2.1 提取 SeriesResolver

```rust
// 新增 trait
pub trait SeriesResolver {
    fn resolve(&self, option: &SeriesOption, theme: &Theme, colors: &[Color], index: usize) -> Result<ResolvedSeries>;
}

// 注册表模式替代巨型 match
pub struct SeriesResolverRegistry {
    resolvers: HashMap<&'static str, Box<dyn SeriesResolver>>,
}

impl SeriesResolverRegistry {
    pub fn new() -> Self {
        let mut registry = HashMap::new();
        registry.insert("bar", Box::new(BarResolver));
        registry.insert("line", Box::new(LineResolver));
        // ...
        Self { resolvers: registry }
    }

    pub fn resolve(&self, option: &SeriesOption, theme: &Theme, colors: &[Color], index: usize) -> Result<ResolvedSeries> {
        let type_name = option.type_name();
        let resolver = self.resolvers.get(type_name).ok_or(/* error */)?;
        resolver.resolve(option, theme, colors, index)
    }
}
```

#### 6.2.2 提取通用数据项解析

```rust
// 通用数据解析工具
pub struct DataParser;
impl DataParser {
    pub fn resolve_data(data: &[DataPoint]) -> Result<Vec<DataItem>>;
    pub fn resolve_scatter_data(data: &[ScatterDataPoint]) -> Result<Vec<ScatterDataItem>>;
    pub fn resolve_candlestick_data(data: &[CandlestickDataPoint]) -> Result<Vec<CandlestickDataItem>>;
}

// 通用样式解析工具
pub struct StyleResolver;
impl StyleResolver {
    pub fn resolve_item_style(option: ItemStyleOption, theme: &Theme) -> ItemStyle;
    pub fn resolve_line_style(option: LineStyleOption, theme: &Theme, default_color: Color) -> LineStyle;
    pub fn resolve_label(option: LabelOption) -> Label;
    pub fn resolve_text_style(option: TextStyleOption, theme: &Theme) -> TextStyle;
}
```

#### 6.2.3 效果

| 指标 | 当前 | 重构后 |
|------|------|--------|
| `model.rs` 行数 | ~950 | ~500 |
| 新增系列改动位置 | 7 处 | 3 处 (option.rs + resolver + component) |
| resolve_series match 分支 | 11 | 0 (注册表模式) |

---

## 7. Phase 3 详细设计：数据驱动的系列渲染

### 7.1 目标

用统一的渲染管线替代每种系列独立的 Pipeline + Component 实现，减少重复代码。

### 7.2 核心思路

将系列渲染从"按类型分发"改为"按标记类型 (MarkType) 分发"：

```
当前：BarComponent → BarMapper → BarVisualBuilder
      LineComponent → LineMapper → LineVisualBuilder
      PieComponent → PieMapper → PieVisualBuilder
      ...

目标：GenericSeriesComponent
      ├── MarkType::Bar  → GenericMapper::map_bar()
      ├── MarkType::Line → GenericMapper::map_line()
      ├── MarkType::Pie  → GenericMapper::map_pie()
      └── ...            → GenericMapper::map_xxx()
      │
      └── GenericVisualBuilder (根据 MarkType + Encoding 构建图元)
```

### 7.3 统一系列表示

```rust
/// 统一系列类型，替代 11 个独立的 ResolvedSeries 变体
pub struct GenericSeries {
    pub name: String,
    pub mark: MarkType,
    pub data: Vec<DataItem>,
    pub encoding: EncodingMeta,  // 编码元信息（x/y/color/stack 等）
    pub style: SeriesStylePack,  // 所有样式（颜色/描边/标签等）
    pub grid_index: usize,
    pub y_axis_index: usize,
}

/// 编码元信息 — 记录数据到视觉通道的映射
pub struct EncodingMeta {
    pub x_field: Option<String>,
    pub y_field: Option<String>,
    pub color_field: Option<String>,
    pub stack_field: Option<String>,
    pub has_area: bool,       // 是否填充面积
    pub smooth: bool,          // 是否平滑
    pub symbol: Symbol,        // 符号类型
    pub symbol_size: f64,
}
```

### 7.4 统一管线

```rust
// 一个统一的管线，根据 MarkType 分派
struct GenericRenderer {
    series: GenericSeries,
    series_index: usize,
    grid_index: usize,
}

impl CartesianRenderer for GenericRenderer {
    fn render_cartesian(&self, ctx: &SeriesContext) -> Vec<VisualElement> {
        let transformed = self.transform(ctx);
        let mapped = self.map(&transformed, ctx);
        self.build(&transformed, &mapped, ctx)
    }
}

impl GenericRenderer {
    fn transform(&self, ctx: &SeriesContext) -> TransformedSeries {
        if self.series.encoding.stack_field.is_some() {
            StackedTransformer::new(...).transform(ctx.resolved.series)
        } else {
            IdentityTransformer.transform(ctx.resolved.series)
        }
    }

    fn map(&self, transformed: &TransformedSeries, ctx: &SeriesContext) -> Vec<MappedGeometry> {
        match self.series.mark {
            MarkType::Bar => {
                let mapper = CartesianBarMapper::new()
                    .with_group(self.series.group_index, group_count);
                mapper.map(transformed, ctx.coord, self.series.y_axis_index)
            }
            MarkType::Line => {
                let mapper = CartesianLineMapper::new(self.series.encoding.smooth);
                mapper.map(transformed, ctx.coord, self.series.y_axis_index)
            }
            MarkType::Point | MarkType::Scatter => {
                let mapper = CartesianScatterMapper::new();
                mapper.map(transformed, ctx.coord, self.series.y_axis_index)
            }
            MarkType::Area => {
                let mapper = CartesianLineMapper::new(true).with_area(true);
                mapper.map(transformed, ctx.coord, self.series.y_axis_index)
            }
            // ...
        }
    }

    fn build(&self, transformed: &TransformedSeries, mapped: &[MappedGeometry], ctx: &SeriesContext) -> Vec<VisualElement> {
        match self.series.mark {
            MarkType::Bar => {
                let builder = BarVisualBuilder::new()
                    .with_series_style(self.series.style.to_pipeline_style())
                    .with_label_config(...);
                builder.build(transformed, mapped, ctx.coord)
            }
            MarkType::Line | MarkType::Area => {
                let builder = LineVisualBuilder::new()
                    .with_series_style(...);
                builder.build(transformed, mapped, ctx.coord)
            }
            // ...
        }
    }
}
```

### 7.5 效果

| 指标 | 当前 | 重构后 |
|------|------|--------|
| 系列组件文件数 | 11+ | 1 (`GenericSeriesComponent`) |
| 系列组件代码行数 | ~1500 | ~300 |
| 新增标记改动位置 | 7 处 | 2-3 处 (map + build 分支) |

---

## 8. Phase 4 详细设计：Layer/分面/统计变换

### 8.1 目标

支持高级可视化特性，使 liecharts 具备接近 Vega-Lite 的表达能力。

### 8.2 Layer（图层叠加）

```rust
/// 复合图层
pub struct Layer {
    pub data: Option<DataFrame>,     // 可选：图层可覆盖数据
    pub mark: MarkType,
    pub encoding: Encoding,
}

/// ChartOption 扩展
pub struct ChartOption {
    // ... 现有字段
    pub layers: Option<Vec<Layer>>,  // 新增：图层叠加
}
```

**编译规则**：
- 如果 `layers` 存在，每个 Layer 编译为一个或多个 `SeriesOption`
- 所有 Layer 共享同一个坐标系（grid/xAxis/yAxis）
- 支持混搭标记：柱状图 + 折线图叠加

**使用示例**：

```rust
ChartBuilder::new()
    .with_data(DataFrame::from_records(data)?)
    .layer(Layer::new()
        .mark(MarkType::Bar)
        .encode(Encoding::new().x("month").y("sales")))
    .layer(Layer::new()
        .mark(MarkType::Line)
        .encode(Encoding::new().x("month").y("growth_rate")))
    .build(800, 600)?
```

### 8.3 Facet（分面）

```rust
pub enum Facet {
    /// 按列分面，每行 n 个
    Wrap { field: String, cols: usize },
    /// 行列分面
    Grid { row_field: String, col_field: String },
}
```

**编译规则**：
- Facet 将数据按分面字段分组
- 每组生成一个独立的子图（映射到 `GridOption`）
- 自动布局：用 `GridManager` 排列多 grid

**使用示例**：

```rust
ChartBuilder::new()
    .with_data(DataFrame::from_records(data)?)
    .mark(MarkType::Bar)
    .encode(Encoding::new().x("month").y("sales"))
    .facet(Facet::Wrap { field: "year".into(), cols: 2 })
    .build(1200, 800)?
```

### 8.4 统计变换

```rust
pub enum StatTransform {
    /// 按指定字段分组聚合
    Aggregate {
        group_by: Vec<String>,
        field: String,
        op: AggregateOp,
    },
    /// 数据分箱（用于直方图）
    Bin { field: String, bins: usize },
    /// 排序
    Sort { field: String, order: SortOrder },
    /// 窗口函数（移动平均等）
    Window { field: String, op: WindowOp, window_size: usize },
}
```

**使用示例**：

```rust
// 统计变换在 Encoding 层面声明
let encoding = Encoding::new()
    .x(Column("month"))
    .y(Channel::new("sales").aggregate(AggregateOp::Sum));

// 或者作为独立的数据变换步骤
ChartBuilder::new()
    .with_data(DataFrame::from_records(data)?)
    .transform(StatTransform::Aggregate {
        group_by: vec!["month".into()],
        field: "sales".into(),
        op: AggregateOp::Sum,
    })
    .mark(MarkType::Bar)
    .encode(Encoding::new().x("month").y("sales"))
    .build(800, 600)?
```

---

## 9. 风险与收益分析

### 9.1 综合评估

```
收益
↑
│        Phase 4
│        (Layer/分面/统计)
│                    Phase 3
│                    (数据驱动渲染)
│              Phase 2
│              (公共管线)
│        Phase 1
│        (DataSpec)
│
└──────────────────────────────→ 风险/投入
   低                            高
```

### 9.2 分阶段评估

| 阶段 | 投入 | 收益 | 风险 | 是否建议立即执行 |
|------|------|------|------|:--------------:|
| Phase 1 | 2-3周 | 中等（改善 API 易用性） | 低 | 是 |
| Phase 2 | 2周 | 中等（降低维护成本） | 低 | 是 |
| Phase 3 | 3-4周 | 高（消除重复代码） | 中 | 是（依赖 Phase 1+2） |
| Phase 4 | 4-8周 | 高（显著提升表达力） | 中-高 | 视项目优先级决定 |

### 9.3 不推荐的做法

1. **不要一次性全部重构** — 风险高、周期长、难以回退
2. **不要废弃 ECharts 风格 API** — JSON 兼容性是核心竞争力，应保留作为底层
3. **不要在 Phase 1 就引入完整的 GoG** — 先验证 DataSpec 概念，再逐步扩展

### 9.4 成功指标

| 指标 | Phase 1 后 | Phase 2 后 | Phase 3 后 |
|------|:---------:|:---------:|:---------:|
| 简单图表代码量减少 | 40%+ | 40%+ | 50%+ |
| model.rs 行数 | ~950 | ~500 | ~300 |
| 新增标记改动位置 | 7 | 3 | 2 |
| 用户学习成本 | 降低 | 不变 | 降低 |
| JSON 兼容性 | 保持 | 保持 | 保持 |

---

## 10. 附录：与 Vega-Lite/ggplot2 的对比

### 10.1 Vega-Lite

Vega-Lite 是数据驱动可视化的参考实现，其核心概念：

```json
{
  "data": { "values": [...] },
  "mark": "bar",
  "encoding": {
    "x": {"field": "month", "type": "nominal"},
    "y": {"field": "sales", "type": "quantitative", "aggregate": "sum"},
    "color": {"field": "region", "type": "nominal"}
  }
}
```

liecharts Phase 1-3 的目标是在 Rust 中实现类似的概念。

### 10.2 ggplot2

ggplot2 的 Layer 模型是 Grammar of Graphics 的经典实现：

```R
ggplot(data, aes(x=month, y=sales, fill=region)) +
  geom_bar(stat="identity", position="stack") +
  facet_wrap(~year) +
  theme_minimal()
```

liecharts Phase 4 的目标是引入类似的 Layer 和 Facet 概念。

### 10.3 差异化定位

| 特性 | Vega-Lite | ggplot2 | liecharts (目标) |
|------|:---------:|:-------:|:---------------:|
| 语言 | JavaScript | R | **Rust** |
| 渲染 | 浏览器 SVG/Canvas | PDF/PNG | **PNG/SVG (服务端)** |
| 交互 | 丰富 | 无 | 无（专注静态） |
| JSON 输入 | 原生 | 无 | **原生** |
| 类型安全 | 无 | 无 | **编译时检查** |
| 性能 | 中 | 中 | **高 (Rust 编译型)** |
| ECharts 兼容 | 无 | 无 | **继承** |

liecharts 的独特定位：**一个类型安全的、高性能的、兼容 ECharts 生态的 Rust 图表库**。

---

## 文档版本记录

| 版本 | 日期 | 变更说明 |
|------|------|---------|
| v1.0 | 2026-05-21 | 初版，涵盖 4 阶段重构方案 |