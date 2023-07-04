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
pub fn solve(board: Board) -> Result<Board, SolverError> {
    let board = SolverBoard::new(board);
    _solve(board)
}

fn _solve(board: SolverBoard) -> Result<Board, SolverError> {
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
                match _solve(try_board) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solvable_difficult() {
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
        let expected_solution = Board::from_str("
            274 685 319
            183 749 265
            965 123 874

            618 534 792
            492 817 653
            357 962 481

            839 256 147
            541 378 926
            726 491 538
        ");
        let actual_solution = solve(board).unwrap();
        assert_eq!(expected_solution, actual_solution);
    }
}