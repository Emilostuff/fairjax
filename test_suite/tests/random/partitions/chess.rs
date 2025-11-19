use test_suite::scenarios::chess::{WantsToPlay, generate_random_messages};

// Declare runners for each strategy
test_suite::declare_chess!(stateful, StatefulTree);
test_suite::partitions_declare_chess!(partitions);

/// Declare top-level test runner
fn run(n_runs: usize, size: usize) {
    for _ in 0..n_runs {
        let messages = generate_random_messages(size, None);
        crate::compare("Partitioned chess", messages, stateful, partitions);
    }
}

#[test]
/// Mini test to be run always
pub fn mini() {
    let runs = 100;
    let size = 100;
    run(runs, size);
}

#[test]
#[ignore]
/// Extensive test to be run on-demand
pub fn extensive() {
    let runs = 100;
    let size = 10000;
    run(runs, size);
}
