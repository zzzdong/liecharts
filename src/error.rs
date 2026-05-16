use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChartError {
    #[error("未设置图表配置选项")]
    NoOption,

    #[error("渲染错误: {0}")]
    RenderError(String),

    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("图片错误: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("不支持的图表类型: {0}")]
    UnsupportedChartType(String),

    #[error("无效的颜色值: {0}")]
    InvalidColor(String),

    #[error("布局错误: {0}")]
    LayoutError(String),
}

pub type Result<T> = std::result::Result<T, ChartError>;
