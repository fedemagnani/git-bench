# git-bench

A Rust implementation of [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) focused on cargo compatibility.

**100% Rust** - No JavaScript dependencies. Dashboard is built with Dioxus/WASM.

## Features

- 📊 **Parse cargo bench output** - Supports both libtest and Criterion benchmark formats
- 💾 **Store benchmark history** - Track performance over time in JSON format
- 📈 **Pure Rust dashboard** - Dioxus/WASM dashboard (no JavaScript!)
- ⚠️ **Alert on regressions** - Detect performance degradation
- 🔗 **GitHub integration** - Commit comments, PR notifications, and GitHub Pages deployment
- 🚀 **Fast & native** - Written in Rust for speed and reliability

## Project Structure

```
git-bench/
├── Cargo.toml                  # Workspace configuration
├── crates/
│   ├── core/                   # Shared types & parsing (WASM-compatible)
│   │   └── src/
│   │       ├── lib.rs          # Library exports
│   │       ├── data.rs         # BenchmarkData, BenchmarkRun, etc.
│   │       ├── parser.rs       # Parse cargo bench output
│   │       ├── compare.rs      # Benchmark comparison logic
│   │       └── error.rs        # Error types
│   ├── cli/                    # CLI binary (native)
│   │   └── src/
│   │       ├── main.rs         # CLI entry point
│   │       ├── alert.rs        # Alerting logic
│   │       ├── git.rs          # Git operations
│   │       └── github.rs       # GitHub API
│   └── dashboard/              # Dioxus WASM dashboard
│       ├── src/
│       │   ├── main.rs         # Dioxus app
│       │   └── styles.rs       # All styling in Rust
│       └── build.sh            # WASM build script
└── examples/                   # Sample benchmark outputs
```

### Crate Architecture

- **`git-bench-core`**: WASM-compatible shared code (types, parsing, comparison)
- **`git-bench`** (cli): Native CLI using git2, reqwest for GitHub integration  
- **`git-bench-dashboard`**: Dioxus WASM frontend using core types

The dashboard is a separate crate because it targets `wasm32-unknown-unknown`, while the CLI uses native dependencies (`git2`, `reqwest`).

## Installation

```bash
# From source
git clone https://github.com/yourusername/git-bench
cd git-bench
cargo install --path crates/cli
```

## Quick Start

### 1. Run your benchmarks and save output

```bash
cargo bench -- --noplot 2>&1 | tee benchmark-output.txt
```

### 2. Store and compare results

```bash
git-bench run --output-file benchmark-output.txt --name "my-benchmarks"
```

### 3. Build and view dashboard

```bash
cd crates/dashboard
./build.sh
cp /path/to/benchmark-data.json dist/data.json
cd dist && python3 -m http.server 8080
# Open http://localhost:8080
```

## Commands

### `run` - Full benchmark workflow

```bash
git-bench run \
  --output-file benchmark-output.txt \
  --name "Rust Benchmarks" \
  --github-token "$GITHUB_TOKEN" \
  --alert-threshold "150%" \
  --fail-on-alert
```

### `store` - Store benchmark results

```bash
git-bench store \
  --output-file benchmark-output.txt \
  --name "my-suite" \
  --data-file benchmark-data.json
```

### `compare` - Compare benchmarks

```bash
git-bench compare \
  --output-file benchmark-output.txt \
  --data-file benchmark-data.json \
  --alert-threshold "200%" \
  --format markdown
```

### `history` - View benchmark history

```bash
git-bench history --data-file benchmark-data.json --limit 10
```

## GitHub Actions Integration

Add this to your `.github/workflows/benchmark.yml`:

```yaml
name: Continuous Benchmarking

on:
  push:
    branches: [main]

permissions:
  contents: write

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Needed for gh-pages branch access

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install WASM target and tools
        run: |
          rustup target add wasm32-unknown-unknown
          cargo install wasm-bindgen-cli

      - name: Run benchmarks
        run: cargo bench 2>&1 | tee benchmark-output.txt

      - name: Install git-bench
        run: cargo install --path crates/cli

      - name: Build dashboard
        run: cd crates/dashboard && ./build.sh

      - name: Configure Git
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"

      - name: Process and deploy benchmarks
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          git-bench run \
            --output-file benchmark-output.txt \
            --name "Rust Benchmarks" \
            --github-token "$GITHUB_TOKEN" \
            --auto-push \
            --gh-pages-branch gh-pages \
            --benchmark-data-dir-path dev/bench \
            --dashboard-dir crates/dashboard/dist \
            --alert-threshold "150%" \
            --fail-on-alert
```

### GitHub Pages Setup

After the first successful run:

1. Go to **Settings → Pages** in your repository
2. Set **Source** to "Deploy from a branch"
3. Select **gh-pages** branch, **dev/bench** folder
4. Your dashboard will be at: `https://username.github.io/repo/dev/bench/`

The `--auto-push` flag will:
- Create the `gh-pages` branch if it doesn't exist
- Deploy dashboard files (if `--dashboard-dir` is provided)
- Update `data.json` with the latest benchmark results
- Push to GitHub Pages automatically

## Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `--output-file` | Path to benchmark output file | Required |
| `--name` | Benchmark suite name | `cargo` |
| `--data-file` | Path to JSON data file | `benchmark-data.json` |
| `--gh-pages-branch` | GitHub Pages branch | `gh-pages` |
| `--benchmark-data-dir-path` | Dashboard output directory | `dev/bench` |
| `--dashboard-dir` | Path to dashboard dist folder | None |
| `--github-token` | GitHub API token | `$GITHUB_TOKEN` |
| `--alert-threshold` | Alert threshold percentage | `200%` |
| `--fail-threshold` | Fail threshold percentage | Same as alert |
| `--comment-always` | Always create commit comment | `false` |
| `--comment-on-alert` | Comment only on alerts | `false` |
| `--fail-on-alert` | Fail workflow on alert | `false` |
| `--auto-push` | Auto-push to gh-pages | `false` |
| `--max-items-in-chart` | Max data points in chart | Unlimited |

## Supported Benchmark Formats

### libtest (built-in Rust benchmarks)

```
test bench_add ... bench:         123 ns/iter (+/- 5)
test bench_multiply ... bench:   1,234 ns/iter (+/- 56)
```

### Criterion

```
bench_fibonacci         time:   [1.2345 µs 1.2456 µs 1.2567 µs]
bench_sorting           time:   [10.123 ns 10.456 ns 10.789 ns]
```

## Dashboard (100% Rust)

The dashboard is built entirely in Rust using Dioxus, compiled to WebAssembly:

```bash
cd crates/dashboard
./build.sh
```

This generates a `dist/` folder containing:
- `index.html` - Generated by Rust
- `*.wasm` - Compiled from Rust
- `*.js` - Auto-generated bindings (not hand-written)

### Dashboard Features

- 📊 Interactive SVG line charts with hierarchical grouping
- 🔍 Hover tooltips with commit details
- ⛶ Expandable charts for detailed view
- 🌙 Light/dark mode toggle
- 📱 Responsive design
- 🦀 **100% Rust** - No manual JavaScript/CSS

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

Inspired by [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) by @rhysd.
