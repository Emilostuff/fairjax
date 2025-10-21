use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng};
use std::{hint::black_box, ops::Range, time::Duration};
use test_suite::scenarios::transitive::{Msg, generate_random_messages};

test_suite::declare_transitive!(run_transitive, StatefulTree);

const SEED: u64 = 123;
const N_CASES: usize = 200;
const SIZE_RANGE: Range<usize> = 30..50;

pub static INPUT: Lazy<Vec<Vec<Msg>>> = Lazy::new(|| {
    let mut rng = rand::rngs::StdRng::seed_from_u64(SEED);
    (0..N_CASES)
        .map(|_| generate_random_messages(rng.random_range(SIZE_RANGE), Some(rng.random())))
        .collect()
});

pub fn bench_transitive(c: &mut Criterion) {
    let mut group = c.benchmark_group("Random");
    group.throughput(Throughput::Elements(
        INPUT.iter().flat_map(|v| v).collect::<Vec<&Msg>>().len() as u64,
    ));
    group.bench_function("Transitive", |b| {
        b.iter(|| {
            for case in INPUT.iter() {
                run_transitive(black_box(&case));
            }
        })
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .sample_size(100);
    targets = bench_transitive
}
criterion_main!(benches);
