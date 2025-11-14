use crate::models::overview::{FeaturesPanelData, StatsPanelData};
use leptos::prelude::*;

/// Stats panel - 1-10 stat rows with auto-assigned labels (flexible count)
#[component]
pub fn StatsPanel(data: StatsPanelData) -> impl IntoView {
    // Use stat rows as provided (1-10 flexible count)
    let rows = data.rows;

    // Use custom title or default to "Analytical Approach"
    let title = data
        .title
        .map(|t| t.to_string())
        .unwrap_or_else(|| "Analytical Approach".to_string());

    // Render rows outside view! to avoid borrowing from rows
    let row_nodes = rows
        .into_iter()
        .map(|s| {
            let label = s.label.to_string();
            let value = s.value.to_string();
            view! {
                <div class="flex items-baseline justify-between" data-testid="stat-row">
                    <span class="stat-label">{label}</span>
                    <span class="stat-value">{value}</span>
                </div>
            }
        })
        .collect_view();

    view! {
        <div class="card" data-testid="stat-list-panel">
            <header class="mb-4">
                <h3 class="panel-title">
                    {title}
                </h3>
            </header>
            <div class="flex flex-col gap-3">
                {row_nodes}
            </div>
        </div>
    }
}

/// Features panel - grid of 0-5 feature cards with auto-assigned headings
/// Only renders if cards exist
#[component]
pub fn FeaturesPanel(data: FeaturesPanelData) -> impl IntoView {
    let cards = data.cards;

    // Only render if we have cards
    (!cards.is_empty()).then(|| {
        let card_nodes = cards
            .into_iter()
            .map(|f| {
                let heading = f.heading.to_string();
                let lines = f.lines;

                // Render line nodes outside inner view!
                let line_nodes = lines
                    .into_iter()
                    .map(|line| {
                        let text = line.to_string();
                        view! { <div class="supporting-text">{text}</div> }
                    })
                    .collect_view();

                view! {
                    <div class="card card-hover" data-testid="feature-card">
                        <div class="panel-title mb-1" data-testid="feature-heading">{heading}</div>
                        {line_nodes}
                    </div>
                }
            })
            .collect_view();

        view! {
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6 py-8" data-testid="features-panel">
                {card_nodes}
            </div>
        }
    })
}
