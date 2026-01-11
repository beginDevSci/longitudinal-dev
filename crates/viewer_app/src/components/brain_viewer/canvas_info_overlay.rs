//! Canvas info overlay component for the brain viewer.
//!
//! Displays stat name/range, threshold slider, and selected vertex info
//! in a compact overlay on the canvas.

use leptos::prelude::*;

use crate::data::StatisticMetadata;
use crate::types::VertexInfo;

/// Canvas info overlay shown on top of the brain visualization.
///
/// Contains stat name/range display, threshold slider, and selection info.
#[component]
pub fn CanvasInfoOverlay(
    /// Statistic metadata (for display name)
    stat_metadata: Signal<Option<StatisticMetadata>>,
    /// Current stat range (min, max)
    stat_range: Signal<Option<(f32, f32)>>,
    /// Whether symmetric mode is enabled (affects displayed range)
    symmetric: Signal<bool>,
    /// Current threshold value
    threshold: Signal<Option<f32>>,
    /// Setter for threshold
    set_threshold: WriteSignal<Option<f32>>,
    /// Currently selected vertex
    selected_vertex: Signal<Option<VertexInfo>>,
    /// Currently selected region name
    selected_region: Signal<Option<String>>,
    /// Setter for selected vertex (to clear selection)
    set_selected_vertex: WriteSignal<Option<VertexInfo>>,
) -> impl IntoView {
    let on_threshold_change = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        if let Ok(v) = value.parse::<f32>() {
            set_threshold.set(Some(v));
        }
    };

    view! {
        <div class="canvas-info-overlay">
            // Stat name and range row
            {move || {
                if let Some(meta) = stat_metadata.get() {
                    let name = meta.display_name.clone();
                    let (min, max) = stat_range.get().unwrap_or((0.0, 0.0));
                    // Use user's symmetric preference to center range at 0
                    let (display_min, display_max) = if symmetric.get() && min < 0.0 && max > 0.0 {
                        let abs_max = min.abs().max(max.abs());
                        (-abs_max, abs_max)
                    } else {
                        (min, max)
                    };
                    view! {
                        <div class="canvas-info-stat">
                            <span class="canvas-info-stat-name">{name}</span>
                            <span class="canvas-info-stat-range">
                                {format!("[{:.1}, {:.1}]", display_min, display_max)}
                            </span>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}

            // Threshold slider row
            <div class="canvas-info-threshold">
                <span class="threshold-label">"T"</span>
                <input
                    type="range"
                    min="0"
                    max="10"
                    step="0.1"
                    class="threshold-slider"
                    prop:value=move || threshold.get().unwrap_or(0.0)
                    on:input=on_threshold_change
                />
                <span class="threshold-value">
                    {move || format!("{:.1}", threshold.get().unwrap_or(0.0))}
                </span>
            </div>

            // Selected vertex info row (only when something is selected)
            {move || {
                if let Some(v) = selected_vertex.get() {
                    let hemi = match v.hemisphere() {
                        Some(crate::types::Hemisphere::Left) => "LH",
                        Some(crate::types::Hemisphere::Right) => "RH",
                        None => "",
                    };
                    let value_str = if v.value.is_nan() {
                        "NaN".to_string()
                    } else if v.value.abs() < 0.01 || v.value.abs() >= 1000.0 {
                        format!("{:.2e}", v.value)
                    } else {
                        format!("{:.3}", v.value)
                    };
                    let region_text = selected_region.get()
                        .map(|r| format!(" [{}]", r))
                        .unwrap_or_default();
                    view! {
                        <div class="canvas-info-selection">
                            <span class="canvas-info-selection-text">
                                {if !hemi.is_empty() { format!("{} ", hemi) } else { String::new() }}
                                "Idx " {v.index} " = " {value_str}
                                {region_text}
                            </span>
                            <button
                                class="canvas-info-selection-clear"
                                on:click=move |_| set_selected_vertex.set(None)
                                title="Clear selection"
                            >
                                "×"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}
        </div>
    }
}
