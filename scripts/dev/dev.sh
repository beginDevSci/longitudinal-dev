#!/usr/bin/env bash
set -euo pipefail

echo "🚀 Starting Leptos development server..."
echo ""
echo "Note: Run Tailwind watch in a separate terminal:"
echo "  npx @tailwindcss/cli@latest -i style/input.css -o target/site/pkg/blog.css --watch"
echo ""

cargo leptos watch --site
