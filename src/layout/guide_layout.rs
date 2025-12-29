//! Guide layout component for individual method guide pages.
//!
//! Renders a guide with:
//! - Title and metadata header
//! - Navigation tabs for hub/tutorial/reference switching
//! - Left sidebar with hierarchical outline navigation
//! - Main content area with rendered HTML
//! - Interactive behaviors (copy buttons, anchor navigation)

use crate::base_path;
use crate::layout::{GuideNavTabs, GuideSidebarNav, GuideTabInfo};
use crate::models::guide::Guide;
use crate::GuideInteractivity;
use leptos::prelude::*;

/// Feature flag: set to false to hide the horizontal TOC once sidebar is verified
const SHOW_HORIZONTAL_TOC: bool = false;

/// Renders a single guide page with sidebar navigation.
///
/// Layout:
/// - Full-width header with title, description, and metadata
/// - Navigation tabs for switching between hub/tutorial/reference
/// - 3-column grid: sidebar | content | (empty right column)
/// - Full-width footer
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
    let outline = guide.outline.clone();

    // Build tab navigation info if this guide is part of a method group
    let tab_info = GuideTabInfo::from_guide(
        &guide.slug,
        guide.guide_type.as_ref().map(|s| s.as_ref()),
        guide.parent_method.as_ref().map(|s| s.as_ref()),
    );

    view! {
        <div class="min-h-screen bg-surface">
            // Header (full-width)
            <header class="border-b border-default bg-subtle">
                <div class="guide-header-container">
                    // Breadcrumb
                    <nav class="mb-3 text-sm">
                        <a href={base_path::join("guides/")} class="text-secondary hover:text-accent transition-colors">
                            "Method Guides"
                        </a>
                        <span class="mx-2 text-muted">"/"</span>
                        <span class="text-primary">{category.clone()}</span>
                    </nav>

                    // Title
                    <h1 class="text-3xl md:text-4xl font-bold text-primary mb-3">
                        {title}
                    </h1>

                    // Description
                    <p class="text-lg text-secondary mb-4">
                        {description}
                    </p>

                    // Metadata pills
                    <div class="flex flex-wrap gap-2">
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

            // Navigation tabs for hub/tutorial/reference
            {tab_info.map(|info| view! { <GuideNavTabs info=info /> })}

            // Horizontal TOC (feature-flagged, kept for rollback)
            {if SHOW_HORIZONTAL_TOC {
                view! {
                    <nav class="guide-toc border-b border-default bg-surface/80 backdrop-blur-sm sticky top-0 z-40">
                        <div class="max-w-4xl mx-auto px-6">
                            <ul class="flex gap-1 overflow-x-auto py-3 text-sm font-medium scrollbar-hide">
                                <li>
                                    <a href="#overview" class="guide-toc-link px-3 py-1.5 rounded-md text-secondary hover:text-accent hover:bg-accent/10 transition-colors whitespace-nowrap">
                                        "Overview"
                                    </a>
                                </li>
                                <li class="text-muted flex items-center">"•"</li>
                                <li>
                                    <a href="#conceptual-foundations" class="guide-toc-link px-3 py-1.5 rounded-md text-secondary hover:text-accent hover:bg-accent/10 transition-colors whitespace-nowrap">
                                        "Concepts"
                                    </a>
                                </li>
                                <li class="text-muted flex items-center">"•"</li>
                                <li>
                                    <a href="#model-specification-fit" class="guide-toc-link px-3 py-1.5 rounded-md text-secondary hover:text-accent hover:bg-accent/10 transition-colors whitespace-nowrap">
                                        "Spec & Fit"
                                    </a>
                                </li>
                                <li class="text-muted flex items-center">"•"</li>
                                <li>
                                    <a href="#interpretation" class="guide-toc-link px-3 py-1.5 rounded-md text-secondary hover:text-accent hover:bg-accent/10 transition-colors whitespace-nowrap">
                                        "Interpretation"
                                    </a>
                                </li>
                                <li class="text-muted flex items-center">"•"</li>
                                <li>
                                    <a href="#worked-example" class="guide-toc-link px-3 py-1.5 rounded-md text-secondary hover:text-accent hover:bg-accent/10 transition-colors whitespace-nowrap">
                                        "Example"
                                    </a>
                                </li>
                                <li class="text-muted flex items-center">"•"</li>
                                <li>
                                    <a href="#reference-resources" class="guide-toc-link px-3 py-1.5 rounded-md text-secondary hover:text-accent hover:bg-accent/10 transition-colors whitespace-nowrap">
                                        "Reference"
                                    </a>
                                </li>
                            </ul>
                        </div>
                    </nav>
                }.into_any()
            } else {
                ().into_any()
            }}

            // 3-column layout: sidebar | content | empty
            <div class="guide-layout-container">
                // Left sidebar (sticky)
                <aside class="guide-sidebar">
                    <div class="guide-sidebar-sticky">
                        <GuideSidebarNav outline=outline />
                    </div>
                </aside>

                // Main content
                <main class="guide-main">
                    <article
                        class="guide-content prose prose-slate dark:prose-invert max-w-none
                               prose-headings:text-primary prose-p:text-secondary
                               prose-a:text-accent prose-a:no-underline hover:prose-a:underline
                               prose-code:text-accent prose-code:bg-subtle prose-code:px-1 prose-code:py-0.5 prose-code:rounded
                               prose-pre:bg-elevated prose-pre:border prose-pre:border-stroke
                               prose-table:border-collapse prose-th:border prose-th:border-stroke prose-th:bg-subtle prose-th:px-4 prose-th:py-2
                               prose-td:border prose-td:border-stroke prose-td:px-4 prose-td:py-2"
                        inner_html=html_content
                    />
                </main>

                // Right column (empty for now, could be used for annotations)
                <aside class="guide-right-rail">
                    // Reserved for future use
                </aside>
            </div>

            // Footer navigation (full-width)
            <footer class="border-t border-default bg-subtle">
                <div class="guide-header-container py-6">
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
