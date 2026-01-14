//! Camera preset toolbar component.

use leptos::callback::Callback;
use leptos::prelude::*;

use crate::types::{BrainViewPreset, Hemisphere};

/// Props for the CameraPresets component.
#[component]
pub fn CameraPresets(
    /// Currently selected hemisphere - filters which presets are shown.
    hemisphere: ReadSignal<Hemisphere>,
    /// Callback when a preset is selected.
    on_preset: Callback<BrainViewPreset>,
    /// Currently selected view preset (if any).
    current_view: ReadSignal<Option<BrainViewPreset>>,
    /// Whether the controls are disabled.
    #[prop(default = false.into())]
    disabled: Signal<bool>,
) -> impl IntoView {
    // Hemisphere-specific presets - filtered based on selected hemisphere
    let left_presets = [BrainViewPreset::LateralLeft, BrainViewPreset::MedialLeft];
    let right_presets = [BrainViewPreset::LateralRight, BrainViewPreset::MedialRight];

    // Standard views available for both hemispheres
    let standard_views = [
        BrainViewPreset::Dorsal,
        BrainViewPreset::Ventral,
        BrainViewPreset::Anterior,
        BrainViewPreset::Posterior,
    ];

    // Friendly labels for standard views
    let friendly_label = |preset: BrainViewPreset| -> &'static str {
        match preset {
            BrainViewPreset::LateralLeft | BrainViewPreset::LateralRight => "Lateral",
            BrainViewPreset::MedialLeft | BrainViewPreset::MedialRight => "Medial",
            BrainViewPreset::Dorsal => "Top",
            BrainViewPreset::Ventral => "Bottom",
            BrainViewPreset::Anterior => "Front",
            BrainViewPreset::Posterior => "Back",
        }
    };

    // Base and active button styles
    let base_class = "px-2.5 py-1 text-[10px] font-medium rounded-[var(--radius-sm)] border disabled:opacity-50 disabled:cursor-not-allowed transition-colors";
    let inactive_class = "bg-[var(--color-bg-subtle)] hover:bg-[var(--color-neutral-200)] border-[var(--color-border-default)] text-[var(--color-text-primary)]";
    let active_class = "bg-[var(--color-accent-emphasis)] border-[var(--color-accent-emphasis)] text-white";

    view! {
        <div class="space-y-1">
            // Hemisphere-specific presets (filtered based on selected hemisphere)
            <div class="flex items-center gap-1.5">
                <span class="text-[9px] text-[var(--color-text-muted)] w-6">
                    {move || match hemisphere.get() {
                        Hemisphere::Left => "Left",
                        Hemisphere::Right => "Right",
                    }}
                </span>
                {move || {
                    let presets = match hemisphere.get() {
                        Hemisphere::Left => &left_presets[..],
                        Hemisphere::Right => &right_presets[..],
                    };
                    presets.iter().map(|preset| {
                        let preset = *preset;
                        let on_preset = on_preset.clone();
                        let on_click = move |_| {
                            on_preset.run(preset);
                        };
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    let is_selected = current_view.get() == Some(preset);
                                    format!("{} {}", base_class, if is_selected { active_class } else { inactive_class })
                                }
                                on:click=on_click
                                disabled=move || disabled.get()
                                title=preset.name()
                            >
                                {friendly_label(preset)}
                            </button>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
            // Standard views row (always available)
            <div class="flex items-center gap-1.5">
                <span class="text-[9px] text-[var(--color-text-muted)] w-6">"View"</span>
                {standard_views.iter().map(|preset| {
                    let preset = *preset;
                    let on_preset = on_preset.clone();
                    let on_click = move |_| {
                        on_preset.run(preset);
                    };
                    view! {
                        <button
                            type="button"
                            class=move || {
                                let is_selected = current_view.get() == Some(preset);
                                format!("{} {}", base_class, if is_selected { active_class } else { inactive_class })
                            }
                            on:click=on_click
                            disabled=move || disabled.get()
                            title=preset.name()
                        >
                            {friendly_label(preset)}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
