use crate::utils::div_ceil;
use std::num::NonZeroU8;
use std::fmt::{self, Debug};

pub const WIDTH: usize = 9;
pub const HEIGHT: usize = 9;
pub const NUM_FIELDS: usize = WIDTH * HEIGHT;

const NUM_BYTES: usize = div_ceil(NUM_FIELDS, 2);
const FIELD_EMPTY: u8 = 0;

/// A [Board] is a 9x9 sudoku board.
/// Each cell can contain a value in 0..=9 where 0 means the cell is empty.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    // Every byte stores two cells. The first 4 bits the first cell, the second 4 bits the second cell.
    // Cells are ordered by columns, first top-to-bottom, then next column left-to-right
    compressed_board: [u8; NUM_BYTES],
}

#[derive(Clone, Copy)]
enum FieldSubindex {
    FirstHalfByte,
    SecondHalfByte,
}

pub struct FieldRef<T> {
    field: T,
    subindex: FieldSubindex,
}

impl FieldRef<&u8> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.get().is_none()
    }

    #[inline]
    pub fn get(&self) -> Option<NonZeroU8> {
        let value = match self.subindex {
            FieldSubindex::FirstHalfByte => self.field & 0x0F,
            FieldSubindex::SecondHalfByte => self.field >> 4,
        };
        assert!(value <= 9);
        NonZeroU8::new(value)
    }
}

impl FieldRef<&mut u8> {
    #[inline]
    pub fn get(&self) -> Option<NonZeroU8> {
        FieldRef::<&u8> {
            field: self.field,
            subindex: self.subindex,
        }
        .get()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        FieldRef::<&u8> {
            field: self.field,
            subindex: self.subindex,
        }
        .is_empty()
    }

    #[inline]
    pub fn set(&mut self, value: Option<NonZeroU8>) {
        let value = value.map(|v| v.get()).unwrap_or(0);
        assert!(value <= 9);
        match self.subindex {
            FieldSubindex::FirstHalfByte => *self.field = (*self.field & 0xF0) | value,
            FieldSubindex::SecondHalfByte => *self.field = (*self.field & 0x0F) | (value << 4),
        }
    }
}

impl Board {
    #[inline]
    pub fn new_empty() -> Self {
        Board {
            compressed_board: [0; NUM_BYTES],
        }
    }

    pub fn from_str(board: &str) -> Self {
        let mut chars = board.chars().filter(|x| !x.is_whitespace());
        let mut board = Board::new_empty();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let c = chars.next().expect("Not enough characters in board string");
                let value = if c == '_' {
                    None
                } else {
                    let value = c.to_digit(10).expect("Invalid characters in board string");
                    assert_ne!(0, value);
                    Some(NonZeroU8::new(u8::try_from(value).unwrap()).unwrap())
                };
                board.field_mut(x, y).set(value);
            }
        }
        assert_eq!(None, chars.next(), "Too many characters in board string");
        board
    }

    fn index(x: usize, y: usize) -> (usize, FieldSubindex) {
        assert!(x < WIDTH);
        assert!(y < HEIGHT);
        let index = x * HEIGHT + y;
        let subindex = if index % 2 == 0 {
            FieldSubindex::FirstHalfByte
        } else {
            FieldSubindex::SecondHalfByte
        };
        (index, subindex)
    }

    #[inline]
    pub fn field(&self, x: usize, y: usize) -> FieldRef<&'_ u8> {
        let (index, subindex) = Self::index(x, y);
        let field = &self.compressed_board[index / 2];
        FieldRef { field, subindex }
    }

    #[inline]
    pub fn field_mut(&mut self, x: usize, y: usize) -> FieldRef<&'_ mut u8> {
        let (index, subindex) = Self::index(x, y);
        let field = &mut self.compressed_board[index / 2];
        FieldRef { field, subindex }
    }

    // TODO Test
    pub fn first_empty_field_index(
        &self,
    ) -> Option<(usize, usize)> {
        // TODO Do this with iterators
        // TODO Better would be to iterate over `self.compressed_board` and `FieldRef::subindex`
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if self.field(x, y).is_empty() {
                    return Some((x, y));
                }
            }
        }
        None
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..HEIGHT {
            if y == 3 || y == 6 {
                // Add a separator line between every 3 rows
                write!(f, "\n")?;
            }
            for x in 0..WIDTH {
                if x == 3 || x == 6 {
                    // Add a separate between every 3 cols
                    write!(f, " ")?;
                }
                write!(f, "{}", self.field(x, y).get().map(|c| c.to_string()).unwrap_or_else(|| "_".to_string()))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let board = Board::new_empty();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                assert_eq!(None, board.field(x, y).get());
                assert!(board.field(x, y).is_empty());
            }
        }
        let mut board = board;
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                assert!(board.field_mut(x, y).is_empty());
            }
        }
    }

    #[test]
    fn random() {
        use rand::{rngs::StdRng, Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(0);
        let mut board = Board::new_empty();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                board.field_mut(x, y).set(NonZeroU8::new(rng.gen_range(0..=9)));
            }
        }

        let mut rng = StdRng::seed_from_u64(0);
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let expected = NonZeroU8::new(rng.gen_range(0..=9));
                assert_eq!(expected, board.field(x, y).get());
                assert_eq!(expected, board.field_mut(x, y).get());
                assert_eq!(expected.is_none(), board.field(x, y).is_empty());
                assert_eq!(expected.is_none(), board.field_mut(x, y).is_empty());
            }
        }
    }

    #[test]
    #[should_panic = "assertion failed: value <= 9"]
    fn invalid_value() {
        let mut board = Board::new_empty();

        board.field_mut(0, 0).set(Some(NonZeroU8::new(10).unwrap()));
    }

    #[test]
    fn from_str() {
        let board = Board::from_str("
            124 367 598
            598 241 36_
            376 895 412

            832 654 179
            _51 9_3 846
            649 718 253

            483 179 625
            217 536 98_
            ___ 482 731
        ");

        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(0, 0).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(1, 0).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(2, 0).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(3, 0).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(4, 0).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(5, 0).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(6, 0).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(7, 0).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(8, 0).get());

        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(0, 1).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(1, 1).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(2, 1).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(3, 1).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(4, 1).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(5, 1).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(6, 1).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(7, 1).get());
        assert_eq!(None, board.field(8, 1).get());

        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(0,2).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(1,2).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(2,2).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(3,2).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(4,2).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(5,2).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(6,2).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(7,2).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(8,2).get());

        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(0,3).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(1,3).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(2,3).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(3,3).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(4,3).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(5,3).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(6,3).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(7,3).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(8,3).get());

        assert_eq!(None, board.field(0,4).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(1,4).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(2,4).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(3,4).get());
        assert_eq!(None, board.field(4,4).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(5,4).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(6,4).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(7,4).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(8,4).get());

        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(0,5).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(1,5).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(2,5).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(3,5).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(4,5).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(5,5).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(6,5).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(7,5).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(8,5).get());


        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(0,6).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(1,6).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(2,6).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(3,6).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(4,6).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(5,6).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(6,6).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(7,6).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(8,6).get());

        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(0,7).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(1,7).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(2,7).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(3,7).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(4,7).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(5,7).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(6,7).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(7,7).get());
        assert_eq!(None, board.field(8,7).get());

        assert_eq!(None, board.field(0,8).get());
        assert_eq!(None, board.field(1,8).get());
        assert_eq!(None, board.field(2,8).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(3,8).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(4,8).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(5,8).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(6,8).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(7,8).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(8,8).get());

    }
}
