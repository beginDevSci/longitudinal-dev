use leptos::prelude::*;

/// Work-in-progress badge with tooltip shown on tutorial pages.
///
/// Displays a compact info badge with tooltip on hover explaining
/// the WIP status and how to provide feedback.
#[component]
pub fn WorkInProgressBadge() -> impl IntoView {
    view! {
        <div class="wip-badge-container">
            <span
                class="wip-badge"
                role="status"
                aria-label="Work in progress - hover for details"
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

                // Tooltip
                <span class="wip-badge-tooltip">
                    "Examples are a work in progress. Please exercise caution when using code examples, as they may not be fully verified. If you spot gaps, errors, or have suggestions, we'd love your feedback—use the \"Suggest changes\" button to help us improve!"
                </span>
            </span>
        </div>
    }
}
