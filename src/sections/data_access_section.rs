use crate::models::data_access::*;
use crate::ui::*;
use leptos::prelude::*;

/// Data Access section with collapsible items
///
/// Displays data access information with collapsible sections for better organization.
#[component]
pub fn DataAccessSection(model: DataAccessModel) -> impl IntoView {
    // Validate at render time (debug builds only)
    model.validate();

    view! {
        <section
            id="data-access"
            data-testid="section:data-access"
            class="mt-8 md:mt-12 lg:mt-16"
            aria-labelledby="data-access-title"
        >
            <div class="card">
                <SectionHeader
                    id="data-access-title"
                    title="Data Access"
                />

                <div class="mt-6 data-access-content">
                    {move || if !model.items.is_empty() {
                        // Render structured items
                        model.items.clone().into_iter().map(|item| {
                            view! {
                                <DataAccessItemView item=item />
                            }
                        }).collect_view().into_any()
                    } else if let Some(ref prose) = model.prose {
                        // Fallback to prose for backward compatibility
                        view! { <div inner_html=prose.to_string()></div> }.into_any()
                    } else {
                        view! { <p>"No data access information available."</p> }.into_any()
                    }}
                </div>
            </div>
        </section>
    }
}

/// Render a single Data Access item
#[component]
fn DataAccessItemView(item: DataAccessItem) -> impl IntoView {
    match item {
        DataAccessItem::Collapsible { title, content, open } => {
            view! {
                <details class="resource-details" open=open>
                    <summary class="resource-summary">
                        <div class="flex items-center justify-between p-3 hover:bg-subtle rounded-lg transition-colors">
                            <span class="panel-title">{title}</span>
                            <svg class="resource-chevron w-5 h-5 transition-transform text-accent-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M19 9l-7 7-7-7"></path>
                            </svg>
                        </div>
                    </summary>
                    <div class="resource-content-wrapper">
                        <div class="resource-content">
                            <div inner_html=content.to_string()></div>
                        </div>
                    </div>
                </details>
            }.into_any()
        }
        DataAccessItem::Prose { content } => {
            view! {
                <div inner_html=content.to_string()></div>
            }.into_any()
        }
    }
}
