#!/bin/bash
set -e

echo "============================================="
echo "  Building next.rs WASM Demo"
echo "============================================="
echo

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "ERROR: wasm-pack is not installed"
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

# Build WASM
echo "Step 1: Building WASM package..."
wasm-pack build --target web --out-dir pkg

echo
echo "Step 2: Building server..."
cargo build --bin server --release

echo
echo "============================================="
echo "  Build complete!"
echo "============================================="
echo
echo "To start the demo:"
echo "  1. cd examples/wasm-demo"
echo "  2. cargo run --bin server"
echo "  3. Open http://localhost:3000 in your browser"
echo
