use thiserror::Error;

use super::board::Board;

mod possible_values;

mod solver;
mod strategies;
use solver::{Generator, Solver};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SolverError {
    #[error("Sudoku is not solvable")]
    NotSolvable,

    #[error("Sudoku has multiple valid solutions")]
    Ambigious,
}

pub fn solve(board: Board) -> Result<Board, SolverError> {
    let mut solver = Solver::new(board);
    match solver.next_solution() {
        None => Err(SolverError::NotSolvable),
        Some(solution) => {
            assert!(board.is_subset_of(&solution));
            if let Some(solution2) = solver.next_solution() {
                assert!(board.is_subset_of(&solution2));
                assert_ne!(solution, solution2);
                Err(SolverError::Ambigious)
            } else {
                assert!(solution.is_filled());
                assert!(!solution.has_conflicts());
                Ok(solution)
            }
        }
    }
}

pub fn generate() -> Board {
    let mut generator = Generator::new(Board::new_empty());
    generator.next_solution().unwrap()
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

    #[test]
    fn generate_some() {
        for _ in 0..100 {
            let solution = generate();
            assert!(solution.is_filled());
            assert!(!solution.has_conflicts());
        }
    }
}
