use test_suite::scenarios::shuffled_pairs::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_shuffled_pairs!(brute_force, BruteForce);
test_suite::declare_shuffled_pairs!(stateful, StatefulTree);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size);
        crate::compare(messages, brute_force, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn test_shuffled_pairs_mini() {
    let runs = 10;
    let size = 10;
    run(runs, size);
}

#[test]
#[ignore]
//#[serial_test::serial]
/// Extensive test to be run on-demand
pub fn test_shuffled_pairs_extensive() {
    let runs = 500;
    let size = 10;
    run(runs, size);
}
