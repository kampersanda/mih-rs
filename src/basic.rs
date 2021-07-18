use crate::popcnt::Popcnt;
use num_traits::int::PrimInt;
use num_traits::{FromPrimitive, ToPrimitive};
use rand::{thread_rng, Rng};

/// Generic trait of binary codes.
pub trait CodeInt: PrimInt + FromPrimitive + ToPrimitive + Popcnt {
    fn dimensions() -> usize;
}

impl CodeInt for u8 {
    fn dimensions() -> usize {
        8
    }
}

impl CodeInt for u16 {
    fn dimensions() -> usize {
        16
    }
}

impl CodeInt for u32 {
    fn dimensions() -> usize {
        32
    }
}

impl CodeInt for u64 {
    fn dimensions() -> usize {
        64
    }
}

impl CodeInt for u128 {
    fn dimensions() -> usize {
        128
    }
}

/// Get the Hamming distance.
pub fn hamdist<T: CodeInt>(x: T, y: T) -> usize {
    (x ^ y).popcnt() as usize
}

/// Generate random binary codes.
pub fn random_codes(size: usize) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut codes = vec![0; size];
    for i in 0..size {
        codes[i] = rng.gen::<u64>();
    }
    codes
}
