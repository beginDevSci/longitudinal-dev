#!/usr/bin/env bash
# Wrapper for check-prereqs Rust binary
# This maintains backward compatibility while using the new Rust implementation

set -euo pipefail

# Use pre-built binary if available (faster), otherwise build it
if [ -x "./target/release/check-prereqs" ]; then
    exec ./target/release/check-prereqs "$@"
elif [ -x "./target/debug/check-prereqs" ]; then
    exec ./target/debug/check-prereqs "$@"
else
    echo "Building check-prereqs binary..."
    cargo build --features ssr --bin check-prereqs --release
    exec ./target/release/check-prereqs "$@"
fi
