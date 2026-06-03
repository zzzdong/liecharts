use crate::{pipeline::types::SubplotVisualData, visual::VisualElement};

/// 视觉元素构建器
///
/// 职责：
/// - 收集所有 subplot 的 VisualElement
/// - 添加全局元素（画布背景、全局标题、全局图例）
/// - 全局标题和图例采用浮层方式
/// - 按 z 索引排序
pub struct VisualElementBuilder {
    background: Option<VisualElement>,
    title_elements: Vec<VisualElement>,
    legend_elements: Vec<VisualElement>,
}

impl Default for VisualElementBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualElementBuilder {
    pub fn new() -> Self {
        Self {
            background: None,
            title_elements: Vec::new(),
            legend_elements: Vec::new(),
        }
    }

    pub fn with_background(mut self, element: VisualElement) -> Self {
        self.background = Some(element);
        self
    }

    pub fn with_title(mut self, elements: Vec<VisualElement>) -> Self {
        self.title_elements = elements;
        self
    }

    pub fn with_legend(mut self, elements: Vec<VisualElement>) -> Self {
        self.legend_elements = elements;
        self
    }

    /// 合并所有元素，按 z-index 排序，输出最终列表
    pub fn build(&self, subplot_data: Vec<SubplotVisualData>) -> Vec<VisualElement> {
        let mut elements: Vec<VisualElement> = Vec::new();

        // 背景层 (z=0)
        if let Some(bg) = &self.background {
            elements.push(bg.clone());
        }

        // subplot 网格线 (z=1)
        for data in &subplot_data {
            elements.extend(data.grid_lines.clone());
        }

        // subplot 轴元素 (z=2)
        for data in &subplot_data {
            elements.extend(data.axis_elements.clone());
        }

        // subplot 系列元素 (z=3)
        for data in &subplot_data {
            elements.extend(data.series_elements.clone());
        }

        // 全局图例 (z=4)
        elements.extend(self.legend_elements.clone());

        // 全局标题 (z=5)
        elements.extend(self.title_elements.clone());

        elements
    }
}
