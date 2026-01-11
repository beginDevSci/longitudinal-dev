use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::data::StatisticMetadata;
use crate::types::ColormapType;

/// Compact color legend showing stat name and range.
///
/// Displays essential info (stat name, min/max values) in a horizontal strip,
/// with detailed explanations available via click-triggered popover.
/// Note: Threshold control has been moved to the on-canvas overlay.
#[component]
pub fn ColorLegend(
    metadata: ReadSignal<Option<StatisticMetadata>>,
    range: ReadSignal<Option<(f32, f32)>>,
    colormap: ReadSignal<ColormapType>,
    symmetric: ReadSignal<bool>,
) -> impl IntoView {
    // Popover open/close state
    let (is_popover_open, set_is_popover_open) = signal(false);

    // Reference for the info button (popover anchor)
    let info_button_ref = NodeRef::<leptos::html::Button>::new();
    // Reference for the popover container (for click-outside detection)
    let popover_ref = NodeRef::<leptos::html::Div>::new();

    // Handle click-outside to close popover
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        let is_open = is_popover_open.get();
        if is_open {
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |ev: web_sys::MouseEvent| {
                if let Some(target) = ev.target() {
                    if let Some(element) = target.dyn_ref::<web_sys::Element>() {
                        // Check if click is outside both the popover and the trigger button
                        let is_inside_popover = popover_ref.get()
                            .map(|p| p.contains(Some(element)))
                            .unwrap_or(false);
                        let is_inside_button = info_button_ref.get()
                            .map(|b| b.contains(Some(element)))
                            .unwrap_or(false);

                        if !is_inside_popover && !is_inside_button {
                            set_is_popover_open.set(false);
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);

            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    let _ = document.add_event_listener_with_callback(
                        "click",
                        closure.as_ref().unchecked_ref(),
                    );
                    // Store closure to be removed later (leak for simplicity in this context)
                    closure.forget();
                }
            }
        }
    });

    // Handle ESC key to close popover
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" && is_popover_open.get_untracked() {
            set_is_popover_open.set(false);
        }
    };

    // Suppress unused variable warning on non-wasm
    let _ = colormap;

    view! {
        <div class="color-legend-compact">
            {move || if let Some(meta) = metadata.get() {
                let name = meta.display_name.clone();
                let (min, max) = range.get().unwrap_or((0.0, 0.0));

                // Use user's symmetric preference to center range at 0
                let (display_min, display_max) = if symmetric.get() && min < 0.0 && max > 0.0 {
                    let abs_max = min.abs().max(max.abs());
                    (-abs_max, abs_max)
                } else {
                    (min, max)
                };

                let name_for_title = name.clone();
                view! {
                    // Compact strip with stat name + range + info toggle
                    <div class="legend-strip">
                        // Stat name (compact)
                        <span
                            class="legend-stat-name"
                            title=name_for_title
                        >
                            {name}
                        </span>

                        // Range display
                        <span class="legend-range">
                            {format!("[{:.1}, {:.1}]", display_min, display_max)}
                        </span>

                        // Info toggle button (popover trigger)
                        <div class="legend-popover-container">
                            <button
                                type="button"
                                node_ref=info_button_ref
                                class="legend-info-toggle"
                                on:click=move |_| set_is_popover_open.update(|v| *v = !*v)
                                on:keydown=on_keydown
                                aria-expanded=move || is_popover_open.get().to_string()
                                aria-haspopup="true"
                                aria-label="Legend details"
                                title="Legend details"
                            >
                                <svg
                                    class="w-3 h-3"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <circle cx="12" cy="12" r="10" stroke-width="2"/>
                                    <path stroke-linecap="round" stroke-width="2" d="M12 16v-4m0-4h.01"/>
                                </svg>
                            </button>

                            // Popover content
                            {move || is_popover_open.get().then(|| {
                                view! {
                                    <div
                                        node_ref=popover_ref
                                        class="legend-popover"
                                        role="dialog"
                                        aria-label="Legend details"
                                    >
                                        // Arrow pointing to trigger
                                        <div class="legend-popover-arrow"></div>

                                        // Close button
                                        <button
                                            type="button"
                                            class="legend-popover-close"
                                            on:click=move |_| set_is_popover_open.set(false)
                                            aria-label="Close"
                                        >
                                            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                            </svg>
                                        </button>

                                        // Content
                                        <div class="legend-popover-content">
                                            <div class="legend-detail-item">
                                                <span class="legend-detail-label">"Units:"</span>
                                                <span class="legend-detail-value">"statistic value"</span>
                                            </div>
                                            <div class="legend-detail-item">
                                                <span class="legend-detail-label">"Range:"</span>
                                                <span class="legend-detail-value">
                                                    {if symmetric.get() && display_min < 0.0 && display_max > 0.0 {
                                                        "centered at zero"
                                                    } else {
                                                        "data range"
                                                    }}
                                                </span>
                                            </div>
                                            {move || {
                                                let show_asym = !symmetric.get() && (min < 0.0 || max > 0.0);
                                                if show_asym {
                                                    view! {
                                                        <div class="legend-detail-item legend-detail-note">
                                                            "Asymmetric range reflects unbalanced data."
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {}.into_any()
                                                }
                                            }}
                                            <div class="legend-detail-item legend-detail-hint">
                                                <span class="text-[var(--color-accent-500)]">"Tip:"</span>
                                                <span>" Use slider in top-right of canvas to adjust threshold"</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div class="text-[var(--color-text-muted)] text-xs">"No statistic loaded"</div> }.into_any()
            }}
        </div>
    }
}
