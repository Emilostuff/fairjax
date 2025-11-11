use test_suite::scenarios::groups::{Msg, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_groups!(stateful, StatefulTree);
test_suite::partitions_declare_groups!(partitions);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("Partitioned groups", messages, partitions, stateful);
    }
}

#[test]
/// Mini test to be run always
pub fn mini() {
    let runs = 10;
    let size = 3;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn extensive() {
    let runs = 50;
    let size = 20;
    run(runs, size);
}
