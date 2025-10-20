use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// pub fn many_to_one(c: &mut Criterion) {
//     use test_suite::scenarios::many_to_one::{Msg, generate_random_messages};
//     test_suite::declare_many_to_one!(run_many_to_one, StatefulTree);

//     let mut group = c.benchmark_group("Many To One");
//     for size in [1, 2, 3, 4, 5, 6].iter() {
//         group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
//             b.iter_custom(|iters| {
//                 let cases = (0..iters).map(|i| generate_random_messages(size, Some(i)));

//                 let start = Instant::now();
//                 for case in cases {
//                     run_many_to_one(black_box(&case));
//                 }
//                 start.elapsed()
//             });
//         });
//     }
//     group.finish();
// }

pub fn many_to_one(c: &mut Criterion) {
    use test_suite::scenarios::many_to_one::{Msg, generate_random_messages};
    test_suite::declare_many_to_one!(run_many_to_one, StatefulTree);

    let mut group = c.benchmark_group("Many To One");
    for size in [2, 4, 6].iter() {
        let mut i = 0;
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || {
                    i += 1;
                    generate_random_messages(size, Some(i))
                },
                |msgs| run_many_to_one(black_box(&msgs)),
            );
        });
    }
    group.finish();
}

// criterion_group! {
//     name = benches;
//     config = Criterion::default()
//         .measurement_time(std::time::Duration::from_secs(5))
//         .warm_up_time(std::time::Duration::from_secs(1))
//         .sample_size(500);
//     targets = many_to_one
// }
// criterion_main!(benches);
