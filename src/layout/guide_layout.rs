//! Guide layout component for individual method guide pages.
//!
//! Renders a guide with:
//! - Title and metadata header
//! - Main content area with rendered HTML
//! - Interactive behaviors (copy buttons, anchor navigation)
//! - Optional table of contents (future)

use crate::base_path;
use crate::models::guide::Guide;
use crate::GuideInteractivity;
use leptos::prelude::*;

/// Renders a single guide page.
///
/// For Phase 1, this is a simple layout that renders the guide HTML.
/// Future phases will add:
/// - Collapsible modules for Worked Example and Reference sections
/// - Table of contents with scrollspy
/// - Code copy buttons
/// - Math rendering
#[component]
pub fn GuideLayout(guide: Guide) -> impl IntoView {
    let title = guide.title.to_string();
    let description = guide.description.to_string();
    let category = guide.category.to_string();
    let tags = guide.tags.iter().map(|t| t.to_string()).collect::<Vec<_>>();
    let r_packages = guide
        .r_packages
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>();
    let html_content = guide.html_content.to_string();

    view! {
        <div class="min-h-screen bg-surface">
            // Header
            <header class="border-b border-default bg-subtle">
                <div class="max-w-4xl mx-auto px-6 py-8 lg:py-12">
                    // Breadcrumb
                    <nav class="mb-4 text-sm">
                        <a href={base_path::join("guides/")} class="text-secondary hover:text-accent transition-colors">
                            "Method Guides"
                        </a>
                        <span class="mx-2 text-muted">"/"</span>
                        <span class="text-primary">{category.clone()}</span>
                    </nav>

                    // Title
                    <h1 class="text-3xl md:text-4xl font-bold text-primary mb-4">
                        {title}
                    </h1>

                    // Description
                    <p class="text-lg text-secondary mb-6">
                        {description}
                    </p>

                    // Metadata pills
                    <div class="flex flex-wrap gap-2">
                        <span class="px-3 py-1 rounded-full text-sm font-medium bg-emerald-100 text-emerald-700 border border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-400 dark:border-emerald-800">
                            "Method Guide"
                        </span>

                        {tags.iter().map(|tag| {
                            view! {
                                <span class="px-3 py-1 rounded-full text-sm font-medium bg-accent/10 text-accent border border-accent/20">
                                    {tag.clone()}
                                </span>
                            }
                        }).collect_view()}

                        {r_packages.iter().map(|pkg| {
                            view! {
                                <span class="px-3 py-1 rounded-full text-sm font-medium bg-blue-50 text-blue-600 dark:bg-blue-900/20 dark:text-blue-400 border border-blue-200 dark:border-blue-800">
                                    {pkg.clone()}
                                </span>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </header>

            // Main content
            <main class="max-w-4xl mx-auto px-6 py-8 lg:py-12">
                <article
                    class="prose prose-slate dark:prose-invert max-w-none
                           prose-headings:text-primary prose-p:text-secondary
                           prose-a:text-accent prose-a:no-underline hover:prose-a:underline
                           prose-code:text-accent prose-code:bg-subtle prose-code:px-1 prose-code:py-0.5 prose-code:rounded
                           prose-pre:bg-elevated prose-pre:border prose-pre:border-stroke
                           prose-table:border-collapse prose-th:border prose-th:border-stroke prose-th:bg-subtle prose-th:px-4 prose-th:py-2
                           prose-td:border prose-td:border-stroke prose-td:px-4 prose-td:py-2"
                    inner_html=html_content
                />
            </main>

            // Footer navigation
            <footer class="border-t border-default bg-subtle">
                <div class="max-w-4xl mx-auto px-6 py-6">
                    <a
                        href={base_path::join("guides/")}
                        class="inline-flex items-center gap-2 text-secondary hover:text-accent transition-colors"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"/>
                        </svg>
                        "Back to Method Guides"
                    </a>
                </div>
            </footer>

            // Island for interactive behaviors (copy buttons, anchor navigation)
            <GuideInteractivity />
        </div>
    }
}
