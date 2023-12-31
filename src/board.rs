use crate::utils::div_ceil;
use std::fmt::{self, Debug};
use std::num::NonZeroU8;

pub const WIDTH: usize = 9;
pub const HEIGHT: usize = 9;
pub const NUM_FIELDS: usize = WIDTH * HEIGHT;
pub const MAX_VALUE: u8 = 9;

const NUM_BYTES: usize = div_ceil(NUM_FIELDS, 2);

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
        (index / 2, subindex)
    }

    #[inline]
    pub fn field(&self, x: usize, y: usize) -> FieldRef<&'_ u8> {
        let (index, subindex) = Self::index(x, y);
        let field = &self.compressed_board[index];
        FieldRef { field, subindex }
    }

    #[inline]
    pub fn field_mut(&mut self, x: usize, y: usize) -> FieldRef<&'_ mut u8> {
        let (index, subindex) = Self::index(x, y);
        let field = &mut self.compressed_board[index];
        FieldRef { field, subindex }
    }

    // TODO Test
    pub fn first_empty_field_index(&self) -> Option<(usize, usize)> {
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

    // TODO Test
    pub fn is_filled(&self) -> bool {
        self.first_empty_field_index().is_none()
    }

    // TODO Test
    pub fn row_iter(&self, row: usize) -> impl Iterator<Item = FieldRef<&'_ u8>> {
        (0..WIDTH).map(move |x| self.field(x, row))
    }

    // TODO Test
    pub fn col_iter(&self, col: usize) -> impl Iterator<Item = FieldRef<&'_ u8>> {
        (0..HEIGHT).map(move |y| self.field(col, y))
    }

    // TODO Test
    pub fn region_iter(
        &self,
        region_x: usize,
        region_y: usize,
    ) -> impl Iterator<Item = FieldRef<&'_ u8>> {
        (0..3)
            .flat_map(move |x| (0..3).map(move |y| self.field(region_x * 3 + x, region_y * 3 + y)))
    }

    // TODO Test
    pub fn has_conflicts(&self) -> bool {
        for row in 0..HEIGHT {
            if self.has_conflicts_in_fields(self.row_iter(row)) {
                return true;
            }
        }
        for col in 0..WIDTH {
            if self.has_conflicts_in_fields(self.col_iter(col)) {
                return true;
            }
        }
        for region_x in 0..3 {
            for region_y in 0..3 {
                if self.has_conflicts_in_fields(self.region_iter(region_x, region_y)) {
                    return true;
                }
            }
        }
        return false;
    }

    fn has_conflicts_in_fields<'a>(
        &'a self,
        fields: impl Iterator<Item = FieldRef<&'a u8>>,
    ) -> bool {
        let mut seen = [false; 9];
        for field in fields {
            if let Some(value) = field.get() {
                let value = value.get() as usize - 1;
                if seen[value] {
                    return true;
                }
                seen[value] = true;
            }
        }
        false
    }

    // TODO Test
    pub fn is_subset_of(&self, rhs: &Board) -> bool {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if let Some(lhs_value) = self.field(x,y).get() {
                    if Some(lhs_value) != rhs.field(x,y).get() {
                        return false;
                    }
                }
            }
        }
        return true;
    }

    // TODO Test
    pub fn num_empty(&self) -> usize {
        let mut num_empty = 0;
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if self.field(x,y).is_empty() {
                    num_empty += 1;
                }
            }
        }
        num_empty
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
                write!(
                    f,
                    "{}",
                    self.field(x, y)
                        .get()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "_".to_string())
                )?;
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
                board
                    .field_mut(x, y)
                    .set(NonZeroU8::new(rng.gen_range(0..=9)));
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
        let board = Board::from_str(
            "
            124 367 598
            598 241 36_
            376 895 412

            832 654 179
            _51 9_3 846
            649 718 253

            483 179 625
            217 536 98_
            ___ 482 731
        ",
        );

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

        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(0, 2).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(1, 2).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(2, 2).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(3, 2).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(4, 2).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(5, 2).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(6, 2).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(7, 2).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(8, 2).get());

        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(0, 3).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(1, 3).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(2, 3).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(3, 3).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(4, 3).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(5, 3).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(6, 3).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(7, 3).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(8, 3).get());

        assert_eq!(None, board.field(0, 4).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(1, 4).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(2, 4).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(3, 4).get());
        assert_eq!(None, board.field(4, 4).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(5, 4).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(6, 4).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(7, 4).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(8, 4).get());

        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(0, 5).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(1, 5).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(2, 5).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(3, 5).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(4, 5).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(5, 5).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(6, 5).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(7, 5).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(8, 5).get());

        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(0, 6).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(1, 6).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(2, 6).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(3, 6).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(4, 6).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(5, 6).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(6, 6).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(7, 6).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(8, 6).get());

        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(0, 7).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(1, 7).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(2, 7).get());
        assert_eq!(Some(NonZeroU8::new(5).unwrap()), board.field(3, 7).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(4, 7).get());
        assert_eq!(Some(NonZeroU8::new(6).unwrap()), board.field(5, 7).get());
        assert_eq!(Some(NonZeroU8::new(9).unwrap()), board.field(6, 7).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(7, 7).get());
        assert_eq!(None, board.field(8, 7).get());

        assert_eq!(None, board.field(0, 8).get());
        assert_eq!(None, board.field(1, 8).get());
        assert_eq!(None, board.field(2, 8).get());
        assert_eq!(Some(NonZeroU8::new(4).unwrap()), board.field(3, 8).get());
        assert_eq!(Some(NonZeroU8::new(8).unwrap()), board.field(4, 8).get());
        assert_eq!(Some(NonZeroU8::new(2).unwrap()), board.field(5, 8).get());
        assert_eq!(Some(NonZeroU8::new(7).unwrap()), board.field(6, 8).get());
        assert_eq!(Some(NonZeroU8::new(3).unwrap()), board.field(7, 8).get());
        assert_eq!(Some(NonZeroU8::new(1).unwrap()), board.field(8, 8).get());
    }
}
