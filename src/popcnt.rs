pub trait Popcnt {
    fn popcnt(&self) -> u32;
}

impl Popcnt for u8 {
    fn popcnt(&self) -> u32 {
        popcnt_64(*self as u64)
    }
}

impl Popcnt for u16 {
    fn popcnt(&self) -> u32 {
        popcnt_64(*self as u64)
    }
}

impl Popcnt for u32 {
    fn popcnt(&self) -> u32 {
        popcnt_64(*self as u64)
    }
}

impl Popcnt for u64 {
    fn popcnt(&self) -> u32 {
        popcnt_64(*self)
    }
}

impl Popcnt for u128 {
    fn popcnt(&self) -> u32 {
        let x1 = *self & (u64::max_value() as u128);
        let x2 = *self >> 64;
        popcnt_64(x1 as u64) + popcnt_64(x2 as u64)
    }
}

pub fn popcnt_64(mut x: u64) -> u32 {
    x = (x & 0x5555555555555555) + ((x >> 1) & 0x5555555555555555);
    x = (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333);
    x = (x & 0x0f0f0f0f0f0f0f0f) + ((x >> 4) & 0x0f0f0f0f0f0f0f0f);
    x = (x & 0x00ff00ff00ff00ff) + ((x >> 8) & 0x00ff00ff00ff00ff);
    x = (x & 0x0000ffff0000ffff) + ((x >> 16) & 0x0000ffff0000ffff);
    x = (x & 0x00000000ffffffff) + ((x >> 32) & 0x00000000ffffffff);
    x as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn popcnt8_works() {
        let mut rng = thread_rng();
        for _ in 0..100 {
            let x: u8 = rng.gen();
            assert_eq!(x.count_ones(), x.popcnt());
        }
    }

    #[test]
    fn popcnt16_works() {
        let mut rng = thread_rng();
        for _ in 0..100 {
            let x: u16 = rng.gen();
            assert_eq!(x.count_ones(), x.popcnt());
        }
    }

    #[test]
    fn popcnt32_works() {
        let mut rng = thread_rng();
        for _ in 0..100 {
            let x: u32 = rng.gen();
            assert_eq!(x.count_ones(), x.popcnt());
        }
    }

    #[test]
    fn popcnt64_works() {
        let mut rng = thread_rng();
        for _ in 0..100 {
            let x: u64 = rng.gen();
            assert_eq!(x.count_ones(), x.popcnt());
        }
    }

    #[test]
    fn popcnt128_works() {
        let mut rng = thread_rng();
        for _ in 0..100 {
            let x: u128 = rng.gen();
            assert_eq!(x.count_ones(), x.popcnt());
        }
    }
}
