use test_suite::scenarios::pairs::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_pairs!(brute_force, BruteForce);
test_suite::declare_pairs!(stateful, StatefulTree);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("pairs", messages, brute_force, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn test_pairs_mini() {
    let runs = 100;
    let size = 20;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn test_pairs_extensive() {
    let runs = 4000;
    let size = 60;
    run(runs, size);
}
