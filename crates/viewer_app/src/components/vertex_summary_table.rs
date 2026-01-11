//! Accessible vertex summary table for screen reader users.
//!
//! This component displays a tabular summary of the top vertices by absolute
//! statistic value, providing an accessible alternative to the 3D visualization
//! for users who cannot see the rendered brain surface.

use leptos::prelude::*;

use crate::data::StatisticData;

/// Displays an accessible summary table of top vertices by statistic value.
///
/// Shows all 10 top vertices in a compressed layout (~18px rows).
#[component]
pub fn VertexSummaryTable(
    /// The statistics data to summarize (None during loading)
    statistics: Signal<Option<StatisticData>>,
    /// The currently selected volume index
    volume_idx: ReadSignal<u32>,
) -> impl IntoView {
    // Compute top 10 vertices by absolute value
    let top_vertices = move || {
        let stats = statistics.get()?;
        let vol = volume_idx.get() as usize;
        let values = stats.volume_slice(vol)?;

        // Collect (index, value) pairs, excluding NaN
        let mut indexed_values: Vec<(usize, f32)> = values
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_finite())
            .map(|(i, v)| (i, *v))
            .collect();

        // Sort by absolute value, descending
        indexed_values.sort_by(|a, b| {
            b.1.abs()
                .partial_cmp(&a.1.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top 10
        Some(indexed_values.into_iter().take(10).collect::<Vec<_>>())
    };

    view! {
        <div class="analysis-tools-section-body vertex-table-wrapper" role="region" aria-label="Top vertices by value">
            <p class="sr-only">
                "This table shows the 10 vertices with the highest absolute statistic values."
            </p>
            {move || match top_vertices() {
                Some(vertices) if !vertices.is_empty() => {
                    view! {
                        <table
                            class="vertex-table-compact w-full border-collapse"
                            aria-label="Top 10 vertices by value"
                        >
                            <thead>
                                <tr class="vertex-table-header">
                                    <th scope="col" class="text-left">"#"</th>
                                    <th scope="col" class="text-left">"Vertex"</th>
                                    <th scope="col" class="text-right">"Value"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {vertices.into_iter().enumerate().map(|(rank, (idx, value))| {
                                    view! {
                                        <tr class="vertex-table-row">
                                            <td class="text-[var(--color-text-muted)]">{rank + 1}</td>
                                            <td class="font-medium">{idx}</td>
                                            <td class="text-right font-mono tabular-nums">
                                                {format!("{:.2}", value)}
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    }.into_any()
                }
                Some(_) => {
                    view! {
                        <p class="analysis-tools-section-intro">"No valid vertices found."</p>
                    }.into_any()
                }
                None => {
                    view! {
                        <p class="analysis-tools-section-intro">"Loading statistics..."</p>
                    }.into_any()
                }
            }}
        </div>
    }
}
