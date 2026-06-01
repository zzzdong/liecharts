# Chart Pipeline 架构设计

## 1. 概述

Chart Pipeline 是整个图表渲染的核心引擎，职责是将用户配置的 `ChartOption` 转换为一组 `VisualElement`，再由渲染器输出为 SVG / Pixmap 等格式。

### 1.1 设计理念

以 **DataFrame** 作为统一的数据载体，贯穿整个处理流程。每个图表类型（Line, Bar, Pie 等）都是一个 `DataProcessor`，遵循 ETL 管线模式：

```
SeriesOption ──▶ DataFrame ──▶ DataFrame（扩展列）──▶ Vec<VisualElement>
            to_dataframe()    transform()              to_visual_elements()
                                    ↑
                           CoordinateMapper
                             (坐标映射)
```

### 1.2 核心流程

```
┌──────────────┐
│  ChartOption │  (用户配置)
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│  1. GridPlanner  │  计算每个 subplot 的布局边界
├──────────────────┤
│  2. AxisBinding- │  解析轴范围（支持双 Y 轴）
│     Resolver     │
├──────────────────┤
│  3. ColorAssigner│  分配颜色 palette + series_colors
├──────────────────┤
│  4. GroupAnalyzer│  将 series 按分组（SideBySide / Stacked / Single）
├──────────────────┤
│  5. DataProcessor│  每个 series 经过:
│     a. to_dataframe()        → 原始 DataFrame
│     b. transform()           → 添加计算列（color, 分组等）
│     c. mapper.map_coordinates() → 像素坐标 px, py
│     d. to_visual_elements()  → Vec<VisualElement>
├──────────────────┤
│  6. 标题/图例/轴  │  全局元素
├──────────────────┤
│  7. 文本布局计算   │
└──────┬───────────┘
       │
       ▼
   Vec<VisualElement> → SVG / Pixmap
```

---

## 2. 模块结构

```
src/pipeline/
├── mod.rs                       # 模块声明 + re-export
├── pipeline.rs                  # 主流程 build_chart / build_chart_with_theme
├── types.rs                     # 共享类型 (SubplotSpec, ColorContext 等)
├── dataframe.rs                 # DataFrame / Series / DataValue / Transformers
├── data_processor.rs            # DataProcessor trait + create_processor()
│
├── grid_planner.rs              # GridPlanner: 布局规划
├── axis_binding_resolver.rs     # AxisBindingResolver: 轴范围解析（双 Y 轴）
├── axis_renderer.rs             # AxisRenderer: 轴刻度线/标签渲染
├── color_assigner.rs            # ColorAssigner: 颜色分配
├── visual_element_builder.rs    # VisualElementBuilder: 全局元素组装
│
├── mapper/                      # ★ CoordinateMapper: 坐标映射器
│   ├── mod.rs                   # CoordinateMapper trait
│   ├── cartesian.rs             # CartesianMapper: X/Y 轴坐标映射
│   ├── polar.rs                 # PolarMapper: 极坐标映射
│   └── noop.rs                  # NoopMapper: 空操作
│
├── accessors/                   # ★ DataFrame 安全访问层
│   ├── mod.rs
│   ├── cartesian_geometry.rs    # CartesianGeometry: px/py 访问器
│   ├── group_info.rs            # GroupInfo: group_position/group_total
│   └── style_access.rs          # StyleAccess: color 访问器
│
├── group/                       # ★ 分组分析（SideBySide / Stacked）
│   ├── mod.rs
│   ├── analyzer.rs              # GroupAnalyzer: 扫描 series 识别分组
│   └── dataframe_builder.rs     # GroupedBarProcessor: 合并 DataFrame
│
└── processor/                   # 各图表类型的 Processor
    ├── mod.rs
    ├── line.rs                  # LineProcessor
    ├── bar.rs                   # BarProcessor
    ├── scatter.rs               # ScatterProcessor
    ├── bubble.rs                # BubbleProcessor
    ├── candlestick.rs           # CandlestickProcessor
    ├── pie.rs                   # PieProcessor
    ├── polar_bar.rs             # PolarBarProcessor
    ├── polar_scatter.rs         # PolarScatterProcessor
    ├── radar.rs                 # RadarProcessor
    ├── gauge.rs                 # GaugeProcessor
    └── table.rs                 # TableProcessor
```

---

## 3. 核心抽象

### 3.1 DataFrame（[dataframe.rs](src/pipeline/dataframe.rs)）

列式数据表，由 `HashMap<String, Series>` 组成，支持动态添加计算列。

```rust
pub struct DataFrame {
    columns: HashMap<String, Series>,
    column_order: Vec<String>,
    row_count: usize,
}
```

关键操作：
- `add_column(series)` — 添加新列
- `get_column(name)` — 按名查找列
- `compute_column(name, |i, &df| -> DataValue)` — 基于现有列计算新列

### 3.2 DataProcessor trait（[data_processor.rs](src/pipeline/data_processor.rs)）

每个图表类型实现此 trait：

```rust
pub trait DataProcessor {
    fn to_dataframe(&self, series, input) -> Result<DataFrame>;
    fn transform(&self, df, input) -> Result<DataFrame>;
    fn to_visual_elements(&self, df, input) -> Result<Vec<VisualElement>>;

    fn resolve_x_axis_idx(&self, series, input) -> usize;
    fn resolve_y_axis_idx(&self, series, input) -> usize;
    fn mapper(&self) -> Box<dyn CoordinateMapper>;

    fn process(&self, series, input) -> Result<Vec<VisualElement>>;
    fn process_dataframe(&self, df, input) -> Result<Vec<VisualElement>>;
}
```

默认流程（`process()`）：

```
to_dataframe → transform → mapper.map_coordinates → to_visual_elements
```

### 3.3 CoordinateMapper（[mapper/mod.rs](src/pipeline/mapper/mod.rs)）

坐标映射器，将数据坐标转换为像素坐标：

```rust
pub trait CoordinateMapper {
    fn map_coordinates(&self, df: &mut DataFrame, input, x_axis_idx, y_axis_idx);
}
```

| Mapper | 适用场景 | 输出列 |
|--------|---------|--------|
| `CartesianMapper` | Line, Bar, Scatter, Bubble, Candlestick | `px`, `py` (可选 `pbase`) |
| `PolarMapper` | PolarBar, PolarScatter | `center_x`, `center_y`, `max_radius` |
| `NoopMapper` | Pie, Radar, Gauge, Table | 无（默认） |

### 3.4 Accessors（[accessors/](src/pipeline/accessors/)）

DataFrame 列的类型安全访问层：

| Accessor | 来源 | 方法 |
|----------|------|------|
| `CartesianGeometry` | `from_df(df)?` | `.px(i)`, `.py(i)`, `.pbase(i, fallback)`, `.collect_points()`, `.row_count()` |
| `GroupInfo` | `from_df(df)` | `.total()`, `.position(i)`, `.center_offset(i)` |
| `StyleAccess` | `from_df(df, fallback)` | `.color(i)` |

### 3.5 GroupAnalyzer（[group/analyzer.rs](src/pipeline/group/analyzer.rs)）

扫描 subplot 内的 series，识别分组：

```rust
pub enum GroupType { Single, SideBySide, Stacked }

pub struct GroupPlan {
    pub series_indices: Vec<usize>,
    pub group_type: GroupType,
}
```

- **Single**：独立的 series（Line, Scatter, etc.）
- **SideBySide**：同 `group_index` 的 Bar 并排
- **Stacked**：同 `stack` 名称的 Bar 堆叠

`GroupedBarProcessor::combine_to_dataframe()` 将同组 Bar 展开为多行，每行带 `group_position` / `group_total` / `stack_base`。

---

## 4. 图表家族分类

```
┌─────────────────────────────────────────────┐
│  Cartesian（笛卡尔坐标系家族）               │
│  ┌─────────────────────────────────────────┐│
│  │ Line    Bar    Scatter    Bubble         ││
│  │ Candlestick                             ││
│  ├─────────────────────────────────────────┤│
│  │ Mapper:  CartesianMapper                ││
│  │ Accessors: CartesianGeometry, GroupInfo  ││
│  │ 轴绑定:  双 Y 轴，via y_axis_index       ││
│  └─────────────────────────────────────────┘│
├─────────────────────────────────────────────┤
│  Polar（极坐标系家族）                       │
│  ┌─────────────────────────────────────────┐│
│  │ Pie    PolarBar    PolarScatter    Radar ││
│  ├─────────────────────────────────────────┤│
│  │ Mapper:  PolarMapper / NoopMapper       ││
│  │ 共同点:  角度 + 半径的几何计算            ││
│  └─────────────────────────────────────────┘│
├─────────────────────────────────────────────┤
│  Standalone（独立家族）                       │
│  ┌─────────────────────────────────────────┐│
│  │ Gauge    Table                          ││
│  ├─────────────────────────────────────────┤│
│  │ Mapper:   NoopMapper                    ││
│  │ 特点:     无坐标轴，自定义渲染逻辑         ││
│  └─────────────────────────────────────────┘│
└─────────────────────────────────────────────┘
```

### 4.1 Cartesian 家族统一模式

```
to_dataframe():
  提取 x, y (+ optional name, size, open/close/low/high)
  添加 cat_idx 列

transform():
  添加 color 列
  Bar: 添加 group_total/group_position（merge 时用）
  Candlestick: 直接在 transform 中计算像素坐标（open_y/close_y 等）
  水平 Bar: 将 y 值复制到 x 列供 mapper 使用

mapper(): CartesianMapper
  自动判断 value/category 轴类型
  计算 px（category → cat_idx 映射 / value → x 列映射）
  计算 py（value → flip / category → top-down）
  堆叠模式: 计算 pbase（stack_base → 像素坐标）

to_visual_elements():
  从 DataFrame 列读取像素坐标和样式
  纯 VisualElement 组装，不做坐标计算
```

### 4.2 双 Y 轴

每个 Cartesian Processor 覆写 `resolve_y_axis_idx()`：

```rust
fn resolve_y_axis_idx(&self, series, input) -> usize {
    match series {
        SeriesOption::Line(l) => l.y_axis_index
            .or_else(|| input.spec.y_axis_indices.first().copied())
            .unwrap_or(0),
        // ...
    }
}
```

CartesianMapper 根据传入的 `y_axis_idx` 从 `input.axis_ranges.get_y_range(y_axis_idx)` 获取对应的轴范围，独立计算 `py`。

---

## 5. 数据三层模型

DataFrame 列按语义分为三层：

| 层 | 来源 | 示例列 |
|----|------|--------|
| **数据层** | `to_dataframe()` | `x`, `y`, `category`, `value`, `open`, `close`, `size`, `name` |
| **样式/分组层** | `transform()` | `color`, `bar_width_ratio`, `group_position`, `group_total`, `stack_base` |
| **几何层** | `CoordinateMapper` | `px`, `py`, `pbase`, `center_x`, `center_y`, `max_radius` |

顺序保证：`transform()` 添加的列可以在 `CoordinateMapper` 中被引用（如 `stack_base` → `pbase`）。

---

## 6. 示例：Bar 分组 + 堆叠的数据流

以 3 个 Bar（直接销售 120, 代理销售 80, 线上销售 60）的 stacked 为例：

```
GroupAnalyzer.analyze()
  → GroupPlan { series_indices: [0,1,2], group_type: Stacked }

GroupedBarProcessor::combine_to_dataframe()
  → stack_cums = [0, 0, 0, 0]
  → 直接销售: y=120, base=0, pos=0, total=1
  → 代理销售: y=200, base=120, pos=0, total=1
  → 线上销售: y=260, base=200, pos=0, total=1
  → 12 行 DataFrame（3 series × 4 categories）

BarProcessor.transform()
  → bar_width_ratio = 0.6

CartesianMapper.map_coordinates()
  → eff_y_max = max(axis_range.y_max, df.y_max) = 260
  → py = bounds.y1 - y * bounds.height / 260
  → pbase = bounds.y1 - stack_base * bounds.height / 260

BarProcessor.to_visual_elements()
  → 直接销售 Q1: top=py(120), bottom=pbase(0) → 从底部开始
  → 代理销售 Q1: top=py(200), bottom=pbase(120) → 从 120 的位置开始
  → 线上销售 Q1: top=py(260), bottom=pbase(200) → 从 200 的位置开始
```

---

## 7. Processor 一览

| Processor | 家族 | Mapper | DataFrame 数据列 | 特殊处理 |
|-----------|------|--------|-----------------|---------|
| LineProcessor | Cartesian | CartesianMapper | x, y, color | Catmull-Rom 平滑, 面积填充 |
| BarProcessor | Cartesian | CartesianMapper | x, y, color, cat_idx, group_*, bar_width_ratio | 分组偏移, 堆叠 base, 横向/纵向 |
| ScatterProcessor | Cartesian | CartesianMapper | x, y, color, name | 标签偏移 |
| BubbleProcessor | Cartesian | CartesianMapper | x, y, color, size, name | 气泡半径 = sqrt(size) |
| CandlestickProcessor | Cartesian | CartesianMapper | open, close, low, high, is_up, cat_idx | 像素坐标在 transform 中计算 |
| PieProcessor | Polar/独立 | NoopMapper | category, value | PieDataTransformer: percent, color, start_angle, end_angle |
| PolarBarProcessor | Polar | PolarMapper(0.85) | value, max_value, data_count | 扇区路径, 极坐标网格 |
| PolarScatterProcessor | Polar | PolarMapper(0.8) | angle, radius, symbol_size | 极坐标网格, 角度→像素 |
| RadarProcessor | Polar/独立 | NoopMapper | name, value（逗号分隔） | 多边形路径, 指标标签 |
| GaugeProcessor | 独立 | NoopMapper | value, name | 半圆弧 + 指针, 渐变色带 |
| TableProcessor | 独立 | NoopMapper | row_idx, col_idx, text, is_header | 行列网格, 斑马纹, 边框 |

---

## 8. 设计决策记录

### 8.1 为什么用 DataFrame 而不是直接操作 SeriesOption？

- 解耦数据与渲染：同一份 DataFrame 可以增删列而不影响原始数据
- 管道组合：`transform()` 添加的列可以被 `CoordinateMapper` 和 `to_visual_elements()` 消费
- 分组支持：将多个 series 展开为 DataFrame 行，合并处理

### 8.2 为什么引入 CoordinateMapper？

- 消除 Cartesian 家族 5 个 processor 中重复的坐标映射代码
- 统一处理 category/value 轴类型判断
- 堆叠模式下的 `pbase` 计算与 `CartesianMapper` 的一体化

### 8.3 为什么 Accessors 不用结构体而用 DataFrame？

- 每个图表类型需要的列不同（Line 不需要 `size`，Bubble 需要）
- DataFrame 的列式存储天然支持可选列
- `CartesianGeometry::from_df(df)?` 在构造时检查列是否存在，兼具灵活性与安全性

### 8.4 为什么 CandlestickProcessor 在 transform 中计算像素坐标？

K 线图需要 4 个 Y 坐标（open_y, close_y, low_y, high_y），CartesianMapper 只提供 `py`（单个 Y 坐标）。这里是一个 trade-off：既享受了 mapper 为 `px`（X 坐标）提供的统一计算，又保留了 K 线特有的多 Y 坐标处理。

### 8.5 Pie 为什么用 NoopMapper 而不是 PolarMapper？

Pie 的 center 和 radius 是用户可配的百分比（`pie.center`, `pie.radius`），不是通用的极坐标映射逻辑。扇区角度来自 `PieDataTransformer` 的累计百分比计算，不需要 mapper。

---

## 9. 配置 → 视觉元素映射

```
ChartOption {
    title: "月度销售数据"
    x_axis: [category: ["1月","2月",...]]
    y_axis: [value: "销售额(万元)"]
    series: [
        Bar { name: "销售额", data: [120, 200, 150, ...] }
    ]
}
    │
    ▼
GridPlanner       → SubplotSpec { bounds: Rect(80, 90, 720, 510), ... }
AxisBindingResolver → ResolvedAxisRanges { y_min=0, y_max=220 }
ColorAssigner     → ColorContext { series_colors: [blue], ... }
GroupAnalyzer     → GroupPlan { type: Single, indices: [0] }
    │
    ▼
BarProcessor.process()
    to_dataframe()
        → DataFrame { x: [0,1,2,3,4,5], y: [120,200,150,80,70,110] }
    transform()
        → +color: [blue×6], +group_total: 1, +group_position: 0, +bar_width_ratio: 0.6
    CartesianMapper.map_coordinates()
        → +px: [160,320,480,640,..], +py: [196,42,147,281,...]
    to_visual_elements()
        → [Rect(x=128, y=196, w=64, h=314), Text("120", x=160, y=210), ...]
    │
    ▼
+ 背景 Rect
+ AxisRenderer → 刻度线 + 标签
+ TitleRenderer → 标题元素 "月度销售数据"
+ LegendRenderer → 图例 "■ 销售额"
+ TextLayout → 最终坐标
    │
    ▼
Vec<VisualElement> → SVG 文件
```
