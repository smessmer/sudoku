use std::num::NonZeroU8;

use crate::{Board, solver::possible_values::PossibleValues};

/// A struct representing a Sudoku board that is currently being solved.
#[derive(Clone, Copy)]
pub struct BoardBeingSolved {
    board: Board,

    // Invariant:
    // * `possible_values` does not contradict `board`, i.e. if it says a value is possible then that value won't cause a conflict with the current board.
    // * However, `possible_values` might be more restrictive than necessary, i.e. it might say that a value is not possible even though it could be placed without causing a conflict.
    //   This is done by [Self::remove_possible_value] and used when we guess a value, to restrict that solution branch from being explored again later.
    possible_values: PossibleValues,
}

impl BoardBeingSolved {
    /// Creates a new `BoardBeingSolved` from the given `Board`.
    pub fn new(board: Board) -> Self {
        let possible_values = PossibleValues::from_board(&board);
        Self {
            board,
            possible_values,
        }
    }

    /// Returns a reference to the current board.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Returns a reference to the possible values for each cell.
    pub fn possible_values(&self) -> &PossibleValues {
        &self.possible_values
    }

    pub fn get_field(&self, x: usize, y: usize) -> Option<NonZeroU8> {
        self.board.field(x, y).get()
    }

    pub fn field_is_empty(&self, x: usize, y: usize) -> bool {
        self.board.field(x, y).is_empty()
    }

    pub fn set_empty_field_to(&mut self, x: usize, y: usize, value: NonZeroU8) {
        let mut field = self.board.field_mut(x, y);
        assert!(field.is_empty());
        field.set(Some(value));
        debug_assert!(!self.board.has_conflicts());
        self.possible_values.remove_conflicting(x, y, value);
    }

    pub fn remove_possible_value(&mut self, x: usize, y: usize, value: NonZeroU8) {
        self.possible_values.remove(x, y, value);
    }
}
