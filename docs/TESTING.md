# Testing Notes

The project ships with a single preflight script that wraps formatting, linting, Rust tests, optional SSG builds, and tutorial validation.

```bash
CHECK_SSG=true ./scripts/dev/check.sh      # run everything
./scripts/dev/check.sh lgcm-basic          # focus on a single tutorial
```

Additional tips:

- Use `cargo test --features ssr` for quick Rust-only checks.
- Run `make ssg` if you want to view the generated site under `dist/`.

As the codebase evolves we can expand this document with more scenarios, but for now the script above keeps the site healthy.

---

## Quick Start

```bash
git clone <repo-url>
cd longitudinal-dev
make bootstrap     # install prerequisites once
make serve         # build + host http://localhost:8000
```

For active work use `make watch` to run the dev watchers, or `make ssg` to generate `dist/` without serving.

## Common Commands

```bash
make bootstrap   # verify/install prerequisites
make watch       # run dev-time watchers (SSG + Tailwind)
make ssg         # build static site into ./dist
make serve       # build + serve on http://localhost:8000
make content     # fmt + clippy + tests + validation
```

---

## Local workflow

1. Install prerequisites (Rust stable, Node 18+, optional R environment).
2. Run `make bootstrap` the first time you clone the repo.
3. Use `make watch` while editing, or `make ssg` for a one-off build.
4. Before submitting changes, run `CHECK_SSG=true ./scripts/dev/check.sh` to format, lint, test, and regenerate the site.
