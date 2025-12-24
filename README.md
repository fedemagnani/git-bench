# git-bench

Rust alternative to [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark). No JavaScript—dashboard runs on Dioxus/WASM.

## Crates

| Crate | Target | Purpose |
|-------|--------|---------|
| `git-bench-core` | wasm32 + native | Types, parsing, comparison logic |
| `git-bench` (cli) | native | CLI with git2/reqwest for GitHub integration |
| `git-bench-dashboard` | wasm32 | Dioxus frontend |

## Install

```bash
# From GitHub (specify package name due to workspace)
cargo install --git https://github.com/fedemagnani/git-bench --tag v0.1.0 git-bench

# From source
cargo install --path crates/cli
```

## Usage

```bash
# Run benchmarks
cargo bench 2>&1 | tee benchmark-output.txt

# Store results and compare
git-bench run --output-file benchmark-output.txt --name "my-benchmarks"
```

## Commands

```bash
# Full workflow (store + compare + optional GitHub comment)
git-bench run --output-file out.txt --alert-threshold "150%" --fail-on-alert

# Just store
git-bench store --output-file out.txt --name "suite"

# Compare with previous
git-bench compare --output-file out.txt --format markdown

# View history
git-bench history --limit 10
```

## GitHub Actions

```yaml
name: Benchmark

on:
  push:
    branches: [main]

permissions:
  contents: write

jobs:
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: dtolnay/rust-toolchain@stable

      - name: Setup
        run: |
          rustup target add wasm32-unknown-unknown
          cargo install dioxus-cli wasm-bindgen-cli
          cargo install --git https://github.com/fedemagnani/git-bench --tag v0.1.0 git-bench
          git clone --depth 1 --branch v0.1.0 https://github.com/fedemagnani/git-bench /tmp/git-bench
          cd /tmp/git-bench/crates/dashboard && dx build --release

      - name: Bench
        run: cargo bench 2>&1 | tee benchmark-output.txt

      - name: Deploy
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git-bench run \
            --output-file benchmark-output.txt \
            --github-token "${{ secrets.GITHUB_TOKEN }}" \
            --auto-push \
            --dashboard-dir /tmp/git-bench/crates/dashboard/dist \
            --alert-threshold "150%"
```

After first run: Settings → Pages → gh-pages branch → `dev/bench` folder.

## Dashboard

```bash
cd crates/dashboard
dx serve              # dev
dx build --release    # prod
```

Features:
- Hierarchical grouping (`grandparent::parent::test`)
- FROM/TO commit comparison with metrics table
- GitHub links for commits and authors
- Dark/light theme
- Search by commit, message, author

## Supported Formats

**libtest:**
```
test bench_add ... bench:         123 ns/iter (+/- 5)
```

**Criterion:**
```
bench_fibonacci         time:   [1.2345 µs 1.2456 µs 1.2567 µs]
```

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--output-file` | required | Benchmark output file |
| `--name` | `cargo` | Suite name |
| `--data-file` | `benchmark-data.json` | JSON storage |
| `--alert-threshold` | `200%` | Regression alert threshold |
| `--fail-on-alert` | `false` | Exit 1 on regression |
| `--auto-push` | `false` | Deploy to gh-pages |
| `--dashboard-dir` | none | Dashboard dist path |
| `--comment-on-alert` | `false` | GitHub comment on regression |

## License

MIT
