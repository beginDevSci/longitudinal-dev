//! Facade crate for the WebGPU brain surface viewer.
//!
//! Exposes a single `BrainViewerIsland` component that can be embedded in
//! longitudinal-dev tutorials. When the `webgpu-viewer` feature is enabled,
//! renders the full interactive 3D viewer. When disabled, renders a static
//! fallback image.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use brain_viewer_facade::{BrainViewerIsland, ViewerData};
//!
//! let data = ViewerData {
//!     manifest_path: "/data/blmm/manifest.json".to_string(),
//!     ..Default::default()
//! };
//!
//! view! { <BrainViewerIsland data=data /> }
//! ```

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration overrides for the viewer's initial state.
///
/// All fields are optional - when `None`, the viewer uses its defaults.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct ViewerOverrides {
    /// Analysis to display: "des1", "des2", "compare"
    #[serde(default)]
    pub analysis: Option<String>,
    /// Statistic type: "tstat", "beta", "conTlp", "sigma2", "Chi2", "Chi2lp"
    #[serde(default)]
    pub statistic: Option<String>,
    /// Volume index (0-based) for multi-volume statistics
    #[serde(default)]
    pub volume_idx: Option<u32>,
    /// Colormap: "hot", "cool", "viridis", "plasma", "rdbu"
    #[serde(default)]
    pub colormap: Option<String>,
    /// Threshold value for statistical maps
    #[serde(default)]
    pub threshold: Option<f32>,
    /// Hemisphere: "lh" (left), "rh" (right)
    #[serde(default)]
    pub hemisphere: Option<String>,
}

/// Configuration for a brain viewer instance.
///
/// This is the primary configuration type passed to `BrainViewerIsland`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ViewerData {
    /// Path to the viewer manifest JSON (contains analysis metadata, file paths)
    pub manifest_path: String,

    /// Optional overrides for the viewer's initial state
    #[serde(default)]
    pub overrides: ViewerOverrides,

    /// Optional caption displayed below the viewer
    #[serde(default)]
    pub caption: Option<String>,

    /// Fallback image to show when WebGPU is unavailable or feature disabled
    #[serde(default)]
    pub fallback_image: Option<String>,

    /// Alt text for the fallback image
    #[serde(default)]
    pub fallback_alt: Option<String>,

    /// If false, show a "Load viewer" button instead of auto-initializing
    /// Useful for reducing initial page load when viewer isn't immediately needed
    #[serde(default)]
    pub auto_start: bool,
}

impl Default for ViewerData {
    fn default() -> Self {
        Self {
            manifest_path: String::new(),
            overrides: ViewerOverrides::default(),
            caption: None,
            fallback_image: None,
            fallback_alt: None,
            auto_start: false,
        }
    }
}

// ============================================================================
// WebGPU-enabled viewer (feature = "webgpu-viewer" + wasm32 target)
// ============================================================================

#[cfg(all(feature = "webgpu-viewer", target_arch = "wasm32"))]
mod webgpu_impl {
    use super::*;
    #[allow(unused_imports)]
    use leptos::prelude::*;

    /// Interactive WebGPU brain surface viewer island.
    ///
    /// When `auto_start` is false, displays a "Load viewer" button over the
    /// fallback image. Clicking the button initializes the WebGPU renderer.
    #[island]
    pub fn BrainViewerIsland(data: ViewerData) -> impl IntoView {
        let (viewer_started, set_viewer_started) = signal(data.auto_start);

        // Derive data_base_path from manifest_path
        // e.g., "/data/blmm/manifest.json" -> "/data/blmm"
        let data_base_path = data
            .manifest_path
            .rsplit_once('/')
            .map(|(base, _)| base.to_string())
            .unwrap_or_else(|| "/data".to_string());

        let fallback_alt = data
            .fallback_alt
            .clone()
            .unwrap_or_else(|| "Interactive brain visualization".to_string());

        let fallback_src = data.fallback_image.clone();
        let caption = data.caption.clone();

        view! {
            <div class="brain-viewer-island">
                {move || {
                    if viewer_started.get() {
                        // Render the actual viewer
                        view! {
                            <div class="viewer-active">
                                <viewer_app::BrainViewerComponent data_base_path=data_base_path.clone() />
                            </div>
                        }.into_any()
                    } else {
                        // Render fallback with "Load viewer" button
                        view! {
                            <div class="viewer-placeholder relative">
                                {fallback_src.clone().map(|src| view! {
                                    <img
                                        src=src
                                        alt=fallback_alt.clone()
                                        class="w-full rounded-lg"
                                        loading="lazy"
                                    />
                                })}
                                <div class="absolute inset-0 flex items-center justify-center bg-black/30 rounded-lg">
                                    <button
                                        class="px-6 py-3 bg-emerald-600 hover:bg-emerald-700 text-white font-medium rounded-lg shadow-lg transition-colors flex items-center gap-2"
                                        on:click=move |_| set_viewer_started.set(true)
                                    >
                                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/>
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                        </svg>
                                        "Load Interactive Viewer"
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
                {caption.map(|cap| view! {
                    <figcaption class="figure-caption mt-2 text-center text-sm text-muted">
                        {cap}
                    </figcaption>
                })}
            </div>
        }
    }
}

#[cfg(all(feature = "webgpu-viewer", target_arch = "wasm32"))]
pub use webgpu_impl::BrainViewerIsland;

// ============================================================================
// Static fallback (used when webgpu-viewer disabled OR on non-wasm32 targets)
// ============================================================================

#[cfg(not(all(feature = "webgpu-viewer", target_arch = "wasm32")))]
mod fallback_impl {
    use super::*;
    #[allow(unused_imports)]
    use leptos::prelude::*;

    /// Static fallback component when WebGPU viewer is disabled.
    ///
    /// Renders the fallback image (if provided) with a message indicating
    /// the interactive viewer is not available.
    #[component]
    pub fn BrainViewerIsland(data: ViewerData) -> impl IntoView {
        let fallback_alt = data
            .fallback_alt
            .unwrap_or_else(|| "Brain visualization (static)".to_string());

        let has_fallback_image = data.fallback_image.is_some();

        view! {
            <div class="viewer-fallback rounded-lg border border-default bg-subtle overflow-hidden">
                {data.fallback_image.map(|src| view! {
                    <figure class="figure-frame">
                        <img
                            src=src
                            alt=fallback_alt.clone()
                            class="w-full"
                            loading="lazy"
                        />
                    </figure>
                })}
                {(!has_fallback_image).then(|| view! {
                    <div class="flex flex-col items-center justify-center p-12 text-center">
                        <svg class="w-16 h-16 text-muted mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"/>
                        </svg>
                        <p class="text-lg font-medium text-secondary mb-2">"Brain Viewer"</p>
                        <p class="text-sm text-muted max-w-md">
                            "Interactive viewer not available. Enable the webgpu-viewer feature to view 3D brain surfaces."
                        </p>
                    </div>
                })}
                {data.caption.map(|cap| view! {
                    <figcaption class="figure-caption mt-2 text-center text-sm text-muted">
                        {cap}
                    </figcaption>
                })}
            </div>
        }
    }
}

#[cfg(not(all(feature = "webgpu-viewer", target_arch = "wasm32")))]
pub use fallback_impl::BrainViewerIsland;
