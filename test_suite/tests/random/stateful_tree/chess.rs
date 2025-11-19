use test_suite::scenarios::chess::{WantsToPlay, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_chess!(stateful, StatefulTree);
test_suite::declare_chess!(bruteforce, BruteForce);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("Chess", messages, bruteforce, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn mini() {
    let runs = 100;
    let size = 20;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn extensive() {
    let runs = 100;
    let size = 500;
    run(runs, size);
}
