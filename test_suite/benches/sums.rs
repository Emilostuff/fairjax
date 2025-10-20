use criterion::{Criterion, criterion_group, criterion_main};
use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng};
use std::{hint::black_box, ops::Range, time::Duration};
use test_suite::scenarios::pairs::{Msg, generate_random_messages};

test_suite::declare_sums!(run_sums, StatefulTree);

const SEED: u64 = 123;
const N_CASES: usize = 200;
const SIZE_RANGE: Range<usize> = 15..30;

pub static INPUT: Lazy<Vec<Vec<Msg>>> = Lazy::new(|| {
    let mut rng = rand::rngs::StdRng::seed_from_u64(SEED);
    (0..N_CASES)
        .map(|_| generate_random_messages(rng.random_range(SIZE_RANGE), Some(rng.random())))
        .collect()
});

pub fn bench_sums(c: &mut Criterion) {
    c.bench_function("Sums", |b| {
        b.iter(|| {
            for case in INPUT.iter() {
                run_sums(black_box(&case));
            }
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .sample_size(100);
    targets = bench_sums
}
criterion_main!(benches);
