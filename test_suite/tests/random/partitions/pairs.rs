use test_suite::scenarios::pairs::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_pairs!(stateful, StatefulTree);
test_suite::partitions_declare_pairs!(partitions);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("Partitioned pairs", messages, stateful, partitions);
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
    let runs = 4000;
    let size = 60;
    run(runs, size);
}
