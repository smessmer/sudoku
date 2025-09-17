use std::num::NonZeroU8;

use super::possible_values::PossibleValues;
use crate::board::{Board, HEIGHT, MAX_VALUE, WIDTH};

pub enum SimpleSolverResult {
    FoundSomething,
    FoundNothing,
    NotSolvable,
}

/// [solve_simple_strategies] tries some fast strategies to add values on the board that can easily be deduced from other values.
/// It modifies the board and possible_values in place and returns whether it found and inserted some values.
pub fn solve_simple_strategies(
    board: &mut Board,
    possible_values: &mut PossibleValues,
) -> SimpleSolverResult {
    let mut previous_iteration_result = SimpleSolverResult::FoundNothing;

    loop {
        match _solve_simple_strategies(board, possible_values) {
            SimpleSolverResult::FoundSomething => {
                previous_iteration_result = SimpleSolverResult::FoundSomething;

                // TODO For some reason, our benchmarks say it would be faster to abort here and return FoundSomething instead of continuing to find more values.
            }
            SimpleSolverResult::FoundNothing => {
                // We may or may not have found something in a previous iteration, just didn't find anything in the current iteration.
                // But we can return what we found previously.
                return previous_iteration_result;
            }
            SimpleSolverResult::NotSolvable => {
                // This might be the first iteration or we may have found something in a previous iteration, but since we ended up in a not solvable dead end,
                // anything we found previously doesn't matter. The whole Sudoku isn't solvable.
                return SimpleSolverResult::NotSolvable;
            }
        }
    }
}

fn _solve_simple_strategies(
    board: &mut Board,
    possible_values: &mut PossibleValues,
) -> SimpleSolverResult {
    let mut result = SimpleSolverResult::FoundNothing;

    match solve_known_values(board, possible_values) {
        SimpleSolverResult::FoundSomething => {
            result = SimpleSolverResult::FoundSomething;
        }
        SimpleSolverResult::FoundNothing => {
            // didn't find anything
        }
        SimpleSolverResult::NotSolvable => return SimpleSolverResult::NotSolvable,
    }

    match solve_hidden_candidates(board, possible_values) {
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
fn solve_known_values(
    board: &mut Board,
    possible_values: &mut PossibleValues,
) -> SimpleSolverResult {
    // TODO Instead of using solve_known_values to run over all fields repeatedly, it would be more efficient to just check number of remaining values whenever we update PossibleValues. Then remove this simple solver strategy here.
    let mut result = SimpleSolverResult::FoundNothing;

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let mut field = board.field_mut(x, y);
            if field.is_empty() {
                let mut possible_values_this_field =
                    possible_values.possible_values_for_field(x, y);
                let Some(first_possible_value) = possible_values_this_field.next() else {
                    // No possible values left for this field. The board is not solvable.
                    return SimpleSolverResult::NotSolvable;
                };
                let second_possible_value = possible_values_this_field.next();
                std::mem::drop(possible_values_this_field);
                if second_possible_value.is_none() {
                    // There is exactly one possible value for this field. Fill it in.
                    field.set(Some(first_possible_value));
                    possible_values.remove_conflicting(x, y, first_possible_value);
                    debug_assert!(!board.has_conflicts());
                    result = SimpleSolverResult::FoundSomething;
                }
            }
        }
    }

    result
}

/// [solve_hidden_candidates] tries to fill hidden candidates, i.e. values that only have one possible position in a row, column or 3x3 region.
fn solve_hidden_candidates(
    board: &mut Board,
    possible_values: &mut PossibleValues,
) -> SimpleSolverResult {
    let mut result = SimpleSolverResult::FoundNothing;

    // Check each row for values that can only be placed in one field
    for row in 0u8..HEIGHT as u8 {
        let cells = (0u8..WIDTH as u8).map(|x| (x, row));
        match _solve_hidden_candidates(board, possible_values, cells) {
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
        let cells = (0u8..HEIGHT as u8).map(|y| (col, y));
        match _solve_hidden_candidates(board, possible_values, cells) {
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
            let cells = (0u8..3u8)
                .flat_map(move |x| (0u8..3u8).map(move |y| (region_x * 3 + x, region_y * 3 + y)));
            match _solve_hidden_candidates(board, possible_values, cells) {
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

#[must_use]
fn _solve_hidden_candidates(
    board: &mut Board,
    possible_values: &mut PossibleValues,
    field_coords: impl Iterator<Item = (u8, u8)> + Clone,
) -> SimpleSolverResult {
    let mut result = SimpleSolverResult::FoundNothing;

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
            result = SimpleSolverResult::FoundSomething;
            debug_assert!(!board.has_conflicts());
        } else {
            // We found no place where we can put this value
            return SimpleSolverResult::NotSolvable;
        }
    }

    result
}
