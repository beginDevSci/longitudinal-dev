use leptos::prelude::*;

/// Fixed outer structure for a section; inner content via `Children`.
///
/// NOTE: The section's <h2> title is now rendered by the SectionHeader component
/// within each section component (e.g., OverviewSection). This wrapper only provides
/// the <section> container with proper a11y attributes.
#[component]
pub fn SectionContainer(
    /// ID of the h2 element (without -title suffix) for aria-labelledby
    label_id: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="section grid gap-3 border-subtle bg-subtle rounded-xl p-6" aria-labelledby=label_id>
            <div class="section__content grid gap-2">{children()}</div>
        </section>
    }
}
