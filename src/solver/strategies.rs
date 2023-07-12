use std::num::NonZeroU8;

use crate::board::{WIDTH, HEIGHT, MAX_VALUE, Board};
use super::possible_values::PossibleValues;
use super::SolverError;

/// [solve_simple_strategies] tries some fast strategies to add values on the board that can easily be deduced from other values.
/// It returns
/// - `Ok(Some((board, possible_values)))` if it found something and the board was changed
/// - `Ok(None)` if it found nothing (this doesn't mean that the board is unsolvable, just that the fast strategy failed)
/// - `Err(SolverError)` if the board is unsolvable
pub fn solve_simple_strategies(mut board: Board, mut possible_values: PossibleValues) -> Result<Option<(Board, PossibleValues)>, SolverError> {
    if solve_hidden_candidates(&mut board, &mut possible_values)? {
        Ok(Some((board, possible_values)))
    } else {
        Ok(None)
    }
}

/// [solve_hidden_candidates] tries to fill hidden candidates, i.e. values that only have one possible position in a row, column or 3x3 region.
/// It returns
/// - `Ok(true)` if it found something and the board was changed
/// - `Ok(false)` if it found nothing (this doesn't mean that the board is unsolvable, just that the strategy failed)
/// - `Err(SolverError)` if the board is unsolvable
fn solve_hidden_candidates(board: &mut Board, possible_values: &mut PossibleValues) -> Result<bool, SolverError> {
    let mut found_something = false;

    // Check each row for values that can only be placed in one field
    for row in 0u8..HEIGHT as u8 {
        let cells = (0u8..WIDTH as u8).map(|x| (x, row));
        if _solve_hidden_candidates(board, possible_values, cells)? {
            found_something = true;
        }
    }

    // Check each col for values that can only be placed in one field
    for col in 0u8..WIDTH as u8 {
        let cells = (0u8..HEIGHT as u8).map(|y| (col, y));
        if _solve_hidden_candidates(board, possible_values, cells)? {
            found_something = true;
        }
    }

    // Check each 3x3 region for values that can only be placed in one field
    for region_x in 0u8..3u8 {
        for region_y in 0u8..3u8 {
            let cells = (0u8..3u8).flat_map(move |x| (0u8..3u8).map(move |y| (region_x * 3 + x, region_y * 3 + y)));
            if _solve_hidden_candidates(board, possible_values, cells)? {
                found_something = true;
            }
        }
    }

    Ok(found_something)
}

#[must_use]
fn _solve_hidden_candidates(board: &mut Board, possible_values: &mut PossibleValues, field_coords: impl Iterator<Item = (u8, u8)> + Clone) -> Result<bool, SolverError> {
    let mut found_something = false;

    'outer: for value in 1u8..=MAX_VALUE {
        let value = NonZeroU8::new(value).unwrap();
        let mut placement = None;
        for (x, y) in field_coords.clone() {
            if let Some(current_value) = board.field(x as usize, y as usize).get() {
                if current_value == value {
                    // We found a field that already has the current value, no need to check other fields for it
                    continue 'outer;
                }
            } else {
                if possible_values.is_possible(x as usize, y as usize, value) {
                    if placement.is_none() {
                        placement = Some((x, y));
                    } else {
                        // We found a second field where the value can be placed. No need to check other fields for it
                        continue 'outer;
                    }
                }
            }
        }

        if let Some((x, y)) = placement {
            // We found exactly one place where we can put this value
            let x = x as usize;
            let y = y as usize;
            board.field_mut(x, y).set(Some(value));
            possible_values.remove_conflicting(x, y, value);
            found_something = true;
            debug_assert!(!board.has_conflicts());
        } else {
            // We found no place where we can put this value
            return Err(SolverError::NotSolvable);
        }
    }

    Ok(found_something)
}
