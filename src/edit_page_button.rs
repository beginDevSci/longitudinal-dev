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

    // Handler to open the editor modal via custom event
    let on_click = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = leptos::web_sys::window() {
                if let Ok(event) = leptos::web_sys::CustomEvent::new("open-editor-modal") {
                    let _ = window.dispatch_event(&event);
                }
            }
        }
    };

    view! {
        <>
            <button
                class="aside-action w-full text-left"
                type="button"
                on:click=on_click
                title="Suggest changes to this page"
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
                <span>"Suggest changes"</span>
            </button>
            <a
                href="https://github.com/beginDevSci/longitudinal-dev/discussions"
                target="_blank"
                rel="noopener noreferrer"
                class="aside-action w-full text-left"
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
                        d="M8 10h8M8 14h5M21 12c0 4.418-4.03 8-9 8-1.084 0-2.12-.172-3.082-.488L3 20l1.488-5.918C4.172 13.12 4 12.084 4 11c0-4.418 4.03-8 9-8s9 3.582 9 9z"
                    />
                </svg>
                <span>"Join the discussion"</span>
            </a>
            <div class="aside-divider"/>
            <a
                href={github_edit_url}
                target="_blank"
                rel="noopener noreferrer"
                class="aside-action w-full text-left"
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
                        d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                    />
                </svg>
                <span>"Edit on GitHub"</span>
            </a>
        </>
    }
}
