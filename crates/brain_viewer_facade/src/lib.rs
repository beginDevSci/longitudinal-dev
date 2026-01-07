//! Facade crate for the WebGPU brain surface viewer.
//!
//! Exposes a single `BrainViewerIsland` component that can be embedded in
//! longitudinal-dev tutorials. The island renders a "Load viewer" button
//! on initial load, and the actual WebGPU viewer when clicked (on wasm32).
//!
//! ## Architecture
//!
//! - SSR (native): Renders fallback with "Load viewer" button, wrapped in `<leptos-island>`
//! - Client (wasm32): Hydrates the island, loads real viewer on button click
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
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct ViewerOverrides {
    #[serde(default)]
    pub analysis: Option<String>,
    #[serde(default)]
    pub statistic: Option<String>,
    #[serde(default)]
    pub volume_idx: Option<u32>,
    #[serde(default)]
    pub colormap: Option<String>,
    #[serde(default)]
    pub threshold: Option<f32>,
    #[serde(default)]
    pub hemisphere: Option<String>,
}

/// Configuration for a brain viewer instance.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ViewerData {
    pub manifest_path: String,
    #[serde(default)]
    pub overrides: ViewerOverrides,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub fallback_image: Option<String>,
    #[serde(default)]
    pub fallback_alt: Option<String>,
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
// Single island that works on both SSR (native) and client (wasm32)
// ============================================================================

/// Renders the fallback/placeholder view with "Load viewer" button.
fn render_fallback(
    fallback_src: Option<String>,
    fallback_alt: String,
    caption: Option<String>,
    on_load_click: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <div class="brain-viewer-island">
            <div class="viewer-placeholder relative rounded-lg border border-default bg-subtle overflow-hidden">
                {fallback_src.map(|src| view! {
                    <img
                        src=src
                        alt=fallback_alt.clone()
                        class="w-full"
                        loading="lazy"
                    />
                })}
                <div class="absolute inset-0 flex items-center justify-center bg-black/40 rounded-lg">
                    <button
                        class="px-6 py-3 bg-emerald-600 hover:bg-emerald-700 text-white font-medium rounded-lg shadow-lg transition-colors flex items-center gap-2"
                        on:click=move |_| on_load_click()
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/>
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                        </svg>
                        "Load Interactive Viewer"
                    </button>
                </div>
            </div>
            {caption.map(|cap| view! {
                <figcaption class="figure-caption mt-2 text-center text-sm text-muted">
                    {cap}
                </figcaption>
            })}
        </div>
    }
}

// When webgpu-viewer feature is enabled, we have the actual viewer available on wasm32
#[cfg(feature = "webgpu-viewer")]
mod viewer_impl {
    use super::*;

    /// Brain viewer island - renders fallback initially, loads WebGPU viewer on click.
    ///
    /// This island compiles on both native (SSR) and wasm32 (client):
    /// - On SSR: Renders the fallback with button, serializes props into `<leptos-island>`
    /// - On client: Hydrates, and when button clicked, loads the actual viewer (wasm32 only)
    #[island]
    pub fn BrainViewerIsland(data: ViewerData) -> impl IntoView {
        let (viewer_started, set_viewer_started) = signal(data.auto_start);

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
                        // Viewer is started - render real viewer on wasm32, message on native
                        #[cfg(target_arch = "wasm32")]
                        {
                            view! {
                                <div class="viewer-active">
                                    <viewer_app::BrainViewerComponent data_base_path=data_base_path.clone() />
                                </div>
                            }.into_any()
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            // This branch runs on SSR if auto_start=true, but typically auto_start=false
                            view! {
                                <div class="viewer-loading p-8 text-center">
                                    <p class="text-muted">"Viewer loading..."</p>
                                </div>
                            }.into_any()
                        }
                    } else {
                        // Show fallback with "Load viewer" button
                        view! {
                            <div class="viewer-placeholder relative rounded-lg border border-default bg-subtle overflow-hidden">
                                {fallback_src.clone().map(|src| view! {
                                    <img
                                        src=src
                                        alt=fallback_alt.clone()
                                        class="w-full"
                                        loading="lazy"
                                    />
                                })}
                                <div class="absolute inset-0 flex items-center justify-center bg-black/40">
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
                {caption.clone().map(|cap| view! {
                    <figcaption class="figure-caption mt-2 text-center text-sm text-muted">
                        {cap}
                    </figcaption>
                })}
            </div>
        }
    }
}

#[cfg(feature = "webgpu-viewer")]
pub use viewer_impl::BrainViewerIsland;

// When webgpu-viewer feature is disabled, provide a simple fallback component
#[cfg(not(feature = "webgpu-viewer"))]
mod fallback_impl {
    use super::*;

    /// Fallback component when webgpu-viewer feature is disabled.
    #[component]
    pub fn BrainViewerIsland(data: ViewerData) -> impl IntoView {
        let fallback_alt = data
            .fallback_alt
            .unwrap_or_else(|| "Brain visualization (static)".to_string());

        let has_fallback = data.fallback_image.is_some();

        view! {
            <div class="viewer-fallback rounded-lg border border-default bg-subtle overflow-hidden">
                {data.fallback_image.map(|src| view! {
                    <figure class="figure-frame">
                        <img src=src alt=fallback_alt.clone() class="w-full" loading="lazy" />
                    </figure>
                })}
                {(!has_fallback).then(|| view! {
                    <div class="flex flex-col items-center justify-center p-12 text-center">
                        <p class="text-muted">"Interactive viewer not available."</p>
                    </div>
                })}
                {data.caption.map(|cap| view! {
                    <figcaption class="figure-caption mt-2 text-center text-sm text-muted">{cap}</figcaption>
                })}
            </div>
        }
    }
}

#[cfg(not(feature = "webgpu-viewer"))]
pub use fallback_impl::BrainViewerIsland;
