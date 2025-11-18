#![recursion_limit = "512"]

use leptos::prelude::*;

pub mod base_path;
pub mod config;
mod copy_code_button;
mod edit_page_button;
mod editor_modal_island;
mod section_container;
mod theme_toggle;

// New architecture modules
pub mod layout;
pub mod models;
pub mod sections;
pub mod ui;

// Multi-post templating modules
pub mod posts;
pub mod tutorial_catalog;

// Syntax highlighting (SSR only)
pub mod syntax_highlight;

pub use copy_code_button::CopyCodeButton;
pub use edit_page_button::EditPageButton;
pub use editor_modal_island::EditorModalIsland;
pub use layout::{PostLayout, SiteLayout};
pub use section_container::SectionContainer;
pub use theme_toggle::ThemeToggle;

/// The application root: render the fixed-layout post (legacy single-page mode)
///
/// NOTE: This component now uses SiteLayout as the canonical wrapper.
#[component]
pub fn App() -> impl IntoView {
    let all_posts = posts::posts();
    let first_post = all_posts
        .into_iter()
        .next()
        .expect("at least one post required");

    // Note: This App component is used for hydration only.
    // In SSG mode, prefill_markdown and baseline_hash are provided by main.rs.
    // For hydration, we use empty strings as these values are embedded in the HTML.
    view! {
        <PostLayout
            post=first_post
            prefill_markdown=""
            baseline_hash=""
        />
    }
}

// ---- Client entry (WASM) ----
// Gate to wasm32 to prevent accidental native builds with --all-features
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn hydrate() {
    // Islands-only hydration: only #[island] components wake up
    leptos::mount::hydrate_islands();
}
