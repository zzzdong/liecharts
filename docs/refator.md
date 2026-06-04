# TypedSeries 改造方案

## 1. 动机

当前管线中，数据从 `ChartSpec.series: Vec<SeriesSpec>` 到 `VisualElement` 的流转过度依赖 `DataFrame` 列式传递。处理器的 `transform()` / `to_visual_elements()` 阶段需要反复从 DataFrame 列中提取字段（color, px, py, group_position...），并通过 accessor 辅助结构间接访问。这导致：

- **渲染器内嵌大量字段提取逻辑**，而非纯渲染
- **类型信息丢失**：`SeriesSpec.config` 是 enum，渲染时仍需 match 解构
- **分组逻辑隐式**：`GroupAnalyzer` 通过 `stack` / `group_index` 字段隐式发现分组，再用 `GroupedBarProcessor.combine_to_dataframe()` 合并 DataFrame
- **测试困难**：渲染器依赖巨大的 `DataProcessorInput`，无法独立测试

### 目标

```
DataFrame (原始输入)
    ↓ Materialize 阶段
LineSeries / BarSeries / GroupedBarSeries / PieSeries / ...
    ↓ Build 阶段
VisualElement
```

- **类型化中间产物**：每种图形有自己的 Series 类型，字段已完全解析
- **Builder 纯构建**：接收 `&LineSeries` + `&RenderContext`，产生 `Vec<VisualElement>`
- **分组显式化**：`GroupedBarSeries` 作为一等类型
- **混合图形自然**：`Vec<TypedSeries>` 按声明顺序迭代渲染

---

## 2. 核心概念：TypedSeries 与"槽位"

### 槽位模型

坐标轴提供了**槽位**——即轴所代表的数据在画布上的具体位置。槽位来源于轴的类型和范围：

- **Value 轴**：数据值通过线性映射得到槽位（像素位置）。如 Y 轴范围 0~100，则值 50 落在像素中间。
- **Category 轴**：每个类别占据一个等分槽位。如 4 个类别 "A/B/C/D"，则每个类别槽宽 = 画布宽度/4。
- **Time/Log 轴**：各自有特定的槽位计算公式。

Materialize 阶段拿到 SubplotSpec 和 ResolvedAxisRanges 后，将 SeriesSpec 中的原始数据分配到对应的槽位上，生成 TypedSeries。TypedSeries 中的坐标已经是**像素空间**的——所有映射已经完成，不依赖任何外部上下文。

```
AxisSpec (声明式)                    ResolvedAxisRange (已解析范围)
    │                                      │
    ├─ Category: ["A","B","C","D"]        ├─ min: 0.0, max: 4.0 (类别索引)
    ├─ Value: 0~100                       ├─ min: 0.0, max: 100.0 (数据范围)
    │                                      │
    │         + SubplotSpec.bounds         │
    │         + DataFrame (原始数据)        │
    │              │                       │
    ▼              ▼                       ▼
        Materialize — 将数据分配到槽位
              │
              ▼
    TypedSeries { points: Vec<Point> (像素空间), color: Color, ... }
```

**关键原则**：TypedSeries 是 **self-contained** 的渲染数据。给定一个 TypedSeries，渲染器不需要再查找任何轴范围或计算任何坐标——直接画出即可。

### TypedSeries enum

```rust
/// 管线中间产物：所有字段已完全解析为像素空间和具体值，
/// 渲染器无需再做任何计算或字段提取。
enum TypedSeries {
    Line(LineSeries),
    Bar(BarSeries),
    GroupedBar(GroupedBarSeries),
    Scatter(ScatterSeries),
    Bubble(BubbleSeries),
    Candlestick(CandlestickSeries),
    Pie(PieSeries),
    Radar(RadarSeries),
    PolarBar(PolarBarSeries),
    PolarScatter(PolarScatterSeries),
    Gauge(GaugeSeries),
    Table(TableSeries),
}
```

### 与当前设计的关键区别

| | 当前 `SeriesSpec` | 新 `TypedSeries` |
|---|---|---|
| 层次 | 声明式规格（配置 + 原始数据） | 已解析的渲染数据（像素空间） |
| 数据载体 | `DataFrame`（列式） | 类型化字段（`points: Vec<Point>` 像素坐标） |
| 坐标空间 | 数据空间（0~100, 类别名等） | **像素空间**（画布上的 (x, y)） |
| 配置访问 | `match series.config { Line(cfg) => cfg.line_width }` | `s.line_width`（直接字段） |
| 颜色 | 从 `ColorContext` 查询 | 已解析为具体 `Color` |
| 分组 | 隐式（`stack` / `group_index` 字段） | 显式（`GroupedBarSeries` variant） |
| 渲染器输入 | `DataProcessorInput`（12 个字段） | `&LineSeries` + `&RenderContext`（仅主题装饰） |

---

## 3. TypedSeries Variant 定义

### 3.1 通用渲染上下文

```rust
/// 渲染器接收的上下文（仅装饰元素需要，系列数据本身已自包含）
struct RenderContext<'a> {
    /// 颜色上下文（仅用于轴/网格线/边框等装饰元素）
    pub colors: &'a ColorContext,

    /// 主题（文本样式等）
    pub theme: &'a Theme,
}
```

注意：`RenderContext` **不再包含 `bounds` 和 `axis_ranges`**——这些在 Materialize 阶段已经消费完毕，像素坐标直接写在 TypedSeries 的字段中。

### 3.2 LineSeries

```rust
struct LineSeries {
    pub name: String,

    // 样式（已解析）
    pub color: Color,
    pub line_width: f64,
    pub smooth: bool,
    pub area_color: Option<Color>,
    pub area_opacity: f64,
    pub symbol_type: SymbolType,
    pub symbol_size: f64,

    // 数据点（★ 像素空间坐标，无需再映射）
    pub points: Vec<Point>,
    // 面积填充的基线 Y（像素空间）
    pub baseline_y: f64,
}
```

### 3.3 BarSeries（单系列柱状图）

```rust
struct BarSeries {
    pub name: String,

    // 样式
    pub color: Color,

    // 数据点（★ 像素空间：每个条目已经算好了像素矩形）
    pub bars: Vec<BarRect>,
}

struct BarRect {
    pub rect: Rect,          // 像素空间的矩形
    pub category: String,   // 类别名（用于 label）
    pub value: f64,          // 原始值（用于 label）
}
```

### 3.4 GroupedBarSeries（并排/堆叠柱状图）

```rust
enum BarGroupType {
    SideBySide,
    Stacked,
}

struct GroupedBarSeries {
    /// 每个子系列的名称和颜色
    pub bars: Vec<BarSubSeries>,
    pub group_type: BarGroupType,

    // 数据（★ 像素空间）
    pub rows: Vec<GroupedBarRow>,
}

struct BarSubSeries {
    pub name: String,
    pub color: Color,
}

struct GroupedBarRow {
    pub bar_rect: Rect,         // 像素空间的矩形
    pub sub_series_idx: usize,  // 指向 bars 的索引
    pub color: Color,           // 子系列颜色
    pub category: String,       // 类别名（用于 label）
    pub value: f64,             // 原始值（用于 label）
}
```

### 3.5 ScatterSeries

```rust
struct ScatterSeries {
    pub name: String,
    pub color: Color,
    pub symbol_size: f64,
    pub points: Vec<Point>,  // ★ 像素空间
}
```

### 3.6 PieSeries

```rust
struct PieSeries {
    pub name: String,

    // 布局参数（已解析）
    pub center: (f64, f64),  // 百分比
    pub radius_inner: f64,   // 百分比
    pub radius_outer: f64,   // 百分比
    pub label_show: bool,
    pub label_position: LabelPosition,
    pub label_font_size: f64,

    // 扇区数据
    pub slices: Vec<PieSlice>,
}

struct PieSlice {
    pub name: String,
    pub value: f64,
    pub color: Color,
    pub percent: f64,  // 0.0~1.0
}
```

### 3.7 其他类型（概要）

```rust
struct BubbleSeries {
    pub name: String,
    pub color: Color,
    pub bubbles: Vec<Bubble>,  // 像素空间：center + radius + name
}

struct Bubble {
    pub center: Point,     // 像素空间中心
    pub radius: f64,       // 像素空间半径
    pub name: String,      // 气泡名（用于 label）
}

struct CandlestickSeries {
    pub name: String,
    pub up_color: Color,
    pub down_color: Color,
    pub candles: Vec<CandleRect>,
}

struct CandleRect {
    pub category: String,
    pub high_line: (Point, Point),   // 上影线端点（像素空间）
    pub low_line: (Point, Point),    // 下影线端点（像素空间）
    pub body_rect: Rect,             // 实体矩形（像素空间）
    pub is_up: bool,                 // 涨跌
}

struct RadarSeries {
    pub name: String,
    pub color: Color,
    pub indicators: Vec<String>,
    pub values: Vec<f64>,
}

struct PolarBarSeries {
    pub name: String,
    pub color: Color,
    pub pad_angle: f64,
    pub start_angle: f64,
    pub bars: Vec<PolarBarPoint>,  // angle, radius
}

struct PolarScatterSeries {
    pub name: String,
    pub color: Color,
    pub symbol_size: f64,
    pub points: Vec<PolarPoint>,
}

struct GaugeSeries {
    pub name: String,
    pub min: f64, pub max: f64,
    pub center: (f64, f64), pub radius: f64,
    pub start_angle: f64, pub end_angle: f64,
    pub split_number: usize,
    pub value: f64,
    pub color: Color,
}

struct TableSeries {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub header_bg: Color,
    pub row_even_bg: Color,
    pub row_odd_bg: Color,
}
```

### 3.8 坐标系归一与槽位分配

不同类型的 Series 数据表达方式不同（`LineSeries` 是数据空间坐标点，`BarSeries` 是类别+值），但它们最终都映射到同一像素坐标系。归一发生在 **Materialize 阶段**，而非 Render 阶段。

#### 槽位分配机制

Materialize 阶段接收三个输入：

```
SubplotSpec.bounds + ResolvedAxisRanges + SeriesSpec
                        │
                        ▼
              为每个数据点分配槽位 → 像素坐标
```

每种图表类型的 Materializer 计算槽位的方式不同：

```rust
// LineMaterializer — 数据空间线性映射到像素（Value × Value 轴）
fn materialize(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color) -> TypedSeries {
    let x_range = axis_ranges.get_x_range(spec.x_axis_index).unwrap();
    let y_range = axis_ranges.get_y_range(spec.y_axis_index).unwrap();

    let points: Vec<Point> = /* DataFrame 中的 (x, y) */ .map(|(x, y)| {
        Point::new(
            bounds.x0 + (x - x_range.min) / (x_range.max - x_range.min) * bounds.width(),
            bounds.y1 - (y - y_range.min) / (y_range.max - y_range.min) * bounds.height(),
        )
    }).collect();

    TypedSeries::Line(LineSeries { points, color, ... })
}

// BarMaterializer — 类别等分（Category × Value 轴）
fn materialize(spec: &SeriesSpec, bounds: Rect, axis_ranges: &ResolvedAxisRanges, color: Color) -> TypedSeries {
    let x_range = axis_ranges.get_x_range(spec.x_axis_index).unwrap();
    let y_range = axis_ranges.get_y_range(spec.y_axis_index).unwrap();
    let cat_count = (x_range.max - x_range.min).max(1.0);
    let bar_width_ratio = /* from BarConfig */;

    let bars: Vec<BarRect> = /* DataFrame 中的 (cat_idx, value) */ .map(|(cat_idx, value)| {
        let px = bounds.x0 + (cat_idx as f64 + 0.5) / cat_count * bounds.width();
        let py = bounds.y1 - (value - y_range.min) / (y_range.max - y_range.min) * bounds.height();
        let bar_w = bounds.width() / cat_count * bar_width_ratio;
        let baseline_y = bounds.y1 - (0.0 - y_range.min) / (y_range.max - y_range.min) * bounds.height();
        BarRect {
            rect: Rect::new(px - bar_w/2.0, py.min(baseline_y), px + bar_w/2.0, py.max(baseline_y)),
            ...
        }
    }).collect();

    TypedSeries::Bar(BarSeries { bars, color, ... })
}
```

#### 归一的核心：共享 bounds，各自选轴

不同类型的 Materializer 使用不同的映射公式，但都面向**同一个 `bounds`**。SeriesSpec 上的 `x_axis_index` / `y_axis_index` 在 Materialize 阶段消费后就消失了——TypedSeries 中不再需要这些索引：

```
系列 1: Line,  x_axis_index=0, y_axis_index=0   →   Materialize(用 x_range(0), y_range(0), bounds)
系列 2: Bar,   x_axis_index=0, y_axis_index=1   →   Materialize(用 x_range(0), y_range(1), bounds)
         ↑ 共用 X 轴                                        ↑ 各自选 Y 轴                ↑ 共用像素区域
```

TypedSeries 产生后，坐标已归一为同一像素空间，按声明顺序叠加即可。

```
TypedSeries::Line → points: [(100, 200), (300, 150), ...]   ← 像素
TypedSeries::Bar  → bars:  [{ rect: (95, 380, 145, 400) }, ...]  ← 像素
                           └─ 同一 bounds，不同 Y 映射 ─┘
```

> **注意**：非笛卡尔坐标系（Pie, Radar, Gauge, Polar*）不经过轴槽位分配。它们使用自己的布局参数（center, radius, angle 等），在 Materialize 阶段直接计算出像素坐标或保持百分比（由具体类型的 Builder 内部处理）。

---

## 4. 数据流

```
ChartSpec  (声明式规格，不变)
  │  series: Vec<SeriesSpec>
  │  grids, x_axes, y_axes, title, legend, ...
  │
  ▼  Materialize 阶段
  │
  ├─ 1. GridPlanner::plan()
  │     → Vec<SubplotSpec>
  │
  ├─ 2. AxisBindingResolver::resolve()
  │     → ResolvedAxisRanges
  │
  ├─ 3. ColorAssigner::assign()
  │     → ColorContext
  │
  ├─ 4. SeriesMaterializer::materialize()  ★ 新增
  │     对每个 subplot，遍历其 series:
  │       - 提取 DataFrame 数据列
  │       - 解析颜色（from ColorContext）
  │       - ★ 将数据点分配到轴槽位 → 像素坐标
  │       - Bar 系列先收集，最后做分组分析
  │     → Vec<TypedSeries>  (所有坐标已是像素空间，保持原始声明顺序)
  │
  ▼  Build 阶段（视觉元素构建）
  │
  ├─ 5. 背景绘制
  ├─ 6. AxisRenderer::render()  (不变)
  ├─ 7. for typed in &series:
  │       match typed {
  │           TypedSeries::Line(s)  => LineBuilder::build(s, &ctx),
  │           TypedSeries::Bar(s)   => BarBuilder::build(s, &ctx),
  │           TypedSeries::GroupedBar(s) => GroupedBarBuilder::build(s, &ctx),
  │           ...
  │       }
  │     → Vec<VisualElement>
  │     ★ 所有坐标映射已在 Materialize 阶段完成，Builder 只做组装
  ├─ 8. 标题/图例/轴名称渲染 (不变)
  │
  ▼
Vec<VisualElement>
```

### 关键设计决策：坐标映射的位置

坐标映射（data-space → pixel-space）放在 **Materialize 阶段**，而非 Build 阶段：

- Materialize 接收 `SubplotSpec.bounds` + `ResolvedAxisRanges` + `SeriesSpec`
- 为每个数据点分配槽位，计算像素坐标，存入 TypedSeries
- Builder 接收已含有像素坐标的 TypedSeries，直接组装为 VisualElement

理由：Materialize 阶段已经拥有 SubplotSpec 和 AxisRanges（从 GridPlanner 和 AxisBindingResolver 获得），具备映射所需的全部信息。将映射前移，Builder 变成纯组装逻辑。

---

## 5. Materialize 阶段详细设计

### 5.1 SeriesMaterializer trait

```rust
/// 每种图表类型实现此 trait，将 SeriesSpec 转换为对应的 TypedSeries
trait SeriesMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,                     // ★ 子图像素边界
        axis_ranges: &ResolvedAxisRanges,  // ★ 轴范围（用于槽位分配）
        color: Color,
        colors: &ColorContext,
    ) -> Result<TypedSeries>;
}

fn create_materializer(chart_type: ChartType) -> Box<dyn SeriesMaterializer>;

// 示例：LineMaterializer::materialize()
impl SeriesMaterializer for LineMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,
        axis_ranges: &ResolvedAxisRanges,
        color: Color,
        _: &ColorContext,
    ) -> Result<TypedSeries> {
        let cfg = match &spec.config {
            SeriesConfig::Line(c) => c,
            _ => return Err(...),
        };

        let x_vals = spec.data.get_column(&cfg.x_col).ok_or(...)?;
        let y_vals = spec.data.get_column(&cfg.y_col).ok_or(...)?;

        let x_range = axis_ranges.get_x_range(spec.x_axis_index).unwrap();
        let y_range = axis_ranges.get_y_range(spec.y_axis_index).unwrap();

        // ★ 在 Materialize 阶段完成数据空间 → 像素空间的映射
        let points: Vec<Point> = (0..spec.data.row_count())
            .filter_map(|i| {
                let x = x_vals.as_f64(i)?;
                let y = y_vals.as_f64(i)?;
                Some(Point::new(
                    bounds.x0 + (x - x_range.min) / (x_range.max - x_range.min) * bounds.width(),
                    bounds.y1 - (y - y_range.min) / (y_range.max - y_range.min) * bounds.height(),
                ))
            })
            .collect();

        Ok(TypedSeries::Line(LineSeries {
            name: spec.name.clone(),
            color,
            line_width: cfg.line_width,
            smooth: cfg.smooth,
            area_color: cfg.area_color,
            area_opacity: cfg.area_opacity,
            symbol_type: cfg.symbol_type,
            symbol_size: cfg.symbol_size,
            points,
            baseline_y: bounds.y1, // 面积填充基线 = 子图底部
        }))
    }
}
```

### 5.2 Bar 分组处理

Materialize 阶段对 Bar 系列做两遍处理：

1. **第一遍**：收集所有 Bar 类型的 SeriesSpec，暂不生成 TypedSeries
2. **分组分析**：复用当前 `GroupAnalyzer` 的逻辑，识别 SideBySide / Stacked 组
3. **第二遍**：
   - Single Bar → `TypedSeries::Bar(BarSeries)`
   - SideBySide / Stacked 组 → `TypedSeries::GroupedBar(GroupedBarSeries)`

非 Bar 类型在第一遍直接生成 TypedSeries。

```rust
fn materialize_all(
    series_indices: &[usize],       // 当前 subplot 内的 series 索引
    spec: &ChartSpec,
    bounds: Rect,                    // ★ 子图像素边界
    axis_ranges: &ResolvedAxisRanges,// ★ 轴范围
    colors: &ColorContext,
) -> Result<Vec<TypedSeries>> {
    let mut result: Vec<(usize, TypedSeries)> = Vec::new();
    let mut bar_specs: Vec<(usize, &SeriesSpec)> = Vec::new();

    for &global_idx in series_indices {
        let s = &spec.series[global_idx];
        match s.config.chart_type() {
            ChartType::Bar => {
                bar_specs.push((global_idx, s));
            }
            chart_type => {
                let color = colors.get_series_color(global_idx);
                let materializer = create_materializer(chart_type);
                let typed = materializer.materialize(s, bounds, axis_ranges, color, colors)?;
                result.push((global_idx, typed));
            }
        }
    }

    // 对 Bar 系列做分组分析
    if !bar_specs.is_empty() {
        let bar_plans = analyze_bar_groups(&bar_specs, &spec.series);
        for plan in bar_plans {
            let typed = materialize_bar_group(&plan, &spec.series, colors)?;
            result.push((plan.first_index, typed));
        }
    }

    // 按原始索引排序，保持声明顺序
    result.sort_by_key(|(idx, _)| *idx);
    Ok(result.into_iter().map(|(_, s)| s).collect())
}
```

---

## 6. VisualElement Builder 层设计

Builder 只做一件事：**将 TypedSeries 的已解析字段组装为 VisualElement**。不包含任何坐标映射、颜色解析或字段提取。命名上它更接近"构建器"而非"渲染器"——真正的渲染（SVG / PNG 输出）在下游的 `src/visual.rs` 完成。

### 6.1 SeriesBuilder trait

```rust
/// 每种 TypedSeries variant 有对应的 VisualElement 构建器
trait SeriesBuilder<T> {
    fn build(series: &T, ctx: &RenderContext) -> Result<Vec<VisualElement>>;
}
```

### 6.2 LineBuilder 示例

```rust
struct LineBuilder;

impl SeriesBuilder<LineSeries> for LineBuilder {
    fn build(series: &LineSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::new();

        if series.points.len() < 2 {
            return Ok(elements);
        }

        // 1. 面积填充（使用已解析的像素基线）
        if let Some(area_color) = &series.area_color {
            let alpha = (255.0 * series.area_opacity).clamp(0.0, 255.0) as u8;
            let mut fill = *area_color;
            fill.a = alpha;
            let area_path = build_area_path(&series.points, series.baseline_y);
            elements.push(VisualElement::Path {
                path: area_path,
                style: FillStrokeStyle { fill: Some(fill), stroke: None },
                z_index: Z_SERIES_FILL,
            });
        }

        // 2. 线条
        let path = if series.smooth {
            build_smooth_path(&series.points)
        } else {
            build_polyline_path(&series.points)
        };
        elements.push(VisualElement::Path {
            path,
            style: FillStrokeStyle {
                fill: None,
                stroke: Some(Stroke { color: series.color, width: series.line_width }),
            },
            z_index: Z_SERIES_LINE,
        });

        // 3. 标记点
        if series.symbol_type != SymbolType::None {
            for pt in &series.points {
                elements.push(build_symbol(pt, series.symbol_type, series.symbol_size, series.color));
            }
        }

        Ok(elements)
    }
}
```

### 6.3 BarBuilder 示例

```rust
struct BarBuilder;

impl SeriesBuilder<BarSeries> for BarBuilder {
    fn build(series: &BarSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::new();

        for bar in &series.bars {
            // ★ 直接使用像素矩形，无需任何计算
            elements.push(VisualElement::Rect {
                rect: bar.rect,
                style: FillStrokeStyle {
                    fill: Some(series.color),
                    stroke: Some(Stroke { color: ctx.colors.border_color, width: 1.0 }),
                },
                z_index: Z_SERIES_FILL,
            });
        }

        Ok(elements)
    }
}
```

### 6.4 GroupedBarBuilder 示例

```rust
struct GroupedBarBuilder;

impl SeriesBuilder<GroupedBarSeries> for GroupedBarBuilder {
    fn build(series: &GroupedBarSeries, ctx: &RenderContext) -> Result<Vec<VisualElement>> {
        let mut elements = Vec::new();

        for row in &series.rows {
            // ★ 直接使用像素矩形，颜色也已解析
            elements.push(VisualElement::Rect {
                rect: row.bar_rect,
                style: FillStrokeStyle {
                    fill: Some(row.color),
                    stroke: Some(Stroke { color: ctx.colors.border_color, width: 1.0 }),
                },
                z_index: Z_SERIES_FILL,
            });
        }

        Ok(elements)
    }
}
```

### 6.5 构建分发

```rust
fn build_visual_elements(
    series: &TypedSeries,
    ctx: &RenderContext,
) -> Result<Vec<VisualElement>> {
    match series {
        TypedSeries::Line(s)       => LineBuilder::build(s, ctx),
        TypedSeries::Bar(s)        => BarBuilder::build(s, ctx),
        TypedSeries::GroupedBar(s) => GroupedBarBuilder::build(s, ctx),
        TypedSeries::Scatter(s)    => ScatterBuilder::build(s, ctx),
        TypedSeries::Bubble(s)     => BubbleBuilder::build(s, ctx),
        TypedSeries::Candlestick(s)=> CandlestickBuilder::build(s, ctx),
        TypedSeries::Pie(s)        => PieBuilder::build(s, ctx),
        TypedSeries::Radar(s)      => RadarBuilder::build(s, ctx),
        TypedSeries::PolarBar(s)   => PolarBarBuilder::build(s, ctx),
        TypedSeries::PolarScatter(s)=> PolarScatterBuilder::build(s, ctx),
        TypedSeries::Gauge(s)      => GaugeBuilder::build(s, ctx),
        TypedSeries::Table(s)      => TableBuilder::build(s, ctx),
    }
}
```

这个 match 只是一层薄路由（thin dispatch），不包含任何字段提取或数据解析。

---

## 7. 混合图形处理

混合图形（同一 chart 内同时有 Line + Bar）在新设计中**自然支持**，无需特殊处理：

```
ChartSpec.series = [
    SeriesSpec { config: Line(...), data: df1 },   // 折线 1
    SeriesSpec { config: Bar(...),  data: df2 },   // 柱状图 1
    SeriesSpec { config: Line(...), data: df3 },   // 折线 2
    SeriesSpec { config: Bar(...),  data: df4 },   // 柱状图 2 (同 grid 内，同 group_index)
]
        ↓ Materialize
Vec<TypedSeries> = [
    TypedSeries::Line(line1),          // 位置 0
    TypedSeries::GroupedBar(bar_group),// 位置 1,3 合并
    TypedSeries::Line(line2),          // 位置 2
]
        ↓ Render (按声明顺序)
  1. LineBuilder::build(line1) → Line 的视觉元素（z_index 较低）
  2. GroupedBarBuilder::build(bar_group) → Bar 的视觉元素（覆盖在 Line 上）
  3. LineBuilder::build(line2) → Line 的视觉元素（在最上层）
```

**渲染顺序即 z-order**：TypedSeries 保持 ChartSpec 中的声明顺序，先声明的先渲染（在底层），后声明的后渲染（在上层）。这符合用户直觉。

**轴绑定**：每个 TypedSeries 自带 `y_axis_index`，双 Y 轴场景下 Line 绑定左轴、Bar 绑定右轴，各自使用对应的 `AxisRange` 进行坐标映射。

---

## 8. 文件结构

```
src/pipeline/
├── types.rs                  # ChartSpec, SeriesSpec 等（保留）
├── typed_series.rs           # ★ 新增：TypedSeries enum + 所有 variant 定义
├── materializer/
│   ├── mod.rs                # SeriesMaterializer trait + 工厂函数 + materialize_all()
│   ├── line.rs               # LineMaterializer
│   ├── bar.rs                # BarMaterializer + BarGroupMaterializer
│   ├── scatter.rs
│   ├── bubble.rs
│   ├── candlestick.rs
│   ├── pie.rs
│   ├── radar.rs
│   ├── polar_bar.rs
│   ├── polar_scatter.rs
│   ├── gauge.rs
│   └── table.rs
├── builder/
│   ├── mod.rs                # SeriesBuilder trait + build_visual_elements() 分发
│   ├── line.rs               # LineBuilder
│   ├── bar.rs                # BarBuilder
│   ├── grouped_bar.rs        # GroupedBarBuilder
│   ├── scatter.rs
│   ├── bubble.rs
│   ├── candlestick.rs
│   ├── pie.rs
│   ├── radar.rs
│   ├── polar_bar.rs
│   ├── polar_scatter.rs
│   ├── gauge.rs
│   └── table.rs
├── pipeline.rs               # 管线编排（简化：materialize → build → collect）
├── compat.rs                 # ChartSpec ↔ ChartOption 互转（保留）
├── data_processor.rs         # 保留过渡期，最终删除或改为内部辅助
├── processor/                # 旧处理器目录，逐步迁移后删除
├── group/                    # GroupAnalyzer 逻辑合并到 materializer/bar.rs
├── mapper/                   # CoordinateMapper 保留，供 materializer 内部使用
├── accessors/                # CartesianGeometry, GroupInfo, StyleAccess → 不再需要
├── dataframe.rs              # DataFrame（保留，作为用户输入）
└── ...
```

---

## 9. 改造计划

### Phase 1：定义 TypedSeries 类型（低风险，无行为变更）

**目标**：在 `src/pipeline/typed_series.rs` 中定义所有 TypedSeries variant 和 `RenderContext`。

**内容**：
- 定义 `TypedSeries` enum（12 个 variant）
- 定义 `LineSeries`, `BarSeries`, `GroupedBarSeries` 等结构体
- 定义 `RenderContext`
- 定义 `Point2D` 等辅助类型
- 此阶段**不修改任何现有代码**，纯新增文件

**检查点**：编译通过

---

### Phase 2：实现 Materializer（新能力，并行开发）

**目标**：实现 `SeriesMaterializer` trait 及所有具体类型。

**内容**：
- 创建 `materializer/` 目录
- 实现 `LineMaterializer::materialize()` — 从 `SeriesSpec` 产生 `LineSeries`
- 实现 `BarMaterializer::materialize()` — Single Bar
- 实现 `materialize_all()` — 整体编排 + Bar 分组
- 实现 `GroupedBarSeries` 的构建逻辑（从 `GroupedBarProcessor.combine_to_dataframe()` 迁移）
- 实现其余 materializer（Scatter, Pie, Bubble, Candlestick, Radar, Polar*, Gauge, Table）

**检查点**：每个 materializer 有独立的单元测试（给定 SeriesSpec，验证产出 TypedSeries 的字段）

---

### Phase 3：实现 Builder（新能力，并行开发）

**目标**：实现 `SeriesBuilder` trait 及所有 VisualElement 构建器。

**内容**：
- 创建 `builder/` 目录
- 实现 `LineBuilder::build()` — 将 Line 处理器中的 `to_visual_elements()` 逻辑迁移，改为接收 `&LineSeries` + `&RenderContext`
- 实现 `BarBuilder::build()` — 同上
- 实现 `GroupedBarBuilder::build()` — 同上
- 实现 `build_visual_elements()` 分发函数
- 实现其余构建器

**当前处理器中的逻辑来源**：
- `processor/line.rs::to_visual_elements()` → `builder/line.rs`
- `processor/bar.rs::to_visual_elements()` → `builder/bar.rs` + `builder/grouped_bar.rs`
- `processor/scatter.rs::to_visual_elements()` → `builder/scatter.rs`
- 等等

**注意**：此阶段 Builder 仍可独立测试，但尚未接入管线。

**检查点**：每个 builder 有独立的单元测试（给定 TypedSeries + RenderContext，验证产出 VisualElement）

---

### Phase 4：接入管线（核心集成）

**目标**：修改 `pipeline.rs` 中的 `build_chart_internal()`，使用新数据流。

**内容**：
1. 在 `pipeline.rs` 中新增 `materialize_and_build()` 函数
2. 将其接入 `build_chart_internal()`，替代旧的处理器调用路径
3. 添加集成测试（从 ChartSpec 到 VisualElement 端到端）

**管线变更**：
```
旧：build_chart_internal()
  ├─ GridPlanner, AxisBindingResolver, ColorAssigner  (保留)
  ├─ 4. 背景绘制                                          (保留)
  ├─ 5. AxisRenderer                                      (保留)
  ├─ 6. for plan in GroupAnalyzer::analyze():             (★ 替换)
  │       processor.process_from_spec() / process_dataframe()
  ├─ 7. 标题/图例/轴名称                                   (保留)
  
新：build_chart_internal()
  ├─ GridPlanner, AxisBindingResolver, ColorAssigner  (保留)
  ├─ 4. 背景绘制                                          (保留)
  ├─ 5. AxisRenderer                                      (保留)
  ├─ 6. typed_series = materialize_all(subplot, spec, bounds, axis_ranges, &colors)  (★ 新增)
  │    for ts in &typed_series:
  │        build_visual_elements(ts, &build_ctx)
  ├─ 7. 标题/图例/轴名称                                   (保留)
```

**检查点**：所有现有集成测试通过

---

### Phase 5：清理旧代码

**目标**：删除或归档被替代的旧代码。

**内容**：
1. 删除 `processor/` 目录中的 `to_dataframe()` 和旧的 `to_visual_elements()` 逻辑
2. 如果 `compat.rs` 的 `chart_spec_to_chart_option` 不再被管线使用，标记 deprecated
3. 删除 `accessors/` 目录（CartesianGeometry, GroupInfo, StyleAccess 不再需要）
4. `DataProcessorInput` 标记 deprecated，逐步移除
5. `DataProcessor` trait → 仅保留 `process()` 用于旧 API 兼容，标记 deprecated

---

### Phase 6：删除 option.rs（P3，远期目标）

**目标**：完全删除 `src/option.rs`，所有路径使用 ChartSpec + TypedSeries。

此项在 spec-model.md 中已列为 P3 优先级的远期任务。

---

## 10. 迁移与兼容策略

### 新旧并行期

在 Phase 4 集成期间，旧处理器路径和新 TypedSeries 路径可以共存：

```rust
fn build_chart_internal(spec: &ChartSpec, ...) -> Result<Vec<VisualElement>> {
    // ...
    if USE_TYPED_SERIES {
        materialize_and_build(spec, ...)
    } else {
        old_processor_path(spec, ...)
    }
}
```

### 风险控制

| 风险 | 缓解措施 |
|------|----------|
| 渲染结果不一致 | Phase 2/3 的单元测试覆盖每个 materializer + renderer 组合 |
| 性能回退 | TypedSeries 减少运行时 match 次数，预期性能不降 |
| 混合图形回归 | Phase 4 集成测试覆盖 Line+Bar 混合场景 |
| 分组逻辑遗漏 | Bar 分组逻辑从 GroupAnalyzer 直接迁移，不重写 |

### 不改变的部分

- `ChartSpec` / `SeriesSpec` / `SeriesConfig` — 声明式规格不变
- `api/` 层（Chart builder, LayerSpec）— 不变
- `GridPlanner`, `AxisBindingResolver`, `ColorAssigner` — 不变
- `AxisRenderer` — 不变
- 标题/图例/轴名称渲染 — 不变
- `DataFrame` — 保留作为用户数据输入
- `CoordinateMapper` (Cartesian, Polar, Noop) — Materializer 内部使用，不再暴露给 Builder

---

## 11. 与 spec-model.md 的关系

| 文档 | 关注点 |
|------|--------|
| `spec-model.md` | **声明式规格层**：`ChartSpec`, `SeriesSpec`, `SeriesConfig` 等 |
| `refator.md` (本文档) | **管线中间层**：`TypedSeries`, `Materializer`, `Builder` |

两者互补：
- `ChartSpec` 是管线的输入（用户可构造、可序列化）
- `TypedSeries` 是管线的中间产物（内部类型，渲染专用）
- `ChartSpec` → Materialize → `TypedSeries` → Render → `VisualElement`
