#!/usr/bin/env bash
set -euo pipefail

# Build mode: dev (faster, ~5-10s) or release (optimized, ~38s)
# Usage: MODE=release ./scripts/build/watch_wasm.sh
MODE="${MODE:-dev}"

echo "🔁 Watching for WASM changes (${MODE} mode)..."
echo ""
echo "This script rebuilds WASM when src/ files change."
echo "Run this alongside Tailwind CSS watch and Leptos watch."
echo ""
echo "💡 Tip: Use MODE=release for production-like builds"
echo "   Default is dev mode for faster iteration (~5-10s vs ~38s)"
echo ""

# Determine build flags and paths based on mode
if [ "$MODE" = "release" ]; then
  BUILD_FLAGS="--release"
  WASM_PATH="target/wasm32-unknown-unknown/release/longitudinal_dev.wasm"
else
  BUILD_FLAGS=""
  WASM_PATH="target/wasm32-unknown-unknown/debug/longitudinal_dev.wasm"
fi

# Check if cargo-watch is installed
if command -v cargo-watch &> /dev/null; then
  echo "✅ Using cargo-watch for efficient rebuilds"
  echo ""
  cargo watch \
    -w src \
    -x "build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate ${BUILD_FLAGS}" \
    -s "wasm-bindgen ${WASM_PATH} --target web --no-typescript --out-dir target/site/pkg --out-name blog"
else
  echo "⚠️  cargo-watch not found, using polling fallback"
  echo "   Install for better performance: cargo install cargo-watch"
  echo ""

  # Fallback: simple polling loop
  while true; do
    echo "Building WASM..."
    cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate ${BUILD_FLAGS}

    echo "Generating bindings..."
    wasm-bindgen "${WASM_PATH}" \
      --target web \
      --no-typescript \
      --out-dir target/site/pkg \
      --out-name blog

    echo "✅ WASM rebuild complete. Waiting for changes..."

    # Wait for file changes (3 second polling interval)
    # This is a simple fallback - cargo-watch is much better
    sleep 3
  done
fi
