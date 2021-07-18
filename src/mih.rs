//! Implements an MIH method.

use crate::sparsehash;
use crate::utils;
use std::collections::HashSet;
use std::io::{Error, ErrorKind};

/// An index implementation of MIH.
pub struct Index<'db> {
    blocks: usize,
    codes: &'db [u64],
    tables: Vec<sparsehash::Table>,
    masks: Vec<u64>,
    begs: Vec<usize>,
}

impl Index<'_> {
    /// Constructs the index from 64-bit codes.
    /// If invalid inputs are given, return ErrorKind::InvalidInput.
    pub fn new<'db>(codes: &'db [u64]) -> Result<Index, Error> {
        let blocks = (64.0 / (codes.len() as f64).log2()).round() as usize;
        if blocks < 2 {
            Index::new_with_blocks(codes, 2)
        } else {
            Index::new_with_blocks(codes, blocks)
        }
    }

    /// Constructs the index from 64-bit codes using manual parameter of blocks.
    /// If invalid inputs are given, return ErrorKind::InvalidInput.
    pub fn new_with_blocks<'db>(codes: &'db [u64], blocks: usize) -> Result<Index, Error> {
        if codes.is_empty() {
            let e = Error::new(ErrorKind::InvalidInput, "codes must not be empty.");
            return Err(e);
        }
        if (u32::max_value() as usize) < codes.len() {
            let e = Error::new(
                ErrorKind::InvalidInput,
                "number of codes must be no more than 2^32.",
            );
            return Err(e);
        }
        if blocks < 2 || 64 < blocks {
            let e = Error::new(ErrorKind::InvalidInput, "blocks must be in [2,64].");
            return Err(e);
        }

        let mut masks = vec![0 as u64; blocks];
        let mut begs = vec![0 as usize; blocks + 1];

        for b in 0..blocks {
            let dim = (b + 64) / blocks;
            masks[b] = (1 << dim) - 1;
            begs[b + 1] = begs[b] + dim;
        }

        let mut tables = Vec::<sparsehash::Table>::with_capacity(blocks);

        for b in 0..blocks {
            let beg = begs[b];
            let dim = begs[b + 1] - begs[b];

            let mut table = sparsehash::Table::new(dim)?;

            for id in 0..codes.len() {
                let pos = (codes[id] >> beg) & masks[b];
                table.count_insert(pos as usize);
            }
            for id in 0..codes.len() {
                let pos = (codes[id] >> beg) & masks[b];
                table.data_insert(pos as usize, id as u32);
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
    pub fn range_search(&self, qcode: u64, radius: usize) -> Vec<usize> {
        let mut answers = Vec::<usize>::with_capacity(1 << 10);
        self.range_search_with_buf(qcode, radius, &mut answers);
        answers
    }

    /// Finds the neighbor codes whose Hamming distances to qcode are within radius.
    /// The ids of the neighbor codes are stored in answers.
    pub fn range_search_with_buf(&self, qcode: u64, radius: usize, answers: &mut Vec<usize>) {
        answers.clear();

        let blocks = self.get_blocks();
        let mut siggen = SigGenerator::default();

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
                            answers.push(*v as usize);
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
                    let dist = utils::hamdist(qcode, self.codes[answers[i]]);
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
    pub fn topk_search(&self, qcode: u64, topk: usize) -> Vec<usize> {
        let mut answers = Vec::<usize>::new();
        self.topk_search_with_buf(qcode, topk, &mut answers);
        answers
    }

    /// Finds the topk codes that are closest to qcode.
    /// The ids of the topk codes are stored in answers.
    pub fn topk_search_with_buf(&self, qcode: u64, topk: usize, answers: &mut Vec<usize>) {
        answers.resize(65 * topk, Default::default());

        let blocks = self.get_blocks();
        let mut siggen = SigGenerator::default();

        let mut n = 0;
        let mut r = 0;

        let mut counts = vec![0 as usize; 65];
        let mut checked = HashSet::<usize>::new();

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
                                let dist = utils::hamdist(qcode, self.codes[id]);
                                if counts[dist] < topk {
                                    answers[dist * topk + counts[dist]] = id;
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

    fn get_blocks(&self) -> usize {
        self.blocks
    }

    fn get_dim(&self, b: usize) -> usize {
        self.begs[b + 1] - self.begs[b]
    }

    fn get_chunk(&self, code: u64, b: usize) -> u64 {
        (code >> self.begs[b]) & self.masks[b]
    }
}

#[derive(Clone)]
pub struct SigGenerator {
    sig: u64,
    base: u64,
    radius: usize,
    bit: isize,
    power: [usize; 64],
}

impl Default for SigGenerator {
    fn default() -> SigGenerator {
        SigGenerator {
            sig: 0,
            base: 0,
            radius: 0,
            bit: 0,
            power: [0; 64],
        }
    }
}

impl SigGenerator {
    fn init(&mut self, base: u64, dim: usize, radius: usize) {
        assert!(radius < dim);

        self.sig = 0;
        self.base = base;
        self.radius = radius;
        self.bit = radius as isize - 1;

        for i in 0..radius {
            self.power[i] = i;
        }
        self.power[radius] = dim + 1;
    }

    fn has_next(&self) -> bool {
        self.bit != self.radius as isize
    }

    fn next(&mut self) -> u64 {
        assert!(self.has_next());

        while self.bit != -1 {
            let idx = self.bit as usize;
            if self.power[idx] == idx {
                self.sig ^= 1 << self.power[idx];
            } else {
                assert!(0 < self.power[idx]);
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

            assert!(0 < self.power[idx]);
            self.sig ^= 1 << (self.power[idx] - 1);
            self.power[idx] = idx;
        }

        tmp ^ self.base
    }
}

#[cfg(test)]
mod tests {
    use crate::ls;
    use crate::mih::*;
    use std::collections::BTreeSet;

    fn naive_topk_search(codes: &[u64], qcode: u64, topk: usize) -> Vec<usize> {
        let mut cands = ls::exhaustive_search(codes, qcode);
        cands.sort_by_key(|x| x.1);

        let max_dist = cands[topk - 1].1;

        let mut i = 0;
        let mut answers = Vec::<usize>::new();
        while cands[i].1 <= max_dist {
            answers.push(cands[i].0);
            i += 1;
        }
        answers
    }

    #[test]
    fn range_search_works() {
        let codes = utils::random_codes(10000);
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

    #[test]
    fn topk_search_works() {
        let codes = utils::random_codes(10000);
        let index = Index::new(&codes).unwrap();

        for topk in &[1, 10, 100] {
            for qi in (0..10000).step_by(100) {
                let qcode = codes[qi];
                let ans1 = naive_topk_search(&codes, qcode, *topk);
                let ans2 = index.topk_search(qcode, *topk);
                let set1: BTreeSet<usize> = ans1.into_iter().collect();
                let set2: BTreeSet<usize> = ans2.into_iter().collect();
                assert_eq!(set2.is_subset(&set1), true);
            }
        }
    }

    #[test]
    fn siggen_works() {
        let mut siggen = SigGenerator::default();
        for k in 1..5 {
            siggen.init(0, 32, k);
            while siggen.has_next() {
                let sig = siggen.next();
                assert_eq!(sig.count_ones(), k as u32);
            }
        }
    }
}
