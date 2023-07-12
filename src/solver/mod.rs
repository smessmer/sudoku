use thiserror::Error;

use super::board::Board;

mod possible_values;
use possible_values::PossibleValues;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SolverError {
    #[error("Sudoku is not solvable")]
    NotSolvable,

    #[error("Sudoku has multiple valid solutions")]
    Ambigious,
}

pub fn solve(mut board: Board) -> Result<Board, SolverError> {
    let possible_values = PossibleValues::from_board(&board);
    let solution = _solve(&mut board, possible_values)?;
    assert!(solution.is_filled());
    assert!(!solution.has_conflicts());
    Ok(solution)
}

// Invariant:
//  - When `_solve` returns, `board` is unchanged. Any changes made to `board` during execution need to have been undone.
fn _solve(board: &mut Board, possible_values: PossibleValues) -> Result<Board, SolverError> {
    // TODO First try faster mechanisms from C++ solver_easy

    match board.first_empty_field_index() {
        None => {
            // No empty fields left. The sudoku is fully solved
            Ok(*board)
        }
        Some((x, y)) => {
            let mut solution = None;
            for value in possible_values.possible_values_for_field(x, y) {
                let mut field = board.field_mut(x, y);
                assert!(field.is_empty());
                field.set(Some(value));
                let mut new_possible_values = possible_values;
                new_possible_values.remove_conflicting(x, y, value);
                match _solve(board, new_possible_values) {
                    Ok(new_solution) => {
                        if solution.is_none() {
                            // We found a solution. Remember it but keep checking for others
                            solution = Some(new_solution);
                        } else {
                            // Undo changes to board before returning
                            board.field_mut(x, y).set(None);

                            // We just found a second solution
                            return Err(SolverError::Ambigious);
                        }
                    }
                    Err(SolverError::Ambigious) => {
                        // Undo changes to the board before returning
                        board.field_mut(x, y).set(None);

                        return Err(SolverError::Ambigious);
                    }
                    Err(SolverError::NotSolvable) => {
                        // This attempt didn't work out. Continue the loop and try other values.
                    }
                }

                // Undo changes to the board before next iteration
                board.field_mut(x, y).set(None);
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
        let board = Board::from_str(
            "
            __4 68_ _19
            __3 __9 2_5
            _6_ ___ __4

            6__ ___ 7_2
            ___ __7 ___
            ___ 9__ __1

            8__ _5_ __7
            _41 3_8 ___
            _2_ _91 ___
        ",
        );
        let expected_solution = Board::from_str(
            "
            274 685 319
            183 749 265
            965 123 874

            618 534 792
            492 817 653
            357 962 481

            839 256 147
            541 378 926
            726 491 538
        ",
        );
        let actual_solution = solve(board).unwrap();
        assert!(actual_solution.is_filled());
        assert!(!actual_solution.has_conflicts());
        assert_eq!(expected_solution, actual_solution);
    }

    #[test]
    fn not_solvable_difficult() {
        let board = Board::from_str(
            "
            __4 68_ _19
            __3 __9 2_5
            _6_ ___ __4

            6__ ___ 7_2
            ___ _27 ___
            ___ 9__ __1

            8__ _5_ __7
            _41 3_8 ___
            _2_ _91 ___
        ",
        );
        let actual_solution = solve(board);
        assert_eq!(Err(SolverError::NotSolvable), actual_solution);
    }

    #[test]
    fn ambigious() {
        let board = Board::from_str(
            "
            __4 6__ _19
            __3 __9 2_5
            _6_ ___ __4

            6__ ___ 7_2
            ___ __7 ___
            ___ 9__ __1

            8__ _5_ __7
            _41 3_8 ___
            _2_ _91 ___
        ",
        );
        let actual_solution = solve(board);
        assert_eq!(Err(SolverError::Ambigious), actual_solution);
    }

    #[test]
    fn empty() {
        let board = Board::new_empty();
        let actual_solution = solve(board);
        assert_eq!(Err(SolverError::Ambigious), actual_solution);
    }

    // TODO More tests
}
