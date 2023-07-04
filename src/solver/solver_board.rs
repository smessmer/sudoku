use bitvec::prelude::*;
use std::num::NonZeroU8;

use crate::board::{NUM_FIELDS, WIDTH, HEIGHT, Board};

const NUM_VALUES_PER_FIELD: usize = 9;

#[derive(Clone, Copy)]
pub struct PossibleValues {
    // Stores 9 bits for each cell. If the bit is set, the value is considered possible.
    values: BitArr!(for NUM_FIELDS*NUM_VALUES_PER_FIELD),
}

impl PossibleValues {
    pub const fn new_all_is_possible() -> Self {
        Self {
            values: bitarr![const 1; NUM_FIELDS*NUM_VALUES_PER_FIELD],
        }
    }

    fn field_start_index(x: usize, y: usize) -> usize {
        assert!(x <= WIDTH && y <= HEIGHT);
        NUM_VALUES_PER_FIELD * (x * HEIGHT + y)
    }

    fn index(x: usize, y: usize, value: NonZeroU8) -> usize {
        assert!(value.get() <= 9);
        let start_index = Self::field_start_index(x, y);
        start_index + usize::from(value.get()) - 1
    }

    pub fn possible_values_for_field(&self, x: usize, y: usize) -> impl Iterator<Item = NonZeroU8> + '_ {
        let start_index = Self::field_start_index(x, y);
        (1u8..=9u8).filter(move |i| self.values[start_index + usize::from(*i) - 1])
            .map(|i| NonZeroU8::new(i).unwrap())
    }

    fn remove_if_set(&mut self, x: usize, y: usize, value: NonZeroU8) {
        let index = Self::index(x, y, value);
        self.values.set(index, false);
    }

    pub fn remove_conflicting(&mut self, x: usize, y: usize, value: NonZeroU8) {
        self.remove_value_from_col(value, x);
        self.remove_value_from_row(value, y);
        self.remove_value_from_cell(value, x/3, y/3);
    }

    fn remove_value_from_col(&mut self, value: NonZeroU8, x: usize) {
        for y in 0..HEIGHT {
            self.remove_if_set(x, y, value);
        }
    }
    
    fn remove_value_from_row(&mut self, value: NonZeroU8, y: usize) {
        for x in 0..WIDTH {
            self.remove_if_set(x, y, value);
        }
    }
    
    fn remove_value_from_cell(&mut self, value: NonZeroU8, cell_x: usize, cell_y: usize) {
        for x in 0..3 {
            for y in 0..3 {
                self.remove_if_set(3 * cell_x + x, 3 * cell_y + y, value);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct SolverBoard {
    // SolverBoard is responsible for upholding the following invariant:
    // Invariant: For the given `board`, `possible_values` contains all the possible values that don't directly cause
    // a conflict with one of the numbers already entered on `Board`. This is deterministically derivable from `board`
    // but we store it separately so we don't have to recompute it again and again.
    board: Board,
    possible_values: PossibleValues,
}

impl SolverBoard {
    pub fn new(board: Board) -> Self {
        let possible_values = possible_values_from_board(&board);
        Self {
            board,
            possible_values,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn possible_values(&self) -> &PossibleValues {
        &self.possible_values
    }

    pub fn set(&mut self, x: usize, y: usize, value: NonZeroU8) {
        let mut field = self.board.field_mut(x, y);
        assert!(field.is_empty(), "SolverBoard.set() is only allowed on empty fields, otherwise we would violate the invariant or would have to re-add some possible values from removing the field");
        field.set(Some(value));
        self.possible_values.remove_conflicting(x, y, value);

    }
}

fn possible_values_from_board(board: &Board) -> PossibleValues {
    let mut possible_values = PossibleValues::new_all_is_possible();
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let field = board.field(x, y);
            if let Some(value) = field.get() {
                possible_values.remove_conflicting(x, y, value);
            }
        }
    }
    possible_values
}
