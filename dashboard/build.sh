#!/bin/bash
# Build script for git-bench-dashboard
#
# This script builds the Dioxus dashboard to WebAssembly and outputs
# the files ready for deployment.
#
# Prerequisites:
#   - rustup target add wasm32-unknown-unknown
#   - cargo install dioxus-cli (optional, for dev server)
#   - cargo install wasm-bindgen-cli

set -e

echo "🔨 Building git-bench-dashboard..."

# Build the WASM binary
cargo build --release --target wasm32-unknown-unknown

# Create output directory
mkdir -p dist

# Run wasm-bindgen to generate JS bindings
wasm-bindgen \
    --target web \
    --out-dir dist \
    --out-name git_bench_dashboard \
    ../target/wasm32-unknown-unknown/release/git_bench_dashboard.wasm

# Copy static files
cp index.html dist/
cp src/styles.css dist/

# Optional: Optimize WASM size with wasm-opt (requires binaryen)
if command -v wasm-opt &> /dev/null; then
    echo "📦 Optimizing WASM with wasm-opt..."
    wasm-opt -Oz -o dist/git_bench_dashboard_bg.wasm dist/git_bench_dashboard_bg.wasm
fi

echo ""
echo "✅ Build complete! Output in ./dist/"
echo ""
echo "To test locally:"
echo "  cd dist && python3 -m http.server 8080"
echo "  Then open http://localhost:8080"
echo ""
echo "Make sure to copy your data.json file to the dist directory!"

