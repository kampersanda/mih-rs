//! # mih-rs
//!
//! Rust implementation of multi-index hashing (MIH) for neighbor searches on binary codes in the Hamming space, described in the paper
//!
//! > Norouzi, Punjani, and Fleet, **Fast exact search in Hamming space with multi-index hashing**, *IEEE TPAMI*, 36(6):1107– 1119, 2014.
//!
//! ## Features
//!
//! - **Two types of neighbor searches:** mih-rs provides the two search operations:
//!   - *Range search* finds neighbor codes whose Hamming distances to a given code are within a radius.
//!   - *Top-K search* finds the top-K codes that are closest to a given code.
//!
//! - **Fast and memory-efficient implementation:** The data structure is built on sparse hash tables, following the original implementation.
//!
//! - **Parameter free:** mih-rs automatically sets an optimal parameter of MIH depending on a given database (although you can also set this manually).

/// An implementation of multi-index hashing.
pub mod index;

/// Exhaustive search functions.
pub mod ls;

/// A generic trait of supported binary codes.
pub mod codeint;

pub use codeint::CodeInt;
pub use index::Index;

/// Gets the Hamming distance between two binary codes.
pub fn hamdist<T: CodeInt>(x: T, y: T) -> usize {
    (x ^ y).popcnt() as usize
}
