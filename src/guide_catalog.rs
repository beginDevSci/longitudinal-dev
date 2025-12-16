//! Guide catalog component for the /guides index page.
//!
//! Displays method guides in a card layout with filtering by category.

use crate::base_path;
use crate::models::guide::GuideCatalogItem;
use leptos::prelude::*;

/// A single guide card in the catalog.
#[component]
pub fn GuideCard(guide: GuideCatalogItem) -> impl IntoView {
    let href = base_path::join(&format!("guides/{}/", guide.slug));

    view! {
        <a
            href={href}
            class="group block rounded-xl transition-all duration-200 hover:scale-102 hover:shadow-xl bg-elevated border border-stroke p-6"
        >
            <div class="flex items-center gap-2 mb-2">
                <span class="text-xs px-2 py-0.5 rounded-full bg-emerald-100 text-emerald-700 border border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-400 dark:border-emerald-800">
                    "Method Guide"
                </span>
                <span class="text-xs px-2 py-0.5 rounded-full bg-slate-100 text-slate-600 border border-slate-200 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700">
                    {guide.category.clone()}
                </span>
            </div>

            <h3 class="text-lg font-semibold group-hover:underline group-hover:text-accent transition-colors duration-200 text-primary">
                {guide.title.clone()}
            </h3>

            <p class="mt-2 text-sm text-secondary line-clamp-3">
                {guide.description.clone()}
            </p>

            // Tags
            {if !guide.tags.is_empty() {
                Some(view! {
                    <div class="mt-4 flex flex-wrap gap-2">
                        {guide.tags.iter().map(|tag| {
                            view! {
                                <span class="px-2 py-1 rounded-full text-xs font-medium bg-accent/5 text-accent border border-accent/20">
                                    {tag.clone()}
                                </span>
                            }
                        }).collect_view()}
                    </div>
                })
            } else {
                None
            }}

            // R packages
            {if !guide.r_packages.is_empty() {
                Some(view! {
                    <div class="mt-3 flex flex-wrap gap-1.5">
                        <span class="text-xs text-muted">"R packages:"</span>
                        {guide.r_packages.iter().map(|pkg| {
                            view! {
                                <span class="text-xs px-1.5 py-0.5 rounded bg-blue-50 text-blue-600 dark:bg-blue-900/20 dark:text-blue-400">
                                    {pkg.clone()}
                                </span>
                            }
                        }).collect_view()}
                    </div>
                })
            } else {
                None
            }}
        </a>
    }
}

/// Guide catalog grid component.
#[component]
pub fn GuideCatalog(guides: Vec<GuideCatalogItem>) -> impl IntoView {
    // Group guides by category
    let mut categories: std::collections::HashMap<String, Vec<GuideCatalogItem>> =
        std::collections::HashMap::new();

    for guide in guides.iter() {
        categories
            .entry(guide.category.clone())
            .or_default()
            .push(guide.clone());
    }

    // Sort categories alphabetically
    let mut category_list: Vec<_> = categories.into_iter().collect();
    category_list.sort_by(|a, b| a.0.cmp(&b.0));

    view! {
        <div class="space-y-10">
            {if guides.is_empty() {
                view! {
                    <div class="text-center py-12">
                        <p class="text-secondary">"No method guides available yet. Check back soon!"</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    // Show all guides in a grid
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                        {guides.into_iter().map(|guide| {
                            view! {
                                <GuideCard guide />
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
