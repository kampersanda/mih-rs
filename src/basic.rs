use crate::popcnt::Popcnt;
use num_traits::int::PrimInt;
use num_traits::{FromPrimitive, ToPrimitive};

/// Trait of binary code
pub trait CodeInt: PrimInt + FromPrimitive + ToPrimitive + Popcnt {}

impl CodeInt for u8 {}

impl CodeInt for u16 {}

impl CodeInt for u32 {}

impl CodeInt for u64 {}

impl CodeInt for u128 {}

/// Get the Hamming distance.
pub fn hamdist<T: CodeInt>(x: T, y: T) -> usize {
    (x ^ y).popcnt() as usize
}
