use test_suite::scenarios::permutations::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_permutations!(brute_force, BruteForce);
test_suite::declare_permutations!(stateful, StatefulTree);

/// Declare top-level test runner
fn run(n_runs: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(None);
        crate::compare("permutations", messages, brute_force, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn mini() {
    let runs = 20;
    run(runs);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn extensive() {
    let runs = 6000;
    run(runs);
}
