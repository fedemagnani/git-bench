//! Libtest benchmarks for git-bench dogfooding
//! These require nightly Rust and #![feature(test)]

#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

fn fib(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

fn sum_range(n: u64) -> u64 {
    (0..n).sum()
}

fn multiply_range(n: u64) -> u64 {
    (1..=n).product()
}

mod libtest {
    use super::*;
    mod arithmetic {
        use super::*;

        #[bench]
        fn sum_1000(b: &mut Bencher) {
            b.iter(|| sum_range(black_box(1000)))
        }

        #[bench]
        fn sum_10000(b: &mut Bencher) {
            b.iter(|| sum_range(black_box(10000)))
        }

        #[bench]
        fn product_10(b: &mut Bencher) {
            b.iter(|| multiply_range(black_box(10)))
        }

        #[bench]
        fn product_15(b: &mut Bencher) {
            b.iter(|| multiply_range(black_box(15)))
        }
    }

    mod fibonacci {
        use super::*;

        #[bench]
        fn fib_10(b: &mut Bencher) {
            b.iter(|| fib(black_box(10)))
        }

        #[bench]
        fn fib_15(b: &mut Bencher) {
            b.iter(|| fib(black_box(15)))
        }

        #[bench]
        fn fib_20(b: &mut Bencher) {
            b.iter(|| fib(black_box(20)))
        }
    }

    mod sorting {
        use super::*;

        #[bench]
        fn sort_100(b: &mut Bencher) {
            b.iter(|| {
                let mut v: Vec<u64> = (0..100).rev().collect();
                v.sort();
                black_box(v)
            })
        }

        #[bench]
        fn sort_1000(b: &mut Bencher) {
            b.iter(|| {
                let mut v: Vec<u64> = (0..1000).rev().collect();
                v.sort();
                black_box(v)
            })
        }
    }
}
