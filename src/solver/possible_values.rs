use bitvec::prelude::*;
use std::num::NonZeroU8;

use crate::board::{Board, HEIGHT, NUM_FIELDS, WIDTH};

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

    pub fn from_board(board: &Board) -> PossibleValues {
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

    fn field_start_index(x: usize, y: usize) -> usize {
        assert!(x <= WIDTH && y <= HEIGHT);
        NUM_VALUES_PER_FIELD * (x * HEIGHT + y)
    }

    fn index(x: usize, y: usize, value: NonZeroU8) -> usize {
        assert!(value.get() <= 9);
        let start_index = Self::field_start_index(x, y);
        start_index + usize::from(value.get()) - 1
    }

    pub fn possible_values_for_field(
        &self,
        x: usize,
        y: usize,
    ) -> impl Iterator<Item = NonZeroU8> + '_ {
        let start_index = Self::field_start_index(x, y);
        (1u8..=9u8)
            .filter(move |i| self.values[start_index + usize::from(*i) - 1])
            .map(|i| NonZeroU8::new(i).unwrap())
    }

    pub fn first_possible_value_for_field(&self, x: usize, y: usize) -> Option<NonZeroU8> {
        // TODO Faster with bit operations that find the first set bit in one assembly instruction?
        self.possible_values_for_field(x, y).next()
    }

    // TODO Test
    pub fn is_possible(&self, x: usize, y: usize, value: NonZeroU8) -> bool {
        let index = Self::index(x, y, value);
        self.values[index]
    }

    // TODO Test
    pub fn remove(&mut self, x: usize, y: usize, value: NonZeroU8) {
        let index = Self::index(x, y, value);
        assert!(self.values[index]);
        self.values.set(index, false);
    }

    fn remove_if_set(&mut self, x: usize, y: usize, value: NonZeroU8) {
        let index = Self::index(x, y, value);
        self.values.set(index, false);
    }

    pub fn remove_conflicting(&mut self, x: usize, y: usize, value: NonZeroU8) {
        self.remove_value_from_col(value, x);
        self.remove_value_from_row(value, y);
        self.remove_value_from_region(value, x / 3, y / 3);
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

    fn remove_value_from_region(&mut self, value: NonZeroU8, cell_x: usize, cell_y: usize) {
        for x in 0..3 {
            for y in 0..3 {
                self.remove_if_set(3 * cell_x + x, 3 * cell_y + y, value);
            }
        }
    }
}
