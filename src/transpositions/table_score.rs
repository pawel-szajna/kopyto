use crate::search::Score;

#[derive(Clone, Copy, Debug)]
pub enum TableScore {
    Exact(Score),
    LowerBound(Score),
    UpperBound(Score),
}

impl TableScore {
    pub fn from_alpha(alpha: Score, is_exact: bool) -> Self {
        match is_exact {
            true => Self::Exact(alpha),
            false => Self::UpperBound(alpha),
        }
    }
}
