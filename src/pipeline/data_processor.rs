use vello_cpu::kurbo::Rect;

/// 基于 DataFrame 的新 DataProcessor trait
///
/// 设计原则：
/// 1. 每个 Processor 负责将 SeriesOption 转换为 DataFrame
/// 2. 使用 Transformer 对 DataFrame 进行转换（添加计算列）
/// 3. 使用 CoordinateMapper 将数据坐标映射为像素坐标
/// 4. 从 DataFrame 生成 VisualElement
/// 5. 不直接操作原始数据，所有操作通过 DataFrame 进行
use crate::error::Result;
use crate::{
    option::{ChartOption, SeriesOption},
    pipeline::{
        dataframe::DataFrame,
        mapper::CoordinateMapper,
        types::{ColorContext, ResolvedAxisRanges, SubplotSpec},
    },
    visual::VisualElement,
};

/// DataProcessorV2 的输入
pub struct DataProcessorInput<'a> {
    pub spec: &'a SubplotSpec,
    pub option: &'a ChartOption,
    pub colors: &'a ColorContext,
    pub axis_ranges: &'a ResolvedAxisRanges,
    pub bounds: Rect,
    pub series_idx: usize, // 当前正在处理的系列索引
}

/// DataProcessorV2 trait
///
/// 每个图表类型实现此 trait，完成从数据到视觉元素的转换
pub trait DataProcessor {
    /// 将 SeriesOption 转换为原始 DataFrame
    fn to_dataframe(&self, series: &SeriesOption, input: &DataProcessorInput) -> Result<DataFrame>;

    /// 转换 DataFrame（添加计算列如 color, percent, position 等）
    fn transform(&self, df: DataFrame, input: &DataProcessorInput) -> Result<DataFrame>;

    /// 从 DataFrame 生成 VisualElement
    fn to_visual_elements(
        &self,
        df: &DataFrame,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>>;

    /// 解析当前 series 使用的 X 轴索引（可被子类覆写）
    fn resolve_x_axis_idx(&self, _series: &SeriesOption, input: &DataProcessorInput) -> usize {
        input.spec.x_axis_indices.first().copied().unwrap_or(0)
    }

    /// 解析当前 series 使用的 Y 轴索引（可被子类覆写）
    fn resolve_y_axis_idx(&self, _series: &SeriesOption, input: &DataProcessorInput) -> usize {
        input.spec.y_axis_indices.first().copied().unwrap_or(0)
    }

    /// 返回坐标映射器（默认 NoopMapper）
    fn mapper(&self) -> Box<dyn CoordinateMapper> {
        Box::new(super::mapper::noop::NoopMapper)
    }

    /// 处理已组合好的 DataFrame（跳过 to_dataframe 阶段）
    /// 用于 GroupProcessor 模式：GroupAnalyzer 已将所有 series 展开为 DataFrame 行
    fn process_dataframe(
        &self,
        df: DataFrame,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let mut df = self.transform(df, input)?;
        let x_axis_idx = input.spec.x_axis_indices.first().copied().unwrap_or(0);
        let y_axis_idx = input.spec.y_axis_indices.first().copied().unwrap_or(0);
        self.mapper()
            .map_coordinates(&mut df, input, x_axis_idx, y_axis_idx);
        self.to_visual_elements(&df, input)
    }

    /// 完整的处理流程（单 series 模式）
    fn process(
        &self,
        series: &SeriesOption,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let df = self.to_dataframe(series, input)?;
        let mut df = self.transform(df, input)?;
        let x_axis_idx = self.resolve_x_axis_idx(series, input);
        let y_axis_idx = self.resolve_y_axis_idx(series, input);
        self.mapper()
            .map_coordinates(&mut df, input, x_axis_idx, y_axis_idx);
        self.to_visual_elements(&df, input)
    }
}

/// 创建对应类型的 DataProcessorV2
pub fn create_processor(series: &SeriesOption) -> Box<dyn DataProcessor> {
    match series {
        SeriesOption::Pie(_) => Box::new(super::processor::pie::PieProcessor::new()),
        SeriesOption::Line(_) => Box::new(super::processor::line::LineProcessor::new()),
        SeriesOption::Bar(_) => Box::new(super::processor::bar::BarProcessor::new()),
        SeriesOption::Scatter(_) => Box::new(super::processor::scatter::ScatterProcessor::new()),
        SeriesOption::Bubble(_) => Box::new(super::processor::bubble::BubbleProcessor::new()),
        SeriesOption::Candlestick(_) => {
            Box::new(super::processor::candlestick::CandlestickProcessor::new())
        }
        SeriesOption::Radar(_) => Box::new(super::processor::radar::RadarProcessor::new()),
        SeriesOption::PolarBar(_) => {
            Box::new(super::processor::polar_bar::PolarBarProcessor::new())
        }
        SeriesOption::PolarScatter(_) => {
            Box::new(super::processor::polar_scatter::PolarScatterProcessor::new())
        }
        SeriesOption::Gauge(_) => Box::new(super::processor::gauge::GaugeProcessor::new()),
        SeriesOption::Table(_) => Box::new(super::processor::table::TableProcessor::new()),
    }
}
