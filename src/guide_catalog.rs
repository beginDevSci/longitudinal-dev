//! Guide catalog component for the /guides index page.
//!
//! Displays method guides in a card layout with filtering by category.
//! Guides are grouped by method (hub + tutorial + reference).
//! Uses a Leptos island for interactive category filtering.

use crate::base_path;
use crate::models::guide::{CategoryMeta, GuideCatalogItem, MethodGroup};
use leptos::prelude::*;
use std::collections::HashMap;

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

    view! {
        <div class="rounded-xl bg-elevated border border-stroke p-6 hover:shadow-lg transition-shadow duration-200">
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
        </div>
    }
}

/// Grouped guide catalog island with interactive category filtering.
#[island]
pub fn GroupedGuideCatalog(groups: Vec<MethodGroup>) -> impl IntoView {
    // Filter state: None = show all, Some(category_id) = filter to that category
    let active_filter = RwSignal::new(None::<String>);

    if groups.is_empty() {
        return view! {
            <div class="text-center py-12">
                <p class="text-secondary">"No method guides available yet. Check back soon!"</p>
            </div>
        }
        .into_any();
    }

    // Group by category (done once, not reactive)
    let mut by_category: HashMap<String, Vec<MethodGroup>> = HashMap::new();
    for group in groups {
        by_category
            .entry(group.category.clone())
            .or_default()
            .push(group);
    }

    // Get categories that have content, sorted by order
    let active_categories: Vec<&'static CategoryMeta> = CategoryMeta::all_sorted()
        .into_iter()
        .filter(|cat| by_category.contains_key(cat.id))
        .collect();

    // Clone for use in closures
    let categories_for_pills = active_categories.clone();
    let categories_for_sections = active_categories.clone();

    view! {
        <div class="space-y-8">
            // Filter pills
            <div class="flex flex-wrap gap-2 pb-4 border-b border-stroke">
                // "All" pill
                <button
                    type="button"
                    class=move || {
                        let base = "px-4 py-2 rounded-full text-sm font-medium border transition-all duration-200 cursor-pointer";
                        if active_filter.get().is_none() {
                            format!("{base} bg-accent text-white border-accent ring-2 ring-offset-2 ring-accent/50")
                        } else {
                            format!("{base} bg-surface border-stroke text-secondary hover:bg-subtle hover:text-primary")
                        }
                    }
                    on:click=move |_| active_filter.set(None)
                >
                    "All"
                </button>

                // Category pills
                {categories_for_pills.into_iter().map(|cat| {
                    let cat_id = cat.id.to_string();
                    let cat_id_for_check = cat_id.clone();
                    let cat_id_for_click = cat_id.clone();
                    let color_classes = cat.color_classes;

                    view! {
                        <button
                            type="button"
                            class=move || {
                                let base = "px-4 py-2 rounded-full text-sm font-medium border transition-all duration-200 cursor-pointer";
                                let is_active = active_filter.get().as_ref() == Some(&cat_id_for_check);
                                if is_active {
                                    format!("{base} {color_classes} ring-2 ring-offset-2 ring-accent/50")
                                } else {
                                    format!("{base} bg-surface border-stroke text-secondary hover:bg-subtle hover:text-primary")
                                }
                            }
                            on:click=move |_| active_filter.set(Some(cat_id_for_click.clone()))
                        >
                            {cat.name}
                        </button>
                    }
                }).collect_view()}
            </div>

            // Category sections (visibility toggled via CSS classes)
            {categories_for_sections.into_iter().map(|cat| {
                let cat_id = cat.id.to_string();
                let cat_groups = by_category.remove(cat.id).unwrap_or_default();

                view! {
                    <section
                        class=move || {
                            let filter = active_filter.get();
                            let is_visible = filter.is_none() || filter.as_ref() == Some(&cat_id);
                            if is_visible {
                                "mb-12"
                            } else {
                                "mb-12 hidden"
                            }
                        }
                    >
                        // Section header
                        <div class="flex items-center gap-3 mb-6">
                            <span class="text-2xl">{cat.icon}</span>
                            <div>
                                <h2 class="text-xl font-bold text-primary">{cat.name}</h2>
                                <p class="text-sm text-secondary">{cat.description}</p>
                            </div>
                        </div>

                        // Cards grid
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                            {cat_groups.into_iter().map(|group| {
                                view! {
                                    <MethodCard group />
                                }
                            }).collect_view()}
                        </div>
                    </section>
                }
            }).collect_view()}
        </div>
    }
    .into_any()
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
