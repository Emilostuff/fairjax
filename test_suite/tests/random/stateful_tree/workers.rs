use test_suite::scenarios::workers::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_workers!(brute_force, BruteForce);
test_suite::declare_workers!(stateful, StatefulTree);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("workers", messages, brute_force, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn mini() {
    let runs = 20;
    let size = 5;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn extensive() {
    let runs = 300;
    let size = 20;
    run(runs, size);
}
