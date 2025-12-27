//! Discussion section renderer.
//!
//! Renders a section with:
//! - Structured items with collapsible subsections (preferred)
//! - Fallback narrative paragraphs (for backward compatibility)

use leptos::prelude::*;

use crate::models::discussion::DiscussionModel;

#[component]
pub fn DiscussionSection(model: DiscussionModel) -> impl IntoView {
    // Move data out before view!
    let items = model.items;
    let paragraphs = model.paragraphs;

    // Prefer structured items if available, otherwise fall back to paragraphs
    let content = if !items.is_empty() {
        // Render structured items with collapsible sections
        let item_nodes = items
            .into_iter()
            .map(|item| {
                view! {
                    <div class="discussion-item">
                        <h3 class="panel-title mb-4">{item.title.to_string()}</h3>
                        <div class="prose prose-sm" inner_html={item.content.to_string()} />
                    </div>
                }
            })
            .collect_view();

        view! {
            <div data-testid="discussion:items" class="space-y-8">
                {item_nodes}
            </div>
        }
        .into_any()
    } else {
        // Fallback: simple paragraph rendering
        let para_nodes = paragraphs
            .into_iter()
            .map(|p| {
                let text = p.to_string();
                view! { <p class="body-text">{text}</p> }
            })
            .collect_view();

        view! {
            <div data-testid="discussion:narrative" class="space-y-6">
                {para_nodes}
            </div>
        }
        .into_any()
    };

    view! {
        <section
            id="discussion"
            data-testid="section:discussion"
            class="mt-8 md:mt-12 lg:mt-16"
            aria-labelledby="discussion-title"
        >
            <div class="card">
                <header class="max-w-prose mb-6">
                    <h2 id="discussion-title" class="section-title text-balance tracking-tight">
                        "Discussion"
                    </h2>
                </header>

                {content}
            </div>
        </section>
    }
}
