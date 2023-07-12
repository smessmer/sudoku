use thiserror::Error;
use std::num::NonZeroU8;

use super::board::{WIDTH, FieldRef, HEIGHT, MAX_VALUE, Board};

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

    if let Some((mut board, possible_values)) = _solve_fast(*board, possible_values)? {
        // Note: calling _solve here means that in it, we re-run _solve_fast again. It's possible that it'll find more things based on the changed board.
        return _solve(&mut board, possible_values);
    }

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
                debug_assert!(!board.has_conflicts());
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

/// [_solve_fast] tries some fast strategies to add values on the board that can easily be deduced from other values.
/// It returns
/// - `Ok(Some((board, possible_values)))` if it found something and the board was changed
/// - `Ok(None)` if it found nothing (this doesn't mean that the board is unsolvable, just that the fast strategy failed)
/// - `Err(SolverError)` if the board is unsolvable
fn _solve_fast(mut board: Board, mut possible_values: PossibleValues) -> Result<Option<(Board, PossibleValues)>, SolverError> {
    let mut found_something = false;

    // Check each row for values that can only be placed in one field
    for row in 0u8..HEIGHT as u8 {
        let cells = (0u8..WIDTH as u8).map(|x| (x, row));
        if _solve_fast_fields(&mut board, &mut possible_values, cells)? {
            found_something = true;
        }
    }

    // Check each col for values that can only be placed in one field
    for col in 0u8..WIDTH as u8 {
        let cells = (0u8..HEIGHT as u8).map(|y| (col, y));
        if _solve_fast_fields(&mut board, &mut possible_values, cells)? {
            found_something = true;
        }
    }

    // Check each 3x3 cell for values that can only be placed in one field
    for cell_x in 0u8..3u8 {
        for cell_y in 0u8..3u8 {
            let cells = (0u8..3u8).flat_map(move |x| (0u8..3u8).map(move |y| (cell_x * 3 + x, cell_y * 3 + y)));
            if _solve_fast_fields(&mut board, &mut possible_values, cells)? {
                found_something = true;
            }
        }
    }

    if found_something {
        Ok(Some((board, possible_values)))
    } else {
        Ok(None)
    }
}

#[must_use]
fn _solve_fast_fields(board: &mut Board, possible_values: &mut PossibleValues, field_coords: impl Iterator<Item = (u8, u8)>) -> Result<bool, SolverError> {
    // Algorithm: Go through one row (or col or cell, based on `field_coords`) and check for each value, if it has only one possible position in this row.

    #[derive(Clone, Copy, Debug)]
    enum ValueInfo {
        NoPossiblePlacementFound,
        AlreadyPlaced,
        CanOnlyBePlacedAtIndex((u8, u8)),
        MultiplePossiblePlacementsFound,
    }
    // value_infos stores for each value, which row index it can be placed in.
    let mut value_infos = [ValueInfo::NoPossiblePlacementFound; MAX_VALUE as usize];


    // First, loop over all fields in the row and check which values are already placed and which values can be placed in which fields.
    for (x, y) in field_coords {
        let field = board.field(x as usize, y as usize);
        if let Some(value) = field.get() {
            // This field is already filled. Remember that the value is taken.
            let entry = &mut value_infos[value.get() as usize - 1];
            match *entry {
                ValueInfo::NoPossiblePlacementFound => 
                    *entry = ValueInfo::AlreadyPlaced,
                ValueInfo::AlreadyPlaced | ValueInfo::CanOnlyBePlacedAtIndex(_) | ValueInfo::MultiplePossiblePlacementsFound => panic!("Something's wrong with our PossibleValues. They shouldn't list any values that are already placed. Board: {board:?}, cell: {x}/{y}, value: {value}, info: {entry:?}"),
            }
        } else {
            // This field isn't filled yet. Remember which values could be placed here.
            for value in possible_values.possible_values_for_field(x as usize, y as usize) {
                match value_infos[value.get() as usize - 1] {
                    ValueInfo::NoPossiblePlacementFound => {
                        value_infos[value.get() as usize - 1] = ValueInfo::CanOnlyBePlacedAtIndex((x, y));
                    }
                    ValueInfo::AlreadyPlaced => panic!("Something's wrong with our PossibleValues. They shouldn't list any values that are already placed"),
                    ValueInfo::CanOnlyBePlacedAtIndex(_) => {
                        value_infos[value.get() as usize - 1] = ValueInfo::MultiplePossiblePlacementsFound;
                    }
                    ValueInfo::MultiplePossiblePlacementsFound => {}
                }
            }
        }
    }

    // Second, check for each value whether it can be placed in only one field.
    let mut found_something = false;
    for value in 1u8..MAX_VALUE {
        let value = NonZeroU8::new(value as u8).unwrap();
        match value_infos[value.get() as usize - 1] {
            ValueInfo::NoPossiblePlacementFound => {
                return Err(SolverError::NotSolvable);
            }
            ValueInfo::CanOnlyBePlacedAtIndex((x, y)) => {
                let x = x as usize;
                let y = y as usize;
                let mut field = board.field_mut(x, y);
                if !field.is_empty() {
                    // We just filled this field in a previous iteration. This means there are two values that need to go here, this is impossible
                    return Err(SolverError::NotSolvable)
                }
                field.set(Some(value));
                possible_values.remove_conflicting(x, y, value);
                debug_assert!(!board.has_conflicts());
                found_something = true;
            }
            ValueInfo::AlreadyPlaced | ValueInfo::MultiplePossiblePlacementsFound => {}
        }
    }

    Ok(found_something)
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
