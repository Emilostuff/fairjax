use test_suite::scenarios::equal_cases::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_equal_cases!(brute_force, BruteForce, [0, 1, 2]);
test_suite::declare_equal_cases!(stateful, StatefulTree, [0, 1, 2]);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("equal_cases", messages, brute_force, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn test_equal_cases_mini() {
    let runs = 10;
    let size = 5;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn test_equal_cases_extensive() {
    let runs = 400;
    let size = 20;
    run(runs, size);
}
