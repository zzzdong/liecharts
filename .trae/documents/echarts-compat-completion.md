# ECharts 兼容性修复 - 收尾阶段

## Context

承接上次会话已批准的 `echarts-compat-tolerance-fixes.md` 计划。Steps 1-7、10 的类型层改造已完成：
- 已新增 5 个容错类型：`StringOrInt`、`NumberOrPercent`、`LenientBool`、`LegendDataItem`、`IntervalOption`
- 已为 `SeriesOption` 添加 `#[serde(other)] Unknown` 兜底
- 已将 `SeriesEncodeOption` 改用 `StringOrInt`
- 已扩展 `ColorOption` 反序列化（接受 `auto`/`transparent`/CSS 关键字/渐变对象）
- 已将 `ChartOption.color`、`AxisLineOption.symbol`、`AxisLabelOption.color` 等改为 `OneOrMany`
- 已将 `AxisLabelOption.interval`、`AxisTickOption.interval` 改为 `IntervalOption`
- 已将 `DataZoomOption.handle_size` 改为 `NumberOrPercent`
- 已将 `LegendOption.data` 改为 `Option<Vec<LegendDataItem>>`
- 已让 `compat.rs` / `pipeline/compat.rs` 用 `filter_map` 跳过 `Unknown` 系列

但是 `LegendOption.data` 的类型变更未同步到下游消费者，导致 `cargo check --lib` 报 6 个编译错误。同时原计划的 Steps 8、9、11、12 仍未完成。

用户核心诉求（不变）：「echarts 的 json 输入不要报错，尽可能绘制出它的声明的，像 tooltips 这些可以先不支持」。

## Current State Analysis

### `cargo check --lib` 当前 6 个错误（全是 `LegendDataItem` 类型变更引发）

1. `src/compat.rs:46` —— `data: Some(legend.data.clone())`
   - 源：`ChartSpec.legend.data` 类型为 `Vec<String>`
   - 目标：`LegendOption.data` 类型为 `Option<Vec<LegendDataItem>>`
   - 修复：将 `Vec<String>` 映射为 `Vec<LegendDataItem::Str>`

2. `src/compat.rs:488` —— `data: l.data.clone().unwrap_or_default()`
   - 源：`LegendOption.data: Option<Vec<LegendDataItem>>`
   - 目标：`LegendSpec.data: Vec<String>`
   - 修复：用 `.iter().map(|i| i.name().to_string()).collect()`

3. `src/pipeline/compat.rs:44` —— 同 #1

4. `src/pipeline/compat.rs:957` —— 同 #2

5. `src/pipeline/pipeline.rs:346` —— `create_text_layout(name, ...)`
   - `name` 类型为 `&LegendDataItem`，函数签名要求 `&str`
   - 修复：`create_text_layout(name.name(), ...)`

6. `src/pipeline/pipeline.rs:407` —— `text: name.clone()`
   - `name` 类型为 `&LegendDataItem`，目标字段要求 `String`
   - 修复：`text: name.name().to_string()`

### 已确认无需修复的位置

- `src/api/chart.rs:1048` 中 `data: l.data.clone()`：此处的 `self.legend` 是 `Option<LegendSpec>`（不是 `Option<LegendOption>`），所以 `l.data` 是 `Vec<String>`，源/目标类型一致，**无需改动**。

### DataPointVisitor 当前状态（位于 `src/option.rs:3338`）

当前 visitor 缺失：
- `visit_unit` / `visit_none`：不支持 `null` 数据点
- `visit_map` 中 `value: Option<f64>`：不支持 ECharts 中常见的 `{value: [x, y]}` 数组 value

### 已存在但未应用的类型

- `LenientBool` 类型定义完整，但尚未应用到任何字段

## Proposed Changes

### Phase A: 修复 6 个编译错误（BLOCKING — 必须最先完成）

#### A1. `src/compat.rs:46`
```rust
// 旧
data: Some(legend.data.clone()),
// 新
data: Some(
    legend
        .data
        .iter()
        .cloned()
        .map(crate::option::LegendDataItem::Str)
        .collect(),
),
```

#### A2. `src/compat.rs:488`
```rust
// 旧
data: l.data.clone().unwrap_or_default(),
// 新
data: l
    .data
    .clone()
    .unwrap_or_default()
    .iter()
    .map(|i| i.name().to_string())
    .collect(),
```

#### A3. `src/pipeline/compat.rs:44`
同 A1 改法。

#### A4. `src/pipeline/compat.rs:957`
同 A2 改法。

#### A5. `src/pipeline/pipeline.rs:346`
```rust
// 旧
let text_layout = create_text_layout(name, &text_style, None);
// 新
let text_layout = create_text_layout(name.name(), &text_style, None);
```

#### A6. `src/pipeline/pipeline.rs:407`
```rust
// 旧
text: name.clone(),
// 新
text: name.name().to_string(),
```

完成后跑 `cargo check --lib` 必须全绿才能进入 Phase B。

### Phase B: DataPoint 容错增强（Step 8）

修改 `src/option.rs` 中 `DataPointVisitor`（行 3338-3414）：

#### B1. 添加 `visit_unit` 和 `visit_none` 处理 `null`
```rust
fn visit_unit<E: de::Error>(self) -> Result<DataPoint, E> {
    // null 数据点 → NaN value（在 line 图中会断线，符合 ECharts connect_nulls=false 语义）
    Ok(DataPoint::Value(f64::NAN))
}

fn visit_none<E: de::Error>(self) -> Result<DataPoint, E> {
    Ok(DataPoint::Value(f64::NAN))
}
```

#### B2. 扩展 `visit_map` 处理 `value: [x, y]` 或 `[x, y, z]` 数组
将 `value: Option<f64>` 改为 `value: Option<serde_json::Value>`，然后根据 Value 类型分支处理：
- `Value::Number(n)` → 提取 f64 作为 value
- `Value::Array(arr)` → 第一个元素作为 x，第二个作为 y，其余忽略（兼容 `[x, y]` 和 `[x, y, z]`）
- `Value::Null` → value = NaN
- 其他 → 跳过

最终根据是否提取到 x 决定返回 `DataPoint::XY` 还是 `DataPoint::Named`/`DataPoint::Value`。

### Phase C: LenientBool 应用（Step 9）— 低优先级

仅在 Phase A、B 完成后且 cargo test 仍全绿时进行。

将以下高风险 bool 字段从 `Option<bool>` 改为 `Option<LenientBool>`：
- `LineSeriesOption.animation`、`clip`、`silent`、`show_symbol`、`legend_hover_link`
- `BarSeriesOption.animation`、`clip`、`silent`
- `PieSeriesOption.animation`、`silent`
- `AxisOption.silent`
- `TooltipOption.show`

下游消费处需要 `.map(|b| b.0)` 或 `.map(|b| b.into())` 转回 bool。

> 注意：此阶段属于「锦上添花」。如果时间紧张或出现回归，可暂缓 LenientBool 应用，仅保留类型定义。原计划的诉求是「JSON 输入不报错」，大多数 LLM 输出的 bool 字段都是合法 bool 值，不会触发字符串 bool 路径。

### Phase D: 新增容错回归测试（Step 11）

在 `tests/echarts_compat_test.rs` 末尾追加 `mod tolerance_tests`，至少包含以下测试：

```rust
mod tolerance_tests {
    use super::*;
    use crate::option::{LegendDataItem, SeriesOption};

    #[test]
    fn test_tolerates_unknown_series_types() {
        // 测试 13 种未知 series 类型都能解析为 Unknown
    }

    #[test]
    fn test_tolerates_dataset_string_encode() {
        // encode.x/y 用字符串列名
    }

    #[test]
    fn test_tolerates_color_auto_and_gradient() {
        // color: ["auto", {type:"linear", colorStops:[...]}, "red"]
    }

    #[test]
    fn test_tolerates_axis_line_symbol_array() {
        // axisLine.symbol: ["none", "arrow"]
    }

    #[test]
    fn test_tolerates_legend_data_objects() {
        // legend.data: [{name:"A", icon:"circle"}, "B"]
    }

    #[test]
    fn test_tolerates_datapoint_null_and_array_value() {
        // series.data: [1, null, 2, {value:[3, 4]}]
    }

    #[test]
    fn test_tolerates_handle_size_percent() {
        // dataZoom.handleSize: "100%"
    }

    #[test]
    fn test_tolerates_axis_label_interval_auto() {
        // axisLabel.interval: "auto"
    }

    #[test]
    fn test_color_field_accepts_single_string() {
        // color: "#c23531"（非数组）
    }

    #[test]
    fn test_unknown_top_level_fields_ignored() {
        // toolbox / polar / graphic / aria / axisPointer 等顶层字段被忽略
    }
}
```

每个测试都断言 `serde_json::from_str::<ChartOption>(json).is_ok()`，并对关键字段值做轻量断言（如 `matches!(opt.series[0], SeriesOption::Unknown)`）。

### Phase E: 端到端验证 + SVG 重生成（Step 12）

按用户项目硬约束：SVG 必须通过重跑 examples 生成并对照 `docs/svg_chart_checklist.md` 验证。

执行顺序：
1. `cargo check --lib` —— 全绿
2. `cargo check --tests --examples` —— 全绿
3. `cargo test` —— 原有 38 个 + 新增 ~10 个测试全过
4. `cargo run --example diagnose_compat` —— 所有 24 个失败案例转为 `[OK]`
5. 重跑现有示例生成 SVG：
   ```powershell
   cargo run --example json_config
   cargo run --example json_bar_tests
   cargo run --example json_line_tests
   cargo run --example json_pie_tests
   cargo run --example scatter_tests
   ```
6. 浏览器打开 `target/*.svg` 或 `site/examples/*.svg`，对照 `docs/svg_chart_checklist.md` 视觉检查关键 case

## Assumptions & Decisions

- **不修改 `LegendSpec.data` 类型**：保持 `Vec<String>`。`LegendSpec` 是内部管线类型，无需暴露 ECharts 的 `LegendDataItem` 复杂性。所有 ChartOption ↔ ChartSpec 转换处统一做 `LegendDataItem::name()` 提取。
- **DataPoint null 用 NaN 表达**：避免新增 `DataPoint::Null` variant 触发大量下游 match 改动。NaN 在 line 图中天然断线，符合 ECharts 语义。
- **`value: [x, y, z]` 中的 z 丢弃**：bubble 图通过 `BubbleDataPoint` 单独路径处理，普通 line/bar/scatter 的 DataPoint 不需要 z。降级不报错即可。
- **LenientBool 阶段可降级跳过**：如果出现回归或时间紧张，仅保留类型定义不应用字段，不影响主目标。
- **不实现新 series 渲染器**：heatmap/funnel/treemap 等仍跳过渲染，仅保证解析不报错。
- **tooltip 字段保持现状**：解析时已忽略（serde 默认行为），不实现交互。

## Verification

### 成功标准

1. `cargo check --lib` 无错误
2. `cargo test` 全绿（原 38 + 新增 ~10 = ~48 个测试）
3. `cargo run --example diagnose_compat` 输出全部 `[OK]`（原 24 个失败点全部转为成功）
4. 重跑 examples 生成的 SVG 与之前一致或更优（对照 `docs/svg_chart_checklist.md`）
5. 新增 10 个容错测试覆盖原 24 个失败模式

### 关键命令

```powershell
# 编译验证
cargo check --lib
cargo check --tests --examples

# 测试验证
cargo test

# 诊断验证（应全部 [OK]）
cargo run --example diagnose_compat

# SVG 重新生成与视觉验证
cargo run --example json_config
cargo run --example json_bar_tests
cargo run --example json_line_tests
cargo run --example json_pie_tests
cargo run --example scatter_tests
```

## Out of Scope

- 不修改 `LegendSpec.data` 类型（保持 `Vec<String>`）
- 不新增 series 渲染器（heatmap/funnel 等仍跳过）
- 不实现 tooltip 实际交互
- 不实现真实渐变渲染（取首色降级）
- 不实现 dataset 多表 source/transform（仅支持基本 source + encode）
- LenientBool 仅做类型定义和应用，不重构现有 bool 字段的下游消费逻辑（除非必要）
