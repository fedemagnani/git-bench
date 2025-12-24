//! Example library with functions to benchmark

#![cfg_attr(feature = "nightly", feature(test))]

#[cfg(all(feature = "nightly", test))]
extern crate test;

/// Fibonacci implementations
pub mod fibonacci {
    /// Recursive fibonacci - O(2^n)
    pub fn recursive(n: u32) -> u64 {
        match n {
            0 => 0,
            1 => 1,
            _ => recursive(n - 1) + recursive(n - 2),
        }
    }

    /// Iterative fibonacci - O(n)
    pub fn iterative(n: u32) -> u64 {
        if n == 0 {
            return 0;
        }
        let (mut a, mut b) = (0u64, 1u64);
        for _ in 1..n {
            (a, b) = (b, a.wrapping_add(b));
        }
        b
    }
}

/// Sorting algorithms
pub mod sorting {
    /// Bubble sort - O(n²)
    pub fn bubble<T: Ord>(arr: &mut [T]) {
        let n = arr.len();
        for i in 0..n {
            for j in 0..n - 1 - i {
                if arr[j] > arr[j + 1] {
                    arr.swap(j, j + 1);
                }
            }
        }
    }

    /// Quick sort - O(n log n)
    pub fn quick<T: Ord>(arr: &mut [T]) {
        if arr.len() <= 1 {
            return;
        }
        let pivot = partition(arr);
        quick(&mut arr[..pivot]);
        quick(&mut arr[pivot + 1..]);
    }

    fn partition<T: Ord>(arr: &mut [T]) -> usize {
        let len = arr.len();
        arr.swap(len / 2, len - 1);
        let mut i = 0;
        for j in 0..len - 1 {
            if arr[j] <= arr[len - 1] {
                arr.swap(i, j);
                i += 1;
            }
        }
        arr.swap(i, len - 1);
        i
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci::recursive(10), 55);
        assert_eq!(fibonacci::iterative(10), 55);
    }

    #[test]
    fn test_sorting() {
        let mut arr = vec![5, 2, 8, 1, 9];
        sorting::quick(&mut arr);
        assert_eq!(arr, vec![1, 2, 5, 8, 9]);
    }
}

// ============================================
// LIBTEST BENCHMARKS (requires nightly Rust)
// Run with: cargo +nightly bench --features nightly
// ============================================
#[cfg(all(feature = "nightly", test))]
mod benches {
    use super::*;
    use test::{black_box, Bencher};

    #[bench]
    fn bench_fib_recursive(b: &mut Bencher) {
        b.iter(|| fibonacci::recursive(black_box(15)));
    }

    #[bench]
    fn bench_fib_iterative(b: &mut Bencher) {
        b.iter(|| fibonacci::iterative(black_box(15)));
    }

    #[bench]
    fn bench_sort_bubble(b: &mut Bencher) {
        b.iter(|| {
            let mut arr: Vec<i32> = (0..100).rev().collect();
            sorting::bubble(&mut arr);
            arr
        });
    }

    #[bench]
    fn bench_sort_quick(b: &mut Bencher) {
        b.iter(|| {
            let mut arr: Vec<i32> = (0..100).rev().collect();
            sorting::quick(&mut arr);
            arr
        });
    }
}
