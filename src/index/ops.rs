use anyhow::{anyhow, Result};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{hamdist, index::*, Index};

impl<T: CodeInt> Index<T> {
    /// Builds an index from binary codes.
    /// The number of blocks for multi-index is set to the optimal one
    /// estimated from the number of input codes.
    /// The input database `codes` is stolen, but the reference can be gotten with [`Index::codes()`].
    ///
    /// # Arguments
    ///
    /// - `codes`: Vector of binary codes of type [`CodeInt`].
    ///
    /// # Errors
    ///
    /// `anyhow::Error` will be returned when
    ///
    ///  - the `codes` is empty, or
    ///  - the number of entries in `codes` is more than `u32::max_value()`.
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

    /// Builds an index from binary codes with a manually specified number of blocks.
    /// The input database `codes` is stolen, but the reference can be gotten with [`Index::codes()`].
    ///
    /// # Arguments
    ///
    /// - `codes`: Vector of binary codes of type [`CodeInt`].
    /// - `num_blocks`: The number of blocks for multi-index.
    ///
    /// # Errors
    ///
    /// `anyhow::Error` will be returned when
    ///
    ///  - the `codes` is empty,
    ///  - the number of entries in `codes` is more than `u32::max_value()`, or
    ///  - `num_blocks` is less than 2 or more than the number of dimensions in a binary code.
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

        let mut tables = Vec::<sparsehash::Table>::with_capacity(num_blocks);

        for b in 0..num_blocks {
            let beg = begs[b];
            let dim = begs[b + 1] - begs[b];

            let mut table = sparsehash::Table::new(dim)?;

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

    /// Returns a searcher [`RangeSearcher`] to find neighbor codes
    /// whose Hamming distances to a query code are within a query radius.
    ///
    /// # Examples
    ///
    /// ```
    /// use mih_rs::Index;
    ///
    /// let codes: Vec<u64> = vec![
    ///     0b1111111111111111111111011111111111111111111111111011101111111111, // #zeros = 3
    ///     0b1111111111111111111111111111111101111111111011111111111111111111, // #zeros = 2
    ///     0b1111111011011101111111111111111101111111111111111111111111111111, // #zeros = 4
    ///     0b1111111111111101111111111111111111111000111111111110001111111110, // #zeros = 8
    ///     0b1101111111111111111111111111111111111111111111111111111111111111, // #zeros = 1
    ///     0b1111111111111111101111111011111111111111111101001110111111111111, // #zeros = 6
    ///     0b1111111111111111111111111111111111101111111111111111011111111111, // #zeros = 2
    ///     0b1110110101011011011111111111111101111111111111111000011111111111, // #zeros = 11
    /// ];
    ///
    /// let index = Index::new(codes).unwrap();
    /// let mut searcher = index.range_searcher();
    ///
    /// let qcode: u64 = 0b1111111111111111111111111111111111111111111111111111111111111111; // #zeros = 0
    /// let answers = searcher.run(qcode, 2);
    /// assert_eq!(answers, vec![1, 4, 6]);
    /// ```
    pub fn range_searcher(&self) -> RangeSearcher<T> {
        RangeSearcher {
            index: self,
            siggen: siggen::SigGenerator64::new(),
            answers: Vec::with_capacity(1 << 10),
        }
    }

    /// Returns a searcher [`TopkSearcher`] to find top-K codes that are closest to a query code.
    ///
    /// # Examples
    ///
    /// ```
    /// use mih_rs::Index;
    ///
    /// let codes: Vec<u64> = vec![
    ///     0b1111111111111111111111011111111111111111111111111011101111111111, // #zeros = 3
    ///     0b1111111111111111111111111111111101111111111011111111111111111111, // #zeros = 2
    ///     0b1111111011011101111111111111111101111111111111111111111111111111, // #zeros = 4
    ///     0b1111111111111101111111111111111111111000111111111110001111111110, // #zeros = 8
    ///     0b1101111111111111111111111111111111111111111111111111111111111111, // #zeros = 1
    ///     0b1111111111111111101111111011111111111111111101001110111111111111, // #zeros = 6
    ///     0b1111111111111111111111111111111111101111111111111111011111111111, // #zeros = 2
    ///     0b1110110101011011011111111111111101111111111111111000011111111111, // #zeros = 11
    /// ];
    ///
    /// let index = Index::new(codes).unwrap();
    /// let mut searcher = index.topk_searcher();
    ///
    /// let qcode: u64 = 0b1111111111111111111111111111111111111111111111111111111111111111; // #zeros = 0
    /// let answers = searcher.run(qcode, 4);
    /// assert_eq!(answers, vec![4, 1, 6, 0]);
    /// ```
    pub fn topk_searcher(&self) -> TopkSearcher<T> {
        TopkSearcher {
            index: self,
            siggen: siggen::SigGenerator64::new(),
            answers: Vec::with_capacity(1 << 10),
            checked: std::collections::HashSet::new(),
        }
    }

    /// Gets the reference of the input database.
    ///
    /// # Examples
    ///
    /// ```
    /// use mih_rs::Index;
    ///
    /// let codes: Vec<u64> = vec![
    ///     0b1111111111111111111111011111111111111111111111111011101111111111, // #zeros = 3
    ///     0b1111111111111111111111111111111101111111111011111111111111111111, // #zeros = 2
    ///     0b1111111011011101111111111111111101111111111111111111111111111111, // #zeros = 4
    ///     0b1111111111111101111111111111111111111000111111111110001111111110, // #zeros = 8
    ///     0b1101111111111111111111111111111111111111111111111111111111111111, // #zeros = 1
    ///     0b1111111111111111101111111011111111111111111101001110111111111111, // #zeros = 6
    ///     0b1111111111111111111111111111111111101111111111111111011111111111, // #zeros = 2
    ///     0b1110110101011011011111111111111101111111111111111000011111111111, // #zeros = 11
    /// ];
    ///
    /// let index = Index::new(codes.clone()).unwrap();
    /// assert_eq!(codes, index.codes());
    /// ```
    pub fn codes(&self) -> &[T] {
        &self.codes
    }

    /// Gets the number of defined blocks in multi-index.
    pub fn num_blocks(&self) -> usize {
        self.num_blocks
    }

    /// Serializes the index into the file.
    pub fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> Result<()> {
        writer.write_u64::<LittleEndian>(self.num_blocks as u64)?;
        writer.write_u64::<LittleEndian>(self.codes.len() as u64)?;
        for x in &self.codes {
            x.serialize_into(&mut writer)?;
        }
        writer.write_u64::<LittleEndian>(self.tables.len() as u64)?;
        for x in &self.tables {
            x.serialize_into(&mut writer)?;
        }
        writer.write_u64::<LittleEndian>(self.masks.len() as u64)?;
        for x in &self.masks {
            x.serialize_into(&mut writer)?;
        }
        writer.write_u64::<LittleEndian>(self.begs.len() as u64)?;
        for &x in &self.begs {
            writer.write_u64::<LittleEndian>(x as u64)?;
        }
        Ok(())
    }

    /// Deserializes the index from the file.
    pub fn deserialize_from<R: std::io::Read>(mut reader: R) -> Result<Self> {
        let num_blocks = reader.read_u64::<LittleEndian>()? as usize;
        let codes = {
            let len = reader.read_u64::<LittleEndian>()? as usize;
            let mut codes = Vec::with_capacity(len);
            for _ in 0..len {
                codes.push(T::deserialize_from(&mut reader)?);
            }
            codes
        };
        let tables = {
            let len = reader.read_u64::<LittleEndian>()? as usize;
            let mut tables = Vec::with_capacity(len);
            for _ in 0..len {
                tables.push(sparsehash::Table::deserialize_from(&mut reader)?);
            }
            tables
        };
        let masks = {
            let len = reader.read_u64::<LittleEndian>()? as usize;
            let mut masks = Vec::with_capacity(len);
            for _ in 0..len {
                masks.push(T::deserialize_from(&mut reader)?);
            }
            masks
        };
        let begs = {
            let len = reader.read_u64::<LittleEndian>()? as usize;
            let mut begs = Vec::with_capacity(len);
            for _ in 0..len {
                begs.push(reader.read_u64::<LittleEndian>()? as usize);
            }
            begs
        };
        Ok(Self {
            num_blocks,
            codes,
            tables,
            masks,
            begs,
        })
    }

    fn get_dim(&self, b: usize) -> usize {
        self.begs[b + 1] - self.begs[b]
    }

    fn get_chunk(&self, code: T, b: usize) -> u64 {
        let chunk = (code >> self.begs[b]) & self.masks[b];
        chunk.to_u64().unwrap()
    }
}

impl<'a, T> RangeSearcher<'a, T>
where
    T: CodeInt,
{
    /// Searches neighbor codes whose Hamming distances to a query code are within a query radius.
    ///
    /// # Arguments
    ///
    /// - `qcode`: Binary code of the query.
    /// - `radius`: Threshold to be searched.
    ///
    /// # Returns
    ///
    /// A slice of ids of codes whose Hamming distances to `qcode` are within `radius`.
    /// The ids are sorted.
    /// Note that the values of the slice will be updated in the next [`RangeSearcher::run()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use mih_rs::Index;
    ///
    /// let codes: Vec<u64> = vec![
    ///     0b1111111111111111111111011111111111111111111111111011101111111111, // #zeros = 3
    ///     0b1111111111111111111111111111111101111111111011111111111111111111, // #zeros = 2
    ///     0b1111111011011101111111111111111101111111111111111111111111111111, // #zeros = 4
    ///     0b1111111111111101111111111111111111111000111111111110001111111110, // #zeros = 8
    ///     0b1101111111111111111111111111111111111111111111111111111111111111, // #zeros = 1
    ///     0b1111111111111111101111111011111111111111111101001110111111111111, // #zeros = 6
    ///     0b1111111111111111111111111111111111101111111111111111011111111111, // #zeros = 2
    ///     0b1110110101011011011111111111111101111111111111111000011111111111, // #zeros = 11
    /// ];
    ///
    /// let index = Index::new(codes).unwrap();
    /// let mut searcher = index.range_searcher();
    ///
    /// let qcode: u64 = 0b1111111111111111111111111111111111111111111111111111111111111111; // #zeros = 0
    /// let answers = searcher.run(qcode, 2);
    /// assert_eq!(answers, vec![1, 4, 6]);
    /// ```
    pub fn run(&mut self, qcode: T, radius: usize) -> &[u32] {
        self.answers.clear();
        let num_blocks = self.index.num_blocks();

        for b in 0..num_blocks {
            // Based on the general pigeonhole principle
            if b + radius + 1 < num_blocks {
                continue;
            }

            let rad = (b + radius + 1 - num_blocks) / num_blocks;
            let dim = self.index.get_dim(b);
            let qcd = self.index.get_chunk(qcode, b);

            let table = &self.index.tables[b];

            // Search with r errors
            for r in 0..rad + 1 {
                self.siggen.init(qcd, dim, r);
                while self.siggen.has_next() {
                    let sig = self.siggen.next();
                    if let Some(a) = table.access(sig as usize) {
                        for v in a {
                            self.answers.push(*v as u32);
                        }
                    }
                }
            }
        }

        let mut n = 0;
        if !self.answers.is_empty() {
            self.answers.sort_unstable();
            for i in 0..self.answers.len() {
                if i == 0 || self.answers[i - 1] != self.answers[i] {
                    let dist = hamdist(qcode, self.index.codes[self.answers[i] as usize]);
                    if dist <= radius {
                        self.answers[n] = self.answers[i];
                        n += 1;
                    }
                }
            }
        }

        self.answers.resize(n, u32::default());
        &self.answers
    }
}

impl<'a, T> TopkSearcher<'a, T>
where
    T: CodeInt,
{
    /// Searches top-K codes that are closest to a query code.
    ///
    /// # Arguments
    ///
    /// - `qcode`: Binary code of the query.
    /// - `topk`: Threshold to be searched.
    ///
    /// # Returns
    ///
    /// A slice of ids of the `topk` nearest neighbor codes to `qcode`.
    /// The ids are sorted in the Hamming distances to `qcode`.
    /// Note that the values of the slice will be updated in the next [`TopkSearcher::run()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use mih_rs::Index;
    ///
    /// let codes: Vec<u64> = vec![
    ///     0b1111111111111111111111011111111111111111111111111011101111111111, // #zeros = 3
    ///     0b1111111111111111111111111111111101111111111011111111111111111111, // #zeros = 2
    ///     0b1111111011011101111111111111111101111111111111111111111111111111, // #zeros = 4
    ///     0b1111111111111101111111111111111111111000111111111110001111111110, // #zeros = 8
    ///     0b1101111111111111111111111111111111111111111111111111111111111111, // #zeros = 1
    ///     0b1111111111111111101111111011111111111111111101001110111111111111, // #zeros = 6
    ///     0b1111111111111111111111111111111111101111111111111111011111111111, // #zeros = 2
    ///     0b1110110101011011011111111111111101111111111111111000011111111111, // #zeros = 11
    /// ];
    ///
    /// let index = Index::new(codes).unwrap();
    /// let mut searcher = index.topk_searcher();
    ///
    /// let qcode: u64 = 0b1111111111111111111111111111111111111111111111111111111111111111; // #zeros = 0
    /// let answers = searcher.run(qcode, 4);
    /// assert_eq!(answers, vec![4, 1, 6, 0]);
    /// ```
    pub fn run(&mut self, qcode: T, topk: usize) -> &[u32] {
        let num_blocks = self.index.num_blocks();
        let num_dimensions = T::dimensions();

        let mut n = 0;
        let mut r = 0;

        let mut counts = vec![0; num_dimensions + 1];

        self.answers
            .resize((num_dimensions + 1) * topk, u32::default());
        self.checked.clear();

        while n < topk {
            for b in 0..num_blocks {
                let dim = self.index.get_dim(b);
                let qcd = self.index.get_chunk(qcode, b);
                let table = &self.index.tables[b];

                self.siggen.init(qcd, dim, r);
                while self.siggen.has_next() {
                    let sig = self.siggen.next();
                    if let Some(a) = table.access(sig as usize) {
                        for &v in a {
                            let id = v as usize;
                            if self.checked.insert(id) {
                                let dist = hamdist(qcode, self.index.codes[id]);
                                if counts[dist] < topk {
                                    self.answers[dist * topk + counts[dist]] = id as u32;
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
                self.answers[n] = self.answers[r * topk + i];
                i += 1;
                n += 1;
            }
            r += 1;
        }

        self.answers.resize(topk, u32::default());
        &self.answers
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
        let mut searcher = index.range_searcher();

        for rad in 0..6 {
            for qi in (0..10000).step_by(100) {
                let qcode = index.codes()[qi];
                let ans1 = ls::range_search(index.codes(), qcode, rad);
                let ans2 = searcher.run(qcode, rad);
                assert_eq!(ans1, ans2);
            }
        }
    }

    fn do_topk_search<T: CodeInt>(codes: Vec<T>) {
        let index = Index::new(codes).unwrap();
        let mut searcher = index.topk_searcher();

        for topk in &[1, 10, 100] {
            for qi in (0..10000).step_by(100) {
                let qcode = index.codes()[qi];
                let ans1 = naive_topk_search(index.codes(), qcode, *topk);
                let ans2 = searcher.run(qcode, *topk);
                let set1: BTreeSet<u32> = ans1.into_iter().collect();
                let set2: BTreeSet<u32> = ans2.into_iter().cloned().collect();
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

    #[test]
    fn serialize_u8_works() {
        let codes = gen_random_codes::<u8>(10000);
        let index = Index::new(codes).unwrap();

        let mut data = vec![];
        index.serialize_into(&mut data).unwrap();
        let other = Index::<u8>::deserialize_from(&data[..]).unwrap();

        assert_eq!(index, other);
    }

    #[test]
    fn serialize_u16_works() {
        let codes = gen_random_codes::<u16>(10000);
        let index = Index::new(codes).unwrap();

        let mut data = vec![];
        index.serialize_into(&mut data).unwrap();
        let other = Index::<u16>::deserialize_from(&data[..]).unwrap();

        assert_eq!(index, other);
    }

    #[test]
    fn serialize_u32_works() {
        let codes = gen_random_codes::<u32>(10000);
        let index = Index::new(codes).unwrap();

        let mut data = vec![];
        index.serialize_into(&mut data).unwrap();
        let other = Index::<u32>::deserialize_from(&data[..]).unwrap();

        assert_eq!(index, other);
    }

    #[test]
    fn serialize_u64_works() {
        let codes = gen_random_codes::<u64>(10000);
        let index = Index::new(codes).unwrap();

        let mut data = vec![];
        index.serialize_into(&mut data).unwrap();
        let other = Index::<u64>::deserialize_from(&data[..]).unwrap();

        assert_eq!(index, other);
    }
}
