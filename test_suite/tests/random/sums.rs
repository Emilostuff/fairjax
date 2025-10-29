use test_suite::scenarios::sums::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_sums!(brute_force, BruteForce);
test_suite::declare_sums!(stateful, StatefulTree);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("sums", messages, brute_force, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn test_sums_mini() {
    let runs = 100;
    let size = 50;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn test_sums_extensive() {
    let runs = 6000;
    let size = 200;
    run(runs, size);
}
