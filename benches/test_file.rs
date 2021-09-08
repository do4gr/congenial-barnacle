use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine_lib::core_logic;

/// was used to iterate on the performance of the sync version
pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("bench core logic", |b| {
        b.iter(|| core_logic(black_box("./files/large_test_file.csv")).unwrap())
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(20);
    targets = criterion_benchmark
);
criterion_main!(benches);
