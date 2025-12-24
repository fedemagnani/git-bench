# Example Benchmarks for git-bench

This crate demonstrates git-bench with both **Criterion** (stable) and **libtest** (nightly) benchmarks.

## Benchmarks Included

| Module | Function | Complexity |
|--------|----------|------------|
| `fibonacci` | `recursive` | O(2^n) - slow |
| `fibonacci` | `iterative` | O(n) - fast |
| `sorting` | `bubble` | O(n²) - slow |
| `sorting` | `quick` | O(n log n) - fast |

## Running Benchmarks

### Criterion (stable Rust)

```bash
cargo bench 2>&1 | tee benchmark-output.txt
```

### Libtest (nightly Rust)

```bash
cargo +nightly bench --features nightly 2>&1 | tee benchmark-output.txt
```

## Using with git-bench

```bash
# From this directory
cargo bench 2>&1 | tee benchmark-output.txt

# From git-bench root
cd ../..
cargo run --bin git-bench -- run \
    --output-file examples/example-bench/benchmark-output.txt \
    --name "example"

# View dashboard
cd crates/dashboard && ./build.sh
cp ../../benchmark-data.json dist/data.json
cd dist && python3 -m http.server 8081
```

## Expected Output

**Criterion:**
```
fibonacci/recursive     time:   [1.0234 µs 1.0345 µs 1.0456 µs]
fibonacci/iterative     time:   [3.1234 ns 3.2345 ns 3.3456 ns]
sorting/bubble          time:   [4.5678 µs 4.6789 µs 4.7890 µs]
sorting/quick           time:   [1.2345 µs 1.3456 µs 1.4567 µs]
```

**Libtest:**
```
test benches::bench_fib_recursive ... bench:       1,023 ns/iter (+/- 45)
test benches::bench_fib_iterative ... bench:           3 ns/iter (+/- 0)
test benches::bench_sort_bubble   ... bench:       4,567 ns/iter (+/- 123)
test benches::bench_sort_quick    ... bench:       1,234 ns/iter (+/- 56)
```
