/// Generator of similar 64-bit codes (or signatures).
pub struct SigGenerator64 {
    sig: u64,
    base: u64,
    radius: usize,
    bit: isize,
    power: [usize; 64],
}

impl SigGenerator64 {
    /// Create a new generator.
    pub const fn new() -> Self {
        Self {
            sig: 0,
            base: 0,
            radius: 0,
            bit: 0,
            power: [0; 64],
        }
    }

    /// Initialize the generator.
    pub fn init(&mut self, base: u64, dim: usize, radius: usize) {
        debug_assert!(radius < dim);

        self.sig = 0;
        self.base = base;
        self.radius = radius;
        self.bit = radius as isize - 1;

        for i in 0..radius {
            self.power[i] = i;
        }
        self.power[radius] = dim + 1;
    }

    /// Check if the next signature exists.
    pub const fn has_next(&self) -> bool {
        self.bit != self.radius as isize
    }

    /// Get the next signature.
    pub fn next(&mut self) -> u64 {
        debug_assert!(self.has_next());

        while self.bit != -1 {
            let idx = self.bit as usize;
            if self.power[idx] == idx {
                self.sig ^= 1 << self.power[idx];
            } else {
                debug_assert!(0 < self.power[idx]);
                self.sig ^= 3 << (self.power[idx] - 1);
            }
            self.power[idx] += 1;
            self.bit -= 1;
        }

        let tmp = self.sig;

        loop {
            self.bit += 1;

            let idx = self.bit as usize;
            if idx >= self.radius || self.power[idx] + 1 != self.power[idx + 1] {
                break;
            }

            debug_assert!(0 < self.power[idx]);
            self.sig ^= 1 << (self.power[idx] - 1);
            self.power[idx] = idx;
        }

        tmp ^ self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn siggen_works() {
        let mut siggen = SigGenerator64::new();
        for k in 1..5 {
            siggen.init(0, 32, k);
            while siggen.has_next() {
                let sig = siggen.next();
                assert_eq!(sig.count_ones(), k as u32);
            }
        }
    }
}
