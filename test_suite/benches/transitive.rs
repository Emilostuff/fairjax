use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

pub fn transitive(c: &mut Criterion) {
    use test_suite::scenarios::transitive::{Msg, generate_random_messages};
    test_suite::declare_transitive!(run_transitive, StatefulTree);

    let mut group = c.benchmark_group("Transitive");
    for size in [1, 2, 3, 4, 5].iter() {
        let mut i = 0;
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || {
                    i += 1;
                    generate_random_messages(size, Some(i))
                },
                |msgs| run_transitive(black_box(&msgs)),
            );
        });
    }
    group.finish();
}

fn custom_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::from_secs(1))
        .warm_up_time(std::time::Duration::from_millis(200))
        .sample_size(100)
}

criterion_group! {
    name = benches;
    config = custom_criterion();
    targets = transitive
}
criterion_main!(benches);
