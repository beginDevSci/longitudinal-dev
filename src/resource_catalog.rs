//! Resource catalog with search and category filtering.
//!
//! Provides interactive filtering for the Resources page via Leptos islands.

use crate::base_path;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Resource category for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceCategory {
    Books,
    Videos,
    Tutorials,
    Cheatsheets,
    RPackages,
}

impl ResourceCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ResourceCategory::Books => "Books",
            ResourceCategory::Videos => "Videos",
            ResourceCategory::Tutorials => "Tutorials",
            ResourceCategory::Cheatsheets => "Cheatsheets",
            ResourceCategory::RPackages => "R Packages",
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            ResourceCategory::Books => "books",
            ResourceCategory::Videos => "videos",
            ResourceCategory::Tutorials => "tutorials",
            ResourceCategory::Cheatsheets => "cheatsheets",
            ResourceCategory::RPackages => "r-packages",
        }
    }

    pub fn all() -> Vec<ResourceCategory> {
        vec![
            ResourceCategory::Books,
            ResourceCategory::Videos,
            ResourceCategory::Tutorials,
            ResourceCategory::Cheatsheets,
            ResourceCategory::RPackages,
        ]
    }
}

/// Additional resource link for packages (vignettes, tutorials, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceLink {
    pub label: String,
    pub url: String,
}

/// Flattened resource item for search/filter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceItem {
    pub title: String,
    pub description: String,
    pub url: String,
    pub category: ResourceCategory,
    /// Optional image URL (for books)
    pub image: Option<String>,
    /// Optional embed URL (for videos)
    pub embed_url: Option<String>,
    /// Optional author (for books)
    pub author: Option<String>,
    /// Optional source (for videos)
    pub source: Option<String>,
    /// Optional platform (for tutorials)
    pub platform: Option<String>,
    /// Optional access type (for tutorials)
    pub access: Option<String>,
    /// Optional format (for cheatsheets)
    pub format: Option<String>,
    /// Optional icon (for cheatsheets)
    pub icon: Option<String>,
    /// Skill level: beginner, intermediate, advanced
    #[serde(default)]
    pub level: Option<String>,
    /// Whether this is an open-source/free resource
    #[serde(default)]
    pub is_open_source: Option<bool>,
    /// Whether this resource is featured/recommended
    #[serde(default)]
    pub is_featured: Option<bool>,
    /// Tags for categorization and filtering
    #[serde(default)]
    pub tags: Vec<String>,
    /// Additional resource links (vignettes, tutorials, GitHub, etc.)
    #[serde(default)]
    pub resource_links: Vec<ResourceLink>,
}

/// Search bar island component for resources
#[island]
pub fn ResourceSearchBar(search_query: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="relative" role="search">
            <svg
                class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-muted pointer-events-none"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                aria-hidden="true"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
            </svg>
            <input
                type="search"
                placeholder="Search resources..."
                aria-label="Search resources"
                class="w-full pl-12 pr-4 py-3 rounded-lg border border-stroke bg-surface text-primary placeholder:text-muted focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:border-accent transition-colors"
                on:input=move |ev| search_query.set(event_target_value(&ev))
                prop:value=move || search_query.get()
            />
        </div>
    }
}

/// Category filter chips island component
#[island]
pub fn ResourceCategoryChips(
    selected_categories: RwSignal<Vec<ResourceCategory>>,
    category_counts: Vec<(ResourceCategory, usize)>,
) -> impl IntoView {
    let total_count: usize = category_counts.iter().map(|(_, c)| c).sum();
    let all_selected = move || {
        let selected = selected_categories.get();
        selected.is_empty() || selected.len() == ResourceCategory::all().len()
    };

    view! {
        <div class="flex flex-wrap gap-2" role="group" aria-label="Filter by category">
            // "All" chip
            <button
                class=move || {
                    let base = "px-4 py-2 rounded-full text-sm font-medium transition-all duration-200";
                    if all_selected() {
                        format!("{base} bg-accent text-white")
                    } else {
                        format!("{base} bg-surface border border-stroke text-secondary hover:border-accent hover:text-accent")
                    }
                }
                on:click=move |_| selected_categories.set(vec![])
            >
                "All (" {total_count} ")"
            </button>

            // Category chips
            {category_counts.into_iter().map(|(category, count)| {
                let cat_for_check = category;
                let cat_for_click = category;
                view! {
                    <button
                        class=move || {
                            let base = "px-4 py-2 rounded-full text-sm font-medium transition-all duration-200";
                            let selected = selected_categories.get();
                            let is_selected = !selected.is_empty() && selected.contains(&cat_for_check);
                            if is_selected {
                                format!("{base} bg-accent text-white")
                            } else {
                                format!("{base} bg-surface border border-stroke text-secondary hover:border-accent hover:text-accent")
                            }
                        }
                        on:click=move |_| {
                            let mut current = selected_categories.get();
                            if current.contains(&cat_for_click) {
                                current.retain(|c| *c != cat_for_click);
                            } else if current.is_empty() || current.len() == ResourceCategory::all().len() - 1 {
                                current = vec![cat_for_click];
                            } else {
                                current.push(cat_for_click);
                            }
                            // If all are selected, clear to show all
                            if current.len() == ResourceCategory::all().len() {
                                current = vec![];
                            }
                            selected_categories.set(current);
                        }
                    >
                        {category.label()} " (" {count} ")"
                    </button>
                }
            }).collect_view()}
        </div>
    }
}


/// Check if a resource matches the search query
fn matches_search(item: &ResourceItem, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query_lower = query.to_lowercase();
    item.title.to_lowercase().contains(&query_lower)
        || item.description.to_lowercase().contains(&query_lower)
        || item.author.as_ref().map(|a| a.to_lowercase().contains(&query_lower)).unwrap_or(false)
        || item.source.as_ref().map(|s| s.to_lowercase().contains(&query_lower)).unwrap_or(false)
        || item.platform.as_ref().map(|p| p.to_lowercase().contains(&query_lower)).unwrap_or(false)
}

/// Check if a resource matches the selected categories
fn matches_category(item: &ResourceItem, categories: &[ResourceCategory]) -> bool {
    if categories.is_empty() {
        return true;
    }
    categories.contains(&item.category)
}

/// Check if a resource matches the selected level
fn matches_level(item: &ResourceItem, level: &str) -> bool {
    if level.is_empty() || level == "all" {
        return true;
    }
    item.level.as_ref().map(|l| l.to_lowercase() == level.to_lowercase()).unwrap_or(false)
}

/// Check if a resource matches the selected topic tag
fn matches_topic(item: &ResourceItem, topic: &str) -> bool {
    if topic.is_empty() {
        return true;
    }
    let topic_lower = topic.to_lowercase();
    item.tags.iter().any(|t| t.to_lowercase().contains(&topic_lower))
}

/// Topic quick filter chips for common research areas
#[island]
pub fn TopicFilterChips(selected_topic: RwSignal<String>) -> impl IntoView {
    let topics = vec![
        ("", "All Topics"),
        ("longitudinal", "Longitudinal"),
        ("mixed models", "Mixed Models"),
        ("sem", "SEM"),
        ("tidyverse", "Tidyverse"),
        ("visualization", "Visualization"),
    ];

    view! {
        <div class="flex flex-wrap gap-2" role="group" aria-label="Filter by topic">
            {topics.into_iter().map(|(value, label)| {
                let value_for_check = value.to_string();
                let value_for_click = value.to_string();
                view! {
                    <button
                        class=move || {
                            let base = "px-3 py-1.5 rounded-full text-sm font-medium transition-all duration-200";
                            let selected = selected_topic.get();
                            let is_selected = selected == value_for_check;
                            if is_selected {
                                format!("{base} bg-accent text-white")
                            } else {
                                format!("{base} bg-surface border border-stroke text-secondary hover:border-accent hover:text-accent")
                            }
                        }
                        on:click=move |_| {
                            selected_topic.set(value_for_click.clone());
                        }
                    >
                        {label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

/// Level filter chips island component
#[island]
pub fn LevelFilterChips(selected_level: RwSignal<String>) -> impl IntoView {
    let levels = vec![
        ("all", "All Levels"),
        ("beginner", "Foundational"),
        ("intermediate", "Intermediate"),
        ("advanced", "Advanced"),
    ];

    view! {
        <div class="flex flex-wrap gap-2" role="group" aria-label="Filter by level">
            {levels.into_iter().map(|(value, label)| {
                let value_for_check = value.to_string();
                let value_for_click = value.to_string();
                view! {
                    <button
                        class=move || {
                            let base = "px-3 py-1.5 rounded-full text-sm font-medium transition-all duration-200";
                            let selected = selected_level.get();
                            let is_selected = (selected.is_empty() && value_for_check == "all") || selected == value_for_check;
                            if is_selected {
                                format!("{base} bg-accent text-white")
                            } else {
                                format!("{base} bg-surface border border-stroke text-secondary hover:border-accent hover:text-accent")
                            }
                        }
                        on:click=move |_| {
                            if value_for_click == "all" {
                                selected_level.set(String::new());
                            } else {
                                selected_level.set(value_for_click.clone());
                            }
                        }
                    >
                        {label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

/// Main resource catalog island that orchestrates filtering
#[island]
pub fn ResourceCatalogIsland(resources: Vec<ResourceItem>) -> impl IntoView {
    let search_query = RwSignal::new(String::new());
    let selected_categories = RwSignal::new(Vec::<ResourceCategory>::new());
    let selected_level = RwSignal::new(String::new());
    let selected_topic = RwSignal::new(String::new());

    // Calculate category counts
    let category_counts: Vec<(ResourceCategory, usize)> = ResourceCategory::all()
        .into_iter()
        .map(|cat| {
            let count = resources.iter().filter(|r| r.category == cat).count();
            (cat, count)
        })
        .collect();

    let total_count = resources.len();
    let resources_signal = StoredValue::new(resources);

    // Filtered resources signal
    let filtered_resources = Memo::new(move |_| {
        let query = search_query.get();
        let categories = selected_categories.get();
        let level = selected_level.get();
        let topic = selected_topic.get();
        resources_signal
            .get_value()
            .into_iter()
            .filter(|r| {
                matches_search(r, &query)
                    && matches_category(r, &categories)
                    && matches_level(r, &level)
                    && matches_topic(r, &topic)
            })
            .collect::<Vec<_>>()
    });

    let filtered_count = Signal::derive(move || filtered_resources.get().len());

    // Group filtered resources by category for display
    let grouped_resources = Memo::new(move |_| {
        let filtered = filtered_resources.get();
        let mut groups: std::collections::HashMap<ResourceCategory, Vec<ResourceItem>> =
            std::collections::HashMap::new();

        for item in filtered {
            groups.entry(item.category).or_default().push(item);
        }

        groups
    });

    view! {
        <div class="space-y-6">
            // Search and filters
            <div class="flex flex-col md:flex-row gap-4 items-start md:items-center justify-between">
                <div class="w-full md:w-96">
                    <ResourceSearchBar search_query=search_query />
                </div>
                <p class="text-sm text-secondary">
                    {move || {
                        let count = filtered_count.get();
                        if count == total_count {
                            format!("Showing all {} resources", total_count)
                        } else {
                            format!("Showing {} of {} resources", count, total_count)
                        }
                    }}
                </p>
            </div>

            // Category chips
            <ResourceCategoryChips
                selected_categories=selected_categories
                category_counts=category_counts
            />

            // Level and Topic filters
            <div class="flex flex-col sm:flex-row gap-4">
                <div class="flex items-center gap-3">
                    <span class="text-sm text-secondary font-medium">"Level:"</span>
                    <LevelFilterChips selected_level=selected_level />
                </div>
                <div class="flex items-center gap-3">
                    <span class="text-sm text-secondary font-medium">"Topic:"</span>
                    <TopicFilterChips selected_topic=selected_topic />
                </div>
            </div>

            // Resource sections
            <div>
                {move || {
                    let groups = grouped_resources.get();
                    let mut index = 0usize;
                    ResourceCategory::all()
                        .into_iter()
                        .filter_map(|cat| {
                            let items = groups.get(&cat)?;
                            if items.is_empty() {
                                return None;
                            }
                            let current_index = index;
                            index += 1;
                            Some(view! {
                                <ResourceSection category=cat items=items.clone() index=current_index />
                            })
                        })
                        .collect_view()
                }}
            </div>

            // Empty state
            {move || {
                if filtered_resources.get().is_empty() {
                    Some(view! {
                        <div class="text-center py-16">
                            <div class="text-6xl mb-4">"🔍"</div>
                            <h3 class="text-xl font-semibold text-primary mb-2">"No resources found"</h3>
                            <p class="text-secondary">
                                "Try adjusting your search or filters"
                            </p>
                        </div>
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}

/// Resource section component (rendered within the island)
#[component]
fn ResourceSection(category: ResourceCategory, items: Vec<ResourceItem>, index: usize) -> impl IntoView {
    let section_id = category.id();
    let section_title = category.label();
    let description = match category {
        ResourceCategory::Books => "From R fundamentals to advanced mixed models and SEM—these texts cover the methods essential for longitudinal research.",
        ResourceCategory::Videos => "Video courses and tutorials covering R programming, statistical modeling, and data analysis workflows.",
        ResourceCategory::Tutorials => "Hands-on interactive tutorials for learning R syntax, data wrangling, and statistical concepts at your own pace.",
        ResourceCategory::Cheatsheets => "Quick reference guides for R syntax, tidyverse verbs, and common statistical functions—keep these handy.",
        ResourceCategory::RPackages => "Core packages for mixed models, growth curves, SEM, Bayesian methods, and missing data handling used throughout longitudinal.dev.",
    };

    let grid_class = match category {
        ResourceCategory::Books => "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6",
        ResourceCategory::Videos => "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
        ResourceCategory::Tutorials => "grid grid-cols-1 md:grid-cols-2 gap-6",
        ResourceCategory::Cheatsheets => "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6",
        ResourceCategory::RPackages => "grid grid-cols-1 md:grid-cols-2 gap-4",
    };

    // Alternating backgrounds: odd indices get bg-subtle. Increased padding for better visual rhythm.
    let section_class = if index % 2 == 1 {
        "py-14 -mx-4 sm:-mx-6 lg:-mx-8 px-4 sm:px-6 lg:px-8 bg-subtle"
    } else {
        "py-14"
    };

    view! {
        <section id=section_id class=section_class>
            <div class="mb-8 pb-4 border-b border-stroke">
                <h2 class="text-2xl font-bold text-primary mb-2">{section_title}</h2>
                <p class="text-secondary">{description}</p>
            </div>
            <div class=grid_class>
                {items.into_iter().map(|item| {
                    view! { <ResourceCard item=item /> }
                }).collect_view()}
            </div>
        </section>
    }
}

/// Individual resource card component
#[component]
fn ResourceCard(item: ResourceItem) -> impl IntoView {
    match item.category {
        ResourceCategory::Books => view! { <BookCardInner item=item /> }.into_any(),
        ResourceCategory::Videos => view! { <VideoCardInner item=item /> }.into_any(),
        ResourceCategory::Tutorials => view! { <TutorialCardInner item=item /> }.into_any(),
        ResourceCategory::Cheatsheets => view! { <CheatsheetCardInner item=item /> }.into_any(),
        ResourceCategory::RPackages => view! { <RPackageCardInner item=item /> }.into_any(),
    }
}

/// Get CSS classes for level badge
fn get_level_badge_class(level: &str) -> &'static str {
    match level.to_lowercase().as_str() {
        "beginner" => "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300",
        "intermediate" => "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300",
        "advanced" => "bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300",
        _ => "bg-slate-100 text-slate-800 dark:bg-slate-800 dark:text-slate-300",
    }
}

/// Get display label for level
fn get_level_label(level: &str) -> &'static str {
    match level.to_lowercase().as_str() {
        "beginner" => "Foundational",
        "intermediate" => "Intermediate",
        "advanced" => "Advanced",
        _ => "All Levels",
    }
}

/// Render badge row with level, open source, and featured badges
#[component]
fn ResourceBadges(
    #[prop(default = String::new())] level: String,
    #[prop(default = false)] is_open_source: bool,
    #[prop(default = false)] is_featured: bool,
) -> impl IntoView {
    let has_level = !level.is_empty();
    let has_any_badge = has_level || is_open_source || is_featured;

    if !has_any_badge {
        return view! { <div></div> }.into_any();
    }

    view! {
        <div class="flex flex-wrap gap-1.5 mb-2">
            // Level badge
            {has_level.then(|| {
                let badge_class = get_level_badge_class(&level);
                let label = get_level_label(&level);
                view! {
                    <span class=format!("px-2 py-0.5 text-xs font-medium rounded-full {}", badge_class)>
                        {label}
                    </span>
                }
            })}
            // Open Source badge
            {is_open_source.then(|| view! {
                <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300">
                    "Open Source"
                </span>
            })}
            // Featured badge
            {is_featured.then(|| view! {
                <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300">
                    "★"
                </span>
            })}
        </div>
    }.into_any()
}

#[component]
fn BookCardInner(item: ResourceItem) -> impl IntoView {
    let has_image = item.image.is_some();
    // Apply base_path to image URLs that start with /
    let image_url = item.image.clone().map(|url| {
        url.strip_prefix('/')
            .map(base_path::join)
            .unwrap_or(url)
    }).unwrap_or_default();
    let author = item.author.clone().unwrap_or_default();
    let is_featured = item.is_featured == Some(true);
    let card_class = if is_featured {
        "resource-card resource-card-featured group block"
    } else {
        "resource-card group block"
    };

    view! {
        <a
            href=item.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class=card_class
        >
            <div class="aspect-[4/3] w-full cover-frame">
                {if has_image {
                    view! {
                        <img
                            src=image_url
                            alt=item.title.clone()
                            class="w-full h-full object-contain"
                            loading="lazy"
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="w-full h-full bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
                            <span class="text-6xl opacity-50">"📚"</span>
                        </div>
                    }.into_any()
                }}
            </div>
            <div class="p-4">
                <ResourceBadges
                    level=item.level.clone().unwrap_or_default()
                    is_open_source=item.is_open_source.unwrap_or(false)
                    is_featured=item.is_featured.unwrap_or(false)
                />
                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors mb-1">
                    {item.title}
                </h3>
                <p class="text-sm text-tertiary mb-2">{author}</p>
                <p class="text-sm text-secondary">{item.description}</p>
            </div>
        </a>
    }
}

#[component]
fn VideoCardInner(item: ResourceItem) -> impl IntoView {
    let has_embed = item.embed_url.is_some();
    let embed_url = item.embed_url.clone().unwrap_or_default();
    let source = item.source.clone().unwrap_or_default();
    let iframe_title = item.title.clone();
    let is_featured = item.is_featured == Some(true);
    let card_class = if is_featured {
        "resource-card resource-card-featured group"
    } else {
        "resource-card group"
    };

    view! {
        <div class=card_class>
            <div class="aspect-video w-full bg-black overflow-hidden">
                {if has_embed {
                    view! {
                        <iframe
                            src=embed_url
                            title=iframe_title
                            class="w-full h-full border-0"
                            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                            allowfullscreen=true
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="w-full h-full bg-gradient-to-br from-red-500/20 to-red-600/10 flex items-center justify-center">
                            <div class="w-16 h-16 rounded-full bg-red-500/80 flex items-center justify-center">
                                <svg class="w-8 h-8 text-white ml-1" fill="currentColor" viewBox="0 0 24 24">
                                    <path d="M8 5v14l11-7z"/>
                                </svg>
                            </div>
                        </div>
                    }.into_any()
                }}
            </div>
            <div class="p-4">
                <ResourceBadges
                    level=item.level.clone().unwrap_or_default()
                    is_open_source=item.is_open_source.unwrap_or(false)
                    is_featured=item.is_featured.unwrap_or(false)
                />
                <a
                    href=item.url.clone()
                    target="_blank"
                    rel="noopener noreferrer"
                    class="block"
                >
                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors mb-1">
                        {item.title}
                    </h3>
                    <p class="text-sm text-tertiary mb-2">{source}</p>
                    <p class="text-sm text-secondary">{item.description}</p>
                </a>
            </div>
        </div>
    }
}

#[component]
fn TutorialCardInner(item: ResourceItem) -> impl IntoView {
    let platform = item.platform.clone().unwrap_or_default();
    let show_platform = !platform.is_empty();
    let access = item.access.clone().unwrap_or_default();
    let access_class = if access.to_lowercase() == "freemium" {
        "px-2 py-0.5 text-xs font-medium rounded-full bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300"
    } else {
        "px-2 py-0.5 text-xs font-medium rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300"
    };
    let access_label = if access.to_lowercase() == "freemium" { "Freemium" } else { "Open" };
    let is_featured = item.is_featured == Some(true);
    let card_class = if is_featured {
        "resource-card resource-card-featured group block p-6"
    } else {
        "resource-card group block p-6"
    };

    view! {
        <a
            href=item.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class=card_class
        >
            <ResourceBadges
                level=item.level.clone().unwrap_or_default()
                is_open_source=item.is_open_source.unwrap_or(false)
                is_featured=item.is_featured.unwrap_or(false)
            />
            <div class="flex items-start gap-4 mb-4">
                <div class="w-14 h-14 rounded-xl bg-gradient-to-br from-blue-500/20 to-purple-500/20 flex items-center justify-center flex-shrink-0 group-hover:scale-110 transition-transform">
                    <span class="text-2xl">"🎓"</span>
                </div>
                <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">
                        {item.title}
                    </h3>
                    <div class="flex items-center gap-2 mt-1">
                        {show_platform.then(|| view! {
                            <span class="text-sm text-tertiary">{platform}</span>
                        })}
                        {(!access.is_empty()).then(|| view! {
                            <span class=access_class>{access_label}</span>
                        })}
                    </div>
                </div>
            </div>
            <p class="text-sm text-secondary line-clamp-3 mb-4">{item.description}</p>
            <div class="flex items-center text-accent text-sm font-medium">
                <span>"Open Tutorial"</span>
                <svg class="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                </svg>
            </div>
        </a>
    }
}

#[component]
fn CheatsheetCardInner(item: ResourceItem) -> impl IntoView {
    let format_badge = item.format.clone().map(|f| f.to_uppercase());
    let is_featured = item.is_featured == Some(true);
    let card_class = if is_featured {
        "resource-card resource-card-featured group block"
    } else {
        "resource-card group block"
    };

    // Apply base_path to image URLs that start with /
    let image_url = item.image.clone().map(|url| {
        url.strip_prefix('/')
            .map(base_path::join)
            .unwrap_or(url)
    });
    let has_image = image_url.is_some();
    let image_src = image_url.unwrap_or_default();

    view! {
        <a
            href=item.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class=card_class
        >
            // PDF preview thumbnail
            <div class="aspect-[4/3] w-full overflow-hidden rounded-t-lg bg-slate-100 dark:bg-slate-800">
                {if has_image {
                    view! {
                        <img
                            src=image_src
                            alt=item.title.clone()
                            class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                            loading="lazy"
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="w-full h-full bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
                            <span class="text-6xl opacity-50">"📄"</span>
                        </div>
                    }.into_any()
                }}
            </div>

            <div class="p-4">
                <ResourceBadges
                    level=item.level.clone().unwrap_or_default()
                    is_open_source=item.is_open_source.unwrap_or(false)
                    is_featured=item.is_featured.unwrap_or(false)
                />
                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors mb-2">
                    {item.title}
                </h3>
                <p class="text-sm text-secondary line-clamp-2 mb-3">{item.description}</p>

                <div class="flex items-center justify-between">
                    {if let Some(badge) = format_badge {
                        view! {
                            <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-accent/10 text-accent">
                                {badge}
                            </span>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                    <svg class="w-5 h-5 text-tertiary group-hover:text-accent group-hover:translate-x-1 group-hover:-translate-y-1 transition-all" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
                    </svg>
                </div>
            </div>
        </a>
    }
}

/// R Package card with resource links (vignettes, tutorials, etc.)
#[component]
fn RPackageCardInner(item: ResourceItem) -> impl IntoView {
    let is_open_source = item.is_open_source == Some(true);
    let level = item.level.clone().unwrap_or_default();
    let has_level = !level.is_empty();
    let has_resource_links = !item.resource_links.is_empty();
    let resource_links = item.resource_links.clone();
    // Apply base_path to logo URLs that start with /
    let logo_url = item.image.clone().map(|url| {
        url.strip_prefix('/')
            .map(base_path::join)
            .unwrap_or(url)
    }).unwrap_or_default();
    let has_logo = !logo_url.is_empty();

    view! {
        <div class="flex flex-col p-4 rounded-lg border border-stroke bg-surface hover:bg-subtle transition-colors">
            // Header row with logo, title, and badges
            <div class="flex items-start gap-3 mb-2">
                // Small hex logo
                {has_logo.then(|| view! {
                    <img
                        src=logo_url
                        alt=""
                        class="w-8 h-8 flex-shrink-0 object-contain"
                        loading="lazy"
                    />
                })}
                <div class="flex-1 min-w-0">
                    <div class="flex items-start justify-between gap-2">
                        <a
                            href=item.url.clone()
                            target="_blank"
                            rel="noopener noreferrer"
                            class="font-semibold text-primary hover:text-accent transition-colors"
                        >
                            {item.title.clone()}
                        </a>
                        <div class="flex items-center gap-1 flex-shrink-0">
                            {has_level.then(|| {
                                let badge_class = get_level_badge_class(&level);
                                let label = get_level_label(&level);
                                view! {
                                    <span class=format!("px-1.5 py-0.5 text-xs font-medium rounded-full {}", badge_class)>
                                        {label}
                                    </span>
                                }
                            })}
                            {is_open_source.then(|| view! {
                                <span class="px-1.5 py-0.5 text-xs font-medium rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300">
                                    "Open"
                                </span>
                            })}
                        </div>
                    </div>
                </div>
            </div>

            // Description
            <p class="text-sm text-secondary mb-3 line-clamp-2">{item.description}</p>

            // Resource links row
            <div class="flex items-center gap-2 flex-wrap mt-auto">
                // Main CRAN/package link
                <a
                    href=item.url.clone()
                    target="_blank"
                    rel="noopener noreferrer"
                    class="inline-flex items-center gap-1 text-xs text-accent hover:text-accent/80 transition-colors"
                >
                    <span>"CRAN"</span>
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
                    </svg>
                </a>
                // Additional resource links
                {has_resource_links.then(move || {
                    view! {
                        <>
                            {resource_links.into_iter().map(|link| {
                                view! {
                                    <span class="text-tertiary">"·"</span>
                                    <a
                                        href=link.url
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="inline-flex items-center gap-1 text-xs text-accent hover:text-accent/80 transition-colors"
                                    >
                                        <span>{link.label}</span>
                                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
                                        </svg>
                                    </a>
                                }
                            }).collect_view()}
                        </>
                    }
                })}
            </div>
        </div>
    }
}
