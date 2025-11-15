#!/usr/bin/env bash
set -euo pipefail

OUTDIR=${1:-dist-about-test}

cargo run -- --outdir "$OUTDIR" >/dev/null

ABOUT_FILE="$OUTDIR/about/index.html"

if [[ ! -f "$ABOUT_FILE" ]]; then
    echo "About page not generated at $ABOUT_FILE" >&2
    exit 1
fi

if ! rg -qi "this is the about page" "$ABOUT_FILE"; then
    echo "Placeholder text not found in $ABOUT_FILE" >&2
    exit 1
fi

echo "✅ About page generated successfully at $ABOUT_FILE"
