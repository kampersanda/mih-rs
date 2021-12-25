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

/// Generic trait for pop-countable integers.
pub trait Popcnt {
    fn popcnt(&self) -> u32;
}

impl Popcnt for u8 {
    fn popcnt(&self) -> u32 {
        self.count_ones()
    }
}

impl Popcnt for u16 {
    fn popcnt(&self) -> u32 {
        self.count_ones()
    }
}

impl Popcnt for u32 {
    fn popcnt(&self) -> u32 {
        self.count_ones()
    }
}

impl Popcnt for u64 {
    fn popcnt(&self) -> u32 {
        self.count_ones()
    }
}
