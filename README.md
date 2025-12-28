# git-bench

Track and visualize Rust benchmark performance across commits. Deploys an interactive WASM dashboard to GitHub Pages.

## Quick Demo

```bash
# Build and run the demo dashboard locally (dogfooding git-bench on itself)
make demo
```

This runs the project's own benchmarks and serves the dashboard at `http://localhost:8080` (make sure it is not already in use).

## GitHub Actions

Copy [`.github/workflows/benchmark-example.yml`](.github/workflows/benchmark-example.yml) to your project's `.github/workflows/` directory.

Dashboard will be available at `https://<username>.github.io/<repo>/dev/bench/`.

## Dashboard Features

- Criterion and libtest support via hierarchical grouping (`grandparent::parent::test`)
- FROM/TO commit comparison with metrics table
- GitHub links for commits and authors

## CLI Options

| Flag | Default | Description |
|------|---------|-------------|
| `--output-file` | required | Benchmark output file |
| `--name` | `cargo` | Suite name |
| `--alert-threshold` | `200%` | Regression alert threshold |
| `--fail-on-alert` | `false` | Exit 1 on regression |
| `--auto-push` | `false` | Deploy to gh-pages |
| `--dashboard-dir` | none | Dashboard dist path |

## Credits

Inspired by [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark).

## License

MIT
