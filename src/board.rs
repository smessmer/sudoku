use crate::utils::div_ceil;

const WIDTH: usize = 9;
const HEIGHT: usize = 9;
const NUM_CELLS: usize = WIDTH * HEIGHT;

const NUM_BYTES: usize = div_ceil(NUM_CELLS, 2);

/// A [Board] is a 9x9 sudoku board.
/// Each cell can contain a value in 0..=9 where 0 means the cell is empty.
pub struct Board {
    // Every byte stores two cells. The first 4 bits the first cell, the second 4 bits the second cell.
    // Cells are ordered by rows, first left-to-right, then top-to-bottom.
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
    pub fn get(&self) -> u8 {
        let value = match self.subindex {
            FieldSubindex::FirstHalfByte => self.field & 0x0F,
            FieldSubindex::SecondHalfByte => self.field >> 4,
        };
        assert!(value <= 9);
        value
    }
}

impl FieldRef<&mut u8> {
    #[inline]
    pub fn get(&self) -> u8 {
        FieldRef::<&u8> {
            field: self.field,
            subindex: self.subindex,
        }.get()
    }

    #[inline]
    pub fn set(&mut self, value: u8) {
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

    fn index(x: usize, y: usize) -> (usize, FieldSubindex) {
        let index = y * WIDTH + x;
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
        FieldRef {
            field,
            subindex,
        }
    }

    #[inline]
    pub fn field_mut(&mut self, x: usize, y: usize) -> FieldRef<&'_ mut u8> {
        let (index, subindex) = Self::index(x, y);
        let field = &mut self.compressed_board[index / 2];
        FieldRef {
            field,
            subindex,
        }
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
                assert_eq!(board.field(x, y).get(), 0);
            }
        }
    }

    #[test]
    fn random() {
        use rand::{SeedableRng, Rng, rngs::StdRng};

        let mut rng = StdRng::seed_from_u64(0);
        let mut board = Board::new_empty();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                board.field_mut(x, y).set(rng.gen_range(0..=9));
            }
        }

        let mut rng = StdRng::seed_from_u64(0);
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let expected = rng.gen_range(0..=9);
                assert_eq!(expected, board.field(x, y).get());
                assert_eq!(expected, board.field_mut(x, y).get());
            }
        }
    }

    #[test]
    #[should_panic = "assertion failed: value <= 9"]
    fn invalid_value() {
        let mut board = Board::new_empty();

        board.field_mut(0, 0).set(10);
    }
}
