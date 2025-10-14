pub mod scenarios {
    pub mod shuffled_pairs;
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchTrace<T: PartialEq + Clone + std::fmt::Debug> {
    pattern_no: usize,
    messages: Vec<T>,
}

impl<T: PartialEq + Clone + std::fmt::Debug> MatchTrace<T> {
    pub fn new(pattern_no: usize, messages: Vec<T>) -> Self {
        MatchTrace {
            pattern_no,
            messages,
        }
    }
}
