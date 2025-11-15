#!/usr/bin/env bash
set -euo pipefail

echo "🔧 Cloudflare Pages Build Script"
echo "=================================="

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    echo "✅ Rust installed"
else
    echo "✅ Rust already available"
fi

# Add WASM target
echo "🎯 Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli (specific version to match Cargo.toml)
echo "🔧 Installing wasm-bindgen-cli..."
cargo install wasm-bindgen-cli --version 0.2.104 --locked

# Install optimization tools (use system packages for speed)
echo "🔧 Installing build tools..."
# Note: Cloudflare build environment is Debian-based
# Using pre-built binaries where available

# Check if binaryen is available, install wasm-opt
if ! command -v wasm-opt &> /dev/null; then
    echo "Installing binaryen (wasm-opt)..."
    # Download pre-built binary for faster builds
    BINARYEN_VERSION=version_118
    wget -q https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}/binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz
    tar xzf binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz
    export PATH="$PWD/binaryen-${BINARYEN_VERSION}/bin:$PATH"
fi

# Check if brotli is available
if ! command -v brotli &> /dev/null; then
    echo "⚠️  brotli not found, compression will be skipped"
fi

# Run the SSG build
echo "🏗️  Building static site..."
SITE_BASE_PATH="${SITE_BASE_PATH:-/}" make ssg

echo "✅ Build complete!"
