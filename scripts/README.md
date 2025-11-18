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
- `bootstrap.sh` – Verifies prerequisites (Rust target, wasm-bindgen CLI, npm deps) and installs missing pieces.
- `check.sh` – Main preflight entry point (fmt, clippy, tests, optional SSG build, tutorial validation).
- `dev.sh` – Convenience launcher for the multi-terminal dev environment.

**Note:** For prerequisite checking only (without installation), use `cargo run --bin check-prereqs` directly.

Most workflows also have Make targets (`make watch`, `make ssg`, `make serve`). Use the scripts directly when you need the raw helper.
