# git-bench

A Rust implementation of [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) focused on cargo compatibility.

## Features

- 📊 **Parse cargo bench output** - Supports both libtest and Criterion benchmark formats
- 💾 **Store benchmark history** - Track performance over time in JSON format
- 📈 **Generate HTML dashboards** - Beautiful Chart.js visualizations
- ⚠️ **Alert on regressions** - Detect performance degradation
- 🔗 **GitHub integration** - Commit comments, PR notifications, and GitHub Pages deployment
- 🚀 **Fast & native** - Written in Rust for speed and reliability

## Installation

```bash
# From crates.io (when published)
cargo install git-bench

# From source
git clone https://github.com/yourusername/git-bench
cd git-bench
cargo install --path .
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

### 3. Generate dashboard

```bash
git-bench dashboard --data-file benchmark-data.json --output-dir ./bench-results
```

## Commands

### `run` - Full benchmark workflow

The main command that combines parsing, storing, comparing, and alerting:

```bash
git-bench run \
  --output-file benchmark-output.txt \
  --name "Rust Benchmarks" \
  --github-token "$GITHUB_TOKEN" \
  --alert-threshold "150%" \
  --fail-on-alert
```

### `store` - Store benchmark results

Parse and store benchmark results without comparison:

```bash
git-bench store \
  --output-file benchmark-output.txt \
  --name "my-suite" \
  --data-file benchmark-data.json
```

### `compare` - Compare benchmarks

Compare current benchmarks against historical data:

```bash
git-bench compare \
  --output-file benchmark-output.txt \
  --data-file benchmark-data.json \
  --alert-threshold "200%" \
  --format markdown
```

### `dashboard` - Generate HTML dashboard

Create a beautiful HTML dashboard with charts:

```bash
git-bench dashboard \
  --data-file benchmark-data.json \
  --output-dir ./dev/bench \
  --title "Performance Dashboard"
```

### `history` - View benchmark history

Display historical benchmark data:

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

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Run benchmarks
        run: cargo bench -- --noplot 2>&1 | tee benchmark-output.txt

      - name: Install git-bench
        run: cargo install git-bench

      - name: Process benchmarks
        run: |
          git-bench run \
            --output-file benchmark-output.txt \
            --name "Rust Benchmarks" \
            --github-token "${{ secrets.GITHUB_TOKEN }}" \
            --auto-push \
            --alert-threshold "150%" \
            --fail-on-alert
```

## Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `--output-file` | Path to benchmark output file | Required |
| `--name` | Benchmark suite name | `cargo` |
| `--data-file` | Path to JSON data file | `benchmark-data.json` |
| `--gh-pages-branch` | GitHub Pages branch | `gh-pages` |
| `--benchmark-data-dir-path` | Dashboard output directory | `dev/bench` |
| `--github-token` | GitHub API token | `$GITHUB_TOKEN` |
| `--alert-threshold` | Alert threshold percentage | `200%` |
| `--fail-threshold` | Fail threshold percentage | Same as alert |
| `--comment-always` | Always create commit comment | `false` |
| `--comment-on-alert` | Comment only on alerts | `false` |
| `--fail-on-alert` | Fail workflow on alert | `false` |
| `--auto-push` | Auto-push to gh-pages | `false` |
| `--max-items-in-chart` | Max data points in chart | Unlimited |
| `--external-data-json-path` | External JSON file path | None |
| `--alert-comment-cc-users` | Users to @mention on alerts | None |

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

## Data Format

Benchmark data is stored in JSON format compatible with the original github-action-benchmark:

```json
{
  "last_update": "2024-01-15T10:30:00Z",
  "entries": {
    "my-suite": [
      {
        "commit": {
          "id": "abc123...",
          "message": "Improve performance",
          "timestamp": "2024-01-15T10:30:00Z"
        },
        "date": "2024-01-15T10:30:00Z",
        "tool": "cargo",
        "benches": [
          {
            "name": "bench_add",
            "value": 123.0,
            "unit": "ns/iter",
            "range": "+/- 5"
          }
        ]
      }
    ]
  }
}
```

## Dashboard

git-bench provides two dashboard options:

### Option 1: Static HTML (default)

The `git-bench dashboard` command generates a self-contained HTML file with embedded Chart.js:

```bash
git-bench dashboard --data-file benchmark-data.json --output-dir ./dev/bench
```

### Option 2: Dioxus (Pure Rust/WASM)

For a fully Rust-based solution, use the Dioxus dashboard in the `dashboard/` directory:

```bash
cd dashboard
cargo install dioxus-cli
dx serve  # Development with hot-reload
# OR
./build.sh  # Production build to dist/
```

Both dashboards include:

- 📊 Interactive line charts for each benchmark suite
- 📋 Tables with latest results
- 🔗 Links to commits
- 🌙 Dark theme optimized for readability
- 📱 Responsive design

## Comparison with github-action-benchmark

| Feature | github-action-benchmark | git-bench |
|---------|------------------------|-----------|
| Language | TypeScript | Rust |
| Cargo support | ✅ | ✅ |
| Other tools | ✅ (Go, Python, etc.) | ❌ (cargo only) |
| Dashboard | ✅ | ✅ |
| GitHub Pages | ✅ | ✅ |
| Alerts | ✅ | ✅ |
| Comments | ✅ | ✅ |
| Native binary | ❌ | ✅ |
| Offline usage | ❌ | ✅ |

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Credits

Inspired by [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) by @rhysd.

