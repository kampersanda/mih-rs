mod ops;
mod siggen;
mod sparsehash;

use crate::CodeInt;

/// Implementation of multi-index hashing.
pub struct Index<'db, T: CodeInt> {
    blocks: usize,
    codes: &'db [T],
    tables: Vec<sparsehash::Table>,
    masks: Vec<T>,
    begs: Vec<usize>,
}
