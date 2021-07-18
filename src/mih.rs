use crate::basic::*;
use crate::sparsehash;

use std::collections::HashSet;
use std::io::{Error, ErrorKind};

/// Implementation of multi-index hashing.
pub struct Index<'db, T: CodeInt> {
    blocks: usize,
    codes: &'db [T],
    tables: Vec<sparsehash::Table>,
    masks: Vec<u64>,
    begs: Vec<usize>,
}

impl<T: CodeInt> Index<'_, T> {
    /// Constructs the index from binary codes.
    /// If invalid inputs are given, return ErrorKind::InvalidInput.
    pub fn new<'db>(codes: &'db [T]) -> Result<Index<T>, Error> {
        let codes_size = codes.len() as f64;
        let dimensions = T::dimensions() as f64;

        let blocks = (dimensions / codes_size.log2()).round() as usize;
        if blocks < 2 {
            Index::new_with_blocks(codes, 2)
        } else {
            Index::new_with_blocks(codes, blocks)
        }
    }

    /// Constructs the index from 64-bit codes using manual parameter of blocks.
    /// If invalid inputs are given, return ErrorKind::InvalidInput.
    pub fn new_with_blocks<'db>(codes: &'db [T], blocks: usize) -> Result<Index<T>, Error> {
        if codes.is_empty() {
            let e = Error::new(ErrorKind::InvalidInput, "codes must not be empty.");
            return Err(e);
        }

        if (u32::max_value() as usize) < codes.len() {
            let e = Error::new(
                ErrorKind::InvalidInput,
                "codes.len() must be no more than 2^32.",
            );
            return Err(e);
        }

        let dimensions = T::dimensions();

        if blocks < 2 || dimensions < blocks {
            let e = Error::new(
                ErrorKind::InvalidInput,
                "blocks must be in [2..dimensions].",
            );
            return Err(e);
        }

        let mut masks = vec![0u64; blocks];
        let mut begs = vec![0usize; blocks + 1];

        for b in 0..blocks {
            let dim = (b + dimensions) / blocks;
            if 64 < dim {
                let e = Error::new(ErrorKind::InvalidInput, "dim must be no more than 64.");
                return Err(e);
            }
            masks[b] = (1 << dim) - 1;
            begs[b + 1] = begs[b] + dim;
        }

        let mut tables = Vec::<sparsehash::Table>::with_capacity(blocks);

        for b in 0..blocks {
            let beg = begs[b];
            let dim = begs[b + 1] - begs[b];

            let mut table = sparsehash::Table::new(dim)?;

            for id in 0..codes.len() {
                let subcode = (codes[id] >> beg).to_u64().unwrap() & masks[b];
                table.count_insert(subcode as usize);
            }

            for id in 0..codes.len() {
                let subcode = (codes[id] >> beg).to_u64().unwrap() & masks[b];
                table.data_insert(subcode as usize, id as u32);
            }

            tables.push(table);
        }

        Ok(Index {
            blocks: blocks,
            codes: codes,
            tables: tables,
            masks: masks,
            begs: begs,
        })
    }

    /// Finds the neighbor codes whose Hamming distances to qcode are within radius.
    /// Returns the ids of the neighbor codes.
    pub fn range_search(&self, qcode: T, radius: usize) -> Vec<u32> {
        let mut answers = Vec::<u32>::with_capacity(1 << 8);
        self.range_search_with_buf(qcode, radius, &mut answers);
        answers
    }

    /// Finds the neighbor codes whose Hamming distances to qcode are within radius.
    /// The ids of the neighbor codes are stored in answers.
    pub fn range_search_with_buf(&self, qcode: T, radius: usize, answers: &mut Vec<u32>) {
        answers.clear();

        let blocks = self.get_blocks();
        let mut siggen = SigGenerator64::new();

        for b in 0..blocks {
            // Based on the general pigeonhole principle
            if b + radius + 1 < blocks {
                continue;
            }

            let rad = (b + radius + 1 - blocks) / blocks;
            let dim = self.get_dim(b);
            let qcd = self.get_chunk(qcode, b);

            let table = &self.tables[b];

            // Search with r errors
            for r in 0..rad + 1 {
                siggen.init(qcd, dim, r);
                while siggen.has_next() {
                    let sig = siggen.next();
                    if let Some(a) = table.access(sig as usize) {
                        for v in a {
                            answers.push(*v as u32);
                        }
                    }
                }
            }
        }

        let mut n = 0;
        if !answers.is_empty() {
            answers.sort();
            for i in 0..answers.len() {
                if i == 0 || answers[i - 1] != answers[i] {
                    let dist = hamdist(qcode, self.codes[answers[i] as usize]);
                    if dist <= radius {
                        answers[n] = answers[i];
                        n += 1;
                    }
                }
            }
        }
        answers.resize(n, Default::default());
    }

    /// Finds the topk codes that are closest to qcode.
    /// Returns the ids of the topk codes.
    pub fn topk_search(&self, qcode: T, topk: usize) -> Vec<u32> {
        let mut answers = Vec::new();
        self.topk_search_with_buf(qcode, topk, &mut answers);
        answers
    }

    /// Finds the topk codes that are closest to qcode.
    /// The ids of the topk codes are stored in answers.
    pub fn topk_search_with_buf(&self, qcode: T, topk: usize, answers: &mut Vec<u32>) {
        let dimensions = T::dimensions();
        answers.resize((dimensions + 1) * topk, Default::default());

        let blocks = self.get_blocks();
        let mut siggen = SigGenerator64::new();

        let mut n = 0;
        let mut r = 0;

        let mut counts = vec![0; dimensions + 1];
        let mut checked = HashSet::new();

        while n < topk {
            for b in 0..blocks {
                let dim = self.get_dim(b);
                let qcd = self.get_chunk(qcode, b);
                let table = &self.tables[b];

                siggen.init(qcd, dim, r);
                while siggen.has_next() {
                    let sig = siggen.next();

                    if let Some(a) = table.access(sig as usize) {
                        for v in a {
                            let id = *v as usize;

                            if checked.insert(id) {
                                let dist = hamdist(qcode, self.codes[id]);
                                if counts[dist] < topk {
                                    answers[dist * topk + counts[dist]] = id as u32;
                                }
                                counts[dist] += 1;
                            }
                        }
                    }
                }

                n += counts[r * blocks + b];
                if topk <= n {
                    break;
                }
            }

            r += 1;
        }

        n = 0;
        r = 0;
        while n < topk {
            let mut i = 0;
            while i < counts[r] && n < topk {
                answers[n] = answers[r * topk + i];
                i += 1;
                n += 1;
            }
            r += 1;
        }
        answers.resize(topk, Default::default());
    }

    pub fn get_blocks(&self) -> usize {
        self.blocks
    }

    fn get_dim(&self, b: usize) -> usize {
        self.begs[b + 1] - self.begs[b]
    }

    fn get_chunk(&self, code: T, b: usize) -> u64 {
        (code >> self.begs[b] as usize).to_u64().unwrap() & self.masks[b]
    }
}

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
    fn new() -> SigGenerator64 {
        SigGenerator64 {
            sig: 0,
            base: 0,
            radius: 0,
            bit: 0,
            power: [0; 64],
        }
    }

    /// Initialize the generator.
    fn init(&mut self, base: u64, dim: usize, radius: usize) {
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
    fn has_next(&self) -> bool {
        self.bit != self.radius as isize
    }

    /// Get the next signature.
    fn next(&mut self) -> u64 {
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
    use crate::{basic, ls};
    use std::collections::BTreeSet;

    fn naive_topk_search<T: CodeInt>(codes: &[T], qcode: T, topk: usize) -> Vec<u32> {
        let mut cands = ls::exhaustive_search(codes, qcode);
        cands.sort_by_key(|x| x.1);

        let max_dist = cands[topk - 1].1;

        let mut i = 0;
        let mut answers = Vec::new();

        while cands[i].1 <= max_dist {
            answers.push(cands[i].0);
            i += 1;
        }
        answers
    }

    fn do_range_search<T: CodeInt>(codes: &[T]) {
        let index = Index::new(&codes).unwrap();
        for rad in 0..6 {
            for qi in (0..10000).step_by(100) {
                let qcode = codes[qi];
                let ans1 = ls::range_search(&codes, qcode, rad);
                let ans2 = index.range_search(qcode, rad);
                assert_eq!(ans1, ans2);
            }
        }
    }

    fn do_topk_search<T: CodeInt>(codes: &[T]) {
        let index = Index::new(&codes).unwrap();
        for topk in &[1, 10, 100] {
            for qi in (0..10000).step_by(100) {
                let qcode = codes[qi];
                let ans1 = naive_topk_search(&codes, qcode, *topk);
                let ans2 = index.topk_search(qcode, *topk);
                let set1: BTreeSet<u32> = ans1.into_iter().collect();
                let set2: BTreeSet<u32> = ans2.into_iter().collect();
                assert_eq!(set2.is_subset(&set1), true);
            }
        }
    }

    #[test]
    fn range_search_works() {
        let codes = basic::random_codes(10000);
        do_range_search(&codes);
    }

    #[test]
    fn topk_search_works() {
        let codes = basic::random_codes(10000);
        do_topk_search(&codes);
    }

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
