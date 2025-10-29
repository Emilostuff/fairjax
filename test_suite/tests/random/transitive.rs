use test_suite::scenarios::transitive::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_transitive!(brute_force, BruteForce);
test_suite::declare_transitive!(stateful, StatefulTree);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("transitive", messages, brute_force, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn test_transitive_mini() {
    let runs = 100;
    let size = 50;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn test_transitive_extensive() {
    let runs = 10000;
    let size = 100;
    run(runs, size);
}
