# git-bench Dashboard

A pure Rust frontend for the benchmark dashboard, built with [Dioxus](https://dioxuslabs.com) and compiled to WebAssembly.

## Features

- 📊 Interactive line charts for benchmark visualization
- 🦀 100% Rust - no JavaScript required
- 🌐 WebAssembly for fast performance
- 🌙 Beautiful dark theme
- 📱 Responsive design

## Prerequisites

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen CLI
cargo install wasm-bindgen-cli

# Optional: Install dioxus-cli for development
cargo install dioxus-cli
```

## Development

Use the Dioxus CLI for hot-reloading development:

```bash
dx serve
```

This will start a development server at `http://localhost:8080`.

## Production Build

### Using the build script

```bash
chmod +x build.sh
./build.sh
```

### Manual build

```bash
# Build the WASM binary
cargo build --release --target wasm32-unknown-unknown

# Generate JS bindings
wasm-bindgen \
    --target web \
    --out-dir dist \
    --out-name git_bench_dashboard \
    target/wasm32-unknown-unknown/release/git_bench_dashboard.wasm

# Copy static files
cp index.html dist/
cp src/styles.css dist/
```

## Deployment

The `dist/` directory contains everything needed:

- `index.html` - Entry point
- `git_bench_dashboard.js` - JS loader
- `git_bench_dashboard_bg.wasm` - WebAssembly module

Copy these files to your web server along with your `data.json` benchmark data.

### GitHub Pages

To deploy to GitHub Pages:

1. Build the dashboard
2. Copy `dist/*` to your `gh-pages` branch
3. Ensure `data.json` is in the same directory

## Data Format

The dashboard expects a `data.json` file with the following format:

```json
{
  "last_update": "2024-01-15T10:30:00Z",
  "entries": {
    "suite-name": [
      {
        "commit": {
          "id": "abc123...",
          "message": "Commit message",
          "timestamp": "2024-01-15T10:30:00Z"
        },
        "date": "2024-01-15T10:30:00Z",
        "tool": "cargo",
        "benches": [
          {
            "name": "benchmark_name",
            "value": 123.45,
            "unit": "ns/iter",
            "range": "+/- 5"
          }
        ]
      }
    ]
  }
}
```

This format is compatible with the `git-bench` CLI tool.

## Customization

### Styling

Edit `src/styles.css` to customize the appearance. The dashboard uses CSS custom properties (variables) for theming:

```css
:root {
    --bg-primary: #0d1117;
    --text-primary: #e6edf3;
    --accent-blue: #58a6ff;
    /* ... */
}
```

### Components

The dashboard is built with modular Dioxus components in `src/main.rs`:

- `App` - Main application
- `Header` / `Footer` - Layout components
- `Dashboard` - Main dashboard view
- `BenchmarkSuite` - Individual suite card
- `BenchmarkChart` - SVG line chart
- `LatestResults` - Results table

