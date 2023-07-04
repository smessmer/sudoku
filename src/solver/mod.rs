use thiserror::Error;
use bitvec::prelude::*;

use super::board::{NUM_FIELDS, WIDTH, HEIGHT, Board};

const NUM_VALUES_PER_FIELD: usize = 9;

mod solver_board;
use solver_board::SolverBoard;

#[derive(Error, Debug)]
pub enum SolverError {
    #[error("Sudoku is not solvable")]
    NotSolvable,

    #[error("Sudoku has multiple valid solutions")]
    Ambigious,
}

pub fn solve(board: SolverBoard) -> Result<Board, SolverError> {
    // TODO First try faster mechanisms from C++ solver_easy

    match board.board().first_empty_field_index() {
        None => {
            // No empty fields left. The sudoku is fully solved
            // TODO Assert the sudoku is valid
            Ok(*board.board())
        }
        Some((x, y)) => {
            let mut solution = None;
            for value in board.possible_values().possible_values_for_field(x, y) {
                let mut try_board = board;
                try_board.set(x, y, value);
                match solve(try_board) {
                    Ok(new_solution) => {
                        if solution.is_none() {
                            // We found a solution. Remember it but keep checking for others
                            solution = Some(new_solution);
                        } else {
                            // We just found a second solution
                            return Err(SolverError::Ambigious);
                        }
                    },
                    Err(SolverError::Ambigious) => {
                        return Err(SolverError::Ambigious);
                    }
                    Err(SolverError::NotSolvable) => {
                        // This attempt didn't work out. Continue the loop and try other values.
                    }
                }
            }
            match solution {
                Some(solution) => Ok(solution),
                None => Err(SolverError::NotSolvable),
            }
        }
    }
}
