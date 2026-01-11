//! Error overlay component for the brain viewer.
//!
//! Displays an error message with an optional retry button when
//! loading or initialization fails.

use leptos::prelude::*;

/// Error overlay shown when loading fails.
///
/// Displays an error icon, message, and optional retry button.
#[component]
pub fn ErrorOverlay(
    /// The error message to display
    message: String,
    /// Whether the retry button should be shown
    can_retry: bool,
    /// Callback when retry is clicked
    on_retry: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="absolute inset-0 flex flex-col items-center justify-center bg-[var(--color-neutral-900)]/95 rounded-[var(--radius-panel)] p-6">
            // Error icon
            <div class="w-16 h-16 mb-4 text-red-500">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"></circle>
                    <line x1="12" y1="8" x2="12" y2="12"></line>
                    <line x1="12" y1="16" x2="12.01" y2="16"></line>
                </svg>
            </div>

            // Error title
            <div class="text-[var(--color-text-primary)] text-lg font-semibold mb-2">
                "Unable to load viewer"
            </div>

            // Error message
            <div
                class="text-[var(--color-text-secondary)] text-sm text-center max-w-sm mb-4 leading-relaxed"
                role="alert"
                aria-live="assertive"
            >
                {message}
            </div>

            // Retry button (if available)
            {if can_retry {
                let on_retry = on_retry.clone();
                view! {
                    <button
                        class="px-4 py-2 bg-[var(--color-accent-500)] hover:bg-[var(--color-accent-600)] text-white text-sm font-medium rounded-[var(--radius-md)] transition-colors flex items-center gap-2"
                        on:click=move |_| on_retry.run(())
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path>
                            <path d="M3 3v5h5"></path>
                        </svg>
                        "Try again"
                    </button>
                }.into_any()
            } else {
                view! {}.into_any()
            }}
        </div>
    }
}
