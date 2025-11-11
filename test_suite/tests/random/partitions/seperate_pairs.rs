use test_suite::scenarios::separate_pairs::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_separate_pairs!(stateful, StatefulTree);
test_suite::partitions_declare_separate_pairs!(partitions);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("Partitioned separate_pairs", messages, partitions, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn mini() {
    let runs = 30;
    let size = 10;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn extensive() {
    let runs = 500;
    let size = 20;
    run(runs, size);
}
