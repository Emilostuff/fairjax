use criterion::{BenchmarkId, Criterion, SamplingMode, criterion_group, criterion_main};
use std::hint::black_box;

pub fn pairs(c: &mut Criterion) {
    use test_suite::scenarios::pairs::{Msg, generate_random_messages};
    test_suite::declare_pairs!(run_pairs, StatefulTree);

    let mut group = c.benchmark_group("Pairs");
    for size in [5, 10, 15, 20].iter() {
        group.sampling_mode(SamplingMode::Flat);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_custom(|iter| {
                let cases = (0..iter).map(|i| generate_random_messages(size, Some(i)));

                let start = std::time::Instant::now();
                for case in cases {
                    run_pairs(black_box(&case));
                }
                start.elapsed()
            });
        });
    }
    group.finish();
}

fn custom_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::from_secs(5))
        .warm_up_time(std::time::Duration::from_secs(3))
        .sample_size(10000)
}

criterion_group! {
    name = benches;
    config = custom_criterion();
    targets = pairs
}
criterion_main!(benches);
