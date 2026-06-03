use vello_cpu::kurbo::Rect;

use crate::{
    error::Result,
    option::{ChartOption, SeriesOption},
    pipeline::{
        ChartType, dataframe::DataFrame, mapper::CoordinateMapper, types::{ChartSpec, ColorContext, ResolvedAxisRanges, SeriesSpec, SubplotSpec}
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
    /// 新增：新 API 的 ChartSpec（可选，用于新管线）
    pub chart_spec: Option<&'a ChartSpec>,
    /// 新增：当前处理的 SeriesSpec（可选，用于新管线）
    pub series_spec: Option<&'a SeriesSpec>,
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

    /// 从 SeriesSpec 直接处理（跳过 to_dataframe，数据已在 DataFrame 中）
    /// 新 API 路径调用此方法，处理器可覆写以使用 SeriesSpec 的配置字段
    fn process_from_spec(
        &self,
        series: &SeriesSpec,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        // 默认实现：使用 SeriesSpec 的数据直接进入 transform 阶段
        let df = series.data.clone();
        let mut df = self.transform(df, input)?;
        self.mapper()
            .map_coordinates(&mut df, input, series.x_axis_index, series.y_axis_index);
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

/// 根据 ChartType 创建 DataProcessorV2（用于新 API 路径）
pub fn create_processor_from_chart_type(chart_type: ChartType) -> Box<dyn DataProcessor> {
    match chart_type {
        ChartType::Pie => Box::new(super::processor::pie::PieProcessor::new()),
        ChartType::Line => Box::new(super::processor::line::LineProcessor::new()),
        ChartType::Bar => Box::new(super::processor::bar::BarProcessor::new()),
        ChartType::Scatter => Box::new(super::processor::scatter::ScatterProcessor::new()),
        ChartType::Bubble => Box::new(super::processor::bubble::BubbleProcessor::new()),
        ChartType::Candlestick => {
            Box::new(super::processor::candlestick::CandlestickProcessor::new())
        }
        ChartType::Radar => Box::new(super::processor::radar::RadarProcessor::new()),
        ChartType::PolarBar => {
            Box::new(super::processor::polar_bar::PolarBarProcessor::new())
        }
        ChartType::PolarScatter => {
            Box::new(super::processor::polar_scatter::PolarScatterProcessor::new())
        }
        ChartType::Gauge => Box::new(super::processor::gauge::GaugeProcessor::new()),
        ChartType::Table => Box::new(super::processor::table::TableProcessor::new()),
    }
}
