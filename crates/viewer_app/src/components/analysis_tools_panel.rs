//! Analysis Tools Panel - Below-canvas collapsible panel for analysis tools.
//!
//! This component renders a collapsible panel containing controls, camera presets,
//! and top vertices in a 3-column grid layout.

use leptos::callback::Callback;
use leptos::prelude::*;

use crate::data::{StatisticData, StatisticMetadata};
use crate::types::{Analysis, BrainViewPreset, ColormapType, Hemisphere, Statistic};

use super::camera_presets::CameraPresets;
use super::controls_card::ControlsCard;
use super::vertex_summary_table::VertexSummaryTable;

/// Analysis Tools Panel - collapsible panel below the legend strip.
///
/// Contains 3 tool cards in a row:
/// - Controls (visualization settings)
/// - Hemisphere + Camera Views
/// - Top Vertices
#[component]
pub fn AnalysisToolsPanel(
    // Hemisphere props (now in Camera Views card)
    hemisphere: ReadSignal<Hemisphere>,
    set_hemisphere: WriteSignal<Hemisphere>,
    // ControlsCard props
    volume_idx: ReadSignal<u32>,
    set_volume_idx: WriteSignal<u32>,
    n_volumes: Signal<u32>,
    stat_metadata: ReadSignal<Option<StatisticMetadata>>,
    colormap: ReadSignal<ColormapType>,
    set_colormap: WriteSignal<ColormapType>,
    symmetric: ReadSignal<bool>,
    set_symmetric: WriteSignal<bool>,
    analysis: ReadSignal<Analysis>,
    set_analysis: WriteSignal<Analysis>,
    statistic: ReadSignal<Statistic>,
    set_statistic: WriteSignal<Statistic>,

    // CameraPresets props
    on_view_preset: Callback<BrainViewPreset>,
    current_view: ReadSignal<Option<BrainViewPreset>>,

    // Shared
    disabled: ReadSignal<bool>,

    // VertexSummaryTable props
    statistics: Signal<Option<StatisticData>>,
) -> impl IntoView {
    // Load initial collapsed state from localStorage - default to EXPANDED since this contains essential controls
    #[allow(unused_variables)]
    let storage_key = "bv_analysis_panel_collapsed";
    let initial_collapsed = {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|storage| storage.get_item(storage_key).ok().flatten())
                .map(|v| v == "true")
                .unwrap_or(false) // Default EXPANDED
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            false
        }
    };

    let (collapsed, set_collapsed) = signal(initial_collapsed);

    // Persist collapsed state to localStorage
    Effect::new(move |_| {
        let is_collapsed = collapsed.get();
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = storage.set_item(storage_key, if is_collapsed { "true" } else { "false" });
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = is_collapsed;
        }
    });

    let toggle = move |_| {
        set_collapsed.update(|c| *c = !*c);
    };

    view! {
        <div class="analysis-tools-panel">
            // Panel header - clickable to toggle
            <button
                type="button"
                class="analysis-tools-panel-header"
                on:click=toggle
                aria-expanded=move || (!collapsed.get()).to_string()
                aria-controls="analysis-tools-content"
            >
                // Chevron indicator
                <span
                    class="analysis-tools-panel-chevron"
                    class:analysis-tools-panel-chevron-open=move || !collapsed.get()
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                    </svg>
                </span>
                <span class="analysis-tools-panel-title">"Analysis Tools"</span>
            </button>

            // Animated content area
            <div
                id="analysis-tools-content"
                class="analysis-tools-panel-content"
                class:analysis-tools-panel-collapsed=move || collapsed.get()
                class:analysis-tools-panel-expanded=move || !collapsed.get()
            >
                <div class="overflow-hidden">
                    <div class="analysis-tools-panel-inner">
                        <div class="analysis-tools-grid-3">
                            // Card 1: Controls
                            <div class="analysis-tools-card">
                                <div class="analysis-tools-card-header">
                                    <svg class="analysis-tools-card-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <circle cx="12" cy="12" r="3"/>
                                        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
                                    </svg>
                                    <span class="analysis-tools-card-title">"Controls"</span>
                                </div>
                                <ControlsCard
                                    volume_idx=volume_idx
                                    set_volume_idx=set_volume_idx
                                    n_volumes=n_volumes
                                    stat_metadata=stat_metadata
                                    colormap=colormap
                                    set_colormap=set_colormap
                                    symmetric=symmetric
                                    set_symmetric=set_symmetric
                                    analysis=analysis
                                    set_analysis=set_analysis
                                    statistic=statistic
                                    set_statistic=set_statistic
                                    disabled=disabled
                                />
                            </div>

                            // Card 2: Hemisphere + Camera Views
                            <div class="analysis-tools-card">
                                <div class="analysis-tools-card-header">
                                    <svg class="analysis-tools-card-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/>
                                        <circle cx="12" cy="13" r="4"/>
                                    </svg>
                                    <span class="analysis-tools-card-title">"Camera Views"</span>
                                </div>
                                <div class="analysis-tools-section-body">
                                    // Hemisphere selector - above camera presets
                                    <div class="controls-row">
                                        <label class="controls-label">"Hemisphere"</label>
                                        <div class="controls-segmented" role="group">
                                            <button
                                                type="button"
                                                class=move || if hemisphere.get() == Hemisphere::Left { "controls-seg-btn active" } else { "controls-seg-btn" }
                                                on:click=move |_| set_hemisphere.set(Hemisphere::Left)
                                                disabled=move || disabled.get()
                                            >
                                                "L"
                                            </button>
                                            <button
                                                type="button"
                                                class=move || if hemisphere.get() == Hemisphere::Right { "controls-seg-btn active" } else { "controls-seg-btn" }
                                                on:click=move |_| set_hemisphere.set(Hemisphere::Right)
                                                disabled=move || disabled.get()
                                            >
                                                "R"
                                            </button>
                                        </div>
                                    </div>
                                    // Camera presets (filtered by selected hemisphere)
                                    <CameraPresets
                                        hemisphere=hemisphere
                                        on_preset=on_view_preset
                                        current_view=current_view
                                        disabled=disabled.into()
                                    />
                                </div>
                            </div>

                            // Card 3: Top Vertices
                            <div class="analysis-tools-card">
                                <div class="analysis-tools-card-header">
                                    <svg class="analysis-tools-card-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M3 3h18v18H3zM3 9h18M9 21V9"/>
                                    </svg>
                                    <span class="analysis-tools-card-title">"Top Vertices"</span>
                                </div>
                                <VertexSummaryTable
                                    statistics=statistics
                                    volume_idx=volume_idx
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
