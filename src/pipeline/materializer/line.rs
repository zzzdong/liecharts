//! Line Materializer: 将 Line SeriesSpec 转换为 LineSeries

use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        materializer::{SeriesMaterializer, map_x_to_pixel, map_y_to_pixel},
        typed_series::{LineSeries, SymbolType, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
    sampling::SamplingProcessor,
    visual::Color,
};

pub struct LineMaterializer;

impl SeriesMaterializer for LineMaterializer {
    fn materialize(
        spec: &SeriesSpec,
        bounds: Rect,
        axis_ranges: &ResolvedAxisRanges,
        color: Color,
        _colors: &ColorContext,
    ) -> Result<TypedSeries> {
        // 验证配置类型
        let cfg = match &spec.config {
            SeriesConfig::Line(c) => c,
            _ => {
                return Err(crate::error::ChartError::InvalidConfig(
                    "Expected LineConfig".into(),
                ));
            }
        };

        // 获取 X/Y 轴范围
        let x_range = axis_ranges.get_x_range(spec.x_axis_index).ok_or_else(|| {
            crate::error::ChartError::InvalidAxisBinding("X axis not found".into())
        })?;
        let y_range = axis_ranges.get_y_range(spec.y_axis_index).ok_or_else(|| {
            crate::error::ChartError::InvalidAxisBinding("Y axis not found".into())
        })?;

        // 应用采样（如果配置了）
        let df = if let Some((sampling_type, threshold)) = spec.sampling {
            SamplingProcessor::sample(&spec.data, threshold, sampling_type)
        } else {
            spec.data.clone()
        };

        // 从 DataFrame 获取数据
        let x_vals = df
            .get_column(&cfg.x_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.x_col.clone()))?;
        let y_vals = df
            .get_column(&cfg.y_col)
            .ok_or_else(|| crate::error::ChartError::MissingColumn(cfg.y_col.clone()))?;

        // 判断 X 轴类型：优先使用 axis_type，而不是 DataFrame 中的数据类型
        let is_category_axis = matches!(
            x_range.axis_type,
            crate::pipeline::types::AxisType::Category
        );
        let is_numeric_x = !is_category_axis && x_vals.as_f64(0).is_some();
        let row_count = df.row_count();
        let range_size = x_range.max - x_range.min;
        let is_category_with_gap = is_category_axis && range_size > row_count as f64 - 1.0;

        // 将数据点映射到像素空间
        let mut points = Vec::with_capacity(df.row_count());
        let mut values = Vec::with_capacity(df.row_count());

        for i in 0..df.row_count() {
            let x = if is_category_axis {
                // 分类轴：使用索引作为 X 值，居中时加 0.5
                let idx = i as f64;
                if is_category_with_gap {
                    Some(idx + 0.5)
                } else {
                    Some(idx)
                }
            } else if is_numeric_x {
                x_vals.as_f64(i)
            } else {
                // 其他情况：使用索引
                Some(i as f64)
            };
            let y = y_vals.as_f64(i);

            if let (Some(x), Some(y)) = (x, y) {
                let px = map_x_to_pixel(x, x_range, bounds);
                let py = map_y_to_pixel(y, y_range, bounds);
                points.push(Point::new(px, py));
                values.push(y);
            }
        }

        // 面积填充基线（Y=0 或 Y 轴最小值）
        let baseline_y = if y_range.min > 0.0 {
            bounds.y1
        } else {
            map_y_to_pixel(0.0, y_range, bounds)
        };

        // 面积填充颜色：用户指定则使用之，否则使用系列颜色
        let area_color = if cfg.area {
            Some(cfg.area_color.unwrap_or(color))
        } else {
            None
        };

        Ok(TypedSeries::Line(LineSeries {
            name: spec.name.clone(),
            color,
            line_width: cfg.line_width,
            smooth: cfg.smooth,
            area_color,
            area_opacity: cfg.area_opacity,
            symbol_type: map_symbol_type(cfg.symbol_type),
            symbol_size: cfg.symbol_size,
            points,
            values,
            baseline_y,
            baseline_points: None,
            label: if cfg.label_show {
                Some(crate::pipeline::typed_series::SeriesLabelConfig {
                    show: true,
                    position: crate::pipeline::typed_series::SeriesLabelPosition::Top,
                    color: crate::visual::Color::new(60, 60, 65),
                    font_size: cfg.label_font_size,
                })
            } else {
                None
            },
        }))
    }
}

pub(crate) fn map_symbol_type(ty: crate::pipeline::types::SymbolType) -> SymbolType {
    match ty {
        crate::pipeline::types::SymbolType::Circle => SymbolType::Circle,
        crate::pipeline::types::SymbolType::Rect => SymbolType::Rect,
        crate::pipeline::types::SymbolType::RoundRect => SymbolType::RoundRect,
        crate::pipeline::types::SymbolType::Triangle => SymbolType::Triangle,
        crate::pipeline::types::SymbolType::Diamond => SymbolType::Diamond,
        crate::pipeline::types::SymbolType::Pin => SymbolType::Pin,
        crate::pipeline::types::SymbolType::Arrow => SymbolType::Arrow,
        crate::pipeline::types::SymbolType::None => SymbolType::None,
    }
}
