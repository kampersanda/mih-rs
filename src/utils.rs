//! Implements some utility functions.
//! Most users do not need to use this module directly.

use rand::{thread_rng, Rng};

pub fn popcnt(mut x: u64) -> usize {
    x = (x & 0x5555555555555555) + ((x >> 1) & 0x5555555555555555);
    x = (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333);
    x = (x & 0x0f0f0f0f0f0f0f0f) + ((x >> 4) & 0x0f0f0f0f0f0f0f0f);
    x = (x & 0x00ff00ff00ff00ff) + ((x >> 8) & 0x00ff00ff00ff00ff);
    x = (x & 0x0000ffff0000ffff) + ((x >> 16) & 0x0000ffff0000ffff);
    x = (x & 0x00000000ffffffff) + ((x >> 32) & 0x00000000ffffffff);
    x as usize
}

pub fn popcnt_mask(x: u64, i: usize) -> usize {
    assert!(i < 64);
    popcnt(x & ((1 << i) - 1))
}

pub fn get(x: u64, i: usize) -> bool {
    assert!(i < 64);
    (x & (1 << i)) != 0
}

pub fn set(x: u64, i: usize) -> u64 {
    assert!(i < 64);
    x | (1 << i)
}

pub fn hamdist(x: u64, y: u64) -> usize {
    popcnt(x ^ y)
}

pub fn random_codes(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut codes = vec![0; size];
    for i in 0..size {
        codes[i] = rng.gen::<u64>();
    }
    codes
}

#[cfg(test)]
mod tests {
    use crate::utils::*;

    #[test]
    fn popcnt_works() {
        let mut rng = thread_rng();
        for _ in 0..100 {
            let x: u64 = rng.gen();
            assert_eq!(x.count_ones(), popcnt(x) as u32);
        }
    }
}
