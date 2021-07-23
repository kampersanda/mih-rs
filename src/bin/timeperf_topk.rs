//! Provides the benchmark of top-K search for MIH and LinearSearch algorithms.
use mih_rs::*;
use std::time;

const SIZES: [usize; 4] = [10_000, 100_000, 1_000_000, 10_000_000];
const TOPKS: [usize; 3] = [1, 10, 100];

fn main() {
    #[cfg(debug_assertions)]
    println!("Debugging enabled");

    let codes = basic::gen_random_codes::<u64>(SIZES[SIZES.len() - 1]);
    let qcodes = basic::gen_random_codes::<u64>(100);

    for size in &SIZES {
        println!("--- N={} ---", *size);

        let ins = time::Instant::now();
        let index = mih::Index::new(&codes[0..*size]).unwrap();
        let elapsed_sec = ins.elapsed().as_secs_f64();
        println!("Constr time: {} sec", elapsed_sec);

        for topk in &TOPKS {
            let ins = time::Instant::now();
            for qcode in &qcodes {
                let answers = index.topk_search(*qcode, *topk);
                assert_eq!(answers.len(), *topk);
            }
            let elapsed_ms = ins.elapsed().as_millis() as f64;
            println!(
                "MIH (K={}):\t{} ms/query",
                *topk,
                elapsed_ms / qcodes.len() as f64
            );
        }

        let ins = time::Instant::now();
        for qcode in &qcodes {
            let mut answers = ls::exhaustive_search(&codes[0..*size], *qcode);
            answers.sort_by_key(|x| x.1);
            assert_eq!(answers.len(), *size);
        }
        let elapsed_ms = ins.elapsed().as_millis() as f64;
        println!(
            "LinearSearch:\t{} ms/query",
            elapsed_ms / qcodes.len() as f64
        );
    }
}
