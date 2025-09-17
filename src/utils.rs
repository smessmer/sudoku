#[inline]
pub const fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

// TODO Tests
