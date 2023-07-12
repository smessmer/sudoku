use criterion::{criterion_group, criterion_main, Criterion};
use sudoku::generate;

fn generate_from_empty(c: &mut Criterion) {
    c.bench_function("generate", |b| b.iter(|| generate()));
}

criterion_group!(
    benches,
    generate_from_empty,
);
criterion_main!(benches);
