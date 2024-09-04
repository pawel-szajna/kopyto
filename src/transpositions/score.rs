#[derive(Clone, Copy, Debug)]
pub enum Score {
    Exact(i64),
    LowerBound(i64),
    UpperBound(i64),
}

impl Score {
    pub fn from_alpha(alpha: i64, is_exact: bool) -> Self {
        match is_exact {
            true => Self::Exact(alpha),
            false => Self::UpperBound(alpha),
        }
    }
}
