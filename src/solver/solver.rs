use super::{
    possible_values::PossibleValues,
    strategies::{solve_simple_strategies, SimpleSolverResult},
};
use crate::board::Board;

pub struct Solver {
    // [board_stack] contains all the branching points after any given guess, with any simple strategies already applied to add additional deterministic fields.
    // At any point, we can find more solutions by taking the top from the stack and applying more guesses, until we get to a fully solved sudoku.
    // When a fully solved sudoku is found, or there are no possible solutions for the top board of the stack, then we should backtrack by removing the top board from the stack.
    // If the stack is empty, then there are no more solutions left.
    // The PossibleValues part of the tuple is equivalent to [PossibleValues::from_board](board), with the difference that we removed values we already guessed to
    // create previous solutions. This ensures we don't guess the same value again after backtracking.
    board_stack: Vec<(Board, PossibleValues)>,
}

impl Solver {
    pub fn new(board: Board) -> Self {
        let possible_values = PossibleValues::from_board(&board);
        let mut res = Self {
            board_stack: vec![],
        };
        res.push(board, possible_values);
        res
    }

    fn push(&mut self, board: Board, possible_values: PossibleValues) {
        match solve_simple_strategies(board, possible_values) {
            SimpleSolverResult::FoundSomething {
                board,
                possible_values,
            } => {
                self.board_stack.push((board, possible_values));
            }
            SimpleSolverResult::FoundNothing => {
                self.board_stack.push((board, possible_values));
            }
            SimpleSolverResult::NotSolvable => {
                // This board is not solvable. Don't even add it.
            }
        }
    }

    pub fn next_solution(&mut self) -> Option<Board> {
        let Some((board, possible_values)) = self.board_stack.last() else {
            // No more solutions left
            return None;
        };
        // TODO No clone
        let board = board.clone();
        let possible_values = possible_values.clone();
        match board.first_empty_field_index() {
            None => {
                // No empty fields left. The sudoku is fully solved.
                self.board_stack.pop().unwrap();
                return Some(board);
            }
            Some((x, y)) => {
                match possible_values.possible_values_for_field(x, y).next() {
                    None => {
                        // No possible values left for this field. This means that the board on top doesn't have any more solutions.
                        // Remove it and continue guessing for boards below it.
                        self.board_stack.pop().unwrap();
                        return self.next_solution();
                    }
                    Some(value) => {
                        // Remove this from the possible values of the *current* board so we don't try it again after backtracking to this stack entry
                        self.board_stack.last_mut().unwrap().1.remove(x, y, value);

                        // Make a guess for the value of this field
                        let mut board = board;
                        let mut field = board.field_mut(x, y);
                        assert!(field.is_empty());
                        field.set(Some(value));
                        debug_assert!(!board.has_conflicts());
                        let mut new_possible_values = possible_values;
                        new_possible_values.remove_conflicting(x, y, value);
                        self.push(board, new_possible_values);

                        return self.next_solution();
                    }
                }
            }
        }
    }
}
