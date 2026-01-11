//! Focus mode header component for the brain viewer.
//!
//! Displays a compact control bar when focus mode is active, with
//! hemisphere toggle, threshold slider, color legend, and exit button.

use leptos::prelude::*;

use crate::components::color_legend::ColorLegend;
use crate::data::StatisticMetadata;
use crate::types::{ColormapType, Hemisphere};

/// Focus mode header shown when the viewer is in focus mode.
///
/// Contains hemisphere toggle, threshold control, compact color legend,
/// and an exit button.
#[component]
pub fn FocusModeHeader(
    /// Current hemisphere selection
    hemisphere: Signal<Hemisphere>,
    /// Setter for hemisphere
    set_hemisphere: WriteSignal<Hemisphere>,
    /// Current threshold value
    threshold: Signal<Option<f32>>,
    /// Setter for threshold
    set_threshold: WriteSignal<Option<f32>>,
    /// Statistic metadata (for color legend) - ReadSignal for pass-through to ColorLegend
    stat_metadata: ReadSignal<Option<StatisticMetadata>>,
    /// Current stat range (for color legend) - ReadSignal for pass-through to ColorLegend
    stat_range: ReadSignal<Option<(f32, f32)>>,
    /// Current colormap (for color legend) - ReadSignal for pass-through to ColorLegend
    colormap: ReadSignal<ColormapType>,
    /// Whether symmetric mode is enabled (for color legend) - ReadSignal for pass-through to ColorLegend
    symmetric: ReadSignal<bool>,
    /// Callback when exit button is clicked
    on_exit: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="brain-viewer-focus-overlay">
            // Focus mode header with key controls
            <div class="brain-viewer-focus-header">
                // Hemisphere toggle
                <div class="focus-header-control">
                    <span class="focus-header-label">"Hemisphere"</span>
                    <div class="segmented-control segmented-control-sm">
                        <button
                            type="button"
                            class=move || format!(
                                "segmented-control-btn {}",
                                if hemisphere.get() == Hemisphere::Left { "active" } else { "" }
                            )
                            on:click=move |_| set_hemisphere.set(Hemisphere::Left)
                        >
                            "L"
                        </button>
                        <button
                            type="button"
                            class=move || format!(
                                "segmented-control-btn {}",
                                if hemisphere.get() == Hemisphere::Right { "active" } else { "" }
                            )
                            on:click=move |_| set_hemisphere.set(Hemisphere::Right)
                        >
                            "R"
                        </button>
                    </div>
                </div>

                // Threshold control
                <div class="focus-header-control focus-header-threshold">
                    <span class="focus-header-label">"Threshold"</span>
                    <div class="threshold-control threshold-control-sm">
                        <input
                            type="range"
                            min="0"
                            max="10"
                            step="0.1"
                            class="threshold-slider"
                            prop:value=move || threshold.get().unwrap_or(0.0)
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                if let Ok(v) = value.parse::<f32>() {
                                    set_threshold.set(Some(v));
                                }
                            }
                        />
                        <input
                            type="number"
                            min="0"
                            max="10"
                            step="0.1"
                            class="threshold-input threshold-input-sm"
                            prop:value=move || threshold.get().map(|t| format!("{:.2}", t)).unwrap_or_default()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                if let Ok(v) = value.parse::<f32>() {
                                    set_threshold.set(Some(v));
                                }
                            }
                        />
                    </div>
                </div>

                // Compact color legend
                <div class="focus-header-legend">
                    <ColorLegend
                        metadata=stat_metadata
                        range=stat_range
                        colormap=colormap
                        symmetric=symmetric
                    />
                </div>

                // Exit button
                <button
                    class="brain-viewer-focus-exit"
                    on:click=move |_| on_exit.run(())
                    title="Exit focus mode (ESC)"
                    aria-label="Exit focus mode"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <line x1="18" y1="6" x2="6" y2="18"></line>
                        <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                </button>
            </div>
        </div>
    }
}
