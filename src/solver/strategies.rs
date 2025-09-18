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

    // Note: at first glance, it sounds smart to repeat this in a loop until no more progress is made.
    //       But our benchmarks show that this actually slows things down by 20%. Probably guessing values
    //       is cheap enough that it's better to just try guessing earlier instead of spending more time
    //       on the off chance that adding a new value will help us deduce more values.

    result
}

/// [solve_known_values] tries to fill in fields that only have one possible value according to `possible_values`.
/// It can also detect situations where a field has no possible values left, meaning that the board is unsolvable.
fn solve_known_values(board: &mut BoardBeingSolved) -> SimpleSolverResult {
    // TODO Instead of using solve_known_values to run over all fields repeatedly, it would be more efficient to just check number of remaining values whenever we update PossibleValues. Then remove this simple solver strategy here.
    //      Also, can we do the same for other strategies? e.g. for hidden values, each time we set a value, check if that creates a hidden candidate in the same row/col/region?
    //      Maybe we should introduce a SolvingBoard struct that wraps Board and PossibleValues and provides methods to set values and with each value being set, it applies all simple strategies for all fields that could be affected by this new value.
    let mut result = SimpleSolverResult::FoundNothing;

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            if board.field_is_empty(x, y) {
                let mut possible_values_this_field =
                    board.possible_values().possible_values_for_field(x, y);
                let Some(first_possible_value) = possible_values_this_field.next() else {
                    // No possible values left for this field. The board is not solvable.
                    return SimpleSolverResult::NotSolvable;
                };
                let second_possible_value = possible_values_this_field.next();
                std::mem::drop(possible_values_this_field);
                if second_possible_value.is_none() {
                    // There is exactly one possible value for this field. Fill it in.
                    board.set_empty_field_to(x, y, first_possible_value);
                    result = SimpleSolverResult::FoundSomething;
                }
            }
        }
    }

    result
}

/// [solve_hidden_candidates] tries to fill hidden candidates, i.e. values that only have one possible position in a row, column or 3x3 region.
fn solve_hidden_candidates(board: &mut BoardBeingSolved) -> SimpleSolverResult {
    let mut result = SimpleSolverResult::FoundNothing;

    // Check each row for values that can only be placed in one field
    for row in 0u8..HEIGHT as u8 {
        let cells = (0u8..WIDTH as u8).map(|x| (x, row));
        match _solve_hidden_candidates(board, cells) {
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
        match _solve_hidden_candidates(board, cells) {
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
            match _solve_hidden_candidates(board, cells) {
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
            board.set_empty_field_to(x, y, value);
            result = SimpleSolverResult::FoundSomething;
        } else {
            // We found no place where we can put this value
            return SimpleSolverResult::NotSolvable;
        }
    }

    result
}
