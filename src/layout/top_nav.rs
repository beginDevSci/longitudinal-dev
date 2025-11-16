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
        <header class="sticky top-0 z-50 w-full border-b border-slate-800/70 bg-slate-950/80 backdrop-blur">
            <div class="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
                <a href=home_href.clone() class="text-sm font-semibold tracking-tight text-white hover:text-teal-200 transition-colors">
                    "Longitudinal.dev"
                </a>

                <nav class="flex items-center gap-3 text-sm font-medium text-slate-200">
                    <a href=home_href class="rounded-lg px-3 py-1.5 hover:bg-white/5 transition-colors">"Home"</a>
                    <a href=tutorials_href class="rounded-lg px-3 py-1.5 hover:bg-white/5 transition-colors">"Tutorials"</a>
                    <a href=posts_href class="rounded-lg px-3 py-1.5 hover:bg-white/5 transition-colors">"Posts"</a>
                    <a href=about_href class="rounded-lg px-3 py-1.5 hover:bg-white/5 transition-colors">"About"</a>
                    <a href=writer_href class="rounded-lg px-3 py-1.5 text-teal-200 hover:bg-teal-500/10 transition-colors">"Writer"</a>
                </nav>
            </div>
        </header>
    }
}
