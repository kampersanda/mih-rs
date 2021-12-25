mod ops;
mod siggen;
mod sparsehash;

use crate::CodeInt;

/// Multi-index hashing for neighbor searches on binary codes in the Hamming space.
///
/// [`Index`] implements the multi-index hashing proposed by
/// [Norouzi et al.](https://arxiv.org/abs/1307.2982), which provides
/// fast and memory-efficient neighbor searches on binary codes in the Hamming space.
///
/// The following two search options are supported:
///
///  - *Range search* finds neighbor codes whose Hamming distances to a given code are within a radius.
///  - *Top-K search* finds the top-K codes that are closest to a given code.
///
/// # Arguments
///
/// [`Index`] takes a generic type parameter of [`CodeInt`] to represent a binary code.
///
/// # Examples
///
/// ```
/// use mih_rs::Index;
///
/// // Database of codes
/// let codes: Vec<u64> = vec![
///     0b1111111111111111111111011111111111111111111111111011101111111111, // #zeros = 3
///     0b1111111111111111111111111111111101111111111011111111111111111111, // #zeros = 2
///     0b1111111011011101111111111111111101111111111111111111111111111111, // #zeros = 4
///     0b1111111111111101111111111111111111111000111111111110001111111110, // #zeros = 8
///     0b1101111111111111111111111111111111111111111111111111111111111111, // #zeros = 1
///     0b1111111111111111101111111011111111111111111101001110111111111111, // #zeros = 6
///     0b1111111111111111111111111111111111101111111111111111011111111111, // #zeros = 2
///     0b1110110101011011011111111111111101111111111111111000011111111111, // #zeros = 11
/// ];
///
/// // Query code
/// let qcode: u64 = 0b1111111111111111111111111111111111111111111111111111111111111111; // #zeros = 0
///
/// // Construct the index
/// let index = Index::new(codes).unwrap();
///
/// // Find the ids of neighbor codes whose Hamming distances are within 2
/// let mut searcher = index.range_searcher();
/// let answers = searcher.run(qcode, 2);
/// assert_eq!(answers, vec![1, 4, 6]);
///
/// // Find the ids of the top-4 nearest neighbor codes
/// let mut searcher = index.topk_searcher();
/// let answers = searcher.run(qcode, 4);
/// assert_eq!(answers, vec![4, 1, 6, 0]);
///
/// // Serialization/Deserialization
/// let mut data = vec![];
/// index.serialize_into(&mut data).unwrap();
/// let other = Index::<u64>::deserialize_from(&data[..]).unwrap();
/// assert_eq!(index, other);
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Index<T: CodeInt> {
    num_blocks: usize,
    codes: Vec<T>,
    tables: Vec<sparsehash::Table>,
    masks: Vec<T>,
    begs: Vec<usize>,
}

/// Range searcher created by [`Index::range_searcher()`].
pub struct RangeSearcher<'a, T: CodeInt> {
    index: &'a Index<T>,
    siggen: siggen::SigGenerator64,
    answers: Vec<u32>,
}

/// Top-K searcher created by [`Index::range_searcher()`].
pub struct TopkSearcher<'a, T: CodeInt> {
    index: &'a Index<T>,
    siggen: siggen::SigGenerator64,
    answers: Vec<u32>,
    checked: std::collections::HashSet<usize>,
}
