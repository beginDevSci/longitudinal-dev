#!/usr/bin/env bash
set -euo pipefail

echo "⚠️  DEPRECATION WARNING: This script is deprecated and will be removed after 2025-12-31"
echo "    Please use 'cargo xtask build-ssg' or 'make ssg' instead."
echo "    See docs for migration: https://github.com/yourusername/longitudinal-dev"
echo ""
echo "🏗️  Building complete SSG site..."

# Output directory
OUTDIR="${1:-dist}"
echo "Output directory: $OUTDIR"

# Optional: Set base path for GitHub Pages (e.g., /repo-name/)
# export SITE_BASE_PATH="${SITE_BASE_PATH:-}"
if [ -n "${SITE_BASE_PATH:-}" ]; then
    echo "📍 Using base path: $SITE_BASE_PATH"
fi

# Check for wasm32-unknown-unknown target
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "❌ Error: wasm32-unknown-unknown target not installed"
    echo ""
    echo "Please install it with:"
    echo "  rustup target add wasm32-unknown-unknown"
    echo ""
    exit 1
fi

# Step 1: Build WASM for islands
echo "📦 Building WASM..."
if ! cargo build --target wasm32-unknown-unknown --release --lib --no-default-features --features hydrate; then
    echo "❌ WASM build failed"
    exit 1
fi

# Check for wasm-bindgen
if ! command -v wasm-bindgen &> /dev/null; then
    echo "❌ Error: wasm-bindgen not found in PATH"
    echo ""
    echo "Please install it with:"
    echo "  cargo install wasm-bindgen-cli"
    echo ""
    echo "Or ensure it's in your PATH"
    exit 1
fi

# Step 2: Run wasm-bindgen
echo "🔗 Running wasm-bindgen..."
mkdir -p "$OUTDIR/pkg"
if ! wasm-bindgen target/wasm32-unknown-unknown/release/longitudinal_dev.wasm \
    --target web \
    --no-typescript \
    --out-dir "$OUTDIR/pkg" \
    --out-name blog; then
    echo "❌ wasm-bindgen failed"
    exit 1
fi

# Step 3: Build Tailwind CSS
echo "🎨 Building Tailwind CSS..."
# Minification now works correctly in v4.1.16 stable
if ! npx tailwindcss -i style/input.css -o "$OUTDIR/pkg/blog.css" --minify; then
    echo "❌ Tailwind CSS build failed"
    echo ""
    echo "Ensure you have run 'npm install' to set up dependencies:"
    echo "  npm install"
    exit 1
fi

# Step 4: Copy public assets
echo "📁 Copying public assets..."
if [ -d "public" ]; then
    mkdir -p "$OUTDIR"
    cp -r public/* "$OUTDIR/"
    echo "✅ Public assets copied"
else
    echo "⚠️  No public directory found, skipping asset copy"
fi

# Step 5: Build and run SSG binary
echo "📄 Generating static HTML..."
if ! cargo build --bin longitudinal_dev --features ssr --release; then
    echo "❌ SSG binary build failed"
    exit 1
fi

if ! ./target/release/longitudinal_dev --outdir "$OUTDIR"; then
    echo "❌ SSG generation failed"
    exit 1
fi

# Step 6: Inject FOUC prevention script into all HTML files
echo "🎨 Injecting theme FOUC prevention script..."
THEME_SCRIPT='<script>(function(){const theme=localStorage.getItem("theme")||"light";document.documentElement.setAttribute("data-theme",theme)})()<\/script>'

# Find all HTML files and inject the script right after <head>
find "$OUTDIR" -name "*.html" -type f -exec perl -i -pe "s/<head>/<head>$THEME_SCRIPT/" {} \;

echo "✅ SSG build complete! Output in $OUTDIR/"
echo "📊 File sizes:"
ls -lh "$OUTDIR"/*.html 2>/dev/null | awk '{print "  " $9 ": " $5}' || true
ls -lh "$OUTDIR/pkg"/* 2>/dev/null | awk '{print "  " $9 ": " $5}' || true
