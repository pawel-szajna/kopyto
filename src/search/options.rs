pub struct Options {
    pub white_time: i32,
    pub black_time: i32,
    #[allow(dead_code)]
    pub white_increment: i32,
    #[allow(dead_code)]
    pub black_increment: i32,
    pub target_time: Option<i32>,
    pub depth: Option<i16>,
}

impl Options {
    pub fn new() -> Self {
        Self {
            white_time: i32::MAX,
            black_time: i32::MAX,
            white_increment: 0,
            black_increment: 0,
            target_time: None,
            depth: None,
        }
    }
}
