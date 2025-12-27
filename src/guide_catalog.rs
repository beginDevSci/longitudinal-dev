//! Guide catalog component for the /guides index page.
//!
//! Displays method guides in a card layout with filtering by category.
//! Guides are grouped by method (hub + tutorial + reference).

use crate::base_path;
use crate::models::guide::{GuideCatalogItem, MethodGroup};
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

/// A method card showing hub, tutorial, and reference links.
#[component]
pub fn MethodCard(group: MethodGroup) -> impl IntoView {
    let hub_href = base_path::join(&format!("guides/{}/", group.hub.slug));
    let tutorial_href = group
        .tutorial
        .as_ref()
        .map(|t| base_path::join(&format!("guides/{}/", t.slug)));
    let reference_href = group
        .reference
        .as_ref()
        .map(|r| base_path::join(&format!("guides/{}/", r.slug)));

    // Category color mapping
    let category_color = match group.category.as_str() {
        "growth-models" => "bg-emerald-100 text-emerald-700 border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-400 dark:border-emerald-800",
        "mixed-models" => "bg-blue-100 text-blue-700 border-blue-200 dark:bg-blue-900/30 dark:text-blue-400 dark:border-blue-800",
        "survival" => "bg-orange-100 text-orange-700 border-orange-200 dark:bg-orange-900/30 dark:text-orange-400 dark:border-orange-800",
        "latent-variable" => "bg-purple-100 text-purple-700 border-purple-200 dark:bg-purple-900/30 dark:text-purple-400 dark:border-purple-800",
        _ => "bg-slate-100 text-slate-600 border-slate-200 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700",
    };

    // Format category name for display
    let category_display = group
        .category
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    view! {
        <div class="rounded-xl bg-elevated border border-stroke p-6 hover:shadow-lg transition-shadow duration-200">
            // Category badge
            <div class="flex items-center gap-2 mb-3">
                <span class={format!("text-xs px-2 py-0.5 rounded-full border {}", category_color)}>
                    {category_display}
                </span>
            </div>

            // Title (links to hub)
            <a href={hub_href.clone()} class="group">
                <h3 class="text-lg font-semibold group-hover:underline group-hover:text-accent transition-colors duration-200 text-primary">
                    {group.hub.title.clone()}
                </h3>
            </a>

            // Description
            <p class="mt-2 text-sm text-secondary line-clamp-3">
                {group.hub.description.clone()}
            </p>

            // Navigation links
            <div class="mt-4 flex flex-wrap gap-3 text-sm">
                <a
                    href={hub_href}
                    class="text-accent hover:underline font-medium"
                >
                    "→ Overview"
                </a>
                {tutorial_href.map(|href| view! {
                    <a
                        href={href}
                        class="text-accent hover:underline font-medium"
                    >
                        "→ Tutorial"
                    </a>
                })}
                {reference_href.map(|href| view! {
                    <a
                        href={href}
                        class="text-accent hover:underline font-medium"
                    >
                        "→ Reference"
                    </a>
                })}
            </div>

            // R packages (merged from all parts)
            {
                let mut all_packages: Vec<String> = group.hub.r_packages.clone();
                if let Some(ref t) = group.tutorial {
                    for pkg in &t.r_packages {
                        if !all_packages.contains(pkg) {
                            all_packages.push(pkg.clone());
                        }
                    }
                }
                if let Some(ref r) = group.reference {
                    for pkg in &r.r_packages {
                        if !all_packages.contains(pkg) {
                            all_packages.push(pkg.clone());
                        }
                    }
                }
                all_packages.sort();

                if !all_packages.is_empty() {
                    Some(view! {
                        <div class="mt-3 flex flex-wrap gap-1.5">
                            <span class="text-xs text-muted">"R:"</span>
                            {all_packages.into_iter().map(|pkg| {
                                view! {
                                    <span class="text-xs px-1.5 py-0.5 rounded bg-blue-50 text-blue-600 dark:bg-blue-900/20 dark:text-blue-400">
                                        {pkg}
                                    </span>
                                }
                            }).collect_view()}
                        </div>
                    })
                } else {
                    None
                }
            }
        </div>
    }
}

/// Grouped guide catalog component.
#[component]
pub fn GroupedGuideCatalog(groups: Vec<MethodGroup>) -> impl IntoView {
    view! {
        <div class="space-y-10">
            {if groups.is_empty() {
                view! {
                    <div class="text-center py-12">
                        <p class="text-secondary">"No method guides available yet. Check back soon!"</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                        {groups.into_iter().map(|group| {
                            view! {
                                <MethodCard group />
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

/// Guide catalog grid component (legacy - shows individual cards).
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
