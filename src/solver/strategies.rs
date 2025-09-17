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
pub fn solve_simple_strategies(
    mut board: Board,
    mut possible_values: PossibleValues,
) -> SimpleSolverResult {
    let mut found_something_previous_iteration = false;
    loop {
        match _solve_simple_strategies(&mut board, &mut possible_values) {
            SimpleSolverResultInternal::FoundSomething => {
                found_something_previous_iteration = true;
            }
            SimpleSolverResultInternal::FoundNothing => {
                if found_something_previous_iteration {
                    // We found something in a previous iteration, just didn't find anything after that.
                    // But we can return what we found previously.
                    return SimpleSolverResult::FoundSomething {
                        board,
                        possible_values,
                    };
                } else {
                    // We didn't find anything and this is the first iteration. Return that we didn't find anything.
                    return SimpleSolverResult::FoundNothing;
                }
            }
            SimpleSolverResultInternal::NotSolvable => {
                // This might be the first iteration or we may have found something in a previous iteration, but since we ended up in a not solvable dead end,
                // anything we found previously doesn't matter. The whole Sudoku isn't solvable.
                return SimpleSolverResult::NotSolvable;
            }
        }
    }
}

enum SimpleSolverResultInternal {
    FoundSomething,
    FoundNothing,
    NotSolvable,
}

fn _solve_simple_strategies(
    board: &mut Board,
    possible_values: &mut PossibleValues,
) -> SimpleSolverResultInternal {
    let mut result = SimpleSolverResultInternal::FoundNothing;

    match solve_known_values(board, possible_values) {
        SimpleSolverResultInternal::FoundSomething => {
            result = SimpleSolverResultInternal::FoundSomething;
        }
        SimpleSolverResultInternal::FoundNothing => {
            // didn't find anything
        }
        SimpleSolverResultInternal::NotSolvable => return SimpleSolverResultInternal::NotSolvable,
    }

    match solve_hidden_candidates(board, possible_values) {
        SimpleSolverResultInternal::FoundSomething => {
            result = SimpleSolverResultInternal::FoundSomething;
        }
        SimpleSolverResultInternal::FoundNothing => {
            // didn't find anything
        }
        SimpleSolverResultInternal::NotSolvable => return SimpleSolverResultInternal::NotSolvable,
    }

    result
}

/// [solve_known_values] tries to fill in fields that only have one possible value according to `possible_values`.
/// It can also detect situations where a field has no possible values left, meaning that the board is unsolvable.
fn solve_known_values(
    board: &mut Board,
    possible_values: &mut PossibleValues,
) -> SimpleSolverResultInternal {
    // TODO Instead of using solve_known_values to run over all fields repeatedly, it would be more efficient to just check number of remaining values whenever we update PossibleValues. Then remove this simple solver strategy here.
    let mut result = SimpleSolverResultInternal::FoundNothing;

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let mut field = board.field_mut(x, y);
            if field.is_empty() {
                let mut possible_values_this_field =
                    possible_values.possible_values_for_field(x, y);
                let Some(first_possible_value) = possible_values_this_field.next() else {
                    // No possible values left for this field. The board is not solvable.
                    return SimpleSolverResultInternal::NotSolvable;
                };
                let second_possible_value = possible_values_this_field.next();
                std::mem::drop(possible_values_this_field);
                if second_possible_value.is_none() {
                    // There is exactly one possible value for this field. Fill it in.
                    field.set(Some(first_possible_value));
                    possible_values.remove_conflicting(x, y, first_possible_value);
                    debug_assert!(!board.has_conflicts());
                    result = SimpleSolverResultInternal::FoundSomething;
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
) -> SimpleSolverResultInternal {
    let mut result = SimpleSolverResultInternal::FoundNothing;

    // Check each row for values that can only be placed in one field
    for row in 0u8..HEIGHT as u8 {
        let cells = (0u8..WIDTH as u8).map(|x| (x, row));
        match _solve_hidden_candidates(board, possible_values, cells) {
            SimpleSolverResultInternal::FoundSomething => {
                result = SimpleSolverResultInternal::FoundSomething;
            }
            SimpleSolverResultInternal::FoundNothing => {}
            SimpleSolverResultInternal::NotSolvable => {
                return SimpleSolverResultInternal::NotSolvable;
            }
        }
    }

    // Check each col for values that can only be placed in one field
    for col in 0u8..WIDTH as u8 {
        let cells = (0u8..HEIGHT as u8).map(|y| (col, y));
        match _solve_hidden_candidates(board, possible_values, cells) {
            SimpleSolverResultInternal::FoundSomething => {
                result = SimpleSolverResultInternal::FoundSomething;
            }
            SimpleSolverResultInternal::FoundNothing => {}
            SimpleSolverResultInternal::NotSolvable => {
                return SimpleSolverResultInternal::NotSolvable;
            }
        }
    }

    // Check each 3x3 region for values that can only be placed in one field
    for region_x in 0u8..3u8 {
        for region_y in 0u8..3u8 {
            let cells = (0u8..3u8)
                .flat_map(move |x| (0u8..3u8).map(move |y| (region_x * 3 + x, region_y * 3 + y)));
            match _solve_hidden_candidates(board, possible_values, cells) {
                SimpleSolverResultInternal::FoundSomething => {
                    result = SimpleSolverResultInternal::FoundSomething;
                }
                SimpleSolverResultInternal::FoundNothing => {}
                SimpleSolverResultInternal::NotSolvable => {
                    return SimpleSolverResultInternal::NotSolvable;
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
) -> SimpleSolverResultInternal {
    let mut result = SimpleSolverResultInternal::FoundNothing;

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
            result = SimpleSolverResultInternal::FoundSomething;
            debug_assert!(!board.has_conflicts());
        } else {
            // We found no place where we can put this value
            return SimpleSolverResultInternal::NotSolvable;
        }
    }

    result
}
