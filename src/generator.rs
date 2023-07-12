use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use super::solver::{SolverError, solve, generate_solved};
use super::board::{Board, HEIGHT, WIDTH};

pub fn generate() -> Board {
    let mut board = generate_solved();
    let mut all_fields: Vec<(u8, u8)> = (0u8..HEIGHT as u8).flat_map(|x| (0u8..WIDTH as u8).map(move |y| (x, y))).collect();
    all_fields.shuffle(&mut rand::thread_rng());
    for (x, y) in all_fields {
        remove_field_if_unambigious(&mut board, x as usize, y as usize);
    }

    assert!(solve(board).is_ok());
    board
}

pub fn generate_max_empty() -> Board {
    let board = generate_solved();
    let board = remove_max(board);
    assert!(solve(board).is_ok());
    board
}

fn remove_max(board: Board) -> Board {
    let best_board = Arc::new(Mutex::new((board.num_empty(), board)));
    _remove_max(board, Arc::clone(&best_board));
    let best_board = best_board.lock().unwrap();
    best_board.1
}

fn _remove_max(board: Board, best_board: Arc<Mutex<(usize, Board)>>) {
    {
        let num_empty = board.num_empty();
        let mut prev_best = best_board.lock().unwrap();
        if num_empty > prev_best.0 {
            println!("Found board with {num_empty} empty fields");
            prev_best.0 = num_empty;
            prev_best.1 = board;
        }
        // and drop the lock
    }

    let mut all_fields: Vec<(u8, u8)> = (0u8..HEIGHT as u8).flat_map(|x| (0u8..WIDTH as u8).map(move |y| (x, y))).collect();
    all_fields.shuffle(&mut rand::thread_rng());
    all_fields.par_iter().for_each(move |(x, y)| {
        let mut board = board;
        if remove_field_if_unambigious(&mut board, *x as usize, *y as usize) {
            _remove_max(board, Arc::clone(&best_board));
        }
    });
}

fn remove_field_if_unambigious(board: &mut Board, x: usize, y: usize) -> bool {
    let mut field = board.field_mut(x, y);
    let value = field.get();
    if value.is_none() {
        return false;
    }
    field.set(None);
    if is_ambigious(*board) {
        board.field_mut(x, y).set(value);
        false
    } else {
        true
    }
}

fn is_ambigious(board: Board) -> bool {
    match solve(board) {
        Err(SolverError::Conflicting) => panic!("Board is conflicting"),
        Err(SolverError::NotSolvable) => panic!("Board is not solvable"),
        Err(SolverError::Ambigious) => true,
        Ok(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_10() {
        for _ in 0..10 {
            let board = generate();
            assert!(solve(board).is_ok());
            assert!(board.num_empty() > 0);
        }
    }

    // TODO More tests
}
