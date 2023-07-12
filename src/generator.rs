use rand::seq::SliceRandom;

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

fn remove_field_if_unambigious(board: &mut Board, x: usize, y: usize) -> bool {
    let mut field = board.field_mut(x, y);
    let value = field.get();
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
