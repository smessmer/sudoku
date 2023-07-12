use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sudoku::{solve, Board};

fn solve_empty(c: &mut Criterion) {
    let board = Board::new_empty();
    c.bench_function("solve empty", |b| b.iter(|| solve(black_box(board))));
}

fn solve_solvable(c: &mut Criterion) {
    let board = Board::from_str("
        __4 68_ _19
        __3 __9 2_5
        _6_ ___ __4

        6__ ___ 7_2
        ___ __7 ___
        ___ 9__ __1

        8__ _5_ __7
        _41 3_8 ___
        _2_ _91 ___
    ");
    c.bench_function("solve solvable", |b| b.iter(|| solve(black_box(board))));
}

fn solve_not_solvable(c: &mut Criterion) {
    let board = Board::from_str("
        __4 68_ _19
        __3 __9 2_5
        _6_ ___ __4

        6__ ___ 7_2
        ___ _27 ___
        ___ 9__ __1

        8__ _5_ __7
        _41 3_8 ___
        _2_ _91 ___
    ");
    c.bench_function("solve not-solvable", |b| b.iter(|| solve(black_box(board))));
}

fn solve_ambigious(c: &mut Criterion) {
    let board = Board::from_str("
        __4 6__ _19
        __3 __9 2_5
        _6_ ___ __4

        6__ ___ 7_2
        ___ __7 ___
        ___ 9__ __1

        8__ _5_ __7
        _41 3_8 ___
        _2_ _91 ___
    ");
    c.bench_function("solve ambigious", |b| b.iter(|| solve(black_box(board))));
}

criterion_group!(benches, solve_empty, solve_solvable, solve_not_solvable, solve_ambigious);
criterion_main!(benches);