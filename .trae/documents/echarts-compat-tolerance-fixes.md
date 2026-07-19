# ECharts JSON 兼容性容错修复

## Context

项目目标之一是支持 ECharts JSON 配置输入渲染图表（LLM 场景）。当前通过诊断脚本 `examples/diagnose_compat.rs` 测试常见 LLM 输出模式，发现 **24 个失败点**，分布在以下几类：

1. **未知 series 类型**（13 处）：`heatmap`/`funnel`/`treemap`/`sunburst`/`sankey`/`graph`/`tree`/`boxplot`/`effectScatter`/`pictorialBar`/`parallel`/`themeRiver`/`custom` —— `SeriesOption` 枚举无 `#[serde(other)]`，serde 直接报错。
2. **`SeriesEncodeOption` 仅接受 usize**：ECharts 5 推荐用列名（字符串）做 encode，如 `"encode": {"x":"product","y":"2015"}`。
3. **`ColorOption` 过于严格**：不认 `"auto"`、不接受渐变对象、不接受单值字符串赋给 `Vec<ColorOption>` 字段。
4. **多个字段单值/数组不兼容**：`AxisLineOption.symbol`、`AxisLabelOption.color`、`AxisLabelOption.interval`、`DataZoomOption.handle_size` 等。
5. **`LegendOption.data` 不接受对象数组**：ECharts 允许 `[{name, icon}, ...]`。
6. **`DataPoint` 边界**：不接受 `null` 元素、不接受 `{"value":[x,y]}` 数组 value。
7. **bool 字段不接受字符串**：LLM 偶发 `"animation":"true"`。

用户明确要求：「echarts 的 json 输入不要报错，尽可能绘制出它的声明的，像 tooltips 这些可以先不支持」。本计划目标：**让任意 LLM 输出的 ECharts JSON 都能解析成功并尽可能渲染**。

## Design Principles

- **解析容错优先**：未知字段/类型用 `#[serde(other)]` 或 `Option<serde_json::Value>` 兜底
- **类型扩展而非替换**：现有 `Option<usize>` 扩展为 `Option<StringOrInt>` 等枚举，保留语义
- **渐变/关键词降级**：渐变颜色取首个 colorStop；`"auto"` 颜色降级为语义合理的默认值
- **零回归**：现有 38 个测试必须全部通过；新增针对每个失败模式的回归测试

## Files to Modify

主战场：`d:\code\rust\liecharts\src\option.rs`（约 3332 行）
- `SeriesOption` 枚举（L1779）
- `SeriesEncodeOption`（L894）
- `ColorOption` 反序列化（L3434）
- `AxisLineOption`（L1420）、`AxisLabelOption`（L1344）、`AxisTickOption`（L1446）
- `LegendOption`（L1059）、`GridOption`（L1139）
- `DataZoomOption`（L494）、`DataPoint`（L2918）
- `ChartOption.color`（L945）
- `AnimationOption`（L359）等 bool 字段

下游适配：
- `d:\code\rust\liecharts\src\compat.rs`（L536 `option_series_to_spec` 全 match 需补 `_` / `Unknown` 臂）
- `d:\code\rust\liecharts\src\pipeline\compat.rs`（L506 `chart_option_to_chart_spec` 改 `filter_map` 跳过 Unknown）
- `d:\code\rust\liecharts\src\lib.rs`（导出新类型）

测试：
- `d:\code\rust\liecharts\tests\echarts_compat_test.rs`（追加容错回归测试）
- `d:\code\rust\liecharts\examples\diagnose_compat.rs`（保留作为可视化诊断工具，最后断言全部 `[OK]`）

## Implementation Steps

### Step 1: 新增通用容错类型（option.rs 顶部）

在 `SingleOrMultiple` 之后新增：

```rust
/// 接受字符串或 usize 的灵活类型（用于 encode.x/y 等）。
#[derive(Debug, Clone, PartialEq)]
pub enum StringOrInt {
    Str(String),
    Int(usize),
}
// 自定义 Serialize/Deserialize，接受 "product" / 0 / "0"
```

```rust
/// 接受数字或百分比字符串（用于 handle_size / size 等）。
#[derive(Debug, Clone, PartialEq)]
pub enum NumberOrPercent {
    Number(f64),
    Percent(f64), // 0~100
}
// "100%" → Percent(100.0); 50 → Number(50.0)
```

```rust
/// 容错 bool：接受 bool 或 "true"/"false" 字符串。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LenientBool(pub bool);
// Deserialize: bool | "true"/"false" | 1/0
```

```rust
/// 图例数据项：字符串或 {name, icon} 对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LegendDataItem {
    Str(String),
    Object { name: String, icon: Option<String> },
}
```

```rust
/// 区间值：数字或 "auto"（用于 axisLabel.interval）。
#[derive(Debug, Clone, PartialEq)]
pub enum IntervalOption {
    Auto,
    Fixed(f64),
}
```

### Step 2: SeriesOption 加 `#[serde(other)]` 兜底

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SeriesOption {
    // ... 已有 11 个 variants ...
    #[serde(other)]
    Unknown,
}
```

- `Unknown` 是单元 variant，匹配时下游 `compat.rs` / `pipeline/compat.rs` 用 `filter_map` 跳过该系列（不渲染但也不报错）。
- 两个 compat 文件中所有 `match s { ... }` 添加 `SeriesOption::Unknown => /* skip */` 臂。

### Step 3: SeriesEncodeOption 改为 StringOrInt

```rust
pub struct SeriesEncodeOption {
    pub x: Option<OneOrMany<StringOrInt>>,
    pub y: Option<OneOrMany<StringOrInt>>,
    // ... 其他字段同步 ...
}
```

下游 materializer 需根据 StringOrInt 解析列名/索引（dataset header 行查找；数字索引直接用）。dataset 头部解析在 `pipeline/compat.rs` 的 `resolve_series_data` 周边扩展。

### Step 4: ColorOption 扩展（关键）

`ColorOption::deserialize` 改造：
- 接受字符串：`"#RRGGBB"`、`"rgb()"`、`"rgba()"`、`"auto"`、`"transparent"`、`"inherit"`、`"none"`、CSS 关键字（`red`/`"blue"` 等常用色）
- 接受对象（渐变）：解析 `colorStops`，返回**首个非空 colorStop 的 color**
- 接受对象（无 `colorStops`）：返回黑色 sentinel `ColorOption::new(0, 0, 0)`，避免报错

关键词映射：
- `"auto"` → `ColorOption::new(0, 0, 0)`（后续可被 series color 覆盖，但解析不报错）
- `"transparent"` / `"none"` → `ColorOption::with_alpha(0, 0, 0, 0)`
- `"red"`/`"green"` 等 CSS 常用色 → 内置 16 色映射表

### Step 5: 单值/数组字段统一为 `OneOrMany`

- `ChartOption.color`: `Option<Vec<ColorOption>>` → `Option<OneOrMany<ColorOption>>`
- `AxisLineOption.symbol`: `Option<String>` → `Option<OneOrMany<String>>`
- `AxisLabelOption.color`: `Option<ColorOption>` → `Option<OneOrMany<ColorOption>>`
- `AxisLabelOption.background_color`、`border_color`、`shadow_color` 同步

### Step 6: NumberOrPercent / IntervalOption 字段替换

- `DataZoomOption.handle_size`: `Option<f64>` → `Option<NumberOrPercent>`
- `AxisLabelOption.interval`: `Option<f64>` → `Option<IntervalOption>`
- `AxisTickOption.interval`: `Option<f64>` → `Option<IntervalOption>`

下游 materializer 取值时用 `unwrap_or_default` 降级为合理默认（如 interval → Auto；handle_size → Number(100.0)）。

### Step 7: LegendOption.data 接受混合数组

```rust
pub struct LegendOption {
    pub data: Option<Vec<LegendDataItem>>,
    // ...
}
```

下游 materializer 在使用 legend.data 时，对 `LegendDataItem::Object` 提取 `name` 字段。

### Step 8: DataPoint 容错增强

修改 `DataPointVisitor`（option.rs L2997）：
- `visit_unit` / `visit_none`：返回 `DataPoint::Value(f64::NAN)`（NaN 在 line 图会断线，符合 ECharts `connect_nulls=false` 语义）
- `visit_map`：当 `value` 是数组 `[x, y]` → `DataPoint::XY(x, y)`；`[x, y, z]` → `DataPoint::XY(x, y)`（z 暂存于 series-level bubble 配置，先丢弃以避免报错）

### Step 9: LenientBool 应用到高风险 bool 字段

仅替换 LLM 最易误发的字段，避免过度改造：
- `LineSeriesOption` / `BarSeriesOption` 等：`animation`、`clip`、`silent`、`universalTransition.enabled`、`show_symbol`、`legend_hover_link`
- `TooltipOption.show`、`AxisOption.silent` 等

保留 `Option<bool>` 类型，但字段改用 `Option<LenientBool>`，下游 `unwrap_or(false)` 即可。注意 LenientBool 实现 `Deref` / `From<bool>` 便于迁移。

### Step 10: 顶层未知组件容错

当前 `ChartOption` 已通过 serde 默认行为静默忽略未知字段（已通过诊断确认 toolbox/polar/geo/calendar/graphic/aria/axisPointer 等都能 OK）。**无需修改**。

### Step 11: 回归测试

在 `tests/echarts_compat_test.rs` 新增 `mod tolerance_tests`，把诊断脚本中的 24 个失败案例作为断言用例：

```rust
#[test]
fn test_tolerates_unknown_series_types() {
    for ty in ["heatmap","funnel","treemap","sunburst","sankey","graph",
               "tree","boxplot","effectScatter","pictorialBar","parallel","themeRiver","custom"] {
        let json = format!(r#"{{"series":[{{"type":"{}","data":[]}}]}}"#, ty);
        let opt: ChartOption = serde_json::from_str(&json)
            .expect("should not error on unknown series type");
        assert!(matches!(opt.series[0], SeriesOption::Unknown));
    }
}

#[test]
fn test_tolerates_dataset_string_encode() { /* ... */ }
#[test]
fn test_tolerates_color_auto_and_gradient() { /* ... */ }
#[test]
fn test_tolerates_axis_line_symbol_array() { /* ... */ }
#[test]
fn test_tolerates_legend_data_objects() { /* ... */ }
#[test]
fn test_tolerates_datapoint_null_and_array_value() { /* ... */ }
#[test]
fn test_tolerates_string_bool_in_animation() { /* ... */ }
#[test]
fn test_tolerates_handle_size_percent() { /* ... */ }
#[test]
fn test_tolerates_axis_label_interval_auto() { /* ... */ }
#[test]
fn test_color_field_accepts_single_string() { /* ... */ }
```

### Step 12: 端到端渲染验证

把 `examples/diagnose_compat.rs` 改造为同时验证「能渲染」：对每个 OK 案例，调用 `ChartBuilder::from_option_json(json)?.build(800, 600)?.render_svg()` 并断言非空字符串（不报错即通过）。

按用户项目约束：跑 `cargo run --example <name>` 重新生成 SVG，对照 `docs/svg_chart_checklist.md` 视觉检查关键 case。

## Verification

```powershell
# 1. 编译
cargo build --release

# 2. 运行诊断：应全部 [OK]
cargo run --example diagnose_compat

# 3. 运行所有测试
cargo test

# 4. 重跑现有示例，对照 SVG checklist
cargo run --example json_config
cargo run --example json_bar_tests
cargo run --example json_line_tests

# 5. 视觉检查生成 SVG
# 浏览器打开 json_bar_*.svg / json_config.svg 等
```

成功标准：
- `cargo test` 全绿（原 38 + 新增 ~10 个容错测试）
- `diagnose_compat` 全部 `[OK]`
- 现有 23 个 `site/examples/*.json` 仍能正常解析与渲染
- 新增容错场景至少能解析（unknown series 跳过、其他尽量渲染）

## Out of Scope

- 不实现真实渐变渲染（取首色降级即可）
- 不实现 dataset 多表 source/transform（仅支持基本 source + encode）
- 不实现 tooltip 实际交互（保持现状：解析但忽略）
- 不新增 series 渲染器（heatmap/funnel 等仍跳过渲染）
