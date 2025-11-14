//! Additional Resources section renderer.
//!
//! Renders a section with resource intro cards (0-8 cards, optional).

use leptos::prelude::*;

use crate::models::additional_resources::ResourcesModel;
use crate::ui::ResourceIntroCard;

#[component]
pub fn AdditionalResourcesSection(model: ResourcesModel) -> impl IntoView {
    let has_cards = !model.items.is_empty();
    let resource_count = model.items.len();

    view! {
        <section
            id="additional-resources"
            data-testid="section:resources"
            class="mt-8 md:mt-12 lg:mt-16"
            aria-labelledby="resources-title"
        >
            <div class="card">
                {if has_cards {
                    view! {
                        <details class="resources-details">
                            <summary class="resources-summary">
                                <div class="resources-summary-content">
                                    <div class="flex items-center gap-3 flex-1">
                                        <div class="icon-circle bg-accent-100 dark:bg-accent-900/30 text-accent-600 dark:text-accent-400">
                                            <svg aria-hidden="true" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>
                                            </svg>
                                        </div>
                                        <h2 id="resources-title" class="section-title text-balance tracking-tight mb-0">
                                            "Additional Resources"
                                        </h2>
                                    </div>
                                    <div class="flex items-center gap-3">
                                        <span class="pill text-xs">{resource_count}</span>
                                        <svg aria-hidden="true" class="resources-chevron h-5 w-5 text-muted transition-transform" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                                            <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7"/>
                                        </svg>
                                    </div>
                                </div>
                            </summary>

                            <div class="resources-content-wrapper">
                                <div data-testid="resources:list" class="resource-list">
                                    {model.items.into_iter().map(|it| {
                                        view! {
                                            <ResourceIntroCard
                                                title=it.title
                                                body=it.body
                                                badge_upper=it.badge_upper
                                                url=it.url.map(|u| u.to_string())
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        </details>
                    }.into_any()
                } else {
                    view! {
                        <header class="max-w-prose mb-6">
                            <h2 id="resources-title" class="section-title text-balance tracking-tight">
                                "Additional Resources"
                            </h2>
                            <p class="text-muted text-sm mt-2">"No additional resources available."</p>
                        </header>
                    }.into_any()
                }}
            </div>
        </section>
    }
}
