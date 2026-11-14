# Contributing to Longitudinal.dev

Thank you for your interest in contributing! This guide covers how to contribute content, report issues, and work with the codebase.

## Ways to Contribute

### 1. Suggest Improvements to Tutorials

The easiest way to contribute is directly from the website:

1. Visit any tutorial page on [longitudinal.dev](https://longitudinal.dev)
2. Click the **"Suggest changes"** button in the right sidebar
3. Fill out the form with your suggestions
4. Submit! Your feedback will create a GitHub issue for review

### 2. Report Issues or Request Features

Found a bug or have an idea? [Open an issue](https://github.com/beginDevSci/longitudinal-dev/issues) on GitHub:

- **Bug reports**: Describe what happened vs. what you expected
- **Feature requests**: Explain the use case and benefit
- **Content gaps**: Suggest topics or tutorials you'd like to see

### 3. Join the Discussion

Connect with the community:

- **GitHub Discussions**: [github.com/beginDevSci/longitudinal-dev/discussions](https://github.com/beginDevSci/longitudinal-dev/discussions)
- **Discord**: [discord.gg/D796Bdy8](https://discord.gg/D796Bdy8)

Share ideas, ask questions, or help others!

### 4. Submit Code or Content

Ready to contribute directly? Here's how to work with the repository.

---

## Development Workflow

### Prerequisites

- **Git**
- **Rust** via [rustup](https://rustup.rs). `rust-toolchain.toml` selects the stable toolchain with the `wasm32-unknown-unknown` target, `rustfmt`, and `clippy`.
- **`wasm-bindgen-cli` 0.2.104**, exactly. It has to match the `wasm-bindgen` version in `Cargo.lock`. `make bootstrap` installs it.
- **Node.js and npm** for the Tailwind CSS build (CI uses Node 22).
- **Python 3** for the local preview server.
- Optional: **`wasm-opt`** (from binaryen) and **`brotli`**. Without them the build skips WASM optimization and Brotli compression and prints a warning.

R, the R packages listed in the `Makefile`, and local ABCD data are needed only to run tutorial analyses through the validation pipeline, not to preview the site.

### Getting Started

1. **Fork and clone:**
   ```bash
   git clone https://github.com/YOUR-USERNAME/longitudinal-dev.git
   cd longitudinal-dev
   ```

2. **Install the build tools:**
   ```bash
   make bootstrap
   ```
   This checks your environment, installs the `wasm32-unknown-unknown` target and `wasm-bindgen-cli` 0.2.104 if they're missing, and then runs `npm install` for Tailwind. It also reports on R, the R packages, and `ABCD_DATA_PATH`. If the only missing items are ones it can't install, such as R, it exits with an error before running `npm install`; run `npm ci` yourself in that case. To re-check without installing anything, run `cargo run --features ssr --bin check-prereqs`.

3. **Create a branch from `main`:**
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feature/your-feature-name
   ```

### Making Changes

#### Adding or Editing Tutorials

Tutorial content lives in `content/tutorials/` as Markdown files. Each tutorial follows a structured format:

- **Overview**: Introduction and goals
- **Data Access**: Where to get the data
- **Data Preparation**: Setup and cleaning
- **Statistical Analysis**: Core analysis code
- **Discussion**: Results interpretation
- **Additional Resources**: References and links

See existing tutorials for examples.

The site renders the generated JSON in `content/posts/`, not the Markdown, so a Markdown edit doesn't change the page until the JSON is regenerated. Regenerating it runs the validation pipeline, which needs local ABCD data. If you don't have that access, propose the Markdown change and a maintainer will regenerate the JSON.

### Previewing Locally

Build the site into `dist/` and serve it at http://localhost:8000:

```bash
make serve
```

Press Ctrl+C to stop the server. `make serve` rebuilds the site every time it starts. To build without serving, run `make ssg`; `cd dist && python3 -m http.server 8000` then serves the result.

`make ssg` and `make serve` leave the WebGPU brain viewer out of the WASM bundle that every page loads; no page uses the viewer at the moment. `make ssg-minimal` is an alias for `make ssg`, kept so existing instructions still work. To include the viewer, use `make ssg-viewer` or `make serve-viewer`. `make watch` (cargo-leptos) still includes the viewer, because the cargo-leptos settings in `Cargo.toml` turn it on.

The `ssg` and `serve` targets, with or without the viewer, use base path `/`, so links behave as they do at the site root. None of them clears `dist/`, so after renaming or removing a page, run `rm -rf dist` before building; otherwise the old page stays in the preview and can hide a broken link.

#### Refreshing After a Guide Edit

The site generator reads `content/guides/*.md` and `content/tutorial_curations.yaml` each time it runs. After one full build, keep the preview server running and regenerate just the HTML from the repository root:

```bash
./target/release/longitudinal_dev --outdir dist
```

This takes a few seconds; reload the page to see the change. Leave `SITE_BASE_PATH` unset so the base path stays `/`. Tutorial JSON, Toolkit YAML, Rust code, and CSS are compiled into the build, so rerun `make serve` or `make ssg` after changing those.

### Branches and Pull Requests

Work on a branch created from `main` and open pull requests against `main`:

```bash
git checkout main
git pull origin main
git checkout -b feature/my-change
# make and commit your changes
git push origin feature/my-change
```

Then open a pull request from your branch to `main`, describing what changed and why. CI (`.github/workflows/ci.yml`) runs on pull requests to `main`: it builds the site, runs the tests and clippy, and audits dependencies.

### Publishing

As configured in this repository:

- `.github/workflows/deploy.yml` runs on every push to `main` and on manual dispatch. It builds with `SITE_BASE_PATH=/longitudinal-dev/` and deploys the result to GitHub Pages. It doesn't wait for CI.
- `scripts/build/cloudflare_build.sh` is a build script for a Cloudflare Pages environment. It builds with base path `/` unless `SITE_BASE_PATH` is set.
- `make deploy` runs `scripts/deploy/deploy_github_pages.sh`, which is disabled: it prints a message and exits before running any Git or file commands. The steps below that guard are written to build with `/longitudinal-dev/` and push the result to a `gh-pages` branch, but after switching branches they delete every top-level entry in the working copy except `.git`, including `dist/` and Git-ignored files such as `target/` and `.envrc`, and then fail. Keep the guard until the publishing workflow is confirmed.

Which of these serves https://longitudinal.dev hasn't been confirmed yet.

### Branch Naming Conventions

- `feature/` - New functionality (e.g., `feature/add-search`)
- `content/` - Tutorial/content updates (e.g., `content/update-lgcm-tutorial`)
- `fix/` - Bug fixes (e.g., `fix/mobile-layout`)
- `docs/` - Documentation changes (e.g., `docs/update-contributing`)

---

## Code Guidelines

### Rust Code

- Follow standard Rust formatting (`cargo fmt`)
- Run clippy before committing (`cargo clippy`)
- Write clear comments for complex logic

### Markdown Content

- Use clear, concise language
- Include code examples with syntax highlighting
- Test all code snippets before submitting
- Add references where appropriate

### Commit Messages

Write clear, descriptive commit messages:

```
Good: "Add missing data preparation step to GLMM tutorial"
Good: "Fix broken link in resources section"
Bad: "updates"
Bad: "fix stuff"
```

---

## Testing

Before submitting a PR:

1. **Build successfully**: `make ssg` should complete without errors. [TESTING.md](TESTING.md) describes the preflight script, `scripts/dev/check.sh`
2. **Check links**: Verify all internal and external links work
3. **Test locally**: Browse the generated site at `http://localhost:8000`
4. **Verify examples**: Run any code examples to ensure they work

---

## Questions?

- Check existing [issues](https://github.com/beginDevSci/longitudinal-dev/issues)
- Ask in [Discussions](https://github.com/beginDevSci/longitudinal-dev/discussions)
- Join our [Discord](https://discord.gg/D796Bdy8)
- Email: support@longitudinal.dev

---

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

Thank you for helping make Longitudinal.dev better! 🎉
