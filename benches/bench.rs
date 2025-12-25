//! Real benchmarks for git-bench dogfooding

use criterion::{black_box, criterion_group, criterion_main, Criterion};

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

fn bench_arithmetic(c: &mut Criterion) {
    c.bench_function("arithmetic::sum_1000", |b| {
        b.iter(|| sum_range(black_box(1000)))
    });

    c.bench_function("arithmetic::sum_10000", |b| {
        b.iter(|| sum_range(black_box(10000)))
    });

    c.bench_function("arithmetic::product_10", |b| {
        b.iter(|| multiply_range(black_box(10)))
    });

    c.bench_function("arithmetic::product_15", |b| {
        b.iter(|| multiply_range(black_box(15)))
    });
}

fn bench_fibonacci(c: &mut Criterion) {
    c.bench_function("fibonacci::fib_10", |b| b.iter(|| fib(black_box(10))));

    c.bench_function("fibonacci::fib_15", |b| b.iter(|| fib(black_box(15))));

    c.bench_function("fibonacci::fib_20", |b| b.iter(|| fib(black_box(20))));
}

fn bench_sorting(c: &mut Criterion) {
    c.bench_function("sorting::sort_100", |b| {
        b.iter_batched(
            || (0..100).rev().collect::<Vec<u64>>(),
            |mut v| {
                v.sort();
                v
            },
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("sorting::sort_1000", |b| {
        b.iter_batched(
            || (0..1000).rev().collect::<Vec<u64>>(),
            |mut v| {
                v.sort();
                v
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(arithmetic, bench_arithmetic);
criterion_group!(fibonacci, bench_fibonacci);
criterion_group!(sorting, bench_sorting);

criterion_main!(arithmetic, fibonacci, sorting);
