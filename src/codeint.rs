use crate::popcnt::Popcnt;
use num_traits::int::PrimInt;
use num_traits::{FromPrimitive, ToPrimitive};

/// Generic trait of binary codes.
pub trait CodeInt: PrimInt + FromPrimitive + ToPrimitive + Popcnt + Default {
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
