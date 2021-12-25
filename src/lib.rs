//! # mih-rs
//!
//! Rust implementation of multi-index hashing (MIH) for neighbor searches on binary codes in the Hamming space, described in the paper
//!
//! > Norouzi, Punjani, and Fleet, [Fast exact search in Hamming space with multi-index hashing](https://arxiv.org/abs/1307.2982), *IEEE TPAMI*, 36(6):1107– 1119, 2014.
//!
//! ## Features
//!
//! - **Two types of neighbor searches:** mih-rs provides the two search operations:
//!   - *Range search* finds neighbor codes whose Hamming distances to a given code are within a radius.
//!   - *Top-K search* finds the top-K codes that are closest to a given code.
//!
//! - **Fast and memory-efficient implementation:** The data structure is built on sparse hash tables, following the [original implementation](https://github.com/norouzi/mih).
//!
//! - **Parameter free:** mih-rs automatically sets an optimal parameter of MIH depending on a given database (although you can also set this manually).
//!
//! - **Serialization:** `mih-rs` supports to serialize/deserialize the index.
//!
//! ## Example
//!
//! ```rust
//! use mih_rs::Index;
//!
//! // Database of codes
//! let codes: Vec<u64> = vec![
//!     0b1111111111111111111111011111111111111111111111111011101111111111, // #zeros = 3
//!     0b1111111111111111111111111111111101111111111011111111111111111111, // #zeros = 2
//!     0b1111111011011101111111111111111101111111111111111111111111111111, // #zeros = 4
//!     0b1111111111111101111111111111111111111000111111111110001111111110, // #zeros = 8
//!     0b1101111111111111111111111111111111111111111111111111111111111111, // #zeros = 1
//!     0b1111111111111111101111111011111111111111111101001110111111111111, // #zeros = 6
//!     0b1111111111111111111111111111111111101111111111111111011111111111, // #zeros = 2
//!     0b1110110101011011011111111111111101111111111111111000011111111111, // #zeros = 11
//! ];
//!
//! // Query code
//! let qcode: u64 = 0b1111111111111111111111111111111111111111111111111111111111111111; // #zeros = 0
//!
//! // Construct the index
//! let index = Index::new(codes).unwrap();
//!
//! // Find the ids of neighbor codes whose Hamming distances are within 2
//! let mut searcher = index.range_searcher();
//! let answers = searcher.run(qcode, 2);
//! assert_eq!(answers, vec![1, 4, 6]);
//!
//! // Find the ids of the top-4 nearest neighbor codes
//! let mut searcher = index.topk_searcher();
//! let answers = searcher.run(qcode, 4);
//! assert_eq!(answers, vec![4, 1, 6, 0]);
//!
//! // Serialization/Deserialization
//! let mut data = vec![];
//! index.serialize_into(&mut data).unwrap();
//! let other = Index::<u32>::deserialize_from(&data[..]).unwrap();
//! assert_eq!(index, other);
//! ```
//!
//! ## Binary code types
//!
//! `mih_rs::Index` can be built from a vector of type `mih_rs::CodeInt`
//! that is a primitive integer trait supporting a popcount operation.
//! Currently, this library defines `mih_rs::CodeInt` for `u8`, `u16`, `u32`, and `u64`.

/// An implementation of multi-index hashing.
pub mod index;

/// Exhaustive search functions (for benchmark).
pub mod ls;

/// A generic trait of supported binary codes.
pub mod codeint;

pub use codeint::CodeInt;
pub use index::Index;

/// Gets the Hamming distance between two binary codes.
pub fn hamdist<T: CodeInt>(x: T, y: T) -> usize {
    (x ^ y).popcnt() as usize
}
