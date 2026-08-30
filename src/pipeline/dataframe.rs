use std::collections::HashMap;

use lievisual::Color;

/// 数据值类型
#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Color(Color),
}

impl From<f64> for DataValue {
    fn from(v: f64) -> Self {
        DataValue::Float(v)
    }
}

impl From<f32> for DataValue {
    fn from(v: f32) -> Self {
        DataValue::Float(v as f64)
    }
}

impl From<i32> for DataValue {
    fn from(v: i32) -> Self {
        DataValue::Float(v as f64)
    }
}

impl From<i64> for DataValue {
    fn from(v: i64) -> Self {
        DataValue::Integer(v)
    }
}

impl From<usize> for DataValue {
    fn from(v: usize) -> Self {
        DataValue::Float(v as f64)
    }
}

impl From<bool> for DataValue {
    fn from(v: bool) -> Self {
        DataValue::Bool(v)
    }
}

impl From<&str> for DataValue {
    fn from(v: &str) -> Self {
        DataValue::String(v.to_string())
    }
}

impl From<String> for DataValue {
    fn from(v: String) -> Self {
        DataValue::String(v)
    }
}

impl From<Color> for DataValue {
    fn from(v: Color) -> Self {
        DataValue::Color(v)
    }
}

/// 数据列
#[derive(Debug, Clone)]
pub struct Series {
    pub name: String,
    pub data: Vec<DataValue>,
}

impl Series {
    pub fn new(name: impl Into<String>, data: Vec<DataValue>) -> Self {
        Self {
            name: name.into(),
            data,
        }
    }

    /// 创建包含重复值的列
    pub fn new_constant(name: impl Into<String>, value: DataValue, count: usize) -> Self {
        Self {
            name: name.into(),
            data: vec![value; count],
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 获取 Float 值（用于数值计算）
    pub fn as_f64(&self, index: usize) -> Option<f64> {
        self.data.get(index).and_then(|v| match v {
            DataValue::Float(f) => Some(*f),
            DataValue::Integer(i) => Some(*i as f64),
            _ => None,
        })
    }

    /// 获取 String 值
    pub fn as_string(&self, index: usize) -> Option<String> {
        self.data.get(index).and_then(|v| match v {
            DataValue::String(s) => Some(s.clone()),
            _ => None,
        })
    }

    /// 获取 Color 值
    pub fn as_color(&self, index: usize) -> Option<Color> {
        self.data.get(index).and_then(|v| match v {
            DataValue::Color(c) => Some(*c),
            _ => None,
        })
    }
}

/// DataFrame - 列式数据表
#[derive(Debug, Clone)]
pub struct DataFrame {
    columns: HashMap<String, Series>,
    column_order: Vec<String>,
    row_count: usize,
}

impl DataFrame {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
            column_order: Vec::new(),
            row_count: 0,
        }
    }

    /// 从列创建 DataFrame
    pub fn from_columns(columns: Vec<Series>) -> Self {
        let mut df = Self::new();
        for col in columns {
            df.add_column(col);
        }
        df
    }

    /// 添加列
    pub fn add_column(&mut self, series: Series) {
        let name = series.name.clone();
        if self.row_count == 0 {
            self.row_count = series.len();
        } else if series.len() != self.row_count {
            panic!(
                "Column length mismatch: expected {}, got {}",
                self.row_count,
                series.len()
            );
        }
        if !self.columns.contains_key(&name) {
            self.column_order.push(name.clone());
        }
        self.columns.insert(name, series);
    }

    /// 获取列
    pub fn get_column(&self, name: &str) -> Option<&Series> {
        self.columns.get(name)
    }

    /// 获取可变列
    pub fn get_column_mut(&mut self, name: &str) -> Option<&mut Series> {
        self.columns.get_mut(name)
    }

    /// 行数
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// 列数
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// 获取列名列表（按添加顺序）
    pub fn column_names(&self) -> &[String] {
        &self.column_order
    }

    /// 重命名列
    pub fn rename_column(&mut self, old: &str, new: &str) -> Option<()> {
        let mut series = self.columns.remove(old)?;
        series.name = new.to_string();
        self.columns.insert(new.to_string(), series);
        if let Some(pos) = self.column_order.iter().position(|n| n == old) {
            self.column_order[pos] = new.to_string();
        }
        Some(())
    }

    /// 获取某行某列的值
    pub fn get(&self, row: usize, col: &str) -> Option<&DataValue> {
        self.columns.get(col)?.data.get(row)
    }

    /// 计算新列（基于现有列的函数计算）
    pub fn compute_column<F>(&mut self, name: impl Into<String>, compute: F)
    where
        F: Fn(usize, &DataFrame) -> DataValue,
    {
        let name = name.into();
        let data: Vec<DataValue> = (0..self.row_count).map(|i| compute(i, self)).collect();
        self.add_column(Series::new(name, data));
    }

    /// 从函数创建 DataFrame（用于函数曲线演示）
    ///
    /// 生成两个列：`x_col` 和 `y_col`，其中 x 在 `range` 内均匀取 `steps` 个点，
    /// y = f(x)。
    ///
    /// # Example
    ///
    /// ```
    /// use liecharts::pipeline::dataframe::DataFrame;
    ///
    /// let df = DataFrame::from_function("x", "y", 0.0..=1.0, 100, |x| x * x);
    /// assert_eq!(df.row_count(), 101);
    /// assert!((df.get_column("x").unwrap().as_f64(0).unwrap() - 0.0).abs() < 1e-6);
    /// assert!((df.get_column("y").unwrap().as_f64(50).unwrap() - 0.25).abs() < 1e-2);
    /// ```
    pub fn from_function(
        x_col: &str,
        y_col: &str,
        range: std::ops::RangeInclusive<f64>,
        steps: usize,
        f: impl Fn(f64) -> f64,
    ) -> Self {
        let start = *range.start();
        let end = *range.end();
        let step = if steps > 0 {
            (end - start) / steps as f64
        } else {
            end - start
        };

        let mut df = DataFrame::new();
        let mut x_data = Vec::with_capacity(steps + 1);
        let mut y_data = Vec::with_capacity(steps + 1);

        for i in 0..=steps {
            let x = start + i as f64 * step;
            let y = f(x);
            x_data.push(DataValue::Float(x));
            y_data.push(DataValue::Float(y));
        }

        df.add_column(Series::new(x_col, x_data));
        df.add_column(Series::new(y_col, y_data));
        df
    }
}

impl Default for DataFrame {
    fn default() -> Self {
        Self::new()
    }
}

/// 饼图数据转换器
pub struct PieDataTransformer;

impl PieDataTransformer {
    /// 将原始数据转换为饼图 DataFrame
    /// 输入: category, value 列
    /// 输出: category, value, percent, color, start_angle, end_angle 列
    pub fn transform(df: &DataFrame, palette: &[Color]) -> DataFrame {
        let mut result = df.clone();

        // 计算总值
        let total: f64 = (0..df.row_count())
            .filter_map(|i| df.get_column("value").and_then(|c| c.as_f64(i)))
            .sum();

        // 添加 percent 列
        result.compute_column("percent", |i, df| {
            if let Some(value) = df.get_column("value").and_then(|c| c.as_f64(i)) {
                if total > 0.0 {
                    DataValue::Float(value / total)
                } else {
                    DataValue::Float(0.0)
                }
            } else {
                DataValue::Null
            }
        });

        // 添加 color 列
        result.compute_column("color", |i, _| {
            palette
                .get(i)
                .map(|&c| DataValue::Color(c))
                .unwrap_or(DataValue::Color(Color::rgb(128, 128, 128)))
        });

        // 添加 start_angle, end_angle 列（用于绘制扇区）
        let mut current_angle = -std::f64::consts::FRAC_PI_2; // 从12点方向开始
        let mut start_angles = Vec::new();
        let mut end_angles = Vec::new();

        for i in 0..result.row_count() {
            if let Some(percent) = result.get_column("percent").and_then(|c| c.as_f64(i)) {
                let sweep_angle = 2.0 * std::f64::consts::PI * percent;
                start_angles.push(DataValue::Float(current_angle));
                current_angle += sweep_angle;
                end_angles.push(DataValue::Float(current_angle));
            } else {
                start_angles.push(DataValue::Null);
                end_angles.push(DataValue::Null);
            }
        }

        result.add_column(Series::new("start_angle", start_angles));
        result.add_column(Series::new("end_angle", end_angles));

        result
    }
}

/// 折线图数据转换器
pub struct LineDataTransformer;

impl LineDataTransformer {
    /// 将原始数据转换为折线图 DataFrame
    /// 输入: x, y 列
    /// 输出: x, y, color, point_x, point_y（像素坐标）列
    pub fn transform(
        df: &DataFrame,
        color: Color,
        x_range: (f64, f64),
        y_range: (f64, f64),
        bounds: &vello_cpu::kurbo::Rect,
    ) -> DataFrame {
        let mut result = df.clone();

        // 添加 color 列
        result.compute_column("color", |_, _| DataValue::Color(color));

        // 计算像素坐标
        let x_scale = bounds.width() / (x_range.1 - x_range.0).max(0.001);
        let y_scale = bounds.height() / (y_range.1 - y_range.0).max(0.001);

        result.compute_column("point_x", |i, df| {
            if let Some(x) = df.get_column("x").and_then(|c| c.as_f64(i)) {
                let px = bounds.x0 + (x - x_range.0) * x_scale;
                DataValue::Float(px)
            } else {
                DataValue::Null
            }
        });

        result.compute_column("point_y", |i, df| {
            if let Some(y) = df.get_column("y").and_then(|c| c.as_f64(i)) {
                // Y轴向下为正，需要翻转
                let py = bounds.y1 - (y - y_range.0) * y_scale;
                DataValue::Float(py)
            } else {
                DataValue::Null
            }
        });

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataframe_basic() {
        let mut df = DataFrame::new();
        df.add_column(Series::new(
            "category",
            vec![DataValue::String("A".into()), DataValue::String("B".into())],
        ));
        df.add_column(Series::new(
            "value",
            vec![DataValue::Float(10.0), DataValue::Float(20.0)],
        ));

        assert_eq!(df.row_count(), 2);
        assert_eq!(df.column_count(), 2);

        let col = df.get_column("value").unwrap();
        assert_eq!(col.as_f64(0), Some(10.0));
        assert_eq!(col.as_f64(1), Some(20.0));
    }

    #[test]
    fn test_pie_transformer() {
        let mut df = DataFrame::new();
        df.add_column(Series::new(
            "category",
            vec![DataValue::String("A".into()), DataValue::String("B".into())],
        ));
        df.add_column(Series::new(
            "value",
            vec![DataValue::Float(10.0), DataValue::Float(30.0)],
        ));

        let palette = vec![Color::rgb(99, 132, 255), Color::rgb(255, 159, 67)];

        let result = PieDataTransformer::transform(&df, &palette);

        assert!(result.get_column("percent").is_some());
        assert!(result.get_column("color").is_some());
        assert!(result.get_column("start_angle").is_some());
        assert!(result.get_column("end_angle").is_some());

        // 验证 percent 计算
        let percent_col = result.get_column("percent").unwrap();
        assert!((percent_col.as_f64(0).unwrap() - 0.25).abs() < 0.001);
        assert!((percent_col.as_f64(1).unwrap() - 0.75).abs() < 0.001);
    }
}
