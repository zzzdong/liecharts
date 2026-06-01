use crate::{pipeline::dataframe::DataFrame, visual::Color};

pub struct StyleAccess<'a> {
    color_col: Option<&'a crate::pipeline::dataframe::Series>,
    fallback: Color,
}

impl<'a> StyleAccess<'a> {
    pub fn from_df(df: &'a DataFrame, fallback: Color) -> Self {
        let color_col = df.get_column("color");
        Self {
            color_col,
            fallback,
        }
    }

    pub fn color(&self, i: usize) -> Color {
        self.color_col
            .and_then(|c| c.as_color(i))
            .unwrap_or(self.fallback)
    }
}
