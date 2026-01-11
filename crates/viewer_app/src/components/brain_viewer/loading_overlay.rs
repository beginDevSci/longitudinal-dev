//! Loading overlay component for the brain viewer.
//!
//! Displays a brain silhouette with shimmer animation, loading stage text,
//! and a progress bar during initial data loading.

use leptos::prelude::*;

/// Loading overlay shown during initial data loading.
///
/// Displays a brain icon with shimmer effect, current loading stage,
/// and a progress bar.
#[component]
pub fn LoadingOverlay(
    /// Current loading progress (0-100)
    progress: Signal<u8>,
    /// Current loading stage description
    stage: Signal<String>,
) -> impl IntoView {
    view! {
        <div class="absolute inset-0 flex flex-col items-center justify-center bg-[var(--color-neutral-900)]/90 rounded-[var(--radius-panel)] backdrop-blur-sm">
            // Brain silhouette placeholder with shimmer
            <div class="relative w-32 h-32 mb-6">
                // Brain icon SVG
                <svg
                    class="w-full h-full text-[var(--color-neutral-700)] animate-pulse"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path d="M12 2C9.5 2 7.5 3.5 7 5.5C5.5 5.5 4 7 4 8.5C4 10 5 11.5 6.5 12C6 13 6 14 6.5 15C5.5 15.5 5 17 5.5 18C6 19.5 7.5 20 9 19.5C9.5 21 11 22 12.5 22C14 22 15.5 21 16 19.5C17.5 20 19 19.5 19.5 18C20 17 19.5 15.5 18.5 15C19 14 19 13 18.5 12C20 11.5 21 10 21 8.5C21 7 19.5 5.5 18 5.5C17.5 3.5 15.5 2 13 2H12Z"/>
                </svg>
                // Shimmer overlay
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-[var(--color-neutral-600)]/30 to-transparent loading-shimmer"></div>
            </div>

            // Loading stage text
            <div class="text-[var(--color-text-primary)] text-sm font-medium mb-3">
                {move || stage.get()}
            </div>

            // Progress bar
            <div class="w-48 h-1.5 bg-[var(--color-neutral-700)] rounded-full overflow-hidden">
                <div
                    class="h-full bg-[var(--color-accent-500)] transition-all duration-300 ease-out"
                    style:width=move || format!("{}%", progress.get())
                ></div>
            </div>

            // Progress percentage
            <div class="text-[var(--color-text-muted)] text-xs mt-2">
                {move || format!("{}%", progress.get())}
            </div>
        </div>
    }
}
