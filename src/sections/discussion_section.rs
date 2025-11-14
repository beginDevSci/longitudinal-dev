//! Discussion section renderer (simplified).
//!
//! Renders a section with:
//! - Narrative paragraphs (at least 1 required)

use leptos::prelude::*;

use crate::models::discussion::DiscussionModel;

#[component]
pub fn DiscussionSection(model: DiscussionModel) -> impl IntoView {
    // Move data out before view!
    let paragraphs = model.paragraphs;
    let paragraph_count = paragraphs.len();

    // Determine if we should add subheadings (when >= 3 paragraphs)
    let use_subheadings = paragraph_count >= 3;

    // Render content based on paragraph count
    let content = if use_subheadings {
        // Group paragraphs with subheadings for better structure
        // First paragraph(s) -> "Key Findings"
        // Middle paragraph(s) -> "Implications"
        // Last paragraph -> "Conclusion" (if > 3 paragraphs)

        let split_point = if paragraph_count == 3 {
            // 3 paragraphs: 1 + 1 + 1
            (1, 2)
        } else if paragraph_count == 4 {
            // 4 paragraphs: 2 + 2
            (2, 2)
        } else {
            // 5+ paragraphs: ~40% + ~40% + remainder
            let first_group = (paragraph_count * 2 / 5).max(1);
            let second_group = (paragraph_count * 2 / 5).max(1);
            (first_group, second_group)
        };

        let mut paragraphs_iter = paragraphs.into_iter();

        // First group - Key Findings
        let first_group_paras: Vec<_> = paragraphs_iter.by_ref().take(split_point.0).collect();
        let first_nodes = first_group_paras
            .into_iter()
            .map(|p| view! { <p class="body-text">{p.to_string()}</p> })
            .collect_view();

        // Second group - Implications
        let second_group_paras: Vec<_> = paragraphs_iter.by_ref().take(split_point.1).collect();
        let second_nodes = second_group_paras
            .into_iter()
            .map(|p| view! { <p class="body-text">{p.to_string()}</p> })
            .collect_view();

        // Third group - Conclusion (remaining paragraphs)
        let third_group_paras: Vec<_> = paragraphs_iter.collect();
        let has_third_group = !third_group_paras.is_empty();
        let third_nodes = third_group_paras
            .into_iter()
            .map(|p| view! { <p class="body-text">{p.to_string()}</p> })
            .collect_view();

        view! {
            <div data-testid="discussion:narrative" class="space-y-8">
                <div>
                    <h3 class="panel-title mb-4">"Key Findings"</h3>
                    <div class="space-y-4">
                        {first_nodes}
                    </div>
                </div>
                <div>
                    <h3 class="panel-title mb-4">"Implications"</h3>
                    <div class="space-y-4">
                        {second_nodes}
                    </div>
                </div>
                {has_third_group.then(|| view! {
                    <div>
                        <h3 class="panel-title mb-4">"Conclusion"</h3>
                        <div class="space-y-4">
                            {third_nodes}
                        </div>
                    </div>
                })}
            </div>
        }
        .into_any()
    } else {
        // Simple paragraph rendering for < 3 paragraphs
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
