//! Compact controls card for the Analysis Tools panel.
//!
//! Contains essential visualization controls in a dense, vertical layout.
//! Note: Threshold slider has been moved to the on-canvas overlay for quick access.

use leptos::prelude::*;

use crate::data::StatisticMetadata;
use crate::types::{Analysis, ColormapType, Statistic};

/// Compact controls card with essential visualization settings.
#[component]
pub fn ControlsCard(
    // Contrast/Volume
    volume_idx: ReadSignal<u32>,
    set_volume_idx: WriteSignal<u32>,
    n_volumes: Signal<u32>,
    stat_metadata: ReadSignal<Option<StatisticMetadata>>,
    // Colormap
    colormap: ReadSignal<ColormapType>,
    set_colormap: WriteSignal<ColormapType>,
    // Symmetric
    symmetric: ReadSignal<bool>,
    set_symmetric: WriteSignal<bool>,
    // Data options
    analysis: ReadSignal<Analysis>,
    set_analysis: WriteSignal<Analysis>,
    statistic: ReadSignal<Statistic>,
    set_statistic: WriteSignal<Statistic>,
    // Disabled state
    disabled: ReadSignal<bool>,
) -> impl IntoView {
    // Event handlers
    let on_volume_change = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        if let Ok(v) = value.parse::<u32>() {
            set_volume_idx.set(v);
        }
    };

    let on_colormap_change = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        let cm = match value.as_str() {
            "viridis" => ColormapType::Viridis,
            "hot" => ColormapType::Hot,
            "cividis" => ColormapType::Cividis,
            "plasma" => ColormapType::Plasma,
            _ => ColormapType::RdBu,
        };
        set_colormap.set(cm);
    };

    let on_symmetric_change = move |ev: leptos::ev::Event| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);
        set_symmetric.set(target.checked());
    };

    let on_analysis_change = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        let ana = match value.as_str() {
            "des2" => Analysis::Design2,
            "compare" => Analysis::Compare,
            _ => Analysis::Design1,
        };
        set_analysis.set(ana);
    };

    let on_stat_change = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        let stat = match value.as_str() {
            "beta" => Statistic::Beta,
            "conTlp" => Statistic::LogP,
            "sigma2" => Statistic::Sigma2,
            "Chi2" => Statistic::Chi2,
            "Chi2lp" => Statistic::Chi2lp,
            _ => Statistic::TStat,
        };
        set_statistic.set(stat);
    };

    view! {
        <div class="controls-card-body">
            // Analysis
            <div class="controls-row">
                <label class="controls-label">"Analysis"</label>
                <select
                    class="controls-select"
                    on:change=on_analysis_change
                    disabled=move || disabled.get()
                >
                    <option value="des1" selected=move || analysis.get() == Analysis::Design1>"Design 1"</option>
                    <option value="des2" selected=move || analysis.get() == Analysis::Design2>"Design 2"</option>
                    <option value="compare" selected=move || analysis.get() == Analysis::Compare>"Compare"</option>
                </select>
            </div>

            // Statistic
            <div class="controls-row">
                <label class="controls-label">"Statistic"</label>
                {move || {
                    let is_compare = analysis.get() == Analysis::Compare;
                    if is_compare {
                        view! {
                            <select
                                class="controls-select"
                                on:change=on_stat_change
                                disabled=move || disabled.get()
                            >
                                <option value="Chi2" selected=move || statistic.get() == Statistic::Chi2>"Chi²"</option>
                                <option value="Chi2lp" selected=move || statistic.get() == Statistic::Chi2lp>"Chi² -log₁₀(p)"</option>
                            </select>
                        }.into_any()
                    } else {
                        view! {
                            <select
                                class="controls-select"
                                on:change=on_stat_change
                                disabled=move || disabled.get()
                            >
                                <option value="conT" selected=move || statistic.get() == Statistic::TStat>"T-stat"</option>
                                <option value="beta" selected=move || statistic.get() == Statistic::Beta>"Beta"</option>
                                <option value="conTlp" selected=move || statistic.get() == Statistic::LogP>"Log p"</option>
                                <option value="sigma2" selected=move || statistic.get() == Statistic::Sigma2>"σ²"</option>
                            </select>
                        }.into_any()
                    }
                }}
            </div>

            // Contrast - only show if multiple volumes
            {move || {
                let num_vols = n_volumes.get();
                if num_vols > 1 {
                    let volume_labels = stat_metadata.get()
                        .map(|m| m.volumes.clone())
                        .unwrap_or_default();
                    view! {
                        <div class="controls-row">
                            <label class="controls-label">"Contrast"</label>
                            <select
                                class="controls-select"
                                on:change=on_volume_change
                                disabled=move || disabled.get()
                            >
                                {(0..num_vols).map(|v| {
                                    let label = volume_labels
                                        .iter()
                                        .find(|vl| vl.index == v)
                                        .map(|vl| vl.label.clone())
                                        .unwrap_or_else(|| format!("Contrast {}", v));
                                    view! {
                                        <option value=v.to_string() selected=move || volume_idx.get() == v>
                                            {label}
                                        </option>
                                    }
                                }).collect::<Vec<_>>()}
                            </select>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}

            // Colormap
            <div class="controls-row">
                <label class="controls-label">"Colormap"</label>
                <select
                    class="controls-select"
                    on:change=on_colormap_change
                    disabled=move || disabled.get()
                >
                    <option value="rdbu" selected=move || colormap.get() == ColormapType::RdBu>"RdBu"</option>
                    <option value="viridis" selected=move || colormap.get() == ColormapType::Viridis>"Viridis"</option>
                    <option value="cividis" selected=move || colormap.get() == ColormapType::Cividis>"Cividis"</option>
                    <option value="plasma" selected=move || colormap.get() == ColormapType::Plasma>"Plasma"</option>
                    <option value="hot" selected=move || colormap.get() == ColormapType::Hot>"Hot"</option>
                </select>
            </div>

            // Symmetric toggle - inline checkbox
            <div class="controls-row controls-row-inline">
                <label class="controls-checkbox-label">
                    <input
                        type="checkbox"
                        class="controls-checkbox"
                        on:change=on_symmetric_change
                        prop:checked=move || symmetric.get()
                        disabled=move || disabled.get()
                    />
                    "Symmetric"
                </label>
            </div>
        </div>
    }
}
