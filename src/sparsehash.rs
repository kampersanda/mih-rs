//! Implements a sparse hash table of the internal data structure of MIH.
//! Most users do not need to use this module directly.

use crate::utils;
use std::io::{Error, ErrorKind};

const GROUP_SIZE: usize = 64;
const COUNT_FLAG: u32 = u32::max_value();

#[derive(Default, Clone)]
pub struct Table {
    size: usize,
    bits: usize,
    groups: Vec<Group>,
}

impl Table {
    pub fn new(bits: usize) -> Result<Table, Error> {
        if bits < 1 || 32 < bits {
            let e = Error::new(ErrorKind::InvalidInput, "bits needs to be in [6,37]");
            return Err(e);
        }

        let size = 1 << bits;
        let groups = vec![Group::default(); if size >= GROUP_SIZE { size / GROUP_SIZE } else { 1 }];

        Ok(Table {
            size: size,
            bits: bits,
            groups: groups,
        })
    }

    pub fn access(&self, idx: usize) -> Option<&[u32]> {
        let gpos = idx / GROUP_SIZE;
        let gmod = idx % GROUP_SIZE;
        self.groups[gpos].access(gmod)
    }

    pub fn insert(&mut self, idx: usize, dat: u32) {
        let gpos = idx / GROUP_SIZE;
        let gmod = idx % GROUP_SIZE;
        self.groups[gpos].insert(gmod, dat);
    }

    pub fn count_insert(&mut self, idx: usize) {
        let gpos = idx / GROUP_SIZE;
        let gmod = idx % GROUP_SIZE;
        self.groups[gpos].count_insert(gmod);
    }

    pub fn data_insert(&mut self, idx: usize, dat: u32) {
        let gpos = idx / GROUP_SIZE;
        let gmod = idx % GROUP_SIZE;
        self.groups[gpos].data_insert(gmod, dat);
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
    pub fn get_bits(&self) -> usize {
        self.bits
    }

    pub fn get_array_size(&self, idx: usize) -> usize {
        let gpos = idx / GROUP_SIZE;
        let gmod = idx % GROUP_SIZE;
        self.groups[gpos].get_size(gmod)
    }
}

#[derive(Default, Clone)]
struct Group {
    bitmap: u64,
    array: Vec<u32>,
}

impl Group {
    fn access(&self, idx: usize) -> Option<&[u32]> {
        debug_assert!(idx < GROUP_SIZE);

        if !utils::get(self.bitmap, idx) {
            return None;
        }

        let howmany = utils::popcnt_mask(self.bitmap, idx);
        let totones = utils::popcnt(self.bitmap);

        let bpos = totones + 1 + self.array[howmany] as usize;
        let epos = bpos + (self.array[howmany + 1] - self.array[howmany]) as usize;

        Some(&self.array[bpos..epos])
    }

    fn insert(&mut self, idx: usize, dat: u32) {
        debug_assert!(idx < GROUP_SIZE);

        if self.bitmap == 0 {
            self.bitmap = utils::set(self.bitmap, idx);
            self.array = vec![0, 1, dat]; // beg, end, dat
            return;
        }

        let howmany = utils::popcnt_mask(self.bitmap, idx);

        if !utils::get(self.bitmap, idx) {
            self.array.insert(howmany, self.array[howmany]);
            self.bitmap = utils::set(self.bitmap, idx);
        }

        let totones = utils::popcnt(self.bitmap);
        self.array.insert(totones + 1 + self.array[howmany + 1] as usize, dat);

        for i in howmany + 1..totones + 1 {
            self.array[i] += 1;
        }
    }

    fn count_insert(&mut self, idx: usize) {
        debug_assert!(idx < GROUP_SIZE);

        if self.bitmap == 0 {
            self.array.push(COUNT_FLAG);
        }

        let howmany = utils::popcnt_mask(self.bitmap, idx);
        if !utils::get(self.bitmap, idx) {
            self.array.insert(howmany + 1, 1);
            self.bitmap = utils::set(self.bitmap, idx);
        } else {
            self.array[howmany + 1] += 1;
        }
    }

    fn data_insert(&mut self, idx: usize, dat: u32) {
        debug_assert!(idx < GROUP_SIZE);
        debug_assert!(utils::get(self.bitmap, idx));

        if self.array[0] == COUNT_FLAG {
            self.allocate_mem_based_on_counts();
        }

        let totones = utils::popcnt(self.bitmap);
        let howmany = utils::popcnt_mask(self.bitmap, idx);

        let offset = self.array[howmany + 1] as usize;
        self.array[totones + 1 + offset] = dat;
        self.array[howmany + 1] += 1;
    }

    fn allocate_mem_based_on_counts(&mut self) {
        debug_assert_ne!(self.bitmap, 0);
        debug_assert_eq!(self.array[0], COUNT_FLAG);

        let totones = utils::popcnt(self.bitmap);
        debug_assert_eq!(totones + 1, self.array.len());

        self.array[0] = 0;
        for i in 0..totones {
            self.array[i + 1] += self.array[i];
        }

        let new_size = self.array.len() + self.array[totones] as usize;
        self.array.resize(new_size, Default::default());

        for i in (0..totones).rev() {
            self.array[i + 1] = self.array[i];
        }
    }

    fn get_size(&self, idx: usize) -> usize {
        debug_assert!(idx < GROUP_SIZE);

        if !utils::get(self.bitmap, idx) {
            0
        } else {
            let howmany = utils::popcnt_mask(self.bitmap, idx);
            (self.array[howmany + 1] - self.array[howmany]) as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sparsehash::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn table_works() {
        let mut obj1 = vec![Vec::<u32>::default(); 1 << 10];
        let mut obj2 = Table::new(10).unwrap();
        assert_eq!(obj2.get_bits(), 10);
        assert_eq!(obj2.get_size(), obj1.len());

        let mut rng = thread_rng();
        for i in 0..1000 {
            let idx: usize = rng.gen_range(0, obj2.get_size());
            obj1[idx].push(i);
            obj2.insert(idx, i);
        }

        for idx in 0..obj1.len() {
            let org = &obj1[idx];
            match obj2.access(idx) {
                None => assert_eq!(org.is_empty(), true),
                Some(a) => assert_eq!(&org[..], a),
            }
        }
    }

    #[test]
    fn table_works_in_balk_manner() {
        let mut obj1 = vec![Vec::<u32>::default(); 1 << 10];
        let mut obj2 = Table::new(10).unwrap();
        assert_eq!(obj2.get_bits(), 10);
        assert_eq!(obj2.get_size(), obj1.len());

        let mut rng = thread_rng();
        let mut idxs = vec![0 as usize; 1000];

        for i in 0..1000 {
            idxs[i] = rng.gen_range(0, obj2.get_size());
        }

        for i in 0..1000 {
            let idx = idxs[i];
            obj2.count_insert(idx);
        }

        for i in 0..1000 {
            let idx = idxs[i];
            obj1[idx].push(i as u32);
            obj2.data_insert(idx, i as u32);
        }

        for idx in 0..obj1.len() {
            let org = &obj1[idx];
            match obj2.access(idx) {
                None => assert_eq!(org.is_empty(), true),
                Some(a) => assert_eq!(&org[..], a),
            }
        }
    }

    #[test]
    fn group_works() {
        let mut rng = thread_rng();

        let mut obj1 = vec![Vec::<u32>::default(); GROUP_SIZE];
        let mut obj2 = Group::default();

        for i in 0..100 {
            let idx: usize = rng.gen_range(0, GROUP_SIZE);
            obj1[idx].push(i);
            obj2.insert(idx, i);
        }

        for idx in 0..GROUP_SIZE {
            let org = &obj1[idx];
            match obj2.access(idx) {
                None => assert_eq!(org.is_empty(), true),
                Some(a) => assert_eq!(&org[..], a),
            }
        }
    }

    #[test]
    fn group_works_in_balk_manner() {
        let mut rng = thread_rng();

        let mut obj1 = vec![Vec::<u32>::default(); GROUP_SIZE];
        let mut obj2 = Group::default();

        let mut idxs = vec![0 as usize; 100];
        for i in 0..100 {
            idxs[i] = rng.gen_range(0, GROUP_SIZE);
        }

        for i in 0..100 {
            let idx = idxs[i];
            obj2.count_insert(idx);
        }
        for i in 0..100 {
            let idx = idxs[i];
            obj1[idx].push(i as u32);
            obj2.data_insert(idx, i as u32);
        }

        for idx in 0..GROUP_SIZE {
            let org = &obj1[idx];
            match obj2.access(idx) {
                None => assert_eq!(org.is_empty(), true),
                Some(a) => assert_eq!(&org[..], a),
            }
        }
    }
}
