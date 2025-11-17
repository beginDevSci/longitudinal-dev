use leptos::prelude::*;

use crate::base_path;

/// Simple top navigation rendered on every page (SSR-only).
#[component]
pub fn TopNav() -> impl IntoView {
    let home_href = base_path::base_path();
    let tutorials_href = base_path::join("tutorials/");
    let posts_href = base_path::join("posts/");
    let about_href = base_path::join("about/");
    let writer_href = base_path::join("writer/");

    view! {
        <header class="sticky top-0 z-50 w-full border-b border-default bg-surface backdrop-blur-md shadow-sm">
            <div class="mx-auto flex h-16 max-w-7xl items-center justify-between px-6 sm:px-8 lg:px-10">
                <a href=home_href.clone() class="text-base font-bold tracking-tight text-primary hover:text-accent transition-colors">
                    "Longitudinal.dev"
                </a>

                <nav class="flex items-center gap-1 text-sm font-medium">
                    <a href=home_href class="rounded-lg px-4 py-2 text-secondary hover:text-primary hover:bg-subtle transition-all duration-200">"Home"</a>
                    <a href=tutorials_href class="rounded-lg px-4 py-2 text-secondary hover:text-primary hover:bg-subtle transition-all duration-200">"Tutorials"</a>
                    <a href=posts_href class="rounded-lg px-4 py-2 text-secondary hover:text-primary hover:bg-subtle transition-all duration-200">"Posts"</a>
                    <a href=about_href class="rounded-lg px-4 py-2 text-secondary hover:text-primary hover:bg-subtle transition-all duration-200">"About"</a>
                    <a href=writer_href class="rounded-lg px-4 py-2 text-secondary hover:text-primary hover:bg-subtle transition-all duration-200">"Writer"</a>
                </nav>
            </div>
        </header>
    }
}
