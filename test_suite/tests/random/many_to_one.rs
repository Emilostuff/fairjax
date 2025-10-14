use test_suite::scenarios::many_to_one::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_many_to_one!(brute_force, BruteForce);
test_suite::declare_many_to_one!(stateful, StatefulTree);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size);
        crate::compare("pairs", messages, brute_force, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn test_many_to_one_mini() {
    let runs = 100;
    let size = 5;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn test_many_to_one_extensive() {
    let runs = 100;
    let size = 20;
    run(runs, size);
}
