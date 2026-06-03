# Spec 体系设计文档

## 1. 设计原则

Spec 体系是 `pipeline` 模块的核心数据模型，所有 Spec 类型定义于 `src/pipeline/types.rs`。

| 原则 | 说明 |
|------|------|
| **值确定性** | 所有字段的值都是确定的。不存在"此图表类型不需要该字段所以是 `None`"的情况 |
| **默认值** | 每个 Config 实现 `Default`，创建即就绪，零配置即可使用 |
| **可修改性** | 创建后通过直接字段赋值修改，无需额外 setter 方法 |
| **类型安全** | 通过 `enum` 承载类型特化配置，编译期保证字段存在 |
| **单一数据源** | `ChartSpec` 是 pipeline 的唯一输入，消除了新旧双源混杂 |

---

## 2. 库整体架构

```
liecharts
├── src/
│   ├── api/                    # 公开 API 层（用户入口）
│   │   ├── chart.rs            # Chart 构建器 + to_chart_spec()
│   │   └── layer.rs            # LayerSpec + 各图层类型定义
│   │
│   ├── pipeline/               # 渲染管线
│   │   ├── types.rs            # ★ 核心 Spec 类型定义
│   │   ├── pipeline.rs         # 管线编排（build_chart_internal）
│   │   ├── compat.rs           # 旧 API 兼容转换（Spec → Option）
│   │   ├── data_processor.rs   # DataProcessor trait
│   │   ├── grid_planner.rs     # 网格布局计算
│   │   ├── axis_binding_resolver.rs  # 轴范围解析
│   │   ├── color_assigner.rs   # 颜色分配
│   │   ├── axis_renderer.rs    # 轴渲染
│   │   ├── dataframe.rs        # DataFrame 数据结构
│   │   ├── processor/          # 各图表类型处理器
│   │   ├── mapper/             # 坐标映射器
│   │   └── group/              # 分组逻辑（堆叠/并排）
│   │
│   ├── option.rs               # 旧架构（仅保留作兼容）
│   ├── sampling.rs             # 数据采样
│   ├── theme.rs                # 主题
│   └── visual.rs               # 渲染图元（VisualElement）
│
└── docs/
    └── spec-model.md           # 本文档
```

---

## 3. 完整数据流

```
┌─────────────────────────────────────────────────────────────────────┐
│                    用户 API 层                                       │
│                                                                     │
│   Chart::new(800, 600)                                              │
│     .add_bar(Bar::new().data(df))                                   │
│     .add_line(Line::new().data(df))                                 │
│     .render_svg()                                                   │
│       │                                                             │
│       │ to_chart_spec()  ← 通过 LayerSpec trait 方法消除 match 冗余  │
│       ▼                                                             │
├─────────────────────────────────────────────────────────────────────┤
│                    Pipeline 层                                       │
│                                                                     │
│   ChartSpec {                                                       │
│       width, height,                                                │
│       grids:    Vec<GridSpec>,                                      │
│       x_axes:   Vec<AxisSpec>,                                      │
│       y_axes:   Vec<AxisSpec>,                                      │
│       series:   Vec<SeriesSpec>,                                    │
│       title:    Option<TitleSpec>,                                  │
│       legend:   Option<LegendSpec>,                                 │
│       ...                                                            │
│   }                                                                 │
│       │                                                             │
│       │ build_chart_internal(spec, theme)  ← 只接受 ChartSpec       │
│       ▼                                                             │
│   1. GridPlanner.plan() → Vec<SubplotSpec>                          │
│   2. AxisBindingResolver.resolve() → ResolvedAxisRanges             │
│   3. ColorAssigner.assign() → ColorContext                          │
│   4. 背景绘制                                                        │
│   5. AxisRenderer 渲染轴                                             │
│   6. GroupAnalyzer 分组 → processor.process(input)                  │
│         input.chart_spec.series[idx].config → 直接取配置             │
│   7. 标题/图例/轴名称                                                │
│       │                                                             │
│       ▼                                                             │
├─────────────────────────────────────────────────────────────────────┤
│                    Render 层                                         │
│                                                                     │
│   Vec<VisualElement> → SvgRenderer / PixmapRenderer                 │
│       │                                                             │
│       ▼                                                             │
│   SVG String / PNG Bytes                                             │
└─────────────────────────────────────────────────────────────────────┘
```

**关键设计决策：ChartSpec 为唯一数据源。** `build_chart_internal` 只接受 `(&ChartSpec, &Theme)`，不再同时传入 `ChartOption`。所有组件从 `ChartSpec` 取值。

---

## 4. ChartSpec — 图表顶层规格

```rust
pub struct ChartSpec {
    pub width: u32,                       // 画布宽度
    pub height: u32,                      // 画布高度
    pub grids: Vec<GridSpec>,             // 网格布局
    pub x_axes: Vec<AxisSpec>,            // X 轴列表
    pub y_axes: Vec<AxisSpec>,            // Y 轴列表
    pub series: Vec<SeriesSpec>,          // 系列数据
    pub title: Option<TitleSpec>,         // 标题
    pub legend: Option<LegendSpec>,       // 图例
    pub background: Color,                // 背景色
    pub palette: Vec<Color>,              // 调色板（来自主题）
    pub theme_name: Option<String>,       // 主题名
}
```

### 创建入口

| 入口 | 来源 | 说明 |
|------|------|------|
| `api::Chart::to_chart_spec()` | 新 API | 构建器模式，推荐 |
| `compat::chart_option_to_chart_spec()` | 旧 API | 从 `option::ChartOption` 转换 |
| 直接构造 | 手动 | 程序化构建 |

---

## 5. GridSpec — 网格布局

```rust
pub struct GridSpec {
    pub left: Option<f64>,     // 左边距 (px)，None = auto
    pub right: Option<f64>,    // 右边距 (px)
    pub top: Option<f64>,      // 上边距 (px)
    pub bottom: Option<f64>,   // 下边距 (px)
    pub contain_label: bool,   // 边距是否包含轴标签
}
```

多网格场景下，每个 `GridSpec` 对应一个独立的绘图区域。`series` 通过 `grid_index` 绑定到网格。

---

## 6. AxisSpec — 坐标轴规格

```rust
pub struct AxisSpec {
    pub axis_type: AxisType,        // Category / Value / Time / Log
    pub position: AxisPosition,     // Top / Bottom / Left / Right
    pub grid_index: usize,          // 绑定的网格索引
    pub min: Option<f64>,           // 用户指定最小值
    pub max: Option<f64>,           // 用户指定最大值
    pub name: Option<String>,       // 轴名称
    pub categories: Vec<String>,    // Category 轴标签
    pub boundary_gap: bool,         // 两端留白
}
```

### 辅助枚举

```rust
pub enum AxisType { Category, Value, Time, Log }
pub enum AxisPosition { Top, Bottom, Left, Right }
```

---

## 7. SeriesSpec + SeriesConfig — 系列规格

### 7.1 设计动机

旧设计中，`SeriesSpec` 把所有图表类型的配置字段平铺在一个结构体里，用 `Option<T>` 表示"此图表类型不需要"：

```rust
// 旧设计（反模式）
pub struct SeriesSpec {
    pub bar_width: Option<f64>,         // 只有 Bar 需要
    pub line_width: Option<f64>,        // 只有 Line 需要
    pub pie_center: Option<Vec<String>>, // 只有 Pie 需要
    pub pad_angle: Option<f64>,         // 只有 PolarBar 需要
    // ... 10+ 个互不相关的 Option 字段
}
```

新设计通过 **enum 承载类型特化配置**，每个图表类型拥有独立的 Config 结构体：

```rust
pub struct SeriesSpec {
    // ── 所有图表类型共用的基础字段 ──
    pub name: String,
    pub data: DataFrame,
    pub grid_index: usize,
    pub x_axis_index: usize,
    pub y_axis_index: usize,
    pub stack: Option<String>,
    pub group_index: usize,
    pub sampling: Option<(SamplingType, usize)>,
    pub item_style: ItemStyleSpec,

    // ── 类型特化配置（从 config 推导 chart_type）──
    pub config: SeriesConfig,
}

pub enum SeriesConfig {
    Line(LineConfig),
    Bar(BarConfig),
    Pie(PieConfig),
    Scatter(ScatterConfig),
    Bubble(BubbleConfig),
    Candlestick(CandlestickConfig),
    Radar(RadarConfig),
    PolarBar(PolarBarConfig),
    PolarScatter(PolarScatterConfig),
    Gauge(GaugeConfig),
    Table(TableConfig),
}
```

**注意：`SeriesSpec` 不再包含 `chart_type` 字段。** 图表类型由 `config.chart_type()` 推导，避免冗余。

### 7.2 共用字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 系列名称，用于图例和提示 |
| `data` | `DataFrame` | 列式数据，列名由各 Config 指定 |
| `grid_index` | `usize` | 绑定的网格索引 |
| `x_axis_index` | `usize` | 使用的 X 轴索引 |
| `y_axis_index` | `usize` | 使用的 Y 轴索引 |
| `stack` | `Option<String>` | 堆叠组名（用于堆叠柱状图/折线图） |
| `group_index` | `usize` | 并排分组索引（用于并排柱状图） |
| `sampling` | `Option<...>` | 数据采样策略（LTTB/Average/Max/Min） |
| `item_style` | `ItemStyleSpec` | 图形样式（颜色、边框、透明度） |
| `config` | `SeriesConfig` | 类型特化配置（枚举分发） |

### 7.3 各图表类型 Config

#### LineConfig — 折线图

```rust
pub struct LineConfig {
    pub x_col: String,             // 默认 "x"
    pub y_col: String,             // 默认 "y"
    pub smooth: bool,              // 默认 false，是否平滑曲线
    pub line_width: f64,           // 默认 2.0，线条宽度
    pub area_color: Option<Color>, // 面积填充色
    pub area_opacity: f64,         // 默认 0.5，面积透明度
    pub symbol_type: SymbolType,   // 默认 Circle，标记点类型
    pub symbol_size: f64,          // 默认 4.0，标记点大小
}
```

#### BarConfig — 柱状图

```rust
pub struct BarConfig {
    pub x_col: String,             // 默认 "x"
    pub y_col: String,             // 默认 "y"
    pub bar_width: f64,            // 默认 0.6，宽度比例 (0.0~1.0)
}
```

#### PieConfig — 饼图

```rust
pub struct PieConfig {
    pub category_col: String,      // 默认 "category"
    pub value_col: String,         // 默认 "value"
    pub center: (f64, f64),        // 默认 (50.0, 50.0)，圆心百分比
    pub radius: (f64, f64),        // 默认 (0.0, 75.0)，(内半径, 外半径) 百分比
    pub label_show: bool,          // 默认 false
    pub label_position: LabelPosition, // 默认 Outside
    pub label_font_size: f64,      // 默认 12.0
}
```

#### ScatterConfig — 散点图

```rust
pub struct ScatterConfig {
    pub x_col: String,             // 默认 "x"
    pub y_col: String,             // 默认 "y"
    pub symbol_size: f64,          // 默认 10.0
}
```

#### BubbleConfig — 气泡图

```rust
pub struct BubbleConfig {
    pub x_col: String,             // 默认 "x"
    pub y_col: String,             // 默认 "y"
    pub size_col: Option<String>,  // 气泡大小列
    pub name_col: Option<String>,  // 名称列
    pub symbol_size_scale: f64,    // 默认 1.0
}
```

#### CandlestickConfig — K线图

```rust
pub struct CandlestickConfig {
    pub category_col: String,      // 默认 "category"
    pub open_col: String,          // 默认 "open"
    pub close_col: String,         // 默认 "close"
    pub low_col: String,           // 默认 "low"
    pub high_col: String,          // 默认 "high"
}
```

#### RadarConfig — 雷达图

```rust
pub struct RadarConfig {
    pub value_col: String,         // 默认 "value"
    pub indicators: Vec<String>,   // 默认空，指示器名称列表
}
```

#### PolarBarConfig — 极坐标柱状图

```rust
pub struct PolarBarConfig {
    pub angle_col: String,         // 默认 "angle"
    pub radius_col: String,        // 默认 "radius"
    pub pad_angle: f64,            // 默认 2.0，间隔角度(度)
    pub start_angle: f64,          // 默认 0.0，起始角度(度)
}
```

#### PolarScatterConfig — 极坐标散点图

```rust
pub struct PolarScatterConfig {
    pub angle_col: String,         // 默认 "angle"
    pub radius_col: String,        // 默认 "radius"
    pub symbol_size: f64,          // 默认 8.0
}
```

#### GaugeConfig — 仪表盘

```rust
pub struct GaugeConfig {
    pub value_col: String,         // 默认 "value"
    pub min: f64,                  // 默认 0.0
    pub max: f64,                  // 默认 100.0
    pub center: (f64, f64),        // 默认 (50.0, 75.0)
    pub radius: f64,               // 默认 75.0
    pub start_angle: f64,          // 默认 -225.0
    pub end_angle: f64,            // 默认 45.0
    pub split_number: usize,       // 默认 10，刻度分段数
}
```

#### TableConfig — 表格

```rust
pub struct TableConfig;  // 无额外配置，使用 DataFrame 的所有列
```

### 7.4 SeriesConfig 辅助方法

```rust
impl SeriesConfig {
    /// 获取图表类型（用于处理器分发）
    pub fn chart_type(&self) -> ChartType;

    /// 获取 X 轴列名（用于轴范围计算、compat 转换）
    pub fn x_col_name(&self) -> &str;

    /// 获取 Y 轴列名（用于轴范围计算、compat 转换）
    pub fn y_col_name(&self) -> &str;
}
```

### 7.5 辅助类型

```rust
pub enum ChartType {
    Line, Bar, Pie, Scatter, Bubble,
    Candlestick, Radar, PolarBar, PolarScatter, Gauge, Table,
}

pub enum SymbolType {
    Circle, Rect, RoundRect, Triangle, Diamond, Pin, Arrow, None,
}

pub enum LabelPosition { Outside, Inside }

pub struct ItemStyleSpec {
    pub color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<f64>,
    pub opacity: Option<f64>,
}
```

### 7.6 处理器取配置示例（对比旧设计）

```rust
// 旧设计（运行时 unwrap + 字段混杂）
fn process_from_spec(&self, series: &SeriesSpec, _input: &DataProcessorInput) -> Result<Vec<VisualElement>> {
    let line_width = series.line_width.unwrap_or(2.0);   // 运行时才知道值
    let smooth = series.smooth;                          // 对 Bar 类型无意义但存在
    let bar_width = series.bar_width.unwrap_or(0.6);     // 对 Line 类型无意义但存在
    // ...
}

// 新设计（编译期保证 + 字段隔离）
fn process(&self, input: &DataProcessorInput) -> Result<Vec<VisualElement>> {
    let series = &input.chart_spec.series[input.series_idx];
    let SeriesConfig::Line(cfg) = &series.config else {
        return Err(...);
    };
    let line_width = cfg.line_width;   // 确定值，不 unwrap
    let smooth = cfg.smooth;           // 确定值，不 unwrap
    // BarConfig 的字段完全不可见，编译期隔离
}
```

---

## 8. 装饰元素规格

### TitleSpec

```rust
pub struct TitleSpec {
    pub text: Option<String>,
    pub subtext: Option<String>,
}
```

### LegendSpec

```rust
pub struct LegendSpec {
    pub show: bool,
    pub data: Vec<String>,
    pub symbol_size: f64,
}
```

---

## 9. Pipeline 渲染管线

`pipeline::pipeline::build_chart_internal(&ChartSpec, &Theme)` 是核心管线函数。

```
ChartSpec  ──→  build_chart_internal(spec, theme)
                    │
                    ├─ 1. GridPlanner         → Vec<SubplotSpec>
                    │   计算每个 subplot 的像素边界
                    │
                    ├─ 2. AxisBindingResolver → ResolvedAxisRanges
                    │   收集数据范围 + 用户 min/max → 最终轴范围
                    │
                    ├─ 3. ColorAssigner       → ColorContext
                    │   从主题分配调色板颜色
                    │
                    ├─ 4. 背景绘制
                    │
                    ├─ 5. AxisRenderer        → 轴元素
                    │   渲染轴刻度、标签、网格线
                    │
                    ├─ 6. DataProcessor       → Vec<VisualElement>
                    │   │ GroupAnalyzer 分组
                    │   │   ├─ Single    → processor.process(input)
                    │   │   ├─ SideBySide → processor.process_dataframe(df, input)
                    │   │   └─ Stacked   → processor.process_dataframe(df, input)
                    │   │
                    │   │ 每个处理器内部 (process):
                    │   │   a. 从 input.chart_spec.series[idx] 取 SeriesSpec
                    │   │   b. 匹配 series.config 获取类型特化配置
                    │   │   c. transform() — 添加计算列 (color, position...)
                    │   │   d. CoordinateMapper::map_coordinates()
                    │   │   e. to_visual_elements() — 生成图元
                    │
                    ├─ 7. 标题 / 图例 / 轴名称 渲染
                    │
                    └─→ Vec<VisualElement>  (渲染输出)
```

---

## 10. Pipeline 中间类型

### SubplotSpec — 网格规划输出

```rust
pub struct SubplotSpec {
    pub id: usize,                    // subplot 编号
    pub bounds: Rect,                 // 像素边界
    pub series_indices: Vec<usize>,   // 关联的 series 索引
    pub x_axis_indices: Vec<usize>,   // 关联的 X 轴索引
    pub y_axis_indices: Vec<usize>,   // 关联的 Y 轴索引
}
```

### ResolvedAxisRange — 单轴解析结果

```rust
pub struct ResolvedAxisRange {
    pub axis_index: usize,
    pub position: AxisPosition,
    pub min: f64,                     // 最终最小值
    pub max: f64,                     // 最终最大值
    pub is_user_defined: bool,        // 是否用户指定
    pub tick_count_hint: Option<usize>,
}
```

### ResolvedAxisRanges — 所有轴解析结果

```rust
pub struct ResolvedAxisRanges {
    pub ranges: Vec<ResolvedAxisRange>,
}
// 提供 get_x_range(idx) / get_y_range(idx) 查询方法
```

### ColorContext — 颜色上下文

```rust
pub struct ColorContext {
    pub palette: Vec<Color>,           // 主题调色板
    pub background: Color,             // 背景色
    pub series_colors: Vec<Color>,     // 每个 series 的颜色
    pub axis_line_color: Color,        // 轴线颜色
    pub axis_label_color: Color,       // 轴标签颜色
    pub grid_line_color: Color,        // 网格线颜色
    pub border_color: Color,           // 边框颜色
    pub text_color: Color,             // 主文字颜色
    pub text_secondary_color: Color,   // 次要文字颜色
    pub up_color: Color,               // 涨/正值颜色 (K线)
    pub down_color: Color,             // 跌/负值颜色 (K线)
    pub table_header_bg: Color,        // 表头背景
    pub table_row_even_bg: Color,      // 偶数行背景
    pub table_row_odd_bg: Color,       // 奇数行背景
}
```

---

## 11. DataProcessor trait — 处理器接口（优化后）

**核心设计变更：** 删除对旧 `ChartOption`/`SeriesOption` 的依赖，`DataProcessorInput` 只持有 `ChartSpec` 一个数据源。

```rust
/// 处理器入参 — 从 ChartSpec 统一取值
pub struct DataProcessorInput<'a> {
    pub chart_spec: &'a ChartSpec,          // ★ 唯一数据源（不是 Option）
    pub subplot: &'a SubplotSpec,           // 当前 subplot
    pub colors: &'a ColorContext,           // 颜色上下文
    pub axis_ranges: &'a ResolvedAxisRanges,// 轴范围
    pub bounds: Rect,                       // subplot 像素边界
    pub series_idx: usize,                  // 当前 series 在 chart_spec.series 中的索引
}

/// 处理器 trait（优化后）
pub trait DataProcessor {
    /// ★ 主流程：从 ChartSpec 直接处理单 series
    fn process(&self, input: &DataProcessorInput) -> Result<Vec<VisualElement>>;

    /// 分组模式：处理已合并的 DataFrame（SideBySide / Stacked）
    fn process_dataframe(&self, df: DataFrame, input: &DataProcessorInput) -> Result<Vec<VisualElement>>;

    /// 返回坐标映射器
    fn mapper(&self) -> Box<dyn CoordinateMapper>;
}
```

**对比旧 trait 的关键变化：**

| 旧方法 | 新状态 | 原因 |
|--------|--------|------|
| `process(series: &SeriesOption, ...)` | **删除** | 单向迁移到 `process(input)` |
| `to_dataframe(series: &SeriesOption, ...)` | **删除** | `SeriesSpec.data` 已是 DataFrame |
| `transform(df, input)` | **保留** | 内部使用，不再从 `option` 取值 |
| `to_visual_elements(df, input)` | **保留** | 内部使用，不再从 `option` 取值 |
| `process_from_spec(series, input)` | **删除** | 合并入 `process(input)` |
| `resolve_x_axis_idx` / `resolve_y_axis_idx` | **删除** | 直接从 `series.x_axis_index` 取值 |

### 处理器工厂

```rust
/// 从 SeriesConfig 创建处理器
pub fn create_processor(config: &SeriesConfig) -> Box<dyn DataProcessor>;
```

### 处理器列表

| 处理器 | 文件 | 坐标系 | 映射器 |
|--------|------|--------|--------|
| `LineProcessor` | `processor/line.rs` | 笛卡尔 | `CartesianMapper` |
| `BarProcessor` | `processor/bar.rs` | 笛卡尔 | `CartesianMapper` |
| `ScatterProcessor` | `processor/scatter.rs` | 笛卡尔 | `CartesianMapper` |
| `BubbleProcessor` | `processor/bubble.rs` | 笛卡尔 | `CartesianMapper` |
| `CandlestickProcessor` | `processor/candlestick.rs` | 笛卡尔 | `CartesianMapper` |
| `PieProcessor` | `processor/pie.rs` | 无 | `NoopMapper` |
| `RadarProcessor` | `processor/radar.rs` | 极坐标 | `PolarMapper` |
| `PolarBarProcessor` | `processor/polar_bar.rs` | 极坐标 | `PolarMapper` |
| `PolarScatterProcessor` | `processor/polar_scatter.rs` | 极坐标 | `PolarMapper` |
| `GaugeProcessor` | `processor/gauge.rs` | 无 | `NoopMapper` |
| `TableProcessor` | `processor/table.rs` | 无 | `NoopMapper` |

### 处理器实现示例（LineProcessor）

```rust
impl DataProcessor for LineProcessor {
    fn process(&self, input: &DataProcessorInput) -> Result<Vec<VisualElement>> {
        let series = &input.chart_spec.series[input.series_idx];

        // 编译期仅匹配 Line variant，其他类型不可见
        let SeriesConfig::Line(cfg) = &series.config else {
            return Err(ChartError::DataError("Expected Line series".into()));
        };

        // 从 DataFrame 提取数据
        let mut df = series.data.clone();

        // 应用采样
        if let Some((sampling_type, threshold)) = &series.sampling {
            df = SamplingProcessor::sample(&df, *threshold, *sampling_type);
        }

        // transform：添加计算列
        let series_color = input.colors.get_series_color(input.series_idx);
        df.add_column(Series::new_constant("color", DataValue::Color(series_color), df.row_count()));
        df.add_column(Series::new_constant("line_width", DataValue::Float(cfg.line_width), df.row_count()));
        df.add_column(Series::new_constant("smooth", DataValue::Bool(cfg.smooth), df.row_count()));

        // 坐标映射
        self.mapper().map_coordinates(&mut df, input, series.x_axis_index, series.y_axis_index);

        // 生成图元
        self.to_visual_elements(&df, &series.config, input)
    }

    fn mapper(&self) -> Box<dyn CoordinateMapper> {
        Box::new(CartesianMapper)
    }
}
```

---

## 12. CoordinateMapper — 坐标映射

将数据值映射到像素坐标。

```rust
pub trait CoordinateMapper {
    fn map_coordinates(
        &self,
        df: &mut DataFrame,
        input: &DataProcessorInput,
        x_axis_idx: usize,
        y_axis_idx: usize,
    );
}
```

| 映射器 | 适用场景 |
|--------|----------|
| `CartesianMapper` | 笛卡尔坐标系（Line, Bar, Scatter, Bubble, Candlestick） |
| `PolarMapper` | 极坐标系（Radar, PolarBar, PolarScatter） |
| `NoopMapper` | 无坐标系（Pie, Gauge, Table） |

---

## 13. Group System — 分组系统

位于 `pipeline/group/`，处理多个 series 的组合渲染。

### GroupAnalyzer

分析 subplot 内的 series 列表，生成分组方案：

```rust
pub enum GroupType {
    Single,       // 单独渲染
    SideBySide,   // 并排（多个 Bar series 并列）
    Stacked,      // 堆叠（多个 Bar/Line series 堆叠）
}

pub struct GroupPlan {
    pub group_type: GroupType,
    pub series_indices: Vec<usize>,
}
```

### GroupedBarProcessor

将多个 series 合并为一个 DataFrame，每行包含所有 series 的数据，用于并排/堆叠渲染。

---

## 14. API 层 — 用户入口

### Chart 构建器

```rust
let svg = Chart::new(800, 600)
    .title("销售趋势")
    .data(df)
    .x_axis(Axis::category().data(["A", "B", "C"]))
    .y_axis(Axis::value())
    .add_bar(Bar::new().name("销售额").x("cat").y("val"))
    .add_line(Line::new().name("趋势线").x("cat").y("trend").smooth(true).line_width(1.5))
    .render_svg()?;
```

### LayerSpec — 图层枚举 + trait 方法

```rust
pub enum LayerSpec {
    Line(Line), Bar(Bar), Pie(Pie), Scatter(Scatter), Bubble(Bubble),
    Candlestick(Candlestick), Radar(Radar), PolarBar(PolarBar),
    PolarScatter(PolarScatter), Gauge(Gauge), Table(Table),
}

impl LayerSpec {
    // 统一访问层（消除 to_chart_spec 中的冗余 match）
    pub(crate) fn name(&self) -> &str;
    pub(crate) fn data(&self) -> Option<&DataFrame>;
    pub(crate) fn grid_index(&self) -> usize;
    pub(crate) fn y_axis_index(&self) -> usize;
    pub(crate) fn stack(&self) -> Option<&str>;
    pub(crate) fn group_index(&self) -> usize;
    // 生成 pipeline 层的 SeriesConfig
    pub(crate) fn to_series_config(&self) -> (ChartType, SeriesConfig);
}
```

这样做的好处：
- `to_chart_spec()` 从 200+ 行的巨型 match 缩减为约 30 行
- 新图表类型只需在 `LayerSpec` trait 方法中添加一行映射

### API → Pipeline 转换

```
api::Line   → SeriesConfig::Line(LineConfig { x_col, y_col, smooth, line_width, ... })
api::Bar    → SeriesConfig::Bar(BarConfig { x_col, y_col, bar_width })
api::Pie    → SeriesConfig::Pie(PieConfig { category_col, value_col, center, radius, ... })
...
```

### API 层各类型字段完整性

| Builder | 缺失字段 | 修复 |
|---------|----------|------|
| `Line` | `line_width` | 添加 `pub line_width: f64`，默认 `2.0` |
| `Line` | 面积透明度 | `area` 改为 `pub area_opacity: Option<f64>` |
| `Bar` | 无 | 字段完整 |

---

## 15. Compat 层 — 向后兼容

`pipeline/compat.rs` 提供旧架构入口的转换：

| 函数 | 方向 | 用途 |
|------|------|------|
| `chart_spec_to_chart_option()` | Spec → Option | 旧通路渲染时需要 |
| `chart_option_to_chart_spec()` | Option → Spec | 旧 API 入口统一转换为新管线 |

**注意：** `compat.rs` 必须通过 `SeriesConfig` 的方法（`x_col_name`、`y_col_name`）获取列名，不能直接访问 `s.x_col`/`s.y_col`（这些字段已从 `SeriesSpec` 移除）。

```rust
// 正确的转换方式
fn series_to_series_option(s: &SeriesSpec) -> SeriesOption {
    let x_col = s.config.x_col_name();
    let y_col = s.config.y_col_name();
    let data = dataframe_to_datapoints(&s.data, x_col, y_col);

    match &s.config {
        SeriesConfig::Line(cfg) => SeriesOption::Line(LineSeriesOption {
            smooth: Some(cfg.smooth),
            // ...
        }),
        // ...
    }
}
```

---

## 16. 与旧设计对比

| 方面 | 旧设计 | 新设计 |
|------|--------|--------|
| **类型特化字段** | 全部平铺在 `SeriesSpec`，用 `Option<T>` | 每种类型独立 Config 结构体 |
| **默认值** | `Default` 全部设为 `None` | 每个 Config 有合理默认值 |
| **编译期安全** | 运行时 `unwrap_or()` | 编译期保证字段存在 |
| **代码可读性** | `series.bar_width.unwrap_or(0.6)` | `cfg.bar_width` |
| **扩展性** | 新增图表类型需修改 `SeriesSpec` 结构体 + 所有构造处 | 新增 Config 结构体 + 枚举 variant |
| **列映射** | `x_col`/`y_col` 硬编码在基础字段 | 各 Config 独立定义列名语义 |
| **处理器解耦** | 处理器依赖 `series.xxx` 取值 | 处理器仅匹配 `series.config` 的对应 variant |
| **数据源** | `ChartSpec` + `ChartOption` 双源混杂 | `ChartSpec` 为唯一数据源 |
| **trait 复杂度** | 7 个方法（to_dataframe, transform, to_visual_elements, process, process_from_spec, process_dataframe, mapper） | 3 个方法（process, process_dataframe, mapper） |
| **api/chart.rs** | `to_chart_spec` 每个字段 11-way match，200+ 行 | 通过 LayerSpec trait 方法消除冗余，约 30 行 |

---

## 17. 重构路线图

| 优先级 | 任务 | 影响范围 |
|--------|------|----------|
| **P0** | 修复 `compat.rs` 引用 `s.x_col`/`s.y_col`/`s.smooth` | `compat.rs` |
| **P0** | 重构 `DataProcessorInput` — 移除 `option`，`chart_spec`/`series_spec` 从 Option 变必填 | `data_processor.rs`, 所有处理器, `pipeline.rs` |
| **P0** | 迁移 `LineProcessor`/`BarProcessor` 到新 `process(input)` | `processor/line.rs`, `processor/bar.rs` |
| **P1** | 为 `LayerSpec` 实现统一 trait 方法，消除 `api/chart.rs` 冗余 match | `api/layer.rs`, `api/chart.rs` |
| **P1** | 移除 `SeriesSpec.chart_type` 冗余字段 | `types.rs`, 所有引用处 |
| **P1** | 迁移剩余 9 个处理器 | `processor/*.rs` |
| **P2** | 补全 `api/layer.rs` 缺失字段（`Line.line_width` 等） | `api/layer.rs`, `api/chart.rs` |
| **P2** | `build_chart_internal` 移除 `option` 参数 | `pipeline.rs` |
| **P2** | 删除 `DataProcessor` trait 旧方法 | `data_processor.rs`, 所有处理器 |
| **P3** | 完全删除 `option.rs` | 全库 |