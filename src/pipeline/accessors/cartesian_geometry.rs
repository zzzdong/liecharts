use vello_cpu::kurbo;

use crate::{
    error::{ChartError, Result},
    pipeline::dataframe::{DataFrame, Series},
};

pub struct CartesianGeometry<'a> {
    px_col: &'a Series,
    py_col: &'a Series,
    pbase_col: Option<&'a Series>,
}

impl<'a> CartesianGeometry<'a> {
    pub fn from_df(df: &'a DataFrame) -> Result<Self> {
        let px_col = df
            .get_column("px")
            .ok_or_else(|| ChartError::DataError("DataFrame missing 'px' column".into()))?;
        let py_col = df
            .get_column("py")
            .ok_or_else(|| ChartError::DataError("DataFrame missing 'py' column".into()))?;
        let pbase_col = df.get_column("pbase");
        Ok(Self {
            px_col,
            py_col,
            pbase_col,
        })
    }

    pub fn row_count(&self) -> usize {
        self.px_col.len()
    }

    pub fn px(&self, i: usize) -> f64 {
        self.px_col.as_f64(i).unwrap_or(0.0)
    }

    pub fn py(&self, i: usize) -> f64 {
        self.py_col.as_f64(i).unwrap_or(0.0)
    }

    pub fn pbase(&self, i: usize, fallback: f64) -> f64 {
        self.pbase_col.and_then(|c| c.as_f64(i)).unwrap_or(fallback)
    }

    pub fn point(&self, i: usize) -> kurbo::Point {
        kurbo::Point::new(self.px(i), self.py(i))
    }

    pub fn collect_points(&self) -> Vec<kurbo::Point> {
        let mut points = Vec::new();
        for i in 0..self.px_col.len() {
            if let (Some(px), Some(py)) = (self.px_col.as_f64(i), self.py_col.as_f64(i)) {
                points.push(kurbo::Point::new(px, py));
            }
        }
        points
    }
}
