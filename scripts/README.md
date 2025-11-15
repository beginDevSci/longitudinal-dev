# Scripts

```
scripts/
├─ build/   # helpers that build CSS/SSG or watch WASM artifacts
├─ deploy/  # scripts that publish builds
├─ dev/     # local tooling (bootstrap, preflight checks, etc.)
└─ pre-commit
```

## build/
- `build_css.sh` – Runs Tailwind to compile `style/input.css`.
- `build_ssg.sh` – Legacy SSG wrapper (prefer `make ssg`).
- `watch_wasm.sh` – Watches for Rust changes and rebuilds the WASM bundle. Set `MODE=release` for production parity.

## deploy/
- `deploy_github_pages.sh` – Builds with the correct base path and pushes the site to `gh-pages`.

## dev/
- `bootstrap.sh` – Verifies prerequisites (Rust target, wasm-bindgen CLI, npm deps) and installs missing pieces.
- `check-prerequisites.sh` – Helper invoked by the preflight script.
- `check.sh` – Main preflight entry point (fmt, clippy, tests, optional SSG build, tutorial validation).
- `check_about_page.sh` – Smoke test ensuring the About page renders during SSG output.
- `dev.sh` – Convenience launcher for the multi-terminal dev environment.

Most workflows also have Make targets (`make watch`, `make ssg`, `make serve`). Use the scripts directly when you need the raw helper.
