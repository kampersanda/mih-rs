//! Implements a simple exhaustive search algorithm for comparison with MIH.

use crate::basic::*;

/// Finds the neighbors in codes, whose Hamming distances to qcode are within radius.
/// Returns the ids of the neighbor codes.
pub fn range_search<T: CodeInt>(codes: &[T], qcode: T, radius: usize) -> Vec<u32> {
    let mut answers = Vec::<u32>::with_capacity(1 << 8);
    range_search_with_buf(codes, qcode, radius, &mut answers);
    answers
}

/// Finds the neighbors in codes, whose Hamming distances to qcode are within radius.
/// The ids of the neighbor codes are stored in answers.
pub fn range_search_with_buf<T: CodeInt>(
    codes: &[T],
    qcode: T,
    radius: usize,
    answers: &mut Vec<u32>,
) {
    answers.clear();
    for i in 0..codes.len() {
        let dist = hamdist(codes[i], qcode);
        if dist <= radius {
            answers.push(i as u32);
        }
    }
}

/// Computes all the Hamming distances bwtween codes and qcode.
/// Returns the tuples of code id and the distance.
pub fn exhaustive_search<T: CodeInt>(codes: &[T], qcode: T) -> Vec<(u32, u32)> {
    let mut answers = Vec::<(u32, u32)>::new();
    exhaustive_search_with_buf(codes, qcode, &mut answers);
    answers
}

/// Computes all the Hamming distances bwtween codes and qcode.
/// The tuples of code id and the distance are stored in answers.
pub fn exhaustive_search_with_buf<T: CodeInt>(
    codes: &[T],
    qcode: T,
    answers: &mut Vec<(u32, u32)>,
) {
    answers.resize(codes.len(), Default::default());
    for i in 0..codes.len() {
        let dist = hamdist(codes[i], qcode);
        answers[i] = (i as u32, dist as u32);
    }
}
