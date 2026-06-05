use vello_cpu::kurbo::{Point, Rect};

use crate::{
    error::Result,
    option::SeriesOption,
    pipeline::{
        data_processor::{DataProcessor, DataProcessorInput},
        dataframe::{DataFrame, DataValue, Series},
        sampling::SamplingProcessor,
        types::SeriesSpec,
    },
    visual::{FillStrokeStyle, Stroke, VisualElement, Z_SERIES_FILL, Z_SERIES_LINE},
};

pub struct CandlestickProcessor;

impl Default for CandlestickProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CandlestickProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl DataProcessor for CandlestickProcessor {
    fn process_from_spec(
        &self,
        series: &SeriesSpec,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let mut df = series.data.clone();

        // 应用采样（如果配置了）
        if let Some((sampling_type, threshold)) = &series.sampling {
            df = SamplingProcessor::sample(&df, *threshold, *sampling_type);
        }

        let bounds = input.bounds;
        let x_axis_idx = input.spec.x_axis_indices.first().copied().unwrap_or(0);
        let y_axis_idx = series.y_axis_index;

        let x_range = input.axis_ranges.get_x_range(x_axis_idx);
        let y_range = input.axis_ranges.get_y_range(y_axis_idx);

        let (_, x_max) = x_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 1.0));
        let (y_min, y_max) = y_range.map(|r| (r.min, r.max)).unwrap_or((0.0, 100.0));

        let data_len = df.row_count().max(1);
        let cat_count = (x_max - 0.0).max(1.0);
        let cat_width = bounds.width() / cat_count;
        let bar_width = cat_width * 0.6;

        let mut px_values = Vec::new();
        let mut open_y_values = Vec::new();
        let mut close_y_values = Vec::new();
        let mut low_y_values = Vec::new();
        let mut high_y_values = Vec::new();

        for i in 0..df.row_count() {
            let px = bounds.x0 + (i as f64 + 0.5) / data_len as f64 * bounds.width();
            let y_scale = bounds.height() / (y_max - y_min).max(0.001);

            let open = df
                .get_column("open")
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let close = df
                .get_column("close")
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let low = df
                .get_column("low")
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);
            let high = df
                .get_column("high")
                .and_then(|c| c.as_f64(i))
                .unwrap_or(0.0);

            let open_y = bounds.y1 - (open - y_min) * y_scale;
            let close_y = bounds.y1 - (close - y_min) * y_scale;
            let low_y = bounds.y1 - (low - y_min) * y_scale;
            let high_y = bounds.y1 - (high - y_min) * y_scale;

            px_values.push(DataValue::Float(px));
            open_y_values.push(DataValue::Float(open_y));
            close_y_values.push(DataValue::Float(close_y));
            low_y_values.push(DataValue::Float(low_y));
            high_y_values.push(DataValue::Float(high_y));
        }

        df.add_column(Series::new("px", px_values));
        df.add_column(Series::new("open_y", open_y_values));
        df.add_column(Series::new("close_y", close_y_values));
        df.add_column(Series::new("low_y", low_y_values));
        df.add_column(Series::new("high_y", high_y_values));
        df.add_column(Series::new_constant(
            "bar_width",
            DataValue::Float(bar_width),
            data_len,
        ));

        self.to_visual_elements(&df, input)
    }

    fn to_dataframe(
        &self,
        _series: &SeriesOption,
        _input: &DataProcessorInput,
    ) -> Result<DataFrame> {
        // 在新 API 中，数据已经在 DataFrame 中
        Ok(DataFrame::new())
    }

    fn transform(&self, df: DataFrame, _input: &DataProcessorInput) -> Result<DataFrame> {
        Ok(df)
    }

    fn to_visual_elements(
        &self,
        df: &DataFrame,
        input: &DataProcessorInput,
    ) -> Result<Vec<VisualElement>> {
        let up_fill = input.colors.up_color;
        let down_fill = input.colors.down_color;
        let border_color = input.colors.text_color;

        let px_col = df.get_column("px");
        let open_y_col = df.get_column("open_y");
        let close_y_col = df.get_column("close_y");
        let low_y_col = df.get_column("low_y");
        let high_y_col = df.get_column("high_y");
        let is_up_col = df.get_column("is_up");
        let bar_width_col = df.get_column("bar_width");

        let mut elements = Vec::new();
        let row_count = df.row_count();

        for i in 0..row_count {
            let px = px_col.and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let open_y = open_y_col.and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let close_y = close_y_col.and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let low_y = low_y_col.and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let high_y = high_y_col.and_then(|c| c.as_f64(i)).unwrap_or(0.0);
            let is_up = is_up_col
                .and_then(|c| match c.data.get(i) {
                    Some(DataValue::Bool(b)) => Some(*b),
                    _ => None,
                })
                .unwrap_or(false);
            let bar_width = bar_width_col.and_then(|c| c.as_f64(0)).unwrap_or(20.0);

            let fill_color = if is_up { up_fill } else { down_fill };
            let body_top = open_y.min(close_y);
            let body_height = (open_y - close_y).abs().max(1.0);
            let half_w = bar_width / 2.0;

            elements.push(VisualElement::Line {
                start: Point::new(px, high_y),
                end: Point::new(px, low_y),
                style: crate::visual::StrokeStyle {
                    color: border_color,
                    width: 1.0,
                },
                z_index: Z_SERIES_LINE,
            });

            elements.push(VisualElement::Rect {
                rect: Rect::new(px - half_w, body_top, px + half_w, body_top + body_height),
                style: FillStrokeStyle {
                    fill: Some(fill_color),
                    stroke: Some(Stroke {
                        color: border_color,
                        width: 1.0,
                    }),
                },
                z_index: Z_SERIES_FILL,
            });
        }

        Ok(elements)
    }
}
