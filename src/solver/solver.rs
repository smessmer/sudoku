use rand::{rng, rngs::ThreadRng, seq::IndexedRandom as _};
use std::num::NonZeroU8;

use super::{possible_values::PossibleValues, strategies::SimpleSolverResult};
use crate::{board::Board, solver::board_being_solved::BoardBeingSolved};

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
    board_stack: Vec<BoardBeingSolved>,

    guesser: G,
}

impl<G: Guesser> SolverImpl<G> {
    pub fn new(board: Board, guesser: G) -> Self {
        let board = BoardBeingSolved::new(board);
        let mut res = Self {
            board_stack: vec![],
            guesser,
        };
        if let Some(board) = board {
            res.push(board);
        }
        res
    }

    fn push(&mut self, board: BoardBeingSolved) {
        self.board_stack.push(board);
    }

    pub fn next_solution(&mut self) -> Option<Board> {
        loop {
            let Some(board) = self.board_stack.last() else {
                // No more solutions left
                return None;
            };
            match board.board().first_empty_field_index() {
                None => {
                    // No empty fields left. The sudoku is fully solved.
                    let board = *board.board();
                    self.board_stack.pop().unwrap();
                    return Some(board);
                }
                Some((x, y)) => {
                    match self.guesser.guess_value(&board.possible_values(), x, y) {
                        None => {
                            // No possible values left for this field. This means that the board on top doesn't have any more solutions.
                            // Remove it and continue guessing for boards below it.
                            self.board_stack.pop().unwrap();

                            // Now that we removed the top board, continue the loop to try with the next board on the stack.
                            continue;
                        }
                        Some(value) => {
                            let mut board = *board;

                            // Remove this from the possible values of the *current* board so we don't try it again after backtracking to this stack entry
                            self.board_stack
                                .last_mut()
                                .unwrap()
                                .remove_possible_value(x, y, value);

                            // Make a guess for the value of this field
                            match board
                                .set_empty_field_to_value_and_apply_simple_strategies(x, y, value)
                            {
                                SimpleSolverResult::NotSolvable => {
                                    // This board is not solvable. Don't even add it.
                                }
                                SimpleSolverResult::FoundNothing
                                | SimpleSolverResult::FoundSomething => {
                                    // This board might be solvable. Add it to the stack to explore this branch.
                                    self.push(board);
                                }
                            }

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
