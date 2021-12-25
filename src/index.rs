mod ops;
mod siggen;
mod sparsehash;

use crate::CodeInt;

/// Implementation of multi-index hashing.
pub struct Index<T: CodeInt> {
    num_blocks: usize,
    codes: Vec<T>,
    tables: Vec<sparsehash::Table>,
    masks: Vec<T>,
    begs: Vec<usize>,
}
