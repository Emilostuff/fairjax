use test_suite::scenarios::groups::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_groups!(brute_force, BruteForce);
test_suite::declare_groups!(stateful, StatefulTree);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("groups", messages, brute_force, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn test_groups_mini() {
    let runs = 10;
    let size = 3;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn test_groups_extensive() {
    let runs = 25;
    let size = 5;
    run(runs, size);
}
