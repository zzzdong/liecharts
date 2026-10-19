# liecharts 示例

按**图表类型**组织，每个示例同时演示两种构建方式（配置一致，输出可直接对比）：

- `via_api()` —— Rust API 链式构建（`liecharts::api`：`Chart` + `Bar` / `Line` / …）
- `via_json()` —— ECharts option JSON（`ChartBuilder::from_option_json`）

每次 `common::save` 同时输出 **SVG 与 PNG** 两种格式，分别落入仓库根的：

- `svg_output/<name>.svg`
- `png_output/<name>.png`

## 运行

```bash
cargo run --example <name>     # 单个示例，如 cargo run --example bar
cargo run --example showcase   # 能力总览
```

## 示例索引

| 示例 | 图表类型 / 能力 | 场景 |
| --- | --- | --- |
| `bar` | 柱状图 | 分组柱状图（3 系列） |
| `line` | 折线图 | 平滑曲线 + 普通折线（2 系列） |
| `pie` | 饼图 | 五分类（radius 0–75%、标签） |
| `scatter` | 散点图 | XY 二维散点（2 系列） |
| `bubble` | 气泡图 | 编程语言生态（x/y/size/name） |
| `radar` | 雷达图 | 五维度双产品对比 |
| `gauge` | 仪表盘 | 完成率 75.5%（量程 0–100） |
| `heatmap` | 热力图 | 7 天 × 8 小时打卡网格 |
| `boxplot` | 箱线图 | 五分类（min/Q1/median/Q3/max） |
| `candlestick` | K 线图 | 9 个交易日 OHLC |
| `table` | 表格 | 产品销售数据（4 行 × 5 列） |
| `polar` | 极坐标 | 极坐标柱状图 + 极坐标散点图 |
| `combo` | 组合图表 | 双轴组合（柱+线）+ 多子图网格（3 grid） |
| `showcase` | 能力总览 | 暗色主题 / 值标签 / 函数曲线+LTTB 采样 / Hug 自适应 |

## 说明

- 仅提供 API 版本的示例及原因：
  - `bubble` / `table` —— ECharts option JSON 的 bubble / table 路径在兼容层
    （`pipeline::compat`）尚未实现（series 数据被置为空 DataFrame），
    待兼容层补齐后可补上 `via_json`。
  - `showcase` 的「函数曲线」—— 数据由 `DataFrame::from_function` 程序化生成，
    JSON 只能携带显式数据点；「Hug 自适应」—— `FitMode::Hug` 是渲染期布局
    策略，不属于 ECharts option 的描述范围。
- 两种构建方式的输出在版式上可能略有差异（SVG 由浏览器排版、PNG 由
  parley 排版，字体度量口径不同），属预期行为。
