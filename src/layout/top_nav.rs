use leptos::prelude::*;

use crate::base_path;
use crate::ThemeToggle;

/// Simple top navigation rendered on every page (SSR-only).
///
/// Uses theme-aware semantic tokens that respond to the theme picker.
#[component]
pub fn TopNav() -> impl IntoView {
    let home_href = base_path::base_path();
    let tutorials_href = base_path::join("tutorials/");
    let about_href = base_path::join("about/");

    view! {
        <header class="sticky top-0 z-50 w-full border-b border-default bg-surface backdrop-blur-md shadow-sm">
            <div class="mx-auto flex h-16 max-w-7xl items-center justify-between px-6 sm:px-8 lg:px-10">
                <a href=home_href.clone() class="text-base font-bold tracking-tight text-primary hover:text-accent transition-colors">
                    "Longitudinal.dev"
                </a>

                <nav class="flex items-center gap-1 text-sm font-medium">
                    <a href=home_href class="rounded-lg px-4 py-2 text-secondary hover:text-primary hover:bg-subtle transition-all duration-200">"Home"</a>
                    <a href=tutorials_href class="rounded-lg px-4 py-2 text-secondary hover:text-primary hover:bg-subtle transition-all duration-200">"Tutorials"</a>
                    <a href=about_href class="rounded-lg px-4 py-2 text-secondary hover:text-primary hover:bg-subtle transition-all duration-200">"About"</a>
                    <span class="rounded-lg px-4 py-2 text-tertiary cursor-not-allowed opacity-50" title="Coming soon">"Writer"</span>
                    <ThemeToggle/>
                </nav>
            </div>
        </header>
    }
}
