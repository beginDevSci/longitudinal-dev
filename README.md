# Longitudinal.dev

Welcome! 👋

**Longitudinal.dev** is a community-driven, open-source resource hub for longitudinal data science. We're building a platform with tutorials, tools, code examples, and documentation to help researchers and practitioners work with longitudinal data.

## 🚧 Early Days

This project is actively under development—expect new content, features, and improvements to roll out regularly. Things might shift around as we find the best ways to serve the community. We value your feedback.

## 🤝 Get Involved

Open knowledge advances science. This platform thrives on community contributions:

- **Share your expertise** - Submit tutorials, tools, or research insights
- **Suggest improvements** - Found a typo? Have an idea? Use the "Suggest changes" button on any tutorial page
- **Join the discussion** - Connect with us on [GitHub Discussions](https://github.com/beginDevSci/longitudinal-dev/discussions) or [Discord](https://discord.gg/D796Bdy8)

Whether you're an experienced researcher or just getting started with longitudinal analysis, your contributions benefit the community.

## 📬 Contact

Questions, suggestions, or just want to say hi?

- **Email:** support@longitudinal.dev
- **GitHub:** [github.com/beginDevSci/longitudinal-dev](https://github.com/beginDevSci/longitudinal-dev)
- **Discussions:** [Join the conversation](https://github.com/beginDevSci/longitudinal-dev/discussions)
- **Discord:** [Join our community](https://discord.gg/D796Bdy8)

## 🙏 Thank You

Thank you to all contributors and supporters.

---

## 🗺️ Repository Map

To set up the tools and preview the site locally, see [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md).

| What | Where |
|------|-------|
| Method Guides | [`content/guides/`](content/guides/): one Markdown file per page (overview, `-walkthrough`, `-reference`). The `slug` in the frontmatter sets the URL; `TEMPLATE.md` isn't published. |
| ABCD tutorials, source | [`content/tutorials/`](content/tutorials/): the Markdown authors edit. |
| ABCD tutorials, rendered | [`content/posts/`](content/posts/): `*.post.json` generated from that Markdown by the validation pipeline ([`crates/validator`](crates/validator/), [`crates/parser`](crates/parser/)) and compiled into the site by [`build.rs`](build.rs). The site renders this JSON, so editing only the Markdown doesn't change the page. |
| Tutorial catalog curation | [`content/tutorial_curations.yaml`](content/tutorial_curations.yaml): featured tutorials and workflow groups on the ABCD Analyses page. |
| Toolkit | [`content/resources.yaml`](content/resources.yaml) and [`content/tools.yaml`](content/tools.yaml), rendered by the Toolkit pages in [`src/pages/`](src/pages/). |
| Shared layouts | [`src/layout/`](src/layout/): `site_layout.rs` (page shell), `top_nav.rs`, `post_layout.rs` (tutorials), `guide_layout.rs` (guides). Site-wide styles are in [`style/input.css`](style/input.css). |
| Other pages | [`src/main.rs`](src/main.rs) generates every page and contains the homepage and catalog headers; About and the ABCD overview are in [`src/pages/`](src/pages/). |
| Static files | [`public/`](public/): images and data, copied into the build unchanged. |
| Build | [`Makefile`](Makefile) targets run [`src/bin/xtask.rs`](src/bin/xtask.rs), which builds the WASM, CSS, and HTML into `dist/`. |
| Suggestions API | [`workers/suggestions/`](workers/suggestions/): the Cloudflare Worker behind "Suggest changes". |

---

## 🧠 Brain Viewer

The repository includes an interactive 3D brain surface viewer for visualizing neuroimaging statistical results. Built with Rust/WebAssembly and WebGPU.

### Crate Structure

| Crate | Description |
|-------|-------------|
| `brain_viewer_facade` | Entry point - Leptos island component for embedding in tutorials |
| `viewer_app` | Main Leptos viewer application |
| `core_render` | wgpu-based rendering engine |
| `io_formats` | FreeSurfer/GIFTI/NIfTI file format parsers |
| `neuro_surface` | Neuroimaging domain types and colormaps |
| `interaction` | Selection, hover, and ROI state management |

### Building the Viewer

```bash
# Build the viewer facade (requires wasm32 target)
cargo build --target wasm32-unknown-unknown -p brain_viewer_facade --features webgpu-viewer

# Run io_formats tests
cargo test -p io_formats
```

### Usage

The viewer is embedded in tutorials via the `BrainViewerIsland` component from `brain_viewer_facade`. See `crates/brain_viewer_facade/src/lib.rs` for usage examples.

---

**For developers and contributors:** Setup, local preview, and the branch workflow are in [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md); testing is covered in [docs/TESTING.md](docs/TESTING.md). Other files in `docs/` are design notes and plans.
