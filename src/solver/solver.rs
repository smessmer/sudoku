use rand::{rng, rngs::ThreadRng, seq::IndexedRandom as _};
use std::num::NonZeroU8;

use super::{
    possible_values::PossibleValues,
    strategies::{SimpleSolverResult, solve_simple_strategies},
};
use crate::board::Board;

pub struct Solver {
    solver_impl: SolverImpl<GuessFirstPossibleValue>,
}

impl Solver {
    pub fn new(board: Board) -> Self {
        Self {
            solver_impl: SolverImpl::new(board, GuessFirstPossibleValue),
        }
    }

    pub fn next_solution(&mut self) -> Option<Board> {
        self.solver_impl.next_solution()
    }
}

pub struct Generator {
    solver_impl: SolverImpl<GuessRandomPossibleValue>,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            solver_impl: SolverImpl::new(
                Board::new_empty(),
                GuessRandomPossibleValue { rng: rng() },
            ),
        }
    }

    // We're taking `self` by value because this should only be called once. If we call `solver_impl.next_solution` multiple times,
    // the two solutions would be very similar.
    pub fn generate(mut self) -> Board {
        self.solver_impl
            .next_solution()
            .expect("An empty sudoku must have at least one solution")
    }
}

/// A [Guesser] can be used to parameterize a [SolverImpl] so that it either guesses the first possible value for a field, or a random one.
/// Guessing random values is useful for generating new sudokus by running the solver on an empty sudoku with random guesses.
/// For solving a given sudoku, guessing the first possible value is faster.
trait Guesser {
    fn guess_value(
        &mut self,
        possible_values: &PossibleValues,
        x: usize,
        y: usize,
    ) -> Option<NonZeroU8>;
}

struct GuessFirstPossibleValue;
impl Guesser for GuessFirstPossibleValue {
    fn guess_value(
        &mut self,
        possible_values: &PossibleValues,
        x: usize,
        y: usize,
    ) -> Option<NonZeroU8> {
        possible_values.first_possible_value_for_field(x, y)
    }
}

struct GuessRandomPossibleValue {
    rng: ThreadRng,
}
impl Guesser for GuessRandomPossibleValue {
    fn guess_value(
        &mut self,
        possible_values: &PossibleValues,
        x: usize,
        y: usize,
    ) -> Option<NonZeroU8> {
        // TODO Do this without first collecting into Vec. Should be possible if the iterator is ExactSizeIterator.
        let values: Vec<NonZeroU8> = possible_values.possible_values_for_field(x, y).collect();
        values.choose(&mut self.rng).copied()
    }
}

struct SolverImpl<G: Guesser> {
    // [board_stack] contains all the branching points after any given guess, with any simple strategies already applied to add additional deterministic fields.
    // At any point, we can find more solutions by taking the top from the stack and applying more guesses, until we get to a fully solved sudoku.
    // When a fully solved sudoku is found, or there are no possible solutions for the top board of the stack, then we should backtrack by removing the top board from the stack.
    // If the stack is empty, then there are no more solutions left.
    // The PossibleValues part of the tuple is equivalent to [PossibleValues::from_board](board), with the difference that we removed values we already guessed to
    // create previous solutions. This ensures we don't guess the same value again after backtracking.
    board_stack: Vec<(Board, PossibleValues)>,

    guesser: G,
}

impl<G: Guesser> SolverImpl<G> {
    pub fn new(board: Board, guesser: G) -> Self {
        let possible_values = PossibleValues::from_board(&board);
        let mut res = Self {
            board_stack: vec![],
            guesser,
        };
        res.push(board, possible_values);
        res
    }

    fn push(&mut self, board: Board, possible_values: PossibleValues) {
        match solve_simple_strategies(board, possible_values) {
            SimpleSolverResult::FoundSomething {
                board: new_board,
                possible_values: new_possible_values,
            } => {
                debug_assert!(board.is_subset_of(&new_board));
                self.board_stack.push((new_board, new_possible_values));
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
        loop {
            let Some((board, possible_values)) = self.board_stack.last() else {
                // No more solutions left
                return None;
            };
            let board = *board;
            let possible_values = *possible_values;
            match board.first_empty_field_index() {
                None => {
                    // No empty fields left. The sudoku is fully solved.
                    self.board_stack.pop().unwrap();
                    return Some(board);
                }
                Some((x, y)) => {
                    match self.guesser.guess_value(&possible_values, x, y) {
                        None => {
                            // No possible values left for this field. This means that the board on top doesn't have any more solutions.
                            // Remove it and continue guessing for boards below it.
                            // TODO Maybe not just look at the next field, but check all fields of possible_values? As soon as any says that there are no possible values left, we can cut this branch.
                            self.board_stack.pop().unwrap();

                            // Now that we removed the top board, continue the loop to try with the next board on the stack.
                            continue;
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

                            // Now that we guessed a value, continue the loop with the next iteration to either return a solution or keep guessing if necessary.
                            continue;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_ambigious() {
        let board = Board::from_str(
            "
            __4 6__ _19
            __3 __9 2_5
            _6_ ___ __4

            6__ ___ 7_2
            ___ __7 ___
            ___ 9__ __1

            8__ _5_ __7
            _41 3_8 ___
            _2_ _91 ___
        ",
        );
        let mut solver = Solver::new(board);
        let mut solutions = vec![];
        while let Some(solution) = solver.next_solution() {
            assert!(solution.is_filled());
            assert!(!solution.has_conflicts());
            assert!(board.is_subset_of(&solution));

            for other_solution in &solutions {
                assert_ne!(*other_solution, solution);
            }

            solutions.push(solution);
        }
        assert_eq!(10, solutions.len());
    }

    // TODO More tests, including generating based on half-solved sudokus
}
