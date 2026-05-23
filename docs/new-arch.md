# 新架构设计：先 Grid 规划，再数据处理和视觉生成

---

## 术语定义（必读）

以下术语在本方案中有严格定义，避免歧义：

| 术语 | 定义 | 所属层级 |
|------|------|---------|
| **Grid** | 与 ECharts 的 grid 概念一致，指单个子图在画布上的矩形区域配置（如 `{left: 50, top: 50, width: 300, height: 250}`） | GridPlanner |
| **Subplot** | Grid 的运行时实例，携带关联的系列索引、轴索引等绑定关系 | GridPlanner |
| **Chart Bounds** | GridPlanner 输出的具体像素矩形（eg. `(x: 50, y: 50, w: 300, h: 250)`），**不含任何轴刻度/标签信息** | GridPlanner 输出 |
| **Plot Area** | DataProcessor 内部从 chart bounds 中扣除轴标签、标题等占位后得到的**实际绘图区域** | DataProcessor 内部 |
| **Axis Tick** | 坐标轴上的刻度线和标签文本，**完全由数据驱动**，在 DataProcessor 内部计算 | DataProcessor 内部 |
| **VisualElement** | 与渲染后端无关的图元枚举（Rect, Circle, Path, TextRun），渲染器直接消费 | VisualElementBuilder 输出 |

**关键原则**：

> GridPlanner 只回答"每个 chart 放哪里、多大"——纯数学分配，不碰任何数据。
> DataProcessor 只回答"在这个尺寸内，数据画成什么样子"——数据变换→视觉生成。

---

## 一、整体数据流

```
┌─────────────────┐
│  ChartOption    │  用户提供的 ECharts 风格配置
└────────┬────────┘
         ▼
┌───────────────────────────────────────────────────┐
│  ChartOption / ExternalData                       │  用户配置 + 可选外置数据
└─────────────────────┬─────────────────────────────┘
                      ▼
┌───────────────────────────────────────────────────┐
│  GridPlanner          (纯数学分配)                 │
│  → Vec<SubplotSpec>   (含 bounds + 索引绑定)      │
└────────┬──────────────────────────────────────────┘
         ▼
┌───────────────────────────────────────────────────┐
│  AxisBindingResolver  (预处理)                     │
│  → ResolvedAxisRanges (共用轴协调后的轴范围)       │
└────────┬──────────────────────────────────────────┘
         ▼
┌───────────────────────────────────────────────────┐
│  ColorAssigner        (预处理)                     │
│  → ColorContext       (调色板 + 系列颜色分配)      │
└────────┬──────────────────────────────────────────┘
         ▼
┌───────────────────────────────────────────────────┐
│  DataProcessor[0..N]  (每个 subplot 独立执行)      │
│  → SubplotVisualData  (Vec<VisualElement>)         │
│                                                     │
│  内部 6 步骤:                                      │
│  ① data_transform()                                │
│  ② compute_axis_range()                            │
│  ③ generate_candidate_ticks()                      │
│  ④ compute_plot_area()                             │
│  ⑤ refine_ticks()                                 │
│  ⑥ build_geometry()                                │
└────────┬──────────────────────────────────────────┘
         ▼
┌───────────────────────────────────────────────────┐
│  VisualElementBuilder   (合并 + 排序 + 浮层)        │
│  → Vec<VisualElement>                              │
└────────┬──────────────────────────────────────────┘
         ▼
┌─────────────────┐
│  Renderer       │  (SVG/PNG 输出，与现有相同)
└─────────────────┘
```

---

## 二、关键模块详细设计

### 2.1 GridPlanner — 纯画布切分

**职责**：仅根据 `option.grid` 配置和画布尺寸，计算每个 subplot 的像素边界。**完全不接触系列数据、轴标签、文本测量、刻度计算**。

```rust
pub struct GridPlanner {
    total_width: u32,
    total_height: u32,
    grid_configs: Vec<GridOption>,  // 来自 ChartOption.grid
}

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

impl GridPlanner {
    pub fn new(width: u32, height: u32, grids: &[GridOption]) -> Self;
    pub fn plan(&self) -> Vec<SubplotSpec>;
}
```

**规划算法**（纯数学计算）：

- 若 `grids` 为空，创建一个默认 grid 填满整个画布（减去全局标题/图例的预留空间，这些也是纯配置维度）。
- 若 `grids` 不为空，按照 ECharts 布局规则解析每个 `GridOption`：
  - `left` / `right` / `top` / `bottom`：支持像素值或百分比（相对画布）
  - `width` / `height`：支持像素值或百分比（相对画布）
- 多个 grid 之间的间距由配置的 `left`/`top` 等值决定，grid 之间可能有重叠或间隙。

**GridPlanner 不处理的问题**（这些在 DataProcessor 中处理）：

| 问题 | 处理位置 |
|------|---------|
| `containLabel` 模式 | DataProcessor — 知道标签尺寸后才能决定是否需扩大 chart bounds |
| 轴标签重叠自适应 | DataProcessor — 需要知道数据文本的宽度 |
| 刻度数量/间隔 | DataProcessor — 需要知道数据范围 |

---

### 2.2 DataProcessor — 每个 Subplot 独立的数据→视觉管线

**职责**：接收已分配的 chart bounds + 系列数据，完整地完成从数据变换到像素几何的全过程。

**为什么这是一个高内聚的模块而非多个分散组件？**

所有步骤共享同一份 context（chart_bounds + 同一组数据），且**有强顺序依赖**：
- 必须先变换数据，才能知道轴范围
- 必须先知道轴范围，才能生成候选刻度值
- 必须先测量候选刻度文本的尺寸，才能从 chart bounds 扣除出 plot area
- 必须知道 plot area + 轴范围，才能最终确定刻度位置
- 必须知道刻度位置，才能生成系列几何

分散到多个模块反而需要传递复杂的中间结构，形成耦合。

> **关于 "刻度→标签→plot_area" 的循环依赖**：文档旧版将步骤③ compute_plot_area() 放在刻度计算之前，造成"plot_area 需要标签文本尺寸，但标签文本在刻度计算之后才确定"的逻辑矛盾。经评审后修正为：**先生成候选刻度（基于轴范围的标准算法，如 5 等分）→ 测量候选标签文本 → 计算 plot_area → 在 plot_area 内精确刻度**。候选刻度的文本在已知字体大小后，其宽度是 boundable（例如数值轴 0~250 最多 3 位数），所以单次足够，无需迭代。

#### 输入

```rust
pub struct DataProcessorInput<'a> {
    /// GridPlanner 输出的 subplot 分配
    pub spec: &'a SubplotSpec,
    /// 完整的用户配置（含 series, xAxis, yAxis, 样式等）
    pub option: &'a ChartOption,
    /// 颜色上下文（由 ColorAssigner 预先分配）
    pub colors: &'a ColorContext,
    /// 已解析的轴范围（由 AxisBindingResolver 计算，共用轴时全局协调）
    pub axis_ranges: &'a ResolvedAxisRanges,
    /// 原始数据（可选，如果不从 option 中读取）
    pub external_data: Option<&'a DataFrame>,
}
```

#### 输出

```rust
pub struct SubplotVisualData {
    /// 系列相关的视觉元素（柱、线、扇区、数据标签等）
    pub series_elements: Vec<VisualElement>,
    /// 轴相关的视觉元素（轴线、刻度线、刻度标签、轴标题）
    pub axis_elements: Vec<VisualElement>,
    /// 网格线（与轴关联的背景网格）
    pub grid_lines: Vec<VisualElement>,
}
```

> 直接用 `VisualElement` 而非引入 `Geometry` 类型，避免概念重复和转换开销。

#### 内部处理步骤（柱状图示例）

以下以标准的分类 X 轴 + 数值 Y 轴柱状图为例，展示 DataProcessor 内部 6 个步骤：

```
输入: chart_bounds = (x:50, y:50, w:400, h:300)
      series.data = [{"month":"Jan","sales":120},
                     {"month":"Feb","sales":200}, ...]
      y_axis.type = "value", x_axis.type = "category"
      x_axis.data = ["Jan","Feb","Mar","Apr"]
```

**步骤 ① data_transform()**

```
原始数据列表:
  Jan=120, Feb=200, Mar=80, Apr=150
  → 无堆叠、无聚合，直接通过（若有堆叠则累加同组值）
  → 输出: Vec<DataPoint> (已排序、已过滤)
```

**步骤 ② compute_axis_range()**

```
Y 值范围: [80, 200]
  → 上边界: 200 + (200-80)*0.05 = 206
  → 下边界: 0（柱状图从零开始）
  → Y 轴范围: (0, 206)
  → 可选: 取整 → (0, 250)

X 轴: 类目 ["Jan","Feb","Mar","Apr"]
  → boundary_gap=true → 范围 (0, 4)
  → boundary_gap=false → 范围 (0, 3)
```

**步骤 ③ generate_candidate_ticks()**

```
基于轴范围生成候选刻度值（此时还不需要像素坐标）:

  Y 轴 (值域 0~250):
    候选刻度值: [0, 50, 100, 150, 200, 250]
    → 对应的标签文本: ["0", "50", "100", "150", "200", "250"]
    → 这些是候选值，最终刻度位置在步骤⑤中基于 plot_area 精确定位

  X 轴 (类目):
    候选标签文本: ["Jan", "Feb", "Mar", "Apr"]
    → 类目轴的标签是固定的，不需要范围推算
```

> 候选刻度使用标准算法（如 5-10 等分、2-5-10 间隔序列）从数据范围生成，**不依赖像素尺寸**。依赖像素尺寸的精确定位在步骤⑤中完成。

**步骤 ④ compute_plot_area()**

```
这是关键步骤——从 chart_bounds 中扣除轴标签占位:

  1. Y 轴候选标签文本: "0", "50", "100", "150", "200", "250"
  2. 已知字体大小（来自主题配置），测量最宽标签 "250" → ~25px
  3. Y 轴标签预留: 25px（标签）+ 5px（间距）+ 1px（轴线）= 31px
  4. X 轴候选标签文本: "Jan", "Feb", "Mar", "Apr"
  5. 测量最高标签 "Jan" 的文本高度 → ~15px
  6. X 轴标签预留: 15px + 5px = 20px

  plot_area = (x: 50+31, y: 50, w: 400-31, h: 300-20)
            = (x: 81, y: 50, w: 369, h: 280)

  ※ 如果 contain_label=true: 从 chart_bounds 向外扩展，plot_area 更大
  ※ 如果 labels 重叠: 旋转 45° → 重新测量高度 → 调整预留（最多两次迭代）
```

> 候选刻度文本宽度的上界是可预估的（已知字体大小+最大数字位数），因此即使在不知道精确像素位置时也能可靠地预留空间。无需迭代。

**步骤 ⑤ refine_ticks()**

```
在 plot_area (w=369, h=280) 内将候选刻度映射为精确像素坐标:

  Y 轴 (值域 0~250, 像素高度 280):
    刻度值: [0, 50, 100, 150, 200, 250]
    像素 y: [50+280, 50+224, 50+168, 50+112, 50+56, 50]
            = [330, 274, 218, 162, 106, 50]
    （公式: plot_area.y0 + h - (value/max)*h）

  X 轴 (4 个类目, 像素宽度 369, boundary_gap=true):
    每个类目宽度: 369 / 4 = 92.25
    刻度中心 x: [81+46.1, 81+138.4, 81+230.6, 81+322.9]
               = [127.1, 219.4, 311.6, 403.9]
    柱子宽度: 92.25 * 0.6 = 55.35 (60% 占宽比)
```

**步骤 ⑥ build_geometry()**

```
生成 VisualElement:

  柱子:
    Rect { x:99.4, y:162, w:55.4, h:168, fill:blue }     // Jan(120)
    Rect { x:191.7, y:50,  w:55.4, h:280, fill:orange }  // Feb(200)
    Rect { x:284.0, y:218, w:55.4, h:112, fill:green }   // Mar(80)
    Rect { x:376.2, y:106, w:55.4, h:224, fill:red }     // Apr(150)

  Y 轴线:
    Line { x1:81, y1:50, x2:81, y2:330 }

  Y 轴刻度线:
    Line { x1:76, y1:330, x2:81, y2:330 }  // 0
    Line { x1:76, y1:274, x2:81, y2:274 }  // 50
    ...

  Y 轴标签:
    Text { x:66, y:330, text:"0",  align:right }
    Text { x:66, y:274, text:"50", align:right }
    ...

  X 轴线:
    Line { x1:81, y1:330, x2:450, y2:330 }

  X 轴类别标签:
    Text { x:127.1, y:345, text:"Jan", align:center }
    Text { x:219.4, y:345, text:"Feb", align:center }
    ...
```

#### 不同图表类型的 DataProcessor 差异

| 图表类型 | 步骤① 差异 | 步骤② 差异 | 步骤③ 差异 | 步骤④ 差异 | 步骤⑤ 差异 |
|---------|-----------|-----------|-----------|-----------|-----------|
| 柱状图 | 可能需要堆叠 | 从零开始 | 需计算柱宽 | X 轴类目尺 | 生成 Rect |
| 折线图 | 可能需要 area_style | 不强制从零 | 无柱宽计算 | 同上 | 生成 Path |
| 饼图 | 计算百分比 | 不需要轴 | 圆心+半径从 bounds 推算 | 不需要轴 | 生成 Arc/Path |
| 散点图 | 不需要 | X/Y 都是数值 | 不需要 | X/Y 都需刻度 | 生成 Circle |
| 雷达图 | 不需要 | 多维度范围 | 需计算多边形半径 | 角度轴分度 | 生成 Polygon |

#### ColorContext

```rust
pub struct ColorContext {
    pub palette: Vec<Color>,          // 主题色板
    pub background: Color,
    pub series_colors: Vec<Color>,    // 预分配给 series 的颜色（由 ColorAssigner 按索引分配）
    pub axis_line_color: Color,
    pub axis_label_color: Color,
    pub grid_line_color: Color,
}
```

`ColorAssigner` 在 DataProcessor 之外完成颜色分配（可复用当前 `ChartModel` 中的色轮逻辑），作为参数传入 DataProcessor，几何生成时直接使用。分面场景下，同一原始系列在展开的所有分面中保持相同颜色。

#### TextMeasurer（文本测量缓存）

步骤④需要多次测量文本宽度/高度。为避免重复测量相同文本：

```rust
pub struct TextMeasurer {
    cache: HashMap<(String, f64, f64), (f64, f64)>, // (text, font_size, rotation) → (width, height)
}

impl TextMeasurer {
    pub fn measure(&mut self, text: &str, style: &TextStyle) -> (f64, f64);
}
```

`TextMeasurer` 实例可以在 `ChartBuilder` 级别创建，在所有 `DataProcessor` 之间共享，减少重复测量开销。

---

### 2.3 VisualElementBuilder

**职责**：
- 收集所有 subplot 的 `SubplotVisualData` 中的 VisualElement
- 添加全局元素（画布背景、全局标题、全局图例）
- 全局标题和图例采用**浮层方式**（与 ECharts 行为一致）：它们根据用户配置的 `top`/`left`/`right`/`bottom` 计算绝对像素位置，浮动在 subplot 之上，不与 grid 区域交互（可能部分重叠）。`GridPlanner` 不需要为它们预留空间。
- 按 z 索引排序（先背景 → 网格 → 轴 → 系列 → 标签 → 图例 → 标题）
- 输出 `Vec<VisualElement>` 交给渲染器

在 DataProcessor 直接输出 VisualElement 的前提下，VisualElementBuilder 的职责纯粹是**合并和排序**，不需要做类型转换。

---

## 三、完整示例：两个 subplot 的柱状图

### 3.1 用户配置

```json
{
  "grid": [
    { "left": "5%", "top": "10%", "width": "40%", "height": "70%" },
    { "right": "5%", "top": "10%", "width": "40%", "height": "70%" }
  ],
  "xAxis": [
    { "type": "category", "data": ["Jan","Feb","Mar"], "gridIndex": 0 },
    { "type": "category", "data": ["A","B","C"], "gridIndex": 1 }
  ],
  "yAxis": [
    { "type": "value", "gridIndex": 0 },
    { "type": "value", "gridIndex": 1 }
  ],
  "series": [
    { "type": "bar", "data": [100, 200, 150], "xAxisIndex": 0, "yAxisIndex": 0 },
    { "type": "bar", "data": [300, 100, 250], "xAxisIndex": 1, "yAxisIndex": 1 }
  ],
  "title": { "text": "双柱状图对比", "left": "center" }
}
```

### 3.2 各阶段输出

**GridPlanner**（画布 800x500）：

```
SubplotSpec[0]:
  id = 0
  bounds = (40, 50, 320, 350)     // 5% left=40, 10% top=50, 40% w=320, 70% h=350
  series_indices = [0]
  x_axis_indices = [0]
  y_axis_indices = [0]

SubplotSpec[1]:
  id = 1
  bounds = (440, 50, 320, 350)    // 5% right→x=440, 10% top=50, 40% w=320, 70% h=350
  series_indices = [1]
  x_axis_indices = [1]
  y_axis_indices = [1]
```

> GridPlanner 只做加减乘除，不看数据。

**DataProcessor[0]**（bounds=40,50,320,350, series=[100,200,150]）：

```
plot_area = (70, 50, 290, 330)      // 扣除 Y 轴标签 ~30px
Y 范围: (0, 220)
柱子:
  Jan: Rect{x:118.3, y:150,  w:58, h:230, fill:blue}
  Feb: Rect{x:176.7, y:50,   w:58, h:330, fill:orange}
  Mar: Rect{x:235.0, y:100,  w:58, h:280, fill:green}
Y 轴: Line + Ticks + Labels
X 轴: Line + Category Labels
```

**DataProcessor[1]**（bounds=440,50,320,350, series=[300,100,250]）：

```
plot_area = (470, 50, 290, 330)
Y 范围: (0, 330)
柱子:
  A: Rect{x:548.3, y:50,  w:58, h:330, fill:blue}
  B: Rect{x:606.7, y:250, w:58, h:130, fill:orange}
  C: Rect{x:665.0, y:83,  w:58, h:297, fill:green}
Y 轴: Line + Ticks + Labels
X 轴: Line + Category Labels
```

**VisualElementBuilder**：

```
合并顺序（z-index 升序）:
  0: 背景 Rect    (0,0,800,500, fill:white)
  1: 标题 Text    (center, 20, "双柱状图对比")
  2: subplot0 网格线
  3: subplot0 Y 轴线 + 刻度
  4: subplot0 X 轴线 + 标签
  5: subplot0 柱子
  6: subplot1 网格线
  7: subplot1 Y 轴线 + 刻度
  8: subplot1 X 轴线 + 标签
  9: subplot1 柱子
```

---

## 四、与现有 `ChartOption` 的兼容

- 保持 `ChartOption` 结构不变，仍然支持 ECharts 风格的 JSON 配置。
- 在 `ChartBuilder` 内部，将 `ChartOption` 分解为：
  - `grid` 列表 → 传给 `GridPlanner`
  - `series` + `xAxis`/`yAxis` → 传给每个 `DataProcessor`
- 现有 `ChartModel` 可以逐步废弃，或简化成只保留样式解析、主题、颜色调色板等辅助功能。
- `DataProcessor` 内部需要解析 `SeriesOption` 枚举，将其转换为对应的处理逻辑。这可以复用现有的 `SeriesOption` 变体，只需增加新的转换代码。

**即使使用新架构，用户仍然可以写**：

```json
{
  "grid": [{ "left": "10%", "width": "40%" }, { "right": "10%", "width": "40%" }],
  "xAxis": [{ "type": "category", "data": ["A","B"] }],
  "series": [{ "type": "bar", "data": [1,2] }, { "type": "line", "data": [3,4] }]
}
```

---

## 五、与现有代码的集成策略（渐进式）

1. **Phase 0**：定义新模块骨架 `grid_planner`, `axis_binding_resolver`, `color_assigner`, `data_processor`，建立核心类型（SubplotSpec, ResolvedAxisRange, ColorContext, DataProcessorInput, SubplotVisualData, TextMeasurer），**不实现任何逻辑**。
2. **Phase 1**：实现 `GridPlanner`，替换现有的 `GridManager`（GridManager 也是纯计算，可以直接复用算法），保持输出兼容。
3. **Phase 2**：选择一个简单的图表类型（如饼图——无需轴刻度），用新 `DataProcessor` 实现。在 `ChartBuilder` 中增加 `use_new_pipeline` feature flag，新旧管线并行运行并比较输出。
4. **Phase 3**：逐个迁移其他系列（Line → Bar → Scatter → Candlestick ...），分批移除旧代码（`ResolvedSeries`, `LayoutEngine`, `ChartComponent`, `Pipeline` 等）。

---

## 六、扩展性考虑

### 6.1 共用轴（Shared Axis）

当多个 subplot 共用 Y 轴时，在 DataProcessor 之前增加一个 **AxisRangeCoordinator** 阶段：

```
GridPlanner → AxisRangeCoordinator → DataProcessor[0]
                                   ↘ DataProcessor[1]
```

AxisRangeCoordinator 收集所有相关 subplot 的数据范围，取全局 min/max，然后传递给每个 DataProcessor 使用相同的轴范围。

### 6.2 分面（Facet）

分面 = 根据数据列的值动态生成多个 subplot。在 GridPlanner 之后插入 **FacetExpander**：

```
GridPlanner → FacetExpander → DataProcessor[0..N]
               ↑ 根据 "region" 列的 3 个值
                 将 1 个 subplot 展开为 3 个
```

FacetExpander 将单个 SubplotSpec 拆分为多个，每个新的 SubplotSpec 带有一个数据过滤器（如 `region="North"` / `region="South"` / `region="East"`），DataProcessor 自动应用过滤。

### 6.3 图层（Layer）

图层 = 在同一个 subplot 内叠加不同类型的系列（如 bar + line）。DataProcessor 天然支持：

```rust
// 同一个 SubplotSpec 中，series_indices = [0, 1] 指向 bar 和 line
// DataProcessor 内部:
for &series_idx in spec.series_indices {
    match series_type {
        Bar => build_bar_geometry(data, plot_area, ...),
        Line => build_line_geometry(data, plot_area, ...),
    }
}
```

轴范围计算会自动考虑所有系列的数据范围。

---

## 七、设计边界总结

以下表格明确每个问题应该在哪个阶段解决：

| 问题 | GridPlanner | AxisBindingResolver | ColorAssigner | DataProcessor | VisualElementBuilder |
|------|:-----------:|:-------------------:|:-------------:|:-------------:|:--------------------:|
| 画布切分 | ✅ | — | — | — | — |
| 系列→subplot 分配 | ✅（系列索引） | — | — | — | — |
| 轴实例解析（索引→配置） | — | ✅ | — | — | — |
| 共用轴范围协调 | — | ✅ | — | — | — |
| 颜色分配（色轮+索引） | — | — | ✅ | — | — |
| 数据变换（堆叠/聚合） | — | — | — | ✅ | — |
| 候选刻度生成 | — | — | — | ✅ | — |
| 轴标签文本测量 | — | — | — | ✅ | — |
| Plot area 计算（扣除标签） | — | — | — | ✅ | — |
| containLabel | — | — | — | ✅ | — |
| 刻度精确定位 | — | — | — | ✅ | — |
| 系列几何生成（柱/线/扇区） | — | — | — | ✅ | — |
| 全局标题/图例（浮层） | — | — | — | — | ✅ |
| z-index 排序 | — | — | — | — | ✅ |
| 渲染后端无关图元转换 | — | — | — | — | ✅ |

---

## 八、核心优势总结

1. **GridPlanner 数据无关** — 纯数学计算，易于理解和测试
2. **DataProcessor 高内聚** — 5 个步骤共享 context，避免跨模块传递 DataCoordinateSystem
3. **Per-subplot 并行** — 无共享状态，天然支持并行
4. **Facet/Layer 易扩展** — pipeline 插接模式
5. **ECharts 兼容性保留** — ChartOption 输入不变
6. **渐进式迁移** — 逐个系列替换，风险可控