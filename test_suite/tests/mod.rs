pub mod random {
    pub mod shuffled_pairs;
}

use test_suite::MatchTrace;

fn compare<T: PartialEq + Clone + std::fmt::Debug>(
    messages: Vec<T>,
    oracle: fn(&[T]) -> Vec<MatchTrace<T>>,
    test_subject: fn(&[T]) -> Vec<MatchTrace<T>>,
) {
    let expected = oracle(&messages);
    let actual = test_subject(&messages);
    assert_eq!(expected, actual);
}
