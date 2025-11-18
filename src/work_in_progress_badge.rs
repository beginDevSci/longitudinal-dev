use leptos::prelude::*;

/// Collapsible work-in-progress badge shown on tutorial pages.
///
/// Displays a compact info badge that expands on click to show the full
/// message about work-in-progress examples and requesting feedback.
#[island]
pub fn WorkInProgressBadge() -> impl IntoView {
    let (is_expanded, set_is_expanded) = signal(false);

    let toggle_expanded = move |_| {
        set_is_expanded.update(|expanded| *expanded = !*expanded);
    };

    view! {
        <div class="wip-badge-container">
            <button
                class="wip-badge"
                on:click=toggle_expanded
                aria-expanded=move || is_expanded.get().to_string()
                aria-label="Work in progress notice"
            >
                <svg
                    class="wip-badge-icon"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                </svg>
                <span class="wip-badge-label">"Work in Progress"</span>
                <svg
                    class=move || format!(
                        "wip-badge-chevron {}",
                        if is_expanded.get() { "wip-badge-chevron--expanded" } else { "" }
                    )
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M19 9l-7 7-7-7"
                    />
                </svg>
            </button>

            {move || if is_expanded.get() {
                view! {
                    <div class="wip-badge-details">
                        <p class="wip-badge-text">
                            "Examples are a work in progress. Please exercise caution when using code examples, "
                            "as they may not be fully verified. If you spot gaps, errors, or have suggestions, "
                            "we'd love your feedback—use the "
                            <strong>"\"Suggest changes\""</strong>
                            " button to help us improve!"
                        </p>
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
}
