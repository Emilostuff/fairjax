pub mod scenarios {
    pub mod many_to_one;
    pub mod pairs;
    pub mod sums;
    pub mod transitive;
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

impl<T: PartialEq + Clone + std::fmt::Debug> std::fmt::Display for MatchTrace<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.pattern_no, self.messages)
    }
}
