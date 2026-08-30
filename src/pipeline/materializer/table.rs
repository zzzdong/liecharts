//! Table Materializer: 将 Table SeriesSpec 转换为 TableSeries

use vello_cpu::kurbo::Rect;

use crate::{
    Color,
    error::Result,
    pipeline::{
        dataframe::DataValue,
        materializer::SeriesMaterializer,
        typed_series::{TableSeries, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
};

/// 将 DataValue 格式化为可直接显示的字符串（不带类型包装）
fn format_value(v: &DataValue) -> String {
    match v {
        DataValue::Null => String::new(),
        DataValue::Bool(b) => format!("{}", b),
        DataValue::Integer(i) => format!("{}", i),
        DataValue::Float(f) => {
            // 去除浮点数尾部多余的 0 和小数点
            let s = format!("{}", f);
            if s.contains('.') {
                let trimmed = s.trim_end_matches('0').trim_end_matches('.');
                trimmed.to_string()
            } else {
                s
            }
        }
        DataValue::String(s) => s.clone(),
        DataValue::Color(c) => format!("{:?}", c),
    }
}

pub struct TableMaterializer;

impl SeriesMaterializer for TableMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        _bounds: Rect,
        _axis_ranges: &ResolvedAxisRanges,
        _color: Color,
        colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let _cfg = match &spec.config {
            SeriesConfig::Table(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected TableConfig".into(),
                ));
            }
        };

        // 从 DataFrame 提取表头和行数据
        let mut headers = Vec::new();
        let mut rows: Vec<Vec<String>> = Vec::new();

        // 获取所有列名作为表头
        for col_name in spec.data.column_names() {
            headers.push(col_name.to_string());
        }

        // 获取行数据
        let row_count = spec.data.row_count();
        for i in 0..row_count {
            let mut row = Vec::with_capacity(headers.len());
            for col_name in &headers {
                if let Some(col) = spec.data.get_column(col_name) {
                    let value = col.data.get(i).map(format_value).unwrap_or_default();
                    row.push(value);
                } else {
                    row.push(String::new());
                }
            }
            rows.push(row);
        }

        Ok(TypedSeries::Table(TableSeries {
            name: spec.name.clone(),
            headers,
            rows,
            header_bg: colors.table_header_bg,
            header_border_color: colors.axis_line_color,
            row_even_bg: colors.table_row_even_bg,
            row_odd_bg: colors.table_row_odd_bg,
        }))
    }
}
