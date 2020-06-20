# mih-rs

[![Documentation](https://docs.rs/mih-rs/badge.svg)](https://docs.rs/mih-rs)
[![Crates.io](https://img.shields.io/crates/v/mih-rs.svg)](https://crates.io/crates/mih-rs)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kampersanda/mih-rs/blob/master/LICENSE)

Rust implementation of multi-index hashing (MIH) for neighbor searches on 64-bit codes in the Hamming space, described in the paper

> Norouzi, Punjani, and Fleet, **Fast exact search in Hamming space with multi-index hashing**, *IEEE TPAMI*, 36(6):1107– 1119, 2014.

## Features

- **Two types of neighbor searches:** mih-rs provides the two search operations:
  - *Range search* finds neighbor codes whose Hamming distances to a given code are within a radius.
  - *Top-K search* finds the top-K codes that are closest to a given code.

- **Fast and memory-efficient implementation:** The data structure is built on sparse hash tables, following the original implementation.

- **Parameter free:** mih-rs automatically sets an optimal parameter of MIH depending on a given database (although you can also set this manually).

## Example

```rust
use mih_rs::Index;

fn main() {
    // Database of codes
    let codes: [u64; 8] = [
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
    let qcode: u64 = 0b1111111111111111111111111111111111111111111111111111111111111111;

    // Construct the index
    let index = Index::new(&codes).unwrap();

    // Find the ids of neighbor codes whose Hamming distances are within 2
    let answers = index.range_search(qcode, 2);
    println!("{:?}", answers); // [1, 4, 6]

    // Find the ids of the top-4 nearest neighbor codes
    let answers = index.topk_search(qcode, 4);
    println!("{:?}", answers); // [4, 1, 6, 0]
}
```

## Benchmark

`timeperf_topk.rs` offers the benchmark of top-K search for MIH and LinearSearch algorithms.

The following table shows the result of average search times in milliseconds per query, in the settings:

- **Database**: N random codes from a uniform distribution.
- **Query set**: 100 random codes from a uniform distribution.
- **Machine**: Mac Pro (Late 2013) of 6 core Intel Xeon E5 @3.5 GHz with 32 GB of RAM.

| Algorithm    | N=10,000 | N=100,000 | N=1,000,000 | N=10,000,000 |
| ------------ | -------: | --------: | ----------: | -----------: |
| MIH (K=1)    |      0.1 |       0.4 |         1.7 |          6.5 |
| MIH (K=10)   |      0.2 |       0.8 |         4.0 |         13.6 |
| MIH (K=100)  |      0.4 |       1.6 |         7.9 |         32.0 |
| LinearSearch |      0.4 |       4.9 |        56.1 |        701.4 |

## Licensing

This library is free software provided under MIT.

