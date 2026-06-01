use crate::pipeline::dataframe::DataFrame;

pub struct GroupInfo<'a> {
    pos_col: Option<&'a crate::pipeline::dataframe::Series>,
    total: usize,
}

impl<'a> GroupInfo<'a> {
    pub fn from_df(df: &'a DataFrame) -> Self {
        let total = df
            .get_column("group_total")
            .and_then(|c| c.as_f64(0))
            .map(|v| v as usize)
            .unwrap_or(1);
        let pos_col = df.get_column("group_position");
        Self { pos_col, total }
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn position(&self, i: usize) -> usize {
        self.pos_col
            .and_then(|c| c.as_f64(i))
            .map(|v| v as usize)
            .unwrap_or(0)
    }

    pub fn center_offset(&self, i: usize) -> f64 {
        let pos = self.position(i) as f64;
        let total = self.total as f64;
        (pos - (total - 1.0) / 2.0) / total
    }
}
