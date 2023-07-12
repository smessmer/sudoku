use criterion::{criterion_group, criterion_main, Criterion};
use sudoku::{generate_solved, generate};

fn bench_generate_solved(c: &mut Criterion) {
    c.bench_function("generate solved", |b| b.iter(|| generate_solved()));
}

fn bench_generate_unsolved(c: &mut Criterion) {
    c.bench_function("generate unsolved", |b| b.iter(|| generate()));
}

criterion_group!(
    benches,
    bench_generate_solved,
    bench_generate_unsolved,
);
criterion_main!(benches);
