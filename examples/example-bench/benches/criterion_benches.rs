//! Criterion benchmarks (works on stable Rust)
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use example_bench::{fibonacci, sorting};

fn fibonacci_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");

    group.bench_function("recursive", |b| {
        b.iter(|| fibonacci::recursive(black_box(15)))
    });

    group.bench_function("iterative", |b| {
        b.iter(|| fibonacci::iterative(black_box(15)))
    });

    group.finish();
}

fn sorting_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorting");

    group.bench_function("bubble", |b| {
        b.iter(|| {
            let mut arr: Vec<i32> = (0..100).rev().collect();
            sorting::bubble(&mut arr);
            arr
        })
    });

    group.bench_function("quick", |b| {
        b.iter(|| {
            let mut arr: Vec<i32> = (0..100).rev().collect();
            sorting::quick(&mut arr);
            arr
        })
    });

    group.finish();
}

criterion_group!(benches, fibonacci_benches, sorting_benches);
criterion_main!(benches);
