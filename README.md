# LieCharts

基于 Rust 的可嵌入图表库，受 ECharts 启发，支持将图表渲染为 PNG 和 SVG 格式，并提供了 WebAssembly 构建用于浏览器环境。

## 特性

- **丰富的图表类型**：折线图、柱状图、饼图、面积图、散点图/气泡图、雷达图、仪表盘、K线图等
- **双渲染引擎**：支持 PNG（基于 `vello_cpu`）和 SVG 两种输出格式
- **主题系统**：内置 ECharts 6 设计令牌系统，提供 8 种内置主题（echarts、light、dark、vintage、macarons、infographic、shine、roma）
- **自定义主题**：支持自定义主题色、标题/图例/坐标轴样式
- **字体管理**：支持加载自定义字体（文件或内存字节），自动适配系统字体
- **JSON 配置**：可通过 JSON 或 Rust 结构体配置图表
- **WebAssembly**：可编译为 WASM 在浏览器中运行，附带在线编辑器
- **布局引擎**：支持多 Grid、混合图表、双 Y 轴等复杂布局

## 图表类型

| 类型 | 说明 | 状态 |
|------|------|------|
| line | 折线图 | ✅ |
| bar | 柱状图 | ✅ |
| pie | 饼图 | ✅ |
| area | 面积图 | ✅ |
| scatter | 散点图/气泡图 | ✅ |
| radar | 雷达图 | ✅ |
| gauge | 仪表盘 | ✅ |
| candlestick | K线图 | ✅ |
| mixed | 混合图（折线+柱状） | ✅ |
| stacked_area | 堆叠面积图 | ✅ |
| dual_y_axis | 双 Y 轴图 | ✅ |
| polar_bar | 极坐标柱状图 | ✅ |
| polar_scatter | 极坐标散点图 | ✅ |
| table | 表格 | ✅ |
| multi_grid | 多 Grid 组合 | ✅ |

## 快速开始

### 作为 Rust 库使用

```toml
[dependencies]
liecharts = { git = "https://github.com/zzzdong/liecharts" }
```

### 创建图表

```rust
use liecharts::{LieChart, LieChartOption, AxisType, DataPoint, SeriesOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chart = LieChart::new(800, 600);

    let option = LieChartOption {
        title: Some(liecharts::TitleOption {
            text: Some("月度趋势".to_string()),
            subtext: Some("2024年".to_string()),
            ..Default::default()
        }),
        legend: Some(liecharts::LegendOption {
            show: Some(true),
            data: Some(vec!["销售额".to_string()]),
            ..Default::default()
        }),
        x_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Category),
            data: Some(vec![
                "1月".to_string(), "2月".to_string(), "3月".to_string(),
                "4月".to_string(), "5月".to_string(), "6月".to_string(),
            ]),
            ..Default::default()
        }],
        y_axis: vec![liecharts::AxisOption {
            axis_type: Some(AxisType::Value),
            name: Some("销售额(万元)".to_string()),
            ..Default::default()
        }],
        series: vec![SeriesOption::Bar(liecharts::BarSeriesOption {
            name: Some("销售额".to_string()),
            data: vec![
                DataPoint::Number(120.0),
                DataPoint::Number(200.0),
                DataPoint::Number(150.0),
                DataPoint::Number(80.0),
                DataPoint::Number(70.0),
                DataPoint::Number(110.0),
            ],
            ..Default::default()
        })],
        ..Default::default()
    };

    // 渲染为 PNG
    chart.render_to_image(option, "bar.png")?;

    // 或渲染为 SVG
    // chart.render_to_svg(option, "bar.svg")?;

    Ok(())
}
```

### 使用主题

```rust
use liecharts::{LieChart, Theme};

// 使用内置主题
let chart = LieChart::new(800, 600)
    .with_theme(Theme::dark())
    .with_theme(Theme::vintage());

// 在图表配置中指定主题
let option = LieChartOption {
    theme: Some("dark".to_string()),
    // ... 其他配置
};
```

### 加载自定义字体

```rust
use liecharts::{FontSource, register_font};

// 从文件加载
register_font(FontSource::Path("path/to/font.ttf".into()), Some("MyFont"))?;

// 从内存加载
let bytes: Vec<u8> = // ... 从网络或其它来源获取
register_font(FontSource::Memory(bytes), Some("MyFont"))?;
```

### 渲染到内存

```rust
// 获取 PNG 字节
let png_data: Vec<u8> = chart.render_png(option)?;

// 获取 SVG 字符串
let svg: String = chart.render_svg(option)?;
```

## API 概览

### LieChart

图表实例，持有渲染上下文（宽高、主题注册表）。

| 方法 | 说明 |
|------|------|
| `new(width, height)` | 创建指定尺寸的图表实例 |
| `with_theme(theme)` | 注册主题并返回自身（链式调用） |
| `render_to_image(option, path)` | 渲染为 PNG 并保存到文件 |
| `render_to_svg(option, path)` | 渲染为 SVG 并保存到文件 |
| `render_png(option)` | 渲染为 PNG 并返回字节 |
| `render_svg(option)` | 渲染为 SVG 并返回字符串 |
| `collect_visual_elements(option)` | 收集视觉元素（高级用法） |

### LieChartOption

图表配置，通过 JSON 反序列化或 Rust 结构体构造。字段使用 camelCase 命名规则。

| 字段 | 类型 | 说明 |
|------|------|------|
| `title` | `TitleOption` | 标题配置 |
| `legend` | `LegendOption` | 图例配置 |
| `grid` | `Vec<GridOption>` | 网格布局（支持多 grid） |
| `x_axis` | `Vec<AxisOption>` | X 轴配置 |
| `y_axis` | `Vec<AxisOption>` | Y 轴配置 |
| `series` | `Vec<SeriesOption>` | 系列数据 |
| `color` | `Vec<ColorOption>` | 自定义调色板 |
| `background_color` | `ColorOption` | 背景色 |
| `theme` | `String` | 主题名称 |
| `text_style` | `TextStyleOption` | 全局文本样式 |

### SeriesOption

系列类型枚举，通过 JSON 的 `series[].type` 字段指定。

```json
{"type": "line", ...}
```

| 类型 | 枚举变体 | 说明 |
|------|----------|------|
| `line` | `SeriesOption::Line` | 折线图 |
| `bar` | `SeriesOption::Bar` | 柱状图 |
| `pie` | `SeriesOption::Pie` | 饼图 |
| `area` | `SeriesOption::Area` | 面积图 |
| `scatter` | `SeriesOption::Scatter` | 散点图 |
| `radar` | `SeriesOption::Radar` | 雷达图 |
| `gauge` | `SeriesOption::Gauge` | 仪表盘 |
| `candlestick` | `SeriesOption::Candlestick` | K线图 |
| `bubble` | `SeriesOption::Bubble` | 气泡图 |
| `polar_bar` | `SeriesOption::PolarBar` | 极坐标柱状图 |
| `polar_scatter` | `SeriesOption::PolarScatter` | 极坐标散点图 |
| `table` | `SeriesOption::Table` | 表格 |

### 内置主题

| 名称 | 说明 |
|------|------|
| `echarts` | ECharts 6 默认主题（设计令牌系统） |
| `light` | 浅色主题 |
| `dark` | 深色主题 |
| `vintage` | 复古风格 |
| `macarons` | 马卡龙风格 |
| `infographic` | 信息图风格 |
| `shine` | 闪耀风格 |
| `roma` | 罗马风格 |

## 使用 JSON 配置

LieCharts 支持从 JSON 读取配置，适合动态场景或 Web 编辑器：

```json
{
    "title": {
        "text": "月度趋势",
        "subtext": "2024年"
    },
    "xAxis": [{
        "type": "category",
        "data": ["1月", "2月", "3月", "4月", "5月", "6月"]
    }],
    "yAxis": [{
        "type": "value",
        "name": "销售额(万元)"
    }],
    "series": [{
        "type": "bar",
        "name": "销售额",
        "data": [120, 200, 150, 80, 70, 110]
    }],
    "theme": "dark"
}
```

## WebAssembly / Web 界面

LieCharts 可编译为 WASM 在浏览器中运行，并提供了在线图表编辑器。

### 构建

```bash
wasm-pack build site --target web --out-dir pkg
```

### 运行

```bash
cd site
python -m http.server 8000
# 或使用其它静态文件服务器
```

然后在浏览器打开 `http://localhost:8000`。

### 在线编辑器特性

- **Monaco 编辑器**：支持 JSON 语法高亮、自动补全、格式化
- **主题选择**：下拉选择内置主题，实时生效
- **图表类型切换**：一键切换折线图、柱状图、饼图等
- **SVG/PNG 导出**：支持下载为 SVG 或 PNG 格式（2x 高清）
- **键盘快捷键**：`Ctrl+Enter` 快速生成图表
- **暗色主题**：适配暗色模式的 UI 界面
- **错误提示**：实时 JSON 校验和错误定位

## 设计令牌系统

LieCharts 实现了 ECharts 6 的设计令牌（Design Token）系统，包含完整的色彩阶调：

- **中性色阶**：`neutral00` 到 `neutral99`（22 级）
- **强调色阶**：`accent05` 到 `accent95`（19 级）
- **语义色**：`success`、`warning`、`error`、`info`
- **文字系统**：字体族、字号层级、行高
- **间距系统**：`gap`、`padding`、`margin`
- **边框系统**：圆角、边框宽度
- **效果系统**：阴影层级

## 项目结构

```
liecharts/
├── Cargo.toml          # 主库配置
├── src/
│   ├── lib.rs          # 库入口 & 公开 API
│   ├── chart.rs        # LieChart 主结构体
│   ├── option.rs       # LieChartOption & 组件选项
│   ├── model.rs        # ResolvedOption & 解析逻辑
│   ├── theme.rs        # 主题 & 设计令牌系统
│   ├── text.rs         # 字体管理 & Parley 集成
│   ├── visual.rs       # 视觉元素定义
│   ├── error.rs        # 错误类型
│   ├── component/      # 图表组件（轴、图例、系列等）
│   ├── layout/         # 布局引擎
│   ├── pipeline/       # 数据管道
│   └── render/         # 渲染器（Pixmap & SVG）
├── site/               # WebAssembly 在线编辑器
│   ├── Cargo.toml
│   ├── index.html
│   ├── main.js
│   ├── style.css
│   ├── examples/       # JSON 示例配置
│   └── src/
│       └── lib.rs      # WASM 绑定
└── examples/           # Rust 示例程序
```

## 示例

运行 Rust 示例：

```bash
# 柱状图
cargo run --example bar

# 折线图
cargo run --example line

# 饼图
cargo run --example pie

# 面积图
cargo run --example area

# 雷达图
cargo run --example radar

# 仪表盘
cargo run --example gauge

# K线图
cargo run --example candlestick

# 散点图
cargo run --example scatter

# 极坐标柱状图
cargo run --example polar_bar

# 极坐标散点图
cargo run --example polar_scatter

# 混合图表
cargo run --example mixed

# 多 Grid 布局
cargo run --example multi_grid

# 双 Y 轴图
cargo run --example dual_axis

# 堆叠面积图
cargo run --example stacked_area

# 自定义主题
cargo run --example dark_theme

# JSON 配置
cargo run --example json_config

# 表格
cargo run --example table
```

## 依赖

- [vello_cpu](https://crates.io/crates/vello_cpu) - CPU 路径渲染引擎（PNG 输出）
- [parley](https://crates.io/crates/parley) - 文本布局和字体管理
- [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) - 序列化/反序列化
- [image](https://crates.io/crates/image) - 图像编码（PNG）
- [wasm-bindgen](https://crates.io/crates/wasm-bindgen) - WASM 绑定

## 许可证

MIT