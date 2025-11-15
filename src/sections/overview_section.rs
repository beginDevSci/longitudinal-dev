use crate::models::post::*;
use crate::ui::*;
use leptos::prelude::*;

/// Overview section with flexible layout
///
/// Subcontainers (all optional except summary):
/// 1. Section header (h2) - always shown
/// 2. Summary text - REQUIRED (at least 1 paragraph)
/// 3. Stats panel (right sidebar) - optional (requires {.stats} marker, 0-10 stats)
/// 4. Features panel - optional (requires {.features} marker, 0-5 features)
///
/// Layout behavior:
/// - With stats: Summary on left (2/3 width), stats on right (1/3 width)
/// - Without stats: Summary full-width
#[component]
pub fn OverviewSection(model: OverviewModel) -> impl IntoView {
    // Move data out before view!
    let paragraphs = model.summary_paragraphs;
    let stats_panel = model.stats_panel;
    let features_panel = model.features_panel;

    // Check if panels have content
    let has_stats = stats_panel.as_ref().is_some_and(|p| !p.rows.is_empty());
    let has_features = features_panel.as_ref().is_some_and(|p| !p.cards.is_empty());

    // Render paragraph nodes outside closure
    let para_nodes = paragraphs
        .into_iter()
        .map(|p| {
            let text = p.to_string();
            view! { <p class="body-text max-w-prose md:max-w-3xl mx-auto">{text}</p> }
        })
        .collect_view();

    view! {
        <section
            id="overview"
            data-testid="overview-structure"
            class="mt-8 md:mt-12 lg:mt-16"
            aria-labelledby="overview-title"
        >
            <div class="card">
                // 1) Header row (h2 only)
                <SectionHeader title="Overview" id="overview-title"/>

                // 2) Conditional layout: Split if stats exist, full-width if not
                {if has_stats {
                    let left = view! {
                        <div>
                            <Stack gap=Gap::G6>
                                {para_nodes}
                            </Stack>
                        </div>
                    };
                    let right = view! { <StatsPanel data=stats_panel.unwrap()/> };
                    view! { <Split left right reverse_on_md=false ratio=Ratio::TwoOne/> }.into_any()
                } else {
                    view! {
                        <div>
                            <Stack gap=Gap::G6>
                                {para_nodes}
                            </Stack>
                        </div>
                    }.into_any()
                }}

                // 3) Conditional features panel
                {has_features.then(|| view! {
                    <div class="mt-6">
                        <FeaturesPanel data=features_panel.unwrap()/>
                    </div>
                })}
            </div>
        </section>
    }
}
