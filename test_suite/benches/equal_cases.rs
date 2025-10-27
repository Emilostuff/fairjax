use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng};
use std::{hint::black_box, ops::Range, time::Duration};
use test_suite::scenarios::equal_cases::{Msg, generate_random_messages};

test_suite::declare_equal_cases!(run_equal_cases, StatefulTree, [0, 1, 2, 3, 4, 5, 6, 7]);

const SEED: u64 = 123;
const N_CASES: usize = 100;
const SIZE_RANGE: Range<usize> = 2..5;

pub static INPUT: Lazy<Vec<Vec<Msg>>> = Lazy::new(|| {
    let mut rng = rand::rngs::StdRng::seed_from_u64(SEED);
    (0..N_CASES)
        .map(|_| generate_random_messages(rng.random_range(SIZE_RANGE), Some(rng.random())))
        .collect()
});

pub fn bench_equal_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("Random");
    group.throughput(Throughput::Elements(
        INPUT.iter().flat_map(|v| v).collect::<Vec<&Msg>>().len() as u64,
    ));
    group.bench_function("equal_cases", |b| {
        b.iter(|| {
            for case in INPUT.iter() {
                run_equal_cases(black_box(&case));
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
    targets = bench_equal_cases
}
criterion_main!(benches);
