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
    let _ = symmetric;
    let (display_threshold, set_display_threshold) = signal(0.0f32);
    let (display_initialized, set_display_initialized) = signal(false);

    Effect::new(move |_| {
        if !display_initialized.get() {
            if let Some(value) = threshold.get() {
                set_display_threshold.set(value);
                set_display_initialized.set(true);
            }
        }
    });
    let on_threshold_change = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        if let Ok(v) = value.parse::<f32>() {
            set_display_threshold.set(v);
            set_threshold.set(Some(v.abs()));
        }
    };

    view! {
        <div class="canvas-info-overlay">
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
                        <div class="canvas-info-panel">
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
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}

            // Vertical threshold slider (right side)
            <div class="canvas-info-vertical-slider">
                <div class="canvas-info-vertical-title">
                    {move || {
                        stat_metadata.get()
                            .map(|m| {
                                m.display_name
                                    .replace("T-Statistics", "T-Stats")
                                    .replace(" (DES1)", "")
                            })
                            .unwrap_or_else(|| "T-Stats".to_string())
                    }}
                </div>
                <div class="canvas-info-vertical-strip">
                    <div class="canvas-info-vertical-track">
                        <div class="canvas-info-vertical-ticks" aria-hidden="true">
                            <span></span>
                            <span></span>
                            <span></span>
                        </div>
                        <input
                            type="range"
                            min=move || {
                                stat_range.get()
                                    .map(|(min, max)| {
                                        let abs_max = min.abs().max(max.abs());
                                        -abs_max.ceil()
                                    })
                                    .unwrap_or(-10.0)
                            }
                            max=move || {
                                stat_range.get()
                                    .map(|(min, max)| {
                                        let abs_max = min.abs().max(max.abs());
                                        abs_max.ceil()
                                    })
                                    .unwrap_or(10.0)
                            }
                            step="0.1"
                            class="threshold-slider-vertical"
                            prop:value=move || display_threshold.get()
                            on:input=on_threshold_change
                        />
                    </div>
                    <div class="canvas-info-vertical-labels">
                        <span>{move || {
                            let (min, max) = stat_range.get().unwrap_or((0.0, 0.0));
                            let abs_max = min.abs().max(max.abs());
                            format!("{:.1}", abs_max)
                        }}</span>
                        <span>{move || "0".to_string()}</span>
                        <span>{move || {
                            let (min, max) = stat_range.get().unwrap_or((0.0, 0.0));
                            let abs_max = min.abs().max(max.abs());
                            format!("{:.1}", -abs_max)
                        }}</span>
                    </div>
                </div>
                <div class="canvas-info-vertical-value">
                    {move || format!("T {:.1}", display_threshold.get())}
                </div>
            </div>
        </div>
    }
}
