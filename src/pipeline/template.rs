//! 统一的标签模板渲染引擎。
//!
//! 为 axis label、series label、pie label、legend 提供一致的 ECharts 风格占位符替换，
//! 避免各处各自实现一套 `.replace()` 逻辑。
//!
//! 支持的占位符（与 ECharts formatter 兼容）：
//! - `{a}`：系列名（series name）
//! - `{b}`：数据项名称（类目名 / 数据点名）
//! - `{c}`：数据项数值
//! - `{d}`：百分比（饼图等），带一个「%」
//! - `{value}`：数据项数值（`{c}` 的别名，兼容轴标签旧用法）
//! - `{name}`：系列名（`{a}` 的别名）

/// 模板渲染上下文。
#[derive(Debug, Clone, Default)]
pub struct TemplateContext<'a> {
    /// 系列名（`{a}` / `{name}`）
    pub series_name: Option<&'a str>,
    /// 数据项名称（`{b}`）
    pub name: Option<&'a str>,
    /// 数据项数值（`{c}` / `{value}`）
    pub value: Option<f64>,
    /// 百分比 0~100（`{d}`）
    pub percent: Option<f64>,
}

impl<'a> TemplateContext<'a> {
    /// 仅带数值的上下文（轴标签等场景）
    pub fn value_only(value: f64) -> Self {
        Self {
            series_name: None,
            name: None,
            value: Some(value),
            percent: None,
        }
    }

    /// 仅带名称的上下文（图例等场景）
    pub fn name_only(name: &'a str) -> Self {
        Self {
            series_name: None,
            name: Some(name),
            value: None,
            percent: None,
        }
    }
}

/// 格式化数值：整数不带小数，否则保留 1 位。
pub fn format_number(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{:.0}", v)
    } else if (v * 10.0).fract() == 0.0 {
        format!("{:.1}", v)
    } else {
        format!("{:.2}", v)
    }
}

/// 应用模板替换。
///
/// 未提供模板时返回 `fallback`（各调用方传入自己的默认文本）。
pub fn render_template(tpl: Option<&str>, ctx: &TemplateContext, fallback: &str) -> String {
    let Some(tpl) = tpl else {
        return fallback.to_string();
    };
    if tpl.is_empty() {
        return fallback.to_string();
    }

    let mut out = tpl.to_string();
    if let Some(v) = ctx.value {
        let s = format_number(v);
        out = out.replace("{value}", &s).replace("{c}", &s);
    }
    if let Some(p) = ctx.percent {
        // ECharts 语义：`{d}` 是纯数字百分比（不含 %），由模板自行决定是否追加 "%"
        let s = format!("{:.1}", p);
        out = out.replace("{d}", &s);
    }
    if let Some(name) = ctx.name {
        out = out.replace("{b}", name);
    }
    if let Some(sn) = ctx.series_name {
        out = out.replace("{a}", sn).replace("{name}", sn);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_all_placeholders() {
        let ctx = TemplateContext {
            series_name: Some("告警"),
            name: Some("高危"),
            value: Some(85.0),
            percent: Some(13.7),
        };
        assert_eq!(
            render_template(Some("{a} | {b}: {c} ({d}%)"), &ctx, ""),
            "告警 | 高危: 85 (13.7%)"
        );
        // `{d}` 本身是纯数字（不含 %），模板自行决定是否追加 "%"
        assert_eq!(render_template(Some("{d}"), &ctx, ""), "13.7");
    }

    #[test]
    fn falls_back_when_no_template() {
        let ctx = TemplateContext::value_only(42.0);
        assert_eq!(render_template(None, &ctx, "默认"), "默认");
        assert_eq!(render_template(Some(""), &ctx, "默认"), "默认");
    }

    #[test]
    fn formats_numbers() {
        assert_eq!(format_number(10.0), "10");
        assert_eq!(format_number(10.5), "10.5");
        assert_eq!(format_number(10.55), "10.55");
    }

    #[test]
    fn supports_value_alias() {
        let ctx = TemplateContext::value_only(33.0);
        assert_eq!(render_template(Some("{value} 万人"), &ctx, ""), "33 万人");
    }
}
