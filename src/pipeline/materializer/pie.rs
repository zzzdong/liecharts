//! Pie Materializer: 将 Pie SeriesSpec 转换为 PieSeries

use vello_cpu::kurbo::Rect;

use crate::{
    error::Result,
    pipeline::{
        materializer::SeriesMaterializer,
        typed_series::{LabelPosition, PieSeries, PieSlice, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
    visual::Color,
};

pub struct PieMaterializer;

impl SeriesMaterializer for PieMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        _bounds: Rect,
        _axis_ranges: &ResolvedAxisRanges,
        _color: Color,
        colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let cfg = match &spec.config {
            SeriesConfig::Pie(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected PieConfig".into(),
                ));
            }
        };

        // 从 DataFrame 获取数据
        let category_col = spec
            .data
            .get_column(&cfg.category_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.category_col.clone()))?;
        let value_col = spec
            .data
            .get_column(&cfg.value_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.value_col.clone()))?;

        // 收集数据并计算总和
        let mut total = 0.0;
        let mut entries: Vec<(String, f64)> = Vec::with_capacity(spec.data.row_count());

        for i in 0..spec.data.row_count() {
            let name = category_col.as_string(i).unwrap_or_default();
            let value = value_col.as_f64(i).unwrap_or(0.0);
            if value > 0.0 {
                total += value;
                entries.push((name, value));
            }
        }

        // 创建扇区
        let mut slices = Vec::with_capacity(entries.len());

        for (idx, (name, value)) in entries.into_iter().enumerate() {
            let percent = if total > 0.0 { value / total } else { 0.0 };
            let color = colors.get_data_color(idx);

            slices.push(PieSlice {
                name,
                value,
                color,
                percent,
            });
        }

        Ok(TypedSeries::Pie(PieSeries {
            name: spec.name.clone(),
            radius_inner: cfg.radius.0,
            radius_outer: cfg.radius.1,
            label_show: cfg.label_show,
            label_position: map_label_position(cfg.label_position),
            label_font_size: cfg.label_font_size,
            slices,
        }))
    }
}

fn map_label_position(pos: crate::pipeline::types::LabelPosition) -> LabelPosition {
    match pos {
        crate::pipeline::types::LabelPosition::Outside => LabelPosition::Outside,
        crate::pipeline::types::LabelPosition::Inside => LabelPosition::Inside,
    }
}
