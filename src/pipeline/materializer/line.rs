//! Line Materializer: 将 Line SeriesSpec 转换为 LineSeries

use lievisual::Color;
use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    pipeline::{
        materializer::{SeriesMaterializer, map_x_to_pixel, map_y_to_pixel},
        typed_series::{LineSeries, SymbolType, TypedSeries},
        types::{ColorContext, ResolvedAxisRanges, SeriesConfig, SeriesSpec},
    },
    sampling::SamplingProcessor,
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
                // null 数据点在 DataFrame 里是 NaN（历史：被当成 0 画出一个点）。
                // - `connectNulls: true` → 直接剔除，折线跨越该点连成一条；
                // - 默认 false → 保留 NaN 坐标，由 Builder 在此处断开折线。
                if !y.is_finite() && cfg.connect_nulls {
                    continue;
                }
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

        // 符号抽稀（ECharts `showAllSymbol`）：
        // - `showSymbol: false` → 完全不画符号；
        // - `showAllSymbol: 'auto'`（默认）且主轴为**类目轴** → 按像素槽宽抽稀，
        //   避免稠密数据点画成一串重叠的小圆（ECharts 只在类目轴生效）；
        // - 其余情况（value/time 轴，或显式 `true`）→ 全部显示。
        let symbol_step = if !cfg.symbol_show {
            0
        } else if cfg.show_all_symbol == crate::pipeline::types::ShowAllSymbol::Auto
            && is_category_axis
        {
            auto_symbol_step(cfg.symbol_size, points.len(), bounds.width())
        } else {
            1
        };

        let mark_lines = crate::pipeline::materializer::compute_mark_lines(
            &cfg.mark_line,
            &values,
            y_range,
            bounds,
        );
        let mark_points = crate::pipeline::materializer::compute_mark_points(
            &cfg.mark_point,
            &points,
            &values,
            false,
            |v| map_y_to_pixel(v, y_range, bounds),
        );

        Ok(TypedSeries::Line(LineSeries {
            name: spec.name.clone(),
            color,
            line_width: cfg.line_width,
            line_dash: cfg.line_dash.clone(),
            smooth: cfg.smooth,
            step: cfg.step.map(|s| match s {
                crate::pipeline::types::StepType::Start => {
                    crate::pipeline::typed_series::StepType::Start
                }
                crate::pipeline::types::StepType::Middle => {
                    crate::pipeline::typed_series::StepType::Middle
                }
                crate::pipeline::types::StepType::End => {
                    crate::pipeline::typed_series::StepType::End
                }
            }),
            area_color,
            area_opacity: cfg.area_opacity,
            symbol_type: map_symbol_type(cfg.symbol_type),
            symbol_size: cfg.symbol_size,
            symbol_step,
            points,
            values,
            baseline_y,
            baseline_points: None,
            label: crate::pipeline::materializer::line_label_config(cfg),
            mark_lines,
            mark_points,
        }))
    }
}

/// `showAllSymbol: 'auto'` 的符号抽稀步长：保证相邻符号中心间距 ≥ 符号直径 + 2px。
///
/// `symbol_size` 在本库中是**半径**量级（`build_symbol` 直接把它当半径使用），
/// 故最小中心间距 = `2 × size + 2`。槽宽放得下时返回 1（全部显示）。
pub(crate) fn auto_symbol_step(symbol_size: f64, count: usize, axis_px: f64) -> usize {
    if count <= 1 || axis_px <= 0.0 {
        return 1;
    }
    let slot = axis_px / count as f64;
    if slot <= 0.0 {
        return 1;
    }
    let min_spacing = (symbol_size * 2.0 + 2.0).max(1.0);
    ((min_spacing / slot).ceil() as usize).max(1)
}

pub(crate) fn map_symbol_type(ty: crate::pipeline::types::SymbolType) -> SymbolType {
    match ty {
        crate::pipeline::types::SymbolType::Circle => SymbolType::Circle,
        crate::pipeline::types::SymbolType::EmptyCircle => SymbolType::EmptyCircle,
        crate::pipeline::types::SymbolType::Rect => SymbolType::Rect,
        crate::pipeline::types::SymbolType::RoundRect => SymbolType::RoundRect,
        crate::pipeline::types::SymbolType::Triangle => SymbolType::Triangle,
        crate::pipeline::types::SymbolType::Diamond => SymbolType::Diamond,
        crate::pipeline::types::SymbolType::Pin => SymbolType::Pin,
        crate::pipeline::types::SymbolType::Arrow => SymbolType::Arrow,
        crate::pipeline::types::SymbolType::None => SymbolType::None,
    }
}

#[cfg(test)]
mod tests {
    use super::auto_symbol_step;

    #[test]
    fn auto_symbol_step_matches_pixel_spacing() {
        // 槽宽 60px 远大于符号直径 10px → 不抽稀
        assert_eq!(auto_symbol_step(4.0, 10, 600.0), 1);
        // 槽宽 6px → ceil(10 / 6) = 2
        assert_eq!(auto_symbol_step(4.0, 100, 600.0), 2);
        // 槽宽 1px → 10
        assert_eq!(auto_symbol_step(4.0, 600, 600.0), 10);
        // 退化输入不 panic、不抽稀
        assert_eq!(auto_symbol_step(4.0, 0, 600.0), 1);
        assert_eq!(auto_symbol_step(4.0, 10, 0.0), 1);
    }
}
