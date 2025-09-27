#!/bin/bash

# Build script for WASM with optimizations

set -e

echo "Building oxigraph-wasm for WASM target..."

# Ensure we have the WASM target
rustup target add wasm32-unknown-unknown

# Build the WASM module
cargo build --release --target wasm32-unknown-unknown

# Optional: Use wasm-opt for further optimization if available
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM with wasm-opt..."
    wasm-opt -Os -o target/wasm32-unknown-unknown/release/oxigraph_wasm_optimized.wasm \
        target/wasm32-unknown-unknown/release/oxigraph_wasm.wasm
    echo "Optimized WASM created: target/wasm32-unknown-unknown/release/oxigraph_wasm_optimized.wasm"
else
    echo "wasm-opt not found, skipping optimization"
fi

echo "Build complete!"
echo "WASM file: target/wasm32-unknown-unknown/release/oxigraph_wasm.wasm"