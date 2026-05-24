use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SamplingType {
    Lttb,
    Average,
    Max,
    Min,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingOption {
    #[serde(rename = "type")]
    pub ty: SamplingType,
    pub threshold: usize,
}

impl SamplingOption {
    pub fn new(ty: SamplingType, threshold: usize) -> Self {
        Self { ty, threshold }
    }

    pub fn lttb(threshold: usize) -> Self {
        Self {
            ty: SamplingType::Lttb,
            threshold,
        }
    }
}
