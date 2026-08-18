use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::pipeline::builder::ColorExt;
use lievisual::Color;

/// Design tokens representing a complete chart color and typography palette.
///
/// Each field maps to a visual property used during layout and rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignTokens {
    // 色彩系统
    pub color: ColorTokens,
    // 文字系统
    pub text: TextTokens,
    // 间距系统
    pub spacing: SpacingTokens,
    // 边框与分割线
    pub border: BorderTokens,
    // 阴影与效果
    pub effect: EffectTokens,
}

/// 色彩令牌 - ECharts 6 默认配色方案
/// 参考: https://github.com/apache/echarts/blob/master/src/core/tokens.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorTokens {
    /// 主色调色板（向后兼容，使用 theme）
    pub primary: Vec<String>,
    /// 主题色板（9色）- ECharts 6 新配色
    pub theme: Vec<String>,

    // === 中性色阶 (neutral00 - neutral99) ===
    /// 纯白
    pub neutral00: String,
    /// 极浅灰
    pub neutral05: String,
    /// 很浅灰
    pub neutral10: String,
    /// 浅灰
    pub neutral15: String,
    /// 较浅灰
    pub neutral20: String,
    /// 浅中灰
    pub neutral25: String,
    /// 中浅灰
    pub neutral30: String,
    /// 中灰偏浅
    pub neutral35: String,
    /// 中灰
    pub neutral40: String,
    /// 中灰偏深
    pub neutral45: String,
    /// 中深灰
    pub neutral50: String,
    /// 深灰偏浅
    pub neutral55: String,
    /// 深灰
    pub neutral60: String,
    /// 深灰偏深
    pub neutral65: String,
    /// 较深灰
    pub neutral70: String,
    /// 很深灰
    pub neutral75: String,
    /// 极深灰
    pub neutral80: String,
    /// 接近黑
    pub neutral85: String,
    /// 很接近黑
    pub neutral90: String,
    /// 几乎黑
    pub neutral95: String,
    /// 纯黑
    pub neutral99: String,

    // === 强调色阶 (accent05 - accent95) ===
    /// 极浅强调色
    pub accent05: String,
    /// 很浅强调色
    pub accent10: String,
    /// 浅强调色
    pub accent15: String,
    /// 较浅强调色
    pub accent20: String,
    /// 浅中强调色
    pub accent25: String,
    /// 中浅强调色
    pub accent30: String,
    /// 中强调色偏浅
    pub accent35: String,
    /// 中强调色
    pub accent40: String,
    /// 中强调色偏深
    pub accent45: String,
    /// 中深强调色
    pub accent50: String,
    /// 深强调色偏浅
    pub accent55: String,
    /// 深强调色
    pub accent60: String,
    /// 深强调色偏深
    pub accent65: String,
    /// 较深强调色
    pub accent70: String,
    /// 很深强调色
    pub accent75: String,
    /// 极深强调色
    pub accent80: String,
    /// 接近黑强调色
    pub accent85: String,
    /// 很接近黑强调色
    pub accent90: String,
    /// 几乎黑强调色
    pub accent95: String,

    // === 语义化颜色 ===
    /// 透明
    pub transparent: String,
    /// 主色（向后兼容，使用 primary 语义字段）
    pub text_primary: String,
    /// 次色（向后兼容，使用 secondary 语义字段）
    pub text_secondary: String,
    /// 第三色（向后兼容，使用 tertiary 语义字段）
    pub text_tertiary: String,
    /// 语义化主色
    pub secondary: String,
    /// 语义化第三色
    pub tertiary: String,
    /// 语义化第四色
    pub quaternary: String,
    /// 禁用色
    pub disabled: String,
    /// 高亮色
    pub highlight: String,

    // === 边框颜色 ===
    /// 边框色
    pub border: String,
    /// 边框浅色
    pub border_tint: String,
    /// 边框深色
    pub border_shade: String,

    // === 背景颜色 ===
    /// 背景色
    pub background: String,
    /// 背景浅色
    pub background_tint: String,
    /// 背景透明
    pub background_transparent: String,
    /// 背景深色
    pub background_shade: String,

    // === 阴影颜色 ===
    /// 阴影色
    pub shadow: String,
    /// 阴影浅色
    pub shadow_tint: String,

    // === 轴线颜色 ===
    /// 轴线色
    pub axis_line: String,
    /// 轴线浅色
    pub axis_line_tint: String,
    /// 刻度色
    pub axis_tick: String,
    /// 小刻度色
    pub axis_tick_minor: String,
    /// 轴标签色
    pub axis_label: String,
    /// 分割线色（向后兼容，使用 axis_split_line）
    pub split_line: String,
    /// 分割线色
    pub axis_split_line: String,
    /// 小分割线色
    pub axis_minor_split_line: String,

    // === 状态颜色 ===
    /// 强调色（向后兼容，使用 theme[0]）
    pub accent: String,
    /// 成功色
    pub success: String,
    /// 警告色
    pub warning: String,
    /// 错误色
    pub error: String,
}

/// 文字令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextTokens {
    /// 标题字体大小
    pub title_size: f64,
    /// 副标题字体大小
    pub subtitle_size: f64,
    /// 正文字体大小
    pub body_size: f64,
    /// 辅助文字大小
    pub caption_size: f64,
    /// 字体家族
    pub font_family: String,
    /// 标题字重
    pub title_weight: String,
}

/// 间距令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingTokens {
    /// 超小间距
    pub xs: f64,
    /// 小间距
    pub sm: f64,
    /// 中间距
    pub md: f64,
    /// 大间距
    pub lg: f64,
    /// 超大间距
    pub xl: f64,
}

/// 边框令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderTokens {
    /// 细边框宽度
    pub thin: f64,
    /// 常规边框宽度
    pub normal: f64,
    /// 粗边框宽度
    pub thick: f64,
    /// 边框颜色
    pub color: String,
}

/// 效果令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectTokens {
    /// 阴影颜色
    pub shadow_color: String,
    /// 阴影模糊度
    pub shadow_blur: f64,
    /// 阴影偏移X
    pub shadow_offset_x: f64,
    /// 阴影偏移Y
    pub shadow_offset_y: f64,
}

impl Default for DesignTokens {
    fn default() -> Self {
        Self::echarts_v6()
    }
}

impl DesignTokens {
    /// ECharts 6 默认设计令牌
    /// 参考: https://github.com/apache/echarts/blob/master/src/core/tokens.ts
    pub fn echarts_v6() -> Self {
        // ECharts 6 主题色板
        let theme_colors = vec![
            "#5070dd".to_string(), // 主蓝色
            "#b6d634".to_string(), // 青柠绿
            "#505372".to_string(), // 深灰蓝
            "#ff994d".to_string(), // 橙色
            "#0ca8df".to_string(), // 天蓝
            "#ffd10a".to_string(), // 黄色
            "#fb628b".to_string(), // 粉红
            "#785db0".to_string(), // 紫色
            "#3fbe95".to_string(), // 青绿
        ];

        Self {
            color: ColorTokens {
                // 向后兼容：primary 使用 theme 色板
                primary: theme_colors.clone(),
                // 主题色板（9色）- ECharts 6 新配色
                theme: theme_colors.clone(),

                // 中性色阶 (neutral00 - neutral99)
                neutral00: "#ffffff".to_string(),
                neutral05: "#f4f7fd".to_string(),
                neutral10: "#e8ebf0".to_string(),
                neutral15: "#dbdee4".to_string(),
                neutral20: "#cfd2d7".to_string(),
                neutral25: "#c3c5cb".to_string(),
                neutral30: "#b7b9be".to_string(),
                neutral35: "#aaacb2".to_string(),
                neutral40: "#9ea0a5".to_string(),
                neutral45: "#929399".to_string(),
                neutral50: "#86878c".to_string(),
                neutral55: "#797b7f".to_string(),
                neutral60: "#6d6e73".to_string(),
                neutral65: "#616266".to_string(),
                neutral70: "#54555a".to_string(),
                neutral75: "#48494d".to_string(),
                neutral80: "#3c3c41".to_string(),
                neutral85: "#303034".to_string(),
                neutral90: "#232328".to_string(),
                neutral95: "#17171b".to_string(),
                neutral99: "#000000".to_string(),

                // 强调色阶 (accent05 - accent95) - 基于主题蓝色
                accent05: "#eff1f9".to_string(),
                accent10: "#e0e4f2".to_string(),
                accent15: "#d0d6ec".to_string(),
                accent20: "#c0c9e6".to_string(),
                accent25: "#b1bbdf".to_string(),
                accent30: "#a1aed9".to_string(),
                accent35: "#91a0d3".to_string(),
                accent40: "#8292cc".to_string(),
                accent45: "#7285c6".to_string(),
                accent50: "#6578ba".to_string(),
                accent55: "#5c6da9".to_string(),
                accent60: "#536298".to_string(),
                accent65: "#4a5787".to_string(),
                accent70: "#404c76".to_string(),
                accent75: "#374165".to_string(),
                accent80: "#2e3654".to_string(),
                accent85: "#252b43".to_string(),
                accent90: "#1b2032".to_string(),
                accent95: "#121521".to_string(),

                // 语义化颜色 - 向后兼容
                transparent: "rgba(0,0,0,0)".to_string(),
                text_primary: "#3c3c41".to_string(), // neutral80
                text_secondary: "#54555a".to_string(), // neutral70
                text_tertiary: "#6d6e73".to_string(), // neutral60
                // 新的语义化字段
                secondary: "#54555a".to_string(),  // neutral70
                tertiary: "#6d6e73".to_string(),   // neutral60
                quaternary: "#86878c".to_string(), // neutral50
                disabled: "#cfd2d7".to_string(),   // neutral20
                highlight: "rgba(255,231,130,0.8)".to_string(),

                // 边框颜色
                border: "#b7b9be".to_string(),       // neutral30
                border_tint: "#cfd2d7".to_string(),  // neutral20
                border_shade: "#aaacb2".to_string(), // neutral35

                // 背景颜色
                background: "#f4f7fd".to_string(), // neutral05
                background_tint: "rgba(234,237,245,0.5)".to_string(),
                background_transparent: "rgba(255,255,255,0)".to_string(),
                background_shade: "#e8ebf0".to_string(), // neutral10

                // 阴影颜色
                shadow: "rgba(0,0,0,0.2)".to_string(),
                shadow_tint: "rgba(129,130,136,0.2)".to_string(),

                // 轴线颜色 - 向后兼容
                axis_line: "#54555a".to_string(),       // neutral70
                axis_line_tint: "#9ea0a5".to_string(),  // neutral40
                axis_tick: "#54555a".to_string(),       // neutral70
                axis_tick_minor: "#6d6e73".to_string(), // neutral60
                axis_label: "#54555a".to_string(),      // neutral70
                split_line: "#dbdee4".to_string(),      // neutral15 (向后兼容)
                axis_split_line: "#dbdee4".to_string(), // neutral15
                axis_minor_split_line: "#f4f7fd".to_string(), // neutral05

                // 状态颜色 - 向后兼容
                accent: "#5070dd".to_string(),  // theme[0]
                success: "#b6d634".to_string(), // 主题色2
                warning: "#ffd10a".to_string(), // 主题色6
                error: "#fb628b".to_string(),   // 主题色7
            },
            text: TextTokens {
                title_size: 18.0,
                subtitle_size: 14.0,
                body_size: 12.0,
                caption_size: 10.0,
                font_family: "sans-serif".to_string(),
                title_weight: "normal".to_string(),
            },
            spacing: SpacingTokens {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 16.0,
                xl: 24.0,
            },
            border: BorderTokens {
                thin: 0.5,
                normal: 1.0,
                thick: 2.0,
                color: "#cccccc".to_string(),
            },
            effect: EffectTokens {
                shadow_color: "rgba(0, 0, 0, 0.1)".to_string(),
                shadow_blur: 4.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.0,
            },
        }
    }

    /// Vintage 复古主题的设计令牌
    pub fn vintage() -> Self {
        let mut tokens = Self::echarts_v6();
        tokens.color.primary = vec![
            "#d87c7c".to_string(),
            "#919e8b".to_string(),
            "#d7ab82".to_string(),
            "#6e7074".to_string(),
            "#61a0a8".to_string(),
            "#efa18d".to_string(),
            "#787464".to_string(),
            "#cc7e63".to_string(),
            "#724e58".to_string(),
            "#4b565b".to_string(),
        ];
        tokens.color.theme = tokens.color.primary.clone();
        tokens.color.background = "#fef8ef".to_string();
        tokens
    }

    /// Macarons 马卡龙主题的设计令牌
    pub fn macarons() -> Self {
        let mut tokens = Self::echarts_v6();
        tokens.color.primary = vec![
            "#2ec7c9".to_string(),
            "#b6a2de".to_string(),
            "#5ab1ef".to_string(),
            "#ffb980".to_string(),
            "#d87a80".to_string(),
            "#8d98b3".to_string(),
            "#e5cf0d".to_string(),
            "#97b552".to_string(),
            "#95706d".to_string(),
            "#dc69aa".to_string(),
        ];
        tokens.color.theme = tokens.color.primary.clone();
        tokens
    }

    /// Infographic 信息图主题的设计令牌
    pub fn infographic() -> Self {
        let mut tokens = Self::echarts_v6();
        tokens.color.primary = vec![
            "#c1232b".to_string(),
            "#27727b".to_string(),
            "#fcce10".to_string(),
            "#e87c25".to_string(),
            "#b5c334".to_string(),
            "#fe8463".to_string(),
            "#9bca63".to_string(),
            "#fad860".to_string(),
            "#f3a43b".to_string(),
            "#60c0dd".to_string(),
        ];
        tokens.color.theme = tokens.color.primary.clone();
        tokens
    }

    /// Shine 闪耀主题的设计令牌
    pub fn shine() -> Self {
        let mut tokens = Self::echarts_v6();
        tokens.color.primary = vec![
            "#c12e34".to_string(),
            "#e6b600".to_string(),
            "#0098d9".to_string(),
            "#2b821d".to_string(),
            "#005eaa".to_string(),
            "#339ca8".to_string(),
            "#cda819".to_string(),
            "#32a487".to_string(),
        ];
        tokens.color.theme = tokens.color.primary.clone();
        tokens
    }

    /// Roma 罗马主题的设计令牌
    pub fn roma() -> Self {
        let mut tokens = Self::echarts_v6();
        tokens.color.primary = vec![
            "#ff8a45".to_string(),
            "#e6b600".to_string(),
            "#0098d9".to_string(),
            "#61a0a8".to_string(),
            "#2ec7c9".to_string(),
            "#b6a2de".to_string(),
            "#91ca48".to_string(),
            "#749f83".to_string(),
            "#ca8622".to_string(),
            "#bdaa8d".to_string(),
        ];
        tokens.color.theme = tokens.color.primary.clone();
        tokens
    }

    /// Dark 深色主题的设计令牌
    pub fn dark() -> Self {
        let mut tokens = Self::echarts_v6();
        tokens.color.primary = vec![
            "#4992ff".to_string(),
            "#7cffb2".to_string(),
            "#fddd60".to_string(),
            "#ff6e76".to_string(),
            "#58d9f9".to_string(),
            "#05c091".to_string(),
            "#ff8a45".to_string(),
            "#8d48e3".to_string(),
            "#dd79ff".to_string(),
        ];
        tokens.color.theme = tokens.color.primary.clone();
        tokens.color.background = "#1a1a1a".to_string();
        tokens.color.text_primary = "#eeeeee".to_string();
        tokens.color.text_secondary = "#cccccc".to_string();
        tokens.color.text_tertiary = "#999999".to_string();
        tokens
    }
}

/// 主题配置 - 基于设计令牌构建
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    /// 设计令牌系统
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<DesignTokens>,
    /// 主色调色板（向后兼容）
    pub color: Vec<String>,
    pub background_color: String,
    pub title: TitleTheme,
    pub legend: LegendTheme,
    pub axis: AxisTheme,
    pub series: HashMap<String, SeriesTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleTheme {
    pub text_style: TextStyleTheme,
    pub subtext_style: TextStyleTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendTheme {
    pub text_style: TextStyleTheme,
    pub item_width: f64,
    pub item_height: f64,
    pub symbol_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisTheme {
    pub axis_line: LineStyleTheme,
    pub axis_tick: LineStyleTheme,
    pub axis_label: TextStyleTheme,
    pub split_line: LineStyleTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyleTheme {
    pub color: String,
    pub font_size: f64,
    pub font_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineStyleTheme {
    pub color: String,
    pub width: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesTheme {
    pub item_style: ItemStyleTheme,
    pub line_style: Option<LineStyleTheme>,
    pub label: Option<TextStyleTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStyleTheme {
    pub color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: f64,
}

impl Theme {
    /// 创建基于设计令牌的主题
    pub fn from_tokens(name: &str, tokens: DesignTokens) -> Self {
        let color = tokens.color.primary.clone();
        let bg = tokens.color.background.clone();
        let text_primary = tokens.color.text_primary.clone();
        let text_secondary = tokens.color.text_secondary.clone();
        let font_family = tokens.text.font_family.clone();
        let title_size = tokens.text.title_size;
        let body_size = tokens.text.body_size;
        let axis_line_color = tokens.color.axis_line.clone();
        let axis_label_color = tokens.color.axis_label.clone();
        let split_line_color = tokens.color.split_line.clone();
        let border_thin = tokens.border.thin;

        let mut series_themes = HashMap::new();
        series_themes.insert(
            "bar".to_string(),
            SeriesTheme {
                item_style: ItemStyleTheme {
                    color: None,
                    border_color: None,
                    border_width: 0.0,
                },
                line_style: None,
                label: Some(TextStyleTheme {
                    color: text_primary.clone(),
                    font_size: body_size,
                    font_family: font_family.clone(),
                }),
            },
        );
        series_themes.insert(
            "line".to_string(),
            SeriesTheme {
                item_style: ItemStyleTheme {
                    color: None,
                    border_color: None,
                    border_width: border_thin,
                },
                line_style: Some(LineStyleTheme {
                    color: tokens.color.accent.clone(),
                    width: 2.0,
                }),
                label: Some(TextStyleTheme {
                    color: text_primary.clone(),
                    font_size: body_size,
                    font_family: font_family.clone(),
                }),
            },
        );
        series_themes.insert(
            "pie".to_string(),
            SeriesTheme {
                item_style: ItemStyleTheme {
                    color: None,
                    border_color: Some(bg.clone()),
                    border_width: 1.0,
                },
                line_style: None,
                label: Some(TextStyleTheme {
                    color: text_primary.clone(),
                    font_size: body_size,
                    font_family: font_family.clone(),
                }),
            },
        );
        series_themes.insert(
            "scatter".to_string(),
            SeriesTheme {
                item_style: ItemStyleTheme {
                    color: None,
                    border_color: Some(bg.clone()),
                    border_width: 1.0,
                },
                line_style: None,
                label: Some(TextStyleTheme {
                    color: text_primary.clone(),
                    font_size: body_size,
                    font_family: font_family.clone(),
                }),
            },
        );

        let subtitle_size = tokens.text.subtitle_size;

        Self {
            name: name.to_string(),
            tokens: Some(tokens),
            color,
            background_color: bg,
            title: TitleTheme {
                text_style: TextStyleTheme {
                    color: text_primary.clone(),
                    font_size: title_size,
                    font_family: font_family.clone(),
                },
                subtext_style: TextStyleTheme {
                    color: text_secondary.clone(),
                    font_size: subtitle_size,
                    font_family: font_family.clone(),
                },
            },
            legend: LegendTheme {
                text_style: TextStyleTheme {
                    color: text_primary.clone(),
                    font_size: body_size,
                    font_family: font_family.clone(),
                },
                item_width: 80.0,
                item_height: 20.0,
                symbol_size: 10.0,
            },
            axis: AxisTheme {
                axis_line: LineStyleTheme {
                    color: axis_line_color.clone(),
                    width: 1.0,
                },
                axis_tick: LineStyleTheme {
                    color: axis_line_color.clone(),
                    width: 1.0,
                },
                axis_label: TextStyleTheme {
                    color: axis_label_color.clone(),
                    font_size: body_size,
                    font_family: font_family.clone(),
                },
                split_line: LineStyleTheme {
                    color: split_line_color.clone(),
                    width: 1.0,
                },
            },
            series: series_themes,
        }
    }

    /// ECharts 6 默认主题 - 使用设计令牌系统
    pub fn echarts() -> Self {
        Self::from_tokens("echarts", DesignTokens::echarts_v6())
    }

    /// 旧版浅色主题（向后兼容）
    pub fn light() -> Self {
        let mut theme = Self::echarts();
        theme.name = "light".to_string();
        theme
    }

    /// 旧版深色主题（向后兼容）
    pub fn dark() -> Self {
        Self::from_tokens("dark", DesignTokens::dark())
    }

    /// Vintage 复古主题（简化实现）
    pub fn vintage() -> Self {
        Self::from_tokens("vintage", DesignTokens::vintage())
    }

    /// Macarons 马卡龙主题
    pub fn macarons() -> Self {
        Self::from_tokens("macarons", DesignTokens::macarons())
    }

    /// Infographic 信息图主题
    pub fn infographic() -> Self {
        Self::from_tokens("infographic", DesignTokens::infographic())
    }

    /// Shine 闪耀主题
    pub fn shine() -> Self {
        Self::from_tokens("shine", DesignTokens::shine())
    }

    /// Roma 罗马主题
    pub fn roma() -> Self {
        Self::from_tokens("roma", DesignTokens::roma())
    }

    pub fn get_color(&self, index: usize) -> Result<Color> {
        let color_str = self.color.get(index % self.color.len()).ok_or_else(|| {
            crate::error::ChartError::InvalidColor("No color available".to_string())
        })?;
        Color::from_hex(color_str).ok_or_else(|| {
            crate::error::ChartError::InvalidColor(format!("Invalid color: {}", color_str))
        })
    }

    /// 从 tokens 获取调色板颜色
    pub fn get_theme_color(&self, index: usize) -> Color {
        let tokens = self.tokens();
        let color_str = tokens
            .color
            .theme
            .get(index % tokens.color.theme.len())
            .unwrap_or(&tokens.color.primary[0]);
        Color::from_hex(color_str).unwrap_or(Color::rgb(80, 112, 221))
    }

    /// 获取背景色
    pub fn get_background_color(&self) -> Color {
        Color::from_hex(&self.tokens().color.background).unwrap_or(Color::rgb(255, 255, 255))
    }

    /// 获取标题文本样式
    pub fn get_title_text_style(&self) -> TextStyleTheme {
        let tokens = self.tokens();
        TextStyleTheme {
            color: tokens.color.text_primary.clone(),
            font_size: tokens.text.title_size,
            font_family: tokens.text.font_family.clone(),
        }
    }

    /// 获取副标题文本样式
    pub fn get_subtitle_text_style(&self) -> TextStyleTheme {
        let tokens = self.tokens();
        TextStyleTheme {
            color: tokens.color.text_secondary.clone(),
            font_size: tokens.text.subtitle_size,
            font_family: tokens.text.font_family.clone(),
        }
    }

    /// 获取图例文本样式
    pub fn get_legend_text_style(&self) -> TextStyleTheme {
        let tokens = self.tokens();
        TextStyleTheme {
            color: tokens.color.text_primary.clone(),
            font_size: tokens.text.body_size,
            font_family: tokens.text.font_family.clone(),
        }
    }

    /// 获取图例配置
    pub fn get_legend_config(&self) -> (f64, f64, f64) {
        let _tokens = self.tokens();
        (80.0, 20.0, 10.0) // item_width, item_height, symbol_size
    }

    /// 获取轴标签文本样式
    pub fn get_axis_label_style(&self) -> TextStyleTheme {
        let tokens = self.tokens();
        TextStyleTheme {
            color: tokens.color.axis_label.clone(),
            font_size: tokens.text.body_size,
            font_family: tokens.text.font_family.clone(),
        }
    }

    /// 获取轴线样式
    pub fn get_axis_line_style(&self) -> LineStyleTheme {
        let tokens = self.tokens();
        LineStyleTheme {
            color: tokens.color.axis_line.clone(),
            width: 1.0,
        }
    }

    /// 获取刻度线样式
    pub fn get_axis_tick_style(&self) -> LineStyleTheme {
        let tokens = self.tokens();
        LineStyleTheme {
            color: tokens.color.axis_tick.clone(),
            width: 1.0,
        }
    }

    /// 获取分割线样式
    pub fn get_split_line_style(&self) -> LineStyleTheme {
        let tokens = self.tokens();
        LineStyleTheme {
            color: tokens.color.axis_split_line.clone(),
            width: 1.0,
        }
    }

    /// 获取设计令牌（如果不存在则返回默认）
    pub fn tokens(&self) -> &DesignTokens {
        self.tokens.as_ref().unwrap_or_else(|| {
            // 如果没有设计令牌，创建一个基于当前主题配置的
            static DEFAULT_TOKENS: std::sync::OnceLock<DesignTokens> = std::sync::OnceLock::new();
            DEFAULT_TOKENS.get_or_init(DesignTokens::echarts_v6)
        })
    }
}

#[derive(Debug, Clone)]
pub struct ThemeRegistry {
    themes: HashMap<String, Theme>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            themes: HashMap::new(),
        };
        // ECharts 6 默认主题作为默认主题
        registry.register(Theme::echarts());
        registry.register(Theme::light());
        registry.register(Theme::dark());
        registry.register(Theme::vintage());
        registry.register(Theme::macarons());
        registry.register(Theme::infographic());
        registry.register(Theme::shine());
        registry.register(Theme::roma());
        registry
    }

    /// 获取默认主题（ECharts 6 主题）
    pub fn default_theme(&self) -> &Theme {
        self.themes
            .get("echarts")
            .or_else(|| self.themes.get("light"))
            .expect("Default theme should exist")
    }

    pub fn register(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }

    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// 获取所有可用主题名称
    pub fn available_themes(&self) -> Vec<&String> {
        self.themes.keys().collect()
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_theme(name: &str) -> Result<Theme> {
    let theme_registry = ThemeRegistry::new();
    theme_registry
        .get(name)
        .ok_or_else(|| {
            crate::error::ChartError::ThemeNotFound(format!("Theme not found: {}", name))
        })
        .cloned()
}
