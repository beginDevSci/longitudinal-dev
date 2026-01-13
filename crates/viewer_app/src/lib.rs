//! BLMM Brain Viewer - Interactive 3D visualization of brain surface data.
//!
//! This crate provides a Leptos-based web application for visualizing brain
//! surfaces with statistical overlays. It's designed for viewing BLMM
//! (Bayesian Linear Mixed Models) analysis results on cortical surfaces.
//!
//! # Architecture
//!
//! The viewer is built with several layers:
//!
//! - **UI Layer** (`components/`): Leptos components for the user interface
//! - **Renderer Layer** (`renderer/`): Adapters connecting UI to GPU rendering
//! - **Data Layer** (`data/`): Loading and conversion of brain data files
//! - **Types** (`types/`): Shared type definitions
//!
//! # Feature Flags
//!
//! The crate uses wgpu for WebGPU rendering and supports the following loader configurations:
//!
//! ## Data Loaders
//!
//! - `io-formats-loader`: Use io_formats for loading real FreeSurfer surfaces
//! - `io-formats-overlay-loader`: Use io_formats for loading overlays (curv, GIFTI, NIfTI)
//! - `web-loaders`: Async HTTP loaders for WASM (requires wasm32 target, enabled by default)
//!
//! # Quick Start
//!
//! ```ignore
//! // In your main.rs or lib.rs for WASM
//! use brain_viewer::App;
//!
//! leptos::mount_to_body(|| view! { <App /> });
//! ```
//!
//! # Data Loading
//!
//! The viewer supports loading brain data from various sources:
//!
//! ## Local Files (native)
//!
//! ```ignore
//! use brain_viewer::data::{load_overlay_from_io_formats};
//!
//! let stats = load_overlay_from_io_formats(Path::new("lh.curv"))?;
//! ```
//!
//! ## HTTP Fetch (WASM)
//!
//! With the `web-loaders` feature enabled:
//!
//! ```ignore
//! use brain_viewer::data::{fetch_surface, fetch_overlay};
//!
//! let surface = fetch_surface("/data/lh.pial", Hemisphere::Left).await?;
//! let overlay = fetch_overlay("/data/lh.curv").await?;
//! ```
//!
//! # Expected Data Layout
//!
//! For the demo application, data should be organized under `public/data/`:
//!
//! ```text
//! public/data/
//!   surfaces/
//!     lh.pial           # Left hemisphere surface
//!     rh.pial           # Right hemisphere surface
//!   overlays/
//!     lh.curv           # Left hemisphere curvature
//!     rh.curv           # Right hemisphere curvature
//!   statistics/
//!     analysis_name/
//!       lh.t_stat.bin.gz  # BLMM statistics
//!       rh.t_stat.bin.gz
//! ```

use leptos::prelude::*;

mod components;
pub mod data;
pub mod preferences;
mod renderer;
pub mod types;

#[cfg(target_arch = "wasm32")]
use components::brain_viewer::BrainViewer;

// Re-export BrainViewer for external use (e.g., brain_viewer_facade)
#[cfg(target_arch = "wasm32")]
pub use components::brain_viewer::BrainViewer as BrainViewerComponent;

/// Main application component.
///
/// On WASM targets, this renders the full BrainViewer interface.
/// On other targets, this renders a placeholder (for build verification).
#[component]
pub fn App() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        view! {
            <main class="min-h-screen bg-gray-50 text-gray-900">
                <section class="max-w-6xl mx-auto py-8 px-4">
                    <h1 class="text-2xl font-bold mb-4">"BLMM Brain Viewer"</h1>
                    <p class="mb-4 text-sm text-gray-700">
                        "Interactive 3D visualization of BLMM statistical results on cortical surfaces."
                    </p>
                    <BrainViewer data_base_path="/data".to_string() />
                </section>
            </main>
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        view! {
            <main class="min-h-screen bg-gray-50 text-gray-900">
                <section class="max-w-6xl mx-auto py-8 px-4">
                    <h1 class="text-2xl font-bold mb-4">"BLMM Brain Viewer"</h1>
                    <p class="text-gray-700">
                        "This component requires a WASM target. Build with --target wasm32-unknown-unknown"
                    </p>
                </section>
            </main>
        }
    }
}

#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    leptos::mount::mount_to_body(App);
}
