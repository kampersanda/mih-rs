# mih-rs

![](https://github.com/kampersanda/mih-rs/actions/workflows/rust.yml/badge.svg)
[![Documentation](https://docs.rs/mih-rs/badge.svg)](https://docs.rs/mih-rs)
[![Crates.io](https://img.shields.io/crates/v/mih-rs.svg)](https://crates.io/crates/mih-rs)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kampersanda/mih-rs/blob/master/LICENSE)

Rust implementation of multi-index hashing (MIH) for neighbor searches on binary codes in the Hamming space, described in the paper

> Norouzi, Punjani, and Fleet, [Fast exact search in Hamming space with multi-index hashing](https://arxiv.org/abs/1307.2982), *IEEE TPAMI*, 36(6):1107– 1119, 2014.

As the [benchmark result](https://github.com/kampersanda/mih-rs#benchmark) shows, on 10 million 64-bit codes, `mih-rs` can perform top-k searches 19−94 times faster than linear search when k = 1..100.

## Features

- **Two types of neighbor searches:** `mih-rs` provides the two search operations:
  - *Range search* finds neighbor codes whose Hamming distances to a given code are within a radius.
  - *Top-K search* finds the top-K codes that are closest to a given code.

- **Fast and memory-efficient implementation:** The data structure is built on sparse hash tables, following the [original implementation](https://github.com/norouzi/mih).

- **Parameter free:** `mih-rs` automatically sets an optimal parameter of MIH depending on a given database (although you can also set this manually).

- **Serialization:** `mih-rs` supports to serialize/deserialize the index.

## Example

```rust
use mih_rs::Index;

// Database of codes
let codes: Vec<u64> = vec![
    0b1111111111111111111111011111111111111111111111111011101111111111, // #zeros = 3
    0b1111111111111111111111111111111101111111111011111111111111111111, // #zeros = 2
    0b1111111011011101111111111111111101111111111111111111111111111111, // #zeros = 4
    0b1111111111111101111111111111111111111000111111111110001111111110, // #zeros = 8
    0b1101111111111111111111111111111111111111111111111111111111111111, // #zeros = 1
    0b1111111111111111101111111011111111111111111101001110111111111111, // #zeros = 6
    0b1111111111111111111111111111111111101111111111111111011111111111, // #zeros = 2
    0b1110110101011011011111111111111101111111111111111000011111111111, // #zeros = 11
];

// Query code
let qcode: u64 = 0b1111111111111111111111111111111111111111111111111111111111111111; // #zeros = 0

// Construct the index
let index = Index::new(codes).unwrap();

// Find the ids of neighbor codes whose Hamming distances are within 2
let mut searcher = index.range_searcher();
let answers = searcher.run(qcode, 2);
assert_eq!(answers, vec![1, 4, 6]);

// Find the ids of the top-4 nearest neighbor codes
let mut searcher = index.topk_searcher();
let answers = searcher.run(qcode, 4);
assert_eq!(answers, vec![4, 1, 6, 0]);

// Serialization/Deserialization
let mut data = vec![];
index.serialize_into(&mut data).unwrap();
let other = Index::<u64>::deserialize_from(&data[..]).unwrap();
assert_eq!(index, other);
```

## Binary code types

`mih_rs::Index` can be built from a vector of type `mih_rs::CodeInt`
that is a primitive integer trait supporting a popcount operation.
Currently, this library defines `mih_rs::CodeInt` for `u8`, `u16`, `u32`, and `u64`.

## Benchmark

`timeperf_topk.rs` offers the benchmark of top-K search for MIH and LinearSearch algorithms on binary code types `u32` and `u64`.

The following table shows the result of average search times in milliseconds per query, in the settings:

- **Database**: N random codes from a uniform distribution.
- **Query set**: 100 random codes from a uniform distribution.
- **Machine**: MacBook Pro (2019) of Quad-Core Intel Core i5 @2.4 GHz with 16 GB of RAM.
- **Library version**: v0.2.0

### Result for `u32`

| Algorithm    | N=10,000 | N=100,000 | N=1,000,000 | N=10,000,000 |
| ------------ | -------: | --------: | ----------: | -----------: |
| MIH (K=1)    |     0.01 |      0.02 |        0.07 |         0.38 |
| MIH (K=10)   |     0.04 |      0.08 |        0.30 |         1.06 |
| MIH (K=100)  |     0.13 |      0.22 |        1.22 |         4.35 |
| LinearSearch |     0.36 |      4.40 |       50.96 |       626.87 |

### Result for `u64`

| Algorithm    | N=10,000 | N=100,000 | N=1,000,000 | N=10,000,000 |
| ------------ | -------: | --------: | ----------: | -----------: |
| MIH (K=1)    |     0.10 |      0.36 |        1.46 |          6.7 |
| MIH (K=10)   |     0.20 |      0.76 |        3.72 |         14.8 |
| MIH (K=100)  |     0.41 |      1.53 |        7.02 |         33.2 |
| LinearSearch |     0.36 |      4.36 |       52.28 |        629.1 |

## Licensing

This library is free software provided under MIT.

