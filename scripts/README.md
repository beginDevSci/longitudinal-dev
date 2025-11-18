# Scripts

```
scripts/
├─ build/   # helpers that build CSS/SSG or watch WASM artifacts
├─ deploy/  # scripts that publish builds
├─ dev/     # local tooling (bootstrap, preflight checks, etc.)
└─ pre-commit
```

## build/
- `cloudflare_build.sh` – Build script for Cloudflare Pages deployment environment.
- `watch_wasm.sh` – Watches for Rust changes and rebuilds the WASM bundle. Set `MODE=release` for production parity.

**Note:** CSS and SSG builds are now handled by `cargo xtask build-ssg` (see `src/bin/xtask.rs`).

## deploy/
- `deploy_github_pages.sh` – Builds with the correct base path and pushes the site to `gh-pages`.

## dev/
- `check.sh` – Main preflight entry point (fmt, clippy, tests, optional SSG build, tutorial validation).
- `dev.sh` – Convenience launcher for the multi-terminal dev environment.

**Note:** For prerequisite checking and installation, use:
- `cargo run --bin check-prereqs` - Check only
- `cargo run --bin check-prereqs --install` - Check and install missing prerequisites
- `cargo run --bin check-prereqs --dry-run` - Preview what would be installed
- `make bootstrap` - Recommended setup command (runs check-prereqs --install)

Most workflows also have Make targets (`make watch`, `make ssg`, `make serve`). Use the scripts directly when you need the raw helper.
