# new-arch.md 重构方案评估报告

> 评估日期: 2026-05-21（V2 修正版）
> 评估基准: 当前 liecharts v0.1.0-beta.1 代码库
> 方案文档: [new-arch.md](./new-arch.md)

---

## 目录

1. [方案概述](#1-方案概述)
2. [术语澄清：Grid ≠ 轴刻度](#2-术语澄清grid--轴刻度)
3. [与当前架构的对比](#3-与当前架构的对比)
4. [方案的优势](#4-方案的优势)
5. [识别的关键问题](#5-识别的关键问题)
6. [分项评估](#6-分项评估)
7. [改进建议](#7-改进建议)
8. [总结：可行性与优先级](#8-总结可行性与优先级)

---

## 1. 方案概述

new-arch.md 提出一个三阶段新架构：

```
ChartOption → GridPlanner → DataProcessor (per subplot) → VisualElementBuilder → Renderer
```

核心变化：
- **GridPlanner**: 纯数学分配，将画布按 grid 配置切分成 N 个矩形区域。**不接触任何数据**。
- **DataProcessor**: 每个 subplot 独立运行，在已分配的像素区域内完成全部数据处理和视觉生成。
- **VisualElementBuilder**: 薄转换层，统一合并全局组件和 subplot 输出。

---

## 2. 术语澄清：Grid ≠ 轴刻度

> ⚠️ **V1 评估存在严重误解**——误以为 GridPlanner 的"分配像素区域"包含轴标签空间预留。以下澄清。

在 ECharts 体系和本方案中：

| 术语 | 含义 | 跨越的层级 |
|------|------|-----------|
| **Grid** | 单个图表的**画布分配区域**（如 `{left: 50, top: 50, width: 300, height: 250}`） | GridPlanner |
| **Subplot** | Grid 的运行时表示，携带绑定关系（系列索引、轴索引） | GridPlanner |
| **Chart Bounds** | 分配给某个 subplot 的像素矩形（例如 x=50, y=50, w=300, h=250） | GridPlanner 输出 |
| **Plot Area** | 从 chart bounds 中扣除轴标签、标题等占位后，实际绘图区域 | DataProcessor 内部 |
| **Axis Tick** | 坐标轴上的刻度线和标签，**完全由数据驱动** | DataProcessor 内部 |

**关键边界**：

```
GridPlanner 的输出:
  SubplotSpec.bounds = (50, 50, 300, 250)
                                          ← 这是 chart 的总空间
                                          ← 不含任何轴刻度信息
                                          ← 不含任何数据信息

DataProcessor 的输入:
  接收 SubplotSpec.bounds = (50, 50, 300, 250) + 系列数据

DataProcessor 内部:
  1. 从数据计算出轴范围 (如 Y: 0~100)
  2. 测量 Y 轴标签 "0", "20", "40", ... "100" 的文本宽度 → 约 30px
  3. 从 chart bounds 左侧扣除 30px → 得到 plot_area 左边界为 x=80
  4. 类似地扣除 X 轴标签高度 → 确定完整的 plot_area
  5. 在 plot_area 内建立数据→像素映射
  6. 计算刻度位置、生成几何
```

**所以 GridPlanner 确实是完全数据无关的**——它只做画布切分，不涉及任何数据、文本测量或刻度计算。

---

## 3. 与当前架构的对比

### 3.1 处理顺序对比

| 步骤 | 当前架构 | 新架构 |
|------|---------|--------|
| 1 | 解析 Option → ChartModel（含所有数据解析） | **GridPlanner** — 画布切分 |
| 2 | LayoutEngine 计算 grid bbox + 轴标签布局 | **DataProcessor** — 每个 subplot 数据→几何 |
| 3 | compute_data_coord_for_grid() 收集数据范围 | —（合并到 DataProcessor） |
| 4 | Component + Pipeline → VisualElement | **VisualElementBuilder** — 合并 |
| 5 | Renderer | Renderer（不变） |

### 3.2 核心差异

| 维度 | 当前架构 | 新架构 |
|------|---------|--------|
| **布局策略** | 先 Measure 轴标签尺寸再 Arrange | Grid 纯数学分配 → DataProcessor 内部消化标签 |
| **数据范围计算** | 在 chart 层集中收集（compute_data_coord_for_grid） | 每个 DataProcessor 独立计算 |
| **坐标系统** | DataCoordinateSystem 跨模块传递 | DataProcessor 内部直接映射到像素 |
| **渲染管线** | Component → Pipeline (Transform→Map→Build) | DataProcessor → VisualElementBuilder |
| **并行潜力** | 低（串行） | 高（per-subplot 可并行） |

### 3.3 具体数据流对比（以柱状图为例）

```
当前架构:
  ChartOption
    → ChartModel::new()        解析颜色、主题、轴样式、系列数据
    → LayoutEngine             计算 grid_bbox + 排列轴
    → compute_data_coord()     从系列数据收集范围 → DataCoordinateSystem
    → BarSeriesComponent       使用 SeriesContext(含 DataCoord)
      → IdentityTransformer    或 StackedTransformer
      → CartesianBarMapper     data → 像素坐标
      → BarVisualBuilder       像素 → VisualElement
    → AxisComponent            使用 DataCoord 计算刻度
    → SvgRenderer/PixmapRenderer

新架构:
  ChartOption
    → GridPlanner              画布切分 → SubplotSpec (chart_bounds)
    → DataProcessor (per subplot)
      → 从 series.data 构造数据
      → 执行堆叠/聚合/变换
      → 从数据计算轴范围 min/max
      → 根据数据值文本宽度 + chart_bounds 计算 plot_area
      → 建立数据→像素映射
      → 计算刻度位置、生成几何（柱、轴、标签）
      → 输出 SubplotVisualData（所有几何已是像素坐标）
    → VisualElementBuilder     合并全局元素 + 排序
    → SvgRenderer/PixmapRenderer (不变)
```

---

## 4. 方案的优势

### 4.1 关注点分离更清晰

**当前问题**: `LayoutEngine` 既做 grid 区域分配，又做轴标签排列（Measure-Arrange），`compute_data_coord_for_grid()` 横跨 layout 和 chart 两层。

**新方案**:
- GridPlanner: 只回答 **"每个 chart 放哪里、多大"**
- DataProcessor: 只回答 **"在这个尺寸内，数据画成什么样子"**

没有重叠，没有跨层依赖。

### 4.2 消除了 DataCoordinateSystem 的跨模块传递

**当前问题**（代码证据）:

[chart.rs#L341-540](file:///d:/code/rust/liecharts/src/chart.rs#L341-540) 的 `compute_data_coord_for_grid()` 收集所有系列数据 → 计算每个轴的范围 → 打包成 `DataCoordinateSystem`。然后这个结构被传递给：
- `AxisComponent`（用来定位刻度）
- 各系列 Component（用来映射数据→像素）

这个结构在 5+ 个模块间传递，任何修改都需要同步更新所有消费方。

**新方案**: 每个 DataProcessor 独立计算自己的范围和刻度，不需要共享 DataCoordinateSystem。**内聚性提升，耦合性降低**。

### 4.3 Per-subplot 并行潜力

每个 subplot 的 DataProcessor 无共享状态（只读取自己的 SubplotSpec 和 option），天然支持 `rayon::par_iter()` 并行。对多 grid 场景（如小 multiples / faceted charts）性能提升显著。

### 4.4 分面（Facet）的自然扩展

在 GridPlanner 之后插入 FacetExpander，将单个 SubplotSpec 根据数据列的分组值动态展开为多个 SubplotSpec（每个 bounds 缩小），然后各自进 DataProcessor。这是一个清晰的 pipeline 插接模式，当前架构难以实现。

### 4.5 渐进式迁移策略合理

Phase 0→1→2→3 的路径经过验证是务实的：
- Phase 0: 只定义新模块骨架，零影响
- Phase 1: 替换 GridPlanner（与现有 GridManager 输出对齐）
- Phase 2: 只替换一个系列（如 Pie），新旧管线通过 feature flag 切换
- Phase 3: 逐个替换其他系列

---

## 5. 识别的关键问题

### 问题 1：DataProcessor 需要合理的内部结构化（中等）

> ⚠️ **V1 评估将此标记为"严重"级别**。在正确理解"GridPlanner 只做画布切分"后，重新评估如下：

DataProcessor 内部确实包含多个步骤：

```
输入: chart_bounds + 系列配置 + 原始数据
  │
  ├── 步骤 1: 数据变换（堆叠、聚合、过滤）
  ├── 步骤 2: 从数据计算轴范围（min/max）
  ├── 步骤 3: 测量数据文本 → 扣除标签空间 → 确定 plot_area
  ├── 步骤 4: 建立数据↔像素映射 + 计算刻度位置
  ├── 步骤 5: 生成系列几何（柱/线/扇区）
  └── 步骤 6: 生成标签位置
```

但这些步骤具有**强顺序依赖**和**共享 context**：

- 步骤 2 依赖 1
- 步骤 3 依赖 1 的结果 + chart_bounds
- 步骤 4 依赖 2 和 3
- 步骤 5 和 6 依赖 4

把它们放在一个模块里不是"上帝模块"，而是**高内聚**——所有步骤围绕同一个任务（"在 chart_bounds 内把数据变成像素"），共享同一份 context。

**实际风险评估**:
- 按每个 subplot 一个 DataProcessor 实例计算，代码量预计 400-600 行
- 远小于当前分散在 5+ 文件中的总代码量（约 1200+ 行）
- 每个 steps 可以拆分为 DataProcessor 内部的私有方法，而不是独立模块

**建议**:
- 保持 DataProcessor 作为一个模块，内部按函数（而非独立 trait）划分步骤
- 如果某个步骤变得特别复杂（例如坐标映射），再考虑提取为独立模块
- 避免过早抽象

### 问题 2：SubplotVisualData 中的 Geometry 与现有 VisualElement 重复（中等）

**方案定义**:

```rust
pub struct SeriesShape {
    pub geometries: Vec<Geometry>,  // Rect, Circle, Path, TextRun 等
}
```

`Geometry` 的枚举变体与现有的 `VisualElement` enum（[visual.rs](file:///d:/code/rust/liecharts/src/visual.rs)）几乎相同。引入高度相似的 `Geometry` 类型会导致概念混淆和转换开销。

**建议**: 直接使用 `VisualElement` 替代 `Geometry`。DataProcessor 直接输出 `Vec<VisualElement>`，VisualElementBuilder 只做合并和排序。

### 问题 3：缺少颜色/主题的集成设计（中等）

SubplotVisualData 中缺少颜色和主题信息：

```rust
pub struct SubplotVisualData {
    pub series_shapes: Vec<SeriesShape>,
    pub x_axis: Option<AxisVisual>,
    pub y_axes: Vec<AxisVisual>,
    pub grid_lines: Vec<GridLine>,
}
```

当前颜色分配在 `ChartModel::new()` 中通过色轮完成，需要在 DataProcessor 中复用类似的逻辑。

**建议**: 为 DataProcessor 传入 `&ColorPalette` 或预分配 `Vec<Color>`：

```rust
pub struct DataProcessorContext<'a> {
    pub spec: &'a SubplotSpec,
    pub option: &'a ChartOption,
    pub theme: &'a Theme,
    pub series_colors: &'a [Color],  // 预分配的颜色
}
```

### 问题 4：polars/DataFrame 依赖引入的复杂度（中等）

方案提到"使用 polars"进行数据聚合。这是一个重大新依赖：

| 维度 | 当前方式 (Vec) | 引入 polars |
|------|---------------|------------|
| 依赖数 | 0（内置） | +1（polars） |
| 编译时间 | 基准 | 显著增加（polars 编译很重） |
| 二进制体积 | 基准 | 显著增加 |
| API 复杂度 | 低（Vec） | 高（DataFrame） |
| 聚合能力 | 无 | 强 |
| 适用场景 | 服务端简单图表 | 数据分析场景 |

**建议**:
- **不强制依赖 polars**，数据变换使用轻量级 Rust 原生实现（当前已有 StackedTransformer）
- **DataFrame 作为可选依赖**（feature flag），默认 Vec-based
- 内建简单的 group-by-aggregate（sum/avg/count 覆盖大部分图表需求）

### 问题 5：缺少错误处理设计（低）

方案未提及错误处理。DataProcessor 需要考虑：
- 数据 schema 不匹配（如 x 列不存在）
- NaN/Infinity 处理
- 轴范围计算失败
- 像素坐标溢出

### 问题 6：VisualElementBuilder 职责过于单薄（低）

如果 DataProcessor 直接输出 `Vec<VisualElement>`，VisualElementBuilder 可能退化成一个 10 行的合并函数。但这不影响整体架构——职责单薄不一定是问题，只要边界清晰。

---

## 6. 分项评估

### 6.1 方案完整性评分（V2 修正后）

| 评估项 | 评分 | 说明 |
|--------|:----:|------|
| 架构清晰度 | ⭐⭐⭐⭐⭐ | 三阶段职责明确，边界清晰 |
| 细节完备性 | ⭐⭐⭐⭐ | 核心类型定义完整，颜色/错误处理需补充 |
| 可行性 | ⭐⭐⭐⭐⭐ | 技术上成熟，渐进式迁移路径可行 |
| 与现有代码兼容 | ⭐⭐⭐⭐⭐ | 保持 ChartOption 不变，可逐步替换 |
| 扩展性 | ⭐⭐⭐⭐⭐ | Facet/Layer/共用轴设计自然 |
| 性能考量 | ⭐⭐⭐⭐ | 并行潜力好 |
| 风险评估 | ⭐⭐⭐⭐ | 无严重风险，几个中等问题均可控 |

### 6.2 与 data-driven-refactoring.md 的互补关系

之前我在 [data-driven-refactoring.md](./data-driven-refactoring.md) 中提出的方案是**从用户 API 层叠加 DataSpec**（"图表类型+编码映射"风格），new-arch.md 的方案是**重构内部管线**。两者互补：

```
data-driven-refactoring.md（用户 API 层）:
  DataSpec → GoG Compiler → ChartOption
                              ↓
new-arch.md（内部架构层）:
                    GridPlanner → DataProcessor → VisualElementBuilder
                              ↓
                         Renderer
```

可独立推进行进，在 Phase 2 后合并。

---

## 7. 改进建议

### 7.1 在 new-arch.md 中增加术语表

建议在方案前增加术语定义，明确区分：

| 术语 | 定义 |
|------|------|
| Chart Bounds | GridPlanner 输出的像素矩形，**不含轴/标签/刻度信息** |
| Plot Area | DataProcessor 内部从 chart bounds 再扣除标签后得到的实际绘图区域 |
| Grid | 与 ECharts 的 grid 概念一致，指单个子图区域的配置 |
| Subplot | Grid 的运行时实例，携带关联的系列和轴配置 |

### 7.2 为 DataProcessor 增加内部步骤示意图

```
输入: chart_bounds(400x300)  + 系列数据

DataProcessor {
  ┌─ data_transform() ──────────────────┐
  │  原始 Vec<DataPoint> → 堆叠后数据    │
  └──────────────┬──────────────────────┘
                 ▼
  ┌─ compute_axis_range() ──────────────┐
  │  从堆叠后数据 → Y: 0~100, X: A~E    │
  └──────────────┬──────────────────────┘
                 ▼
  ┌─ compute_plot_area() ───────────────┐
  │  测量 Y 轴标签宽度(30px)            │
  │  测量 X 轴标签高度(20px)            │
  │  chart_bounds(400x300) → plot_area  │
  │    = (30, 0, 370, 280)              │
  └──────────────┬──────────────────────┘
                 ▼
  ┌─ compute_ticks() ───────────────────┐
  │  [0,20,40,60,80,100] → pixel pos   │
  └──────────────┬──────────────────────┘
                 ▼
  ┌─ build_geometry() ──────────────────┐
  │  数据值 → 像素 Rect / Path / Text   │
  └──────────────┬──────────────────────┘
                 ▼
  输出: Vec<VisualElement> (像素坐标)
```

### 7.3 直接使用 VisualElement

消除 Geometry 类型，DataProcessor 直接输出 `Vec<VisualElement>`，按绘制顺序排列（先轴后系列，或按 z 索引排序）。

### 7.4 增加 ColorContext 设计

```rust
pub struct ColorContext {
    pub palette: Vec<Color>,
    pub background: Color,
    pub series_colors: Vec<Color>,  // 预分配给每个 series 的颜色
    pub axis_colors: (Color, Color), // 轴线色 + 标签色
}
```

在 DataProcessor 初始化时传入，几何生成时直接使用。

### 7.5 分步迁移计划调整

建议将 data-driven-refactoring.md 和 new-arch.md 的迁移计划合并：

| Phase | 内容 | 交付物 |
|:-----:|------|--------|
| 0 | 定义新模块骨架（grid_planner, data_processor），建立核心类型 | 新模块骨架 |
| 1 | 实现 GridPlanner，替换现有 GridManager | Grid 规划新管线 |
| 2 | 选一个简单系列（Pie/Line）用 DataProcessor 实现，新旧并行 | 首个新管线系列 |
| 3 | 逐个迁移其他系列，分批移除旧代码（BarSeriesComponent 等） | 全部迁移 |
| 4 | 叠加 DataSpec 层（来自 data-driven-refactoring.md Phase 1） | 数据驱动 API |
| 5 | Facet / Layer 支持（长期） | 高级特性 |

---

## 9. 评审意见响应记录（2026-05-23）

> **背景**：2026-05-23 收到对 new-arch.md 的详细外部评审意见，包含 5 个潜在问题。以下逐条记录评审内容、评估结论和文档更新结果。

### 9.1 问题 1：文本测量与 plot area 的循环依赖

**评审意见**：
> 步骤③ compute_plot_area() 需要轴标签文本尺寸才能扣除占位，但刻度文本是在步骤④才确定的。形成循环：步骤② 确定轴范围 → 步骤③ 需要标签尺寸 → 但标签尺寸取决于步骤④的刻度策略。

**评估**：✅ **这是一个真实的设计缺陷，需要在 DataProcessor 内部步骤顺序上修正。**

**根本原因**：旧版 linear 5-step 顺序确实存在逻辑矛盾——plot_area 需要知道标签文本宽度，但标签文本在刻度计算之后才产生。

**解决方案**：将步骤拆分为 6-step，引入"候选刻度"概念：
- 步骤② compute_axis_range()：确定轴数据范围
- **步骤③ generate_candidate_ticks()**：基于轴范围用标准算法（如 2-5-10 间隔序列）生成候选刻度值和标签文本，**不依赖像素尺寸**
- **步骤④ compute_plot_area()**：测量候选标签文本宽度 → 从 chart_bounds 中扣除 → 得到 plot_area
- **步骤⑤ refine_ticks()**：在 plot_area 内将候选刻度精确映射为像素坐标
- 步骤⑥ build_geometry()

**关键论证**：候选刻度文本的宽度是 boundable（已知字体大小 + 最大数字位数），单次测量足够，无需迭代。

**文档更新**：已更新 new-arch.md 中 DataProcessor 内部步骤为 6-step，并在 reasoning 区块增加了循环依赖的说明。

### 9.2 问题 2：多 Subplot 共用轴的范围协调

**评审意见**：
> 共用轴可能跨 subplot 引用，轴配置本身（min/max 用户指定）需优先于数据范围。建议在 GridPlanner 之后增加 AxisBindingResolver。

**评估**：✅ **这是一个真实的设计缺口，需要添加新层。** 当前文档只在"扩展性"中简略提及 `AxisRangeCoordinator`，但它在管线中的位置和输入/输出不明确。

**解决方案**：引入 `AxisBindingResolver` 预处理层，位于 GridPlanner 之后、DataProcessor 之前：
- 将 `xAxisIndex` / `yAxisIndex` 解析为具体轴配置
- 识别哪些 subplot 共用轴实例
- 对每个轴实例，收集所有关联 subplot 的数据范围，结合用户 min/max → 输出 `ResolvedAxisRange`
- DataProcessor 不再需要独立计算轴范围，直接使用解析结果

**文档更新**：
- new-arch.md 的管线图中增加了 `AxisBindingResolver` 层
- 增加了 `ResolvedAxisRange` 和 `ResolvedAxisRanges` 结构体定义
- DataProcessorInput 中增加了 `axis_ranges: &'a ResolvedAxisRanges` 字段
- 设计边界表中增加了 AxisBindingResolver 列

### 9.3 问题 3：颜色分配的位置

**评审意见**：
> 颜色分配在 DataProcessor 之外完成是正确的。但分面时同一个系列在不同分面中需保持相同颜色。建议增加 ColorAssigner 模块。

**评估**：✅ **建议合理，虽然不是当前阶段的硬性需求，但增加 ColorAssigner 可以使职责更清晰。** 当前文档只有 ColorContext 定义，没有明确说明颜色分配发生在哪个阶段。

**解决方案**：引入 `ColorAssigner` 预处理层：
- 读取主题调色板 + 系列 option.color
- 为每个 series 分配固定颜色
- 分面场景下确保同一系列跨分面颜色一致
- 输出 ColorContext

**文档更新**：
- new-arch.md 的管线图中增加了 `ColorAssigner` 层
- ColorContext 的说明中增加了 ColorAssigner 的职责描述
- 设计边界表中增加了 ColorAssigner 列

### 9.4 问题 4：全局标题/图例的位置计算

**评审意见**：
> 全局标题和图例的 top/left 等配置相对于整个画布，可能需要与 grid 区域避让。建议 GridPlanner 为预留区域做参数，或采用浮层方式。

**评估**：✅ **建议采用浮层方式。** 浮层方式与 ECharts 行为完全一致，实现更简单，不影响现有用户习惯。GridPlanner 无需为标题图例预留空间。

**文档更新**：new-arch.md 中 VisualElementBuilder 的说明增加了浮层方式的描述，设计边界表中增加了"浮层"标注。

### 9.5 问题 5：性能考量——文本测量开销

**评审意见**：
> 每个 DataProcessor 独立测量相同字体、相同文本，可能重复工作。建议引入 TextMeasurer 缓存。

**评估**：✅ **建议合理，但应作为优化而非基础设计。** 性能影响在当前规模下微乎其微，但引入缓存的成本很低，值得做。

**解决方案**：定义 `TextMeasurer` 结构体，内部维护 `HashMap<(text, font_size, rotation), (width, height)>`，在 ChartBuilder 级别创建并在所有 DataProcessor 之间共享。

**文档更新**：new-arch.md 中增加了 `TextMeasurer` 结构体定义和使用说明。

---

## 10. 评审总结

| 评审项 | 严重程度 | 处理结果 | 文档更新 |
|--------|:--------:|:--------:|:--------:|
| 文本测量与 plot area 循环依赖 | **高** | 修正步骤顺序，引入候选刻度概念 | new-arch.md DataProcessor 6-step |
| 共用轴范围协调 | **高** | 新增 AxisBindingResolver 层 | new-arch.md 管线图 + 类型定义 |
| 颜色分配位置 | 中 | 新增 ColorAssigner 层 | new-arch.md 管线图 |
| 标题/图例位置计算 | 中 | 采用浮层方式 | new-arch.md VisualElementBuilder 描述 |
| 文本测量性能开销 | 低 | 增加 TextMeasurer 缓存 | new-arch.md 结构体定义 |

**5 条评审意见全部采纳，均已更新至 new-arch.md 文档。** 其中 2 条高严重度问题（循环依赖、共用轴协调）属于设计缺陷，修正后架构更加健壮；3 条中低严重度问题属于补充完善，不影响核心方向。

## 11. 更新后的完整管线

```
ChartOption / ExternalData
         │
         ▼
  GridPlanner (纯数学分配)
  → Vec<SubplotSpec> (bounds + 索引绑定)
         │
         ▼
  AxisBindingResolver (预处理)
  → ResolvedAxisRanges (共用轴协调后的轴范围)
         │
         ▼
  ColorAssigner (预处理)
  → ColorContext (调色板 + 系列颜色分配)
         │
         ▼
  DataProcessor[0..N] (每个 subplot 独立执行)
  │ 内部 6 步骤:
  │ ① data_transform()
  │ ② compute_axis_range()
  │ ③ generate_candidate_ticks()
  │ ④ compute_plot_area()
  │ ⑤ refine_ticks()
  │ ⑥ build_geometry()
  → SubplotVisualData (Vec<VisualElement>)
         │
         ▼
  VisualElementBuilder (合并 + 排序 + 浮层)
  → Vec<VisualElement>
         │
         ▼
  Renderer (SVG / PNG，与现有相同)
```