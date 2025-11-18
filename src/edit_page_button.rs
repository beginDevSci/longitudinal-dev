use leptos::prelude::*;

/// Button island that triggers the editor modal via custom event
///
/// This is a separate island from the modal so it can be rendered
/// in the aside while the modal appears at the page level.
///
/// Props:
/// - slug: Tutorial slug for the GitHub edit link
#[island]
pub fn EditPageButton(
    slug: String,
) -> impl IntoView {
    let github_edit_url = format!("https://github.com/swhawes/leptos-test/edit/main/content/tutorials/{}.md", slug);

    view! {
        <button
            class="aside-action w-full text-left"
            type="button"
            disabled
            aria-disabled="true"
            title="Coming soon"
        >
            <svg
                class="aside-action-icon"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.8"
                    d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5M15.5 5.5a2.121 2.121 0 113 3L12 15l-3 1 1-3 5.5-5.5z"
                />
            </svg>
            <span>"Edit this page"</span>
            <span class="text-[0.65rem] text-muted">"(coming soon)"</span>
        </button>
    }
}
