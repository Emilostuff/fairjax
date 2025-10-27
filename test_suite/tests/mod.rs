pub mod random {
    pub mod equal_cases;
    pub mod groups;
    pub mod many_to_one;
    pub mod pairs;
    pub mod permutations;
    pub mod sums;
    pub mod transitive;
    pub mod workers;
}

use test_suite::MatchTrace;

fn compare<T: PartialEq + Clone + std::fmt::Debug>(
    title: &str,
    messages: Vec<T>,
    oracle: fn(&[T]) -> Vec<MatchTrace<T>>,
    test_subject: fn(&[T]) -> Vec<MatchTrace<T>>,
) {
    let expected = oracle(&messages);
    let actual = test_subject(&messages);

    if expected != actual {
        panic!(
            "Test: '{title}' FAILED!\nINPUT:\n\t{:?}\nEXPECTED MATCHES:\n{}\nACTUAL MATCHES:\n{}",
            &messages,
            expected
                .iter()
                .map(|m| format!("\t{}", m))
                .collect::<Vec<_>>()
                .join("\n"),
            actual
                .iter()
                .map(|m| format!("\t{}", m))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
}
