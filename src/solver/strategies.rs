use std::num::NonZeroU8;

use super::possible_values::PossibleValues;
use crate::board::{Board, HEIGHT, MAX_VALUE, WIDTH};

pub enum SimpleSolverResult {
    FoundSomething {
        board: Board,
        possible_values: PossibleValues,
    },
    FoundNothing,
    NotSolvable,
}

/// [solve_simple_strategies] tries some fast strategies to add values on the board that can easily be deduced from other values.
/// It returns
pub fn solve_simple_strategies(
    mut board: Board,
    mut possible_values: PossibleValues,
) -> SimpleSolverResult {
    match solve_hidden_candidates(&mut board, &mut possible_values) {
        Some(true) => SimpleSolverResult::FoundSomething {
            board,
            possible_values,
        },
        Some(false) => SimpleSolverResult::FoundNothing,
        None => return SimpleSolverResult::NotSolvable,
    }

    // TODO fill_known_values() (i.e. find fields that only have one possible value in PossibleValues and fill them). This also fixes the TODO in solver.rs to abort early when a field has no possible values left.
}

/// [solve_hidden_candidates] tries to fill hidden candidates, i.e. values that only have one possible position in a row, column or 3x3 region.
/// It returns
/// - `Some(true)` if it found something and the board was changed
/// - `Some(false)` if it found nothing (this doesn't mean that the board is unsolvable, just that the strategy failed)
/// - `None` if the board is unsolvable
fn solve_hidden_candidates(
    board: &mut Board,
    possible_values: &mut PossibleValues,
) -> Option<bool> {
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
            let cells = (0u8..3u8)
                .flat_map(move |x| (0u8..3u8).map(move |y| (region_x * 3 + x, region_y * 3 + y)));
            if _solve_hidden_candidates(board, possible_values, cells)? {
                found_something = true;
            }
        }
    }

    Some(found_something)
}

#[must_use]
fn _solve_hidden_candidates(
    board: &mut Board,
    possible_values: &mut PossibleValues,
    field_coords: impl Iterator<Item = (u8, u8)> + Clone,
) -> Option<bool> {
    let mut found_something = false;

    'outer: for value in 1u8..=MAX_VALUE {
        let value = NonZeroU8::new(value).unwrap();
        let mut placement = None;

        // Find the place(s) where we can put this value
        for (x, y) in field_coords.clone() {
            if let Some(current_value) = board.field(x as usize, y as usize).get() {
                if current_value == value {
                    // We found a field that already has the current value, no need to check other fields for it
                    continue 'outer;
                }
            } else {
                if possible_values.is_possible(x as usize, y as usize, value) {
                    if placement.is_none() {
                        // We found a first place where the value can be placed. Keep looking for more places.
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
            return None;
        }
    }

    Some(found_something)
}
