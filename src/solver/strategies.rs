use std::num::NonZeroU8;

use crate::{
    board::{HEIGHT, MAX_VALUE, WIDTH},
    solver::board_being_solved::BoardBeingSolved,
};

pub enum SimpleSolverResult {
    FoundSomething,
    FoundNothing,
    NotSolvable,
}

/// [solve_simple_strategies] tries some fast strategies to add values on the board that can easily be deduced from other values.
/// It modifies the board and possible_values in place and returns whether it found and inserted some values.
pub fn solve_simple_strategies(board: &mut BoardBeingSolved) -> SimpleSolverResult {
    let mut result = SimpleSolverResult::FoundNothing;

    match solve_known_values(board) {
        SimpleSolverResult::FoundSomething => {
            result = SimpleSolverResult::FoundSomething;
        }
        SimpleSolverResult::FoundNothing => {
            // didn't find anything
        }
        SimpleSolverResult::NotSolvable => return SimpleSolverResult::NotSolvable,
    }

    match solve_hidden_candidates(board) {
        SimpleSolverResult::FoundSomething => {
            result = SimpleSolverResult::FoundSomething;
        }
        SimpleSolverResult::FoundNothing => {
            // didn't find anything
        }
        SimpleSolverResult::NotSolvable => return SimpleSolverResult::NotSolvable,
    }

    // Note: any new values we find here will modify `BoardBeingSolved` and therefore trigger a run of the simple strategies for fields that might be affected by the new values.
    //       So we don't need to loop here to keep trying the simple strategies.

    result
}

/// [solve_simple_strategies_triggered_by_modification] is similar to [solve_simple_strategies], but only checks fields that could be affected by a modification at (modification_x, modification_y).
/// This is faster.
pub fn solve_simple_strategies_triggered_by_modification(
    board: &mut BoardBeingSolved,
    modification_x: u8,
    modification_y: u8,
) -> SimpleSolverResult {
    let mut result = SimpleSolverResult::FoundNothing;

    match solve_known_values_triggered_by_modification(board, modification_x, modification_y) {
        SimpleSolverResult::FoundSomething => {
            result = SimpleSolverResult::FoundSomething;
        }
        SimpleSolverResult::FoundNothing => {
            // didn't find anything
        }
        SimpleSolverResult::NotSolvable => return SimpleSolverResult::NotSolvable,
    }

    match _solve_hidden_candidates_triggered_by_modification(board, modification_x, modification_y)
    {
        SimpleSolverResult::FoundSomething => {
            result = SimpleSolverResult::FoundSomething;
        }
        SimpleSolverResult::FoundNothing => {
            // didn't find anything
        }
        SimpleSolverResult::NotSolvable => return SimpleSolverResult::NotSolvable,
    }

    result
}

/// [solve_known_values] tries to fill in fields that only have one possible value according to `possible_values`.
/// It can also detect situations where a field has no possible values left, meaning that the board is unsolvable.
fn solve_known_values(board: &mut BoardBeingSolved) -> SimpleSolverResult {
    let mut result = SimpleSolverResult::FoundNothing;

    for x in 0..WIDTH as u8 {
        for y in 0..HEIGHT as u8 {
            match _solve_known_values_for_field(board, x, y) {
                SimpleSolverResult::FoundSomething => {
                    result = SimpleSolverResult::FoundSomething;
                }
                SimpleSolverResult::FoundNothing => {
                    // didn't find anything
                }
                SimpleSolverResult::NotSolvable => return SimpleSolverResult::NotSolvable,
            }
        }
    }

    result
}

/// Similar to [solve_known_values], but only checks fields that could be affected by a modification at (modification_x, modification_y).
/// This is faster.
fn solve_known_values_triggered_by_modification(
    board: &mut BoardBeingSolved,
    modification_x: u8,
    modification_y: u8,
) -> SimpleSolverResult {
    // If the field modification_x/modification_y was modified, this can trigger changes in the same row, column and 3x3 region.
    let mut result = SimpleSolverResult::FoundNothing;

    // Check row
    for x in 0..WIDTH as u8 {
        match _solve_known_values_for_field(board, x, modification_y) {
            SimpleSolverResult::FoundSomething => {
                result = SimpleSolverResult::FoundSomething;
            }
            SimpleSolverResult::FoundNothing => {
                // didn't find anything
            }
            SimpleSolverResult::NotSolvable => return SimpleSolverResult::NotSolvable,
        }
    }

    // Check column
    for y in 0..HEIGHT as u8 {
        match _solve_known_values_for_field(board, modification_x, y) {
            SimpleSolverResult::FoundSomething => {
                result = SimpleSolverResult::FoundSomething;
            }
            SimpleSolverResult::FoundNothing => {
                // didn't find anything
            }
            SimpleSolverResult::NotSolvable => return SimpleSolverResult::NotSolvable,
        }
    }

    // Check 3x3 region
    let region_x = modification_x / 3;
    let region_y = modification_y / 3;
    for x in (region_x * 3)..((region_x + 1) * 3) {
        for y in (region_y * 3)..((region_y + 1) * 3) {
            match _solve_known_values_for_field(board, x, y) {
                SimpleSolverResult::FoundSomething => {
                    result = SimpleSolverResult::FoundSomething;
                }
                SimpleSolverResult::FoundNothing => {
                    // didn't find anything
                }
                SimpleSolverResult::NotSolvable => return SimpleSolverResult::NotSolvable,
            }
        }
    }

    result
}

fn _solve_known_values_for_field(board: &mut BoardBeingSolved, x: u8, y: u8) -> SimpleSolverResult {
    if board.field_is_empty(x as usize, y as usize) {
        let mut possible_values_this_field = board
            .possible_values()
            .possible_values_for_field(x as usize, y as usize);
        let Some(first_possible_value) = possible_values_this_field.next() else {
            // No possible values left for this field. The board is not solvable.
            return SimpleSolverResult::NotSolvable;
        };
        let second_possible_value = possible_values_this_field.next();
        std::mem::drop(possible_values_this_field);
        if second_possible_value.is_none() {
            // There is exactly one possible value for this field. Fill it in.
            match board.set_empty_field_to_value_and_apply_simple_strategies(
                x as usize,
                y as usize,
                first_possible_value,
            ) {
                SimpleSolverResult::FoundSomething | SimpleSolverResult::FoundNothing => {
                    // May or may not have found further values, but we at least found the one we just set
                    SimpleSolverResult::FoundSomething
                }
                SimpleSolverResult::NotSolvable => SimpleSolverResult::NotSolvable,
            }
        } else {
            // There are multiple possible values for this field. Can't deduce anything.
            SimpleSolverResult::FoundNothing
        }
    } else {
        // Field is already filled. Nothing to do.
        SimpleSolverResult::FoundNothing
    }
}

/// [solve_hidden_candidates] tries to fill hidden candidates, i.e. values that only have one possible position in a row, column or 3x3 region.
fn solve_hidden_candidates(board: &mut BoardBeingSolved) -> SimpleSolverResult {
    let mut result = SimpleSolverResult::FoundNothing;

    // Check each row for values that can only be placed in one field
    for row in 0u8..HEIGHT as u8 {
        match _solve_hidden_candidates_row(board, row) {
            SimpleSolverResult::FoundSomething => {
                result = SimpleSolverResult::FoundSomething;
            }
            SimpleSolverResult::FoundNothing => {}
            SimpleSolverResult::NotSolvable => {
                return SimpleSolverResult::NotSolvable;
            }
        }
    }

    // Check each col for values that can only be placed in one field
    for col in 0u8..WIDTH as u8 {
        match _solve_hidden_candidates_col(board, col) {
            SimpleSolverResult::FoundSomething => {
                result = SimpleSolverResult::FoundSomething;
            }
            SimpleSolverResult::FoundNothing => {}
            SimpleSolverResult::NotSolvable => {
                return SimpleSolverResult::NotSolvable;
            }
        }
    }

    // Check each 3x3 region for values that can only be placed in one field
    for region_x in 0u8..3u8 {
        for region_y in 0u8..3u8 {
            match _solve_hidden_candidates_region(board, region_x, region_y) {
                SimpleSolverResult::FoundSomething => {
                    result = SimpleSolverResult::FoundSomething;
                }
                SimpleSolverResult::FoundNothing => {}
                SimpleSolverResult::NotSolvable => {
                    return SimpleSolverResult::NotSolvable;
                }
            }
        }
    }

    result
}

/// Similar to [solve_hidden_candidates], but only checks fields that could be affected by a modification at (modification_x, modification_y).
/// This is faster.
fn _solve_hidden_candidates_triggered_by_modification(
    board: &mut BoardBeingSolved,
    modification_x: u8,
    modification_y: u8,
) -> SimpleSolverResult {
    let mut result = SimpleSolverResult::FoundNothing;

    // Check row
    match _solve_hidden_candidates_row(board, modification_y) {
        SimpleSolverResult::FoundSomething => {
            result = SimpleSolverResult::FoundSomething;
        }
        SimpleSolverResult::FoundNothing => {}
        SimpleSolverResult::NotSolvable => {
            return SimpleSolverResult::NotSolvable;
        }
    }

    // Check column
    match _solve_hidden_candidates_col(board, modification_x) {
        SimpleSolverResult::FoundSomething => {
            result = SimpleSolverResult::FoundSomething;
        }
        SimpleSolverResult::FoundNothing => {}
        SimpleSolverResult::NotSolvable => {
            return SimpleSolverResult::NotSolvable;
        }
    }

    // Check 3x3 region
    let region_x = modification_x / 3;
    let region_y = modification_y / 3;
    match _solve_hidden_candidates_region(board, region_x, region_y) {
        SimpleSolverResult::FoundSomething => {
            result = SimpleSolverResult::FoundSomething;
        }
        SimpleSolverResult::FoundNothing => {}
        SimpleSolverResult::NotSolvable => {
            return SimpleSolverResult::NotSolvable;
        }
    }

    result
}

fn _solve_hidden_candidates_row(board: &mut BoardBeingSolved, row: u8) -> SimpleSolverResult {
    let cells = (0u8..WIDTH as u8).map(|x| (x, row));
    _solve_hidden_candidates(board, cells)
}

fn _solve_hidden_candidates_col(board: &mut BoardBeingSolved, col: u8) -> SimpleSolverResult {
    let cells = (0u8..HEIGHT as u8).map(|y| (col, y));
    _solve_hidden_candidates(board, cells)
}

fn _solve_hidden_candidates_region(
    board: &mut BoardBeingSolved,
    region_x: u8,
    region_y: u8,
) -> SimpleSolverResult {
    let cells =
        (0u8..3u8).flat_map(move |x| (0u8..3u8).map(move |y| (region_x * 3 + x, region_y * 3 + y)));
    _solve_hidden_candidates(board, cells)
}

#[must_use]
fn _solve_hidden_candidates(
    board: &mut BoardBeingSolved,
    field_coords: impl Iterator<Item = (u8, u8)> + Clone,
) -> SimpleSolverResult {
    let mut result = SimpleSolverResult::FoundNothing;

    'outer: for value in 1u8..=MAX_VALUE {
        let value = NonZeroU8::new(value).unwrap();
        let mut placement = None;

        // Find the place(s) where we can put this value
        for (x, y) in field_coords.clone() {
            if let Some(current_value) = board.get_field(x as usize, y as usize) {
                if current_value == value {
                    // We found a field that already has the current value, no need to check other fields for it
                    continue 'outer;
                }
            } else {
                if board
                    .possible_values()
                    .is_possible(x as usize, y as usize, value)
                {
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
            match board.set_empty_field_to_value_and_apply_simple_strategies(x, y, value) {
                SimpleSolverResult::FoundSomething | SimpleSolverResult::FoundNothing => {
                    // May or may not have found further values, but we at least found the one we just set
                    result = SimpleSolverResult::FoundSomething;
                }
                SimpleSolverResult::NotSolvable => {
                    return SimpleSolverResult::NotSolvable;
                }
            }
        } else {
            // We found no place where we can put this value
            return SimpleSolverResult::NotSolvable;
        }
    }

    result
}
