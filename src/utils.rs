//! Implements some utility functions.
//! Most users do not need to use this module directly.

use rand::{thread_rng, Rng};

pub fn random_codes(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut codes = vec![0; size];
    for i in 0..size {
        codes[i] = rng.gen::<u64>();
    }
    codes
}
