#!/usr/bin/env bash
set -euo pipefail

echo "🎨 Building Tailwind CSS..."
mkdir -p target/site/pkg
set -x
# Minification now works correctly in v4.1.16 stable
npx tailwindcss -i style/input.css -o target/site/pkg/blog.css --minify
