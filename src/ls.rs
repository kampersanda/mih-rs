//! Implements a simple exhaustive search algorithm for comparison with MIH.

use crate::utils;

/// Finds the neighbors in codes, whose Hamming distances to qcode are within radius.
/// Returns the ids of the neighbor codes.
pub fn range_search(codes: &[u64], qcode: u64, radius: usize) -> Vec<usize> {
    let mut answers = Vec::<usize>::with_capacity(1 << 10);
    range_search_with_buf(codes, qcode, radius, &mut answers);
    answers
}

/// Finds the neighbors in codes, whose Hamming distances to qcode are within radius.
/// The ids of the neighbor codes are stored in answers.
pub fn range_search_with_buf(codes: &[u64], qcode: u64, radius: usize, answers: &mut Vec<usize>) {
    answers.clear();
    for i in 0..codes.len() {
        let dist = utils::hamdist(codes[i], qcode);
        if dist <= radius {
            answers.push(i);
        }
    }
}

/// Computes all the Hamming distances bwtween codes and qcode.
/// Returns the tuples of code id and the distance.
pub fn exhaustive_search(codes: &[u64], qcode: u64) -> Vec<(usize, usize)> {
    let mut answers = Vec::<(usize, usize)>::new();
    exhaustive_search_with_buf(codes, qcode, &mut answers);
    answers
}

/// Computes all the Hamming distances bwtween codes and qcode.
/// The tuples of code id and the distance are stored in answers.
pub fn exhaustive_search_with_buf(codes: &[u64], qcode: u64, answers: &mut Vec<(usize, usize)>) {
    answers.resize(codes.len(), Default::default());
    for i in 0..codes.len() {
        let dist = utils::hamdist(codes[i], qcode);
        answers[i] = (i, dist);
    }
}
