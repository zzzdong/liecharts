use thiserror::Error;

/// Errors that can occur during chart building, layout, or rendering.
#[derive(Error, Debug)]
pub enum ChartError {
    /// Rendering pipeline error (image creation, pixel processing, etc.).
    #[error("Render error: {0}")]
    RenderError(String),

    /// Data parsing or validation error (invalid data points, missing fields, etc.).
    #[error("Data error: {0}")]
    DataError(String),

    /// Invalid color value encountered.
    #[error("Invalid color: {0}")]
    InvalidColor(String),

    /// Theme name not found in the registry.
    #[error("Theme not found: {0}")]
    ThemeNotFound(String),

    /// Font loading failure.
    #[error("Font load error: {0}")]
    FontLoadError(String),

    /// Unsupported chart or series type.
    #[error("Unsupported chart type: {0}")]
    UnsupportedChartType(String),

    /// Layout computation error.
    #[error("Layout error: {0}")]
    LayoutError(String),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// File or stream IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Image encoding/decoding error.
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    /// Invalid configuration error.
    #[error("Invalid config: {0}")]
    InvalidConfig(String),

    /// Invalid axis binding error.
    #[error("Invalid axis binding: {0}")]
    InvalidAxisBinding(String),

    /// Missing column in dataframe.
    #[error("Missing column: {0}")]
    MissingColumn(String),
}

pub type Result<T> = std::result::Result<T, ChartError>;
