use num_traits::int::PrimInt;
use num_traits::{FromPrimitive, ToPrimitive};

pub mod popcnt;
use popcnt::Popcnt;

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
