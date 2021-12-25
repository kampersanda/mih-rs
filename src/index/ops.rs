use std::collections::HashSet;

use anyhow::{anyhow, Result};

use crate::hamdist;
use crate::index::siggen::SigGenerator64;
use crate::index::sparsehash::Table;
use crate::index::CodeInt;
use crate::Index;

impl<T: CodeInt> Index<T> {
    /// Constructs the index from binary codes.
    /// If invalid inputs are given, return ErrorKind::InvalidInput.
    pub fn new(codes: Vec<T>) -> Result<Self> {
        let num_codes = codes.len() as f64;
        let dimensions = T::dimensions() as f64;

        let blocks = (dimensions / num_codes.log2()).round() as usize;
        if blocks < 2 {
            Self::with_blocks(codes, 2)
        } else {
            Self::with_blocks(codes, blocks)
        }
    }

    /// Constructs the index from 64-bit codes using manual parameter of blocks.
    /// If invalid inputs are given, return ErrorKind::InvalidInput.
    pub fn with_blocks(codes: Vec<T>, num_blocks: usize) -> Result<Self> {
        if codes.is_empty() {
            return Err(anyhow!("The input codes must not be empty"));
        }

        if (u32::max_value() as usize) < codes.len() {
            return Err(anyhow!(
                "The number of codes {} must not be no more than {}.",
                codes.len(),
                u32::max_value()
            ));
        }

        let num_dimensions = T::dimensions();
        if num_blocks < 2 || num_dimensions < num_blocks {
            return Err(anyhow!(
                "The number of blocks {} must not be in [2,{}]",
                num_blocks,
                num_dimensions
            ));
        }

        let mut masks = vec![T::default(); num_blocks];
        let mut begs = vec![0; num_blocks + 1];

        for b in 0..num_blocks {
            let dim = (b + num_dimensions) / num_blocks;
            if 64 == dim {
                masks[b] = T::from_u64(u64::max_value()).unwrap();
            } else {
                masks[b] = T::from_u64((1 << dim) - 1).unwrap();
            }
            begs[b + 1] = begs[b] + dim;
        }

        let mut tables = Vec::<Table>::with_capacity(num_blocks);

        for b in 0..num_blocks {
            let beg = begs[b];
            let dim = begs[b + 1] - begs[b];

            let mut table = Table::new(dim)?;

            for &code in &codes {
                let chunk = (code >> beg) & masks[b];
                table.count_insert(chunk.to_u64().unwrap() as usize);
            }

            for (id, &code) in codes.iter().enumerate() {
                let chunk = (code >> beg) & masks[b];
                table.data_insert(chunk.to_u64().unwrap() as usize, id as u32);
            }

            tables.push(table);
        }

        Ok(Self {
            num_blocks,
            codes,
            tables,
            masks,
            begs,
        })
    }

    /// Finds the neighbor codes whose Hamming distances to qcode are within radius.
    /// Returns the ids of the neighbor codes.
    pub fn range_search(&self, qcode: T, radius: usize) -> Vec<u32> {
        let mut answers = Vec::<u32>::with_capacity(1 << 10);
        self.range_search_with_buf(qcode, radius, &mut answers);
        answers
    }

    /// Finds the neighbor codes whose Hamming distances to qcode are within radius.
    /// The ids of the neighbor codes are stored in answers.
    pub fn range_search_with_buf(&self, qcode: T, radius: usize, answers: &mut Vec<u32>) {
        answers.clear();

        let num_blocks = self.num_blocks();
        let mut siggen = SigGenerator64::new();

        for b in 0..num_blocks {
            // Based on the general pigeonhole principle
            if b + radius + 1 < num_blocks {
                continue;
            }

            let rad = (b + radius + 1 - num_blocks) / num_blocks;
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
            answers.sort_unstable();
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

        answers.resize(n, u32::default());
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
        let num_dimensions = T::dimensions();
        answers.resize((num_dimensions + 1) * topk, Default::default());

        let num_blocks = self.num_blocks();
        let mut siggen = SigGenerator64::new();

        let mut n = 0;
        let mut r = 0;

        let mut counts = vec![0; num_dimensions + 1];
        let mut checked = HashSet::new();

        while n < topk {
            for b in 0..num_blocks {
                let dim = self.get_dim(b);
                let qcd = self.get_chunk(qcode, b);
                let table = &self.tables[b];

                siggen.init(qcd, dim, r);
                while siggen.has_next() {
                    let sig = siggen.next();
                    if let Some(a) = table.access(sig as usize) {
                        for &v in a {
                            let id = v as usize;
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

                n += counts[r * num_blocks + b];
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
        answers.resize(topk, u32::default());
    }

    /// Gets the number of defined blocks.
    pub fn num_blocks(&self) -> usize {
        self.num_blocks
    }

    pub fn codes(&self) -> &[T] {
        &self.codes
    }

    fn get_dim(&self, b: usize) -> usize {
        self.begs[b + 1] - self.begs[b]
    }

    fn get_chunk(&self, code: T, b: usize) -> u64 {
        let chunk = (code >> self.begs[b]) & self.masks[b];
        chunk.to_u64().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ls;
    use rand::distributions::{Distribution, Standard};
    use rand::{thread_rng, Rng};
    use std::collections::BTreeSet;

    pub fn gen_random_codes<T>(size: usize) -> Vec<T>
    where
        Standard: Distribution<T>,
    {
        let mut rng = thread_rng();
        let mut codes: Vec<T> = Vec::with_capacity(size);
        for _ in 0..size {
            codes.push(rng.gen::<T>());
        }
        codes
    }

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

    fn do_range_search<T: CodeInt>(codes: Vec<T>) {
        let index = Index::new(codes).unwrap();
        for rad in 0..6 {
            for qi in (0..10000).step_by(100) {
                let qcode = index.codes()[qi];
                let ans1 = ls::range_search(index.codes(), qcode, rad);
                let ans2 = index.range_search(qcode, rad);
                assert_eq!(ans1, ans2);
            }
        }
    }

    fn do_topk_search<T: CodeInt>(codes: Vec<T>) {
        let index = Index::new(codes).unwrap();
        for topk in &[1, 10, 100] {
            for qi in (0..10000).step_by(100) {
                let qcode = index.codes()[qi];
                let ans1 = naive_topk_search(index.codes(), qcode, *topk);
                let ans2 = index.topk_search(qcode, *topk);
                let set1: BTreeSet<u32> = ans1.into_iter().collect();
                let set2: BTreeSet<u32> = ans2.into_iter().collect();
                assert_eq!(set2.is_subset(&set1), true);
            }
        }
    }

    #[test]
    fn range_search_u8_works() {
        let codes = gen_random_codes::<u8>(10000);
        do_range_search(codes);
    }

    #[test]
    fn range_search_u16_works() {
        let codes = gen_random_codes::<u16>(10000);
        do_range_search(codes);
    }

    #[test]
    fn range_search_u32_works() {
        let codes = gen_random_codes::<u32>(10000);
        do_range_search(codes);
    }

    #[test]
    fn range_search_u64_works() {
        let codes = gen_random_codes::<u64>(10000);
        do_range_search(codes);
    }

    #[test]
    fn topk_search_u8_works() {
        let codes = gen_random_codes::<u8>(10000);
        do_topk_search(codes);
    }

    #[test]
    fn topk_search_u16_works() {
        let codes = gen_random_codes::<u16>(10000);
        do_topk_search(codes);
    }

    #[test]
    fn topk_search_u32_works() {
        let codes = gen_random_codes::<u32>(10000);
        do_topk_search(codes);
    }

    #[test]
    fn topk_search_u64_works() {
        let codes = gen_random_codes::<u64>(10000);
        do_topk_search(codes);
    }
}
