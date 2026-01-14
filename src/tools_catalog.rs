//! Tools catalog with search and category filtering.
//!
//! Provides interactive filtering for the Tools page via Leptos islands.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Tool category for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    ProgrammingLanguages,
    RPackages,
    IDEs,
    VersionControl,
    DataFormats,
    Notebooks,
    Databases,
}

impl ToolCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ToolCategory::ProgrammingLanguages => "Languages",
            ToolCategory::RPackages => "R Packages",
            ToolCategory::IDEs => "IDEs",
            ToolCategory::VersionControl => "Version Control",
            ToolCategory::DataFormats => "Data Formats",
            ToolCategory::Notebooks => "Notebooks",
            ToolCategory::Databases => "Databases",
        }
    }

    pub fn full_label(&self) -> &'static str {
        match self {
            ToolCategory::ProgrammingLanguages => "Programming Languages",
            ToolCategory::RPackages => "R Packages for Longitudinal Analysis",
            ToolCategory::IDEs => "Development Environments",
            ToolCategory::VersionControl => "Version Control & Reproducibility",
            ToolCategory::DataFormats => "Data Formats",
            ToolCategory::Notebooks => "Notebooks & Literate Programming",
            ToolCategory::Databases => "Databases",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ToolCategory::ProgrammingLanguages => "Core languages for statistical computing—R and Python power most longitudinal analyses and reproducible workflows.",
            ToolCategory::RPackages => "These R packages power the linear mixed models, growth curves, SEM, and missing data handling used throughout longitudinal.dev.",
            ToolCategory::IDEs => "Development environments optimized for R and data science—write, debug, and visualize your analyses.",
            ToolCategory::VersionControl => "Version control and reproducibility tools—track changes, manage package versions, and create reproducible pipelines.",
            ToolCategory::DataFormats => "File formats for storing and sharing data—from simple CSV to high-performance columnar formats.",
            ToolCategory::Notebooks => "Literate programming environments that combine code, output, and narrative for reproducible research.",
            ToolCategory::Databases => "Database systems for storing and querying structured data, from lightweight SQLite to scalable PostgreSQL.",
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            ToolCategory::ProgrammingLanguages => "languages",
            ToolCategory::RPackages => "r-packages",
            ToolCategory::IDEs => "ides",
            ToolCategory::VersionControl => "version-control",
            ToolCategory::DataFormats => "data-formats",
            ToolCategory::Notebooks => "notebooks",
            ToolCategory::Databases => "databases",
        }
    }

    pub fn all() -> Vec<ToolCategory> {
        vec![
            ToolCategory::ProgrammingLanguages,
            ToolCategory::RPackages,
            ToolCategory::IDEs,
            ToolCategory::VersionControl,
            ToolCategory::DataFormats,
            ToolCategory::Notebooks,
            ToolCategory::Databases,
        ]
    }
}

/// Flattened tool item for search/filter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolItem {
    pub title: String,
    pub description: String,
    pub url: String,
    pub category: ToolCategory,
    pub logo: Option<String>,
    /// Skill level: beginner, intermediate, advanced
    #[serde(default)]
    pub level: Option<String>,
    /// Whether this is an open-source tool
    #[serde(default)]
    pub is_open_source: Option<bool>,
    /// Whether this tool is featured/recommended
    #[serde(default)]
    pub is_featured: Option<bool>,
    /// Tags for categorization and filtering
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Search bar island component for tools
#[island]
pub fn ToolSearchBar(search_query: RwSignal<String>) -> impl IntoView {
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
                placeholder="Search tools..."
                aria-label="Search tools"
                class="w-full pl-12 pr-4 py-3 rounded-lg border border-stroke bg-surface text-primary placeholder:text-muted focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:border-accent transition-colors"
                on:input=move |ev| search_query.set(event_target_value(&ev))
                prop:value=move || search_query.get()
            />
        </div>
    }
}

/// Category filter chips island component
#[island]
pub fn ToolCategoryChips(
    selected_categories: RwSignal<Vec<ToolCategory>>,
    category_counts: Vec<(ToolCategory, usize)>,
) -> impl IntoView {
    let total_count: usize = category_counts.iter().map(|(_, c)| c).sum();
    let all_selected = move || {
        let selected = selected_categories.get();
        selected.is_empty() || selected.len() == ToolCategory::all().len()
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
                            } else if current.is_empty() || current.len() == ToolCategory::all().len() - 1 {
                                current = vec![cat_for_click];
                            } else {
                                current.push(cat_for_click);
                            }
                            // If all are selected, clear to show all
                            if current.len() == ToolCategory::all().len() {
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

/// Check if a tool matches the search query
fn matches_search(item: &ToolItem, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query_lower = query.to_lowercase();
    item.title.to_lowercase().contains(&query_lower)
        || item.description.to_lowercase().contains(&query_lower)
}

/// Check if a tool matches the selected categories
fn matches_category(item: &ToolItem, categories: &[ToolCategory]) -> bool {
    if categories.is_empty() {
        return true;
    }
    categories.contains(&item.category)
}

/// Check if a tool matches the selected level
fn matches_level(item: &ToolItem, level: &str) -> bool {
    if level.is_empty() || level == "all" {
        return true;
    }
    item.level.as_ref().map(|l| l.to_lowercase() == level.to_lowercase()).unwrap_or(false)
}

/// Check if a tool matches the selected focus area
fn matches_focus(item: &ToolItem, focus: &str) -> bool {
    if focus.is_empty() {
        return true;
    }
    let focus_lower = focus.to_lowercase();
    item.tags.iter().any(|t| t.to_lowercase().contains(&focus_lower))
}

/// Focus filter chips for tool categories
#[island]
pub fn FocusFilterChips(selected_focus: RwSignal<String>) -> impl IntoView {
    let focuses = vec![
        ("", "All Focus Areas"),
        ("longitudinal", "Longitudinal"),
        ("reproducibility", "Reproducibility"),
        ("mixed models", "Mixed Models"),
        ("sem", "SEM"),
        ("visualization", "Visualization"),
    ];

    view! {
        <div class="flex flex-wrap gap-2" role="group" aria-label="Filter by focus area">
            {focuses.into_iter().map(|(value, label)| {
                let value_for_check = value.to_string();
                let value_for_click = value.to_string();
                view! {
                    <button
                        class=move || {
                            let base = "px-3 py-1.5 rounded-full text-sm font-medium transition-all duration-200";
                            let selected = selected_focus.get();
                            let is_selected = selected == value_for_check;
                            if is_selected {
                                format!("{base} bg-accent text-white")
                            } else {
                                format!("{base} bg-surface border border-stroke text-secondary hover:border-accent hover:text-accent")
                            }
                        }
                        on:click=move |_| {
                            selected_focus.set(value_for_click.clone());
                        }
                    >
                        {label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

/// Level filter chips island component for tools
#[island]
pub fn ToolLevelFilterChips(selected_level: RwSignal<String>) -> impl IntoView {
    let levels = vec![
        ("all", "All Levels"),
        ("beginner", "Beginner"),
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
        "beginner" => "Beginner",
        "intermediate" => "Intermediate",
        "advanced" => "Advanced",
        _ => "All Levels",
    }
}

/// Render badge row with level, open source, and featured badges for tools
#[component]
fn ToolBadges(
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
        <div class="flex flex-wrap justify-center gap-1 mb-2">
            // Level badge
            {has_level.then(|| {
                let badge_class = get_level_badge_class(&level);
                let label = get_level_label(&level);
                view! {
                    <span class=format!("px-1.5 py-0.5 text-xs font-medium rounded-full {}", badge_class)>
                        {label}
                    </span>
                }
            })}
            // Open Source badge
            {is_open_source.then(|| view! {
                <span class="px-1.5 py-0.5 text-xs font-medium rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300">
                    "Open"
                </span>
            })}
            // Featured badge
            {is_featured.then(|| view! {
                <span class="px-1.5 py-0.5 text-xs font-medium rounded-full bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300">
                    "★"
                </span>
            })}
        </div>
    }.into_any()
}

/// Main tools catalog island that orchestrates filtering
#[island]
pub fn ToolsCatalogIsland(tools: Vec<ToolItem>) -> impl IntoView {
    let search_query = RwSignal::new(String::new());
    let selected_categories = RwSignal::new(Vec::<ToolCategory>::new());
    let selected_level = RwSignal::new(String::new());
    let selected_focus = RwSignal::new(String::new());

    // Calculate category counts
    let category_counts: Vec<(ToolCategory, usize)> = ToolCategory::all()
        .into_iter()
        .map(|cat| {
            let count = tools.iter().filter(|t| t.category == cat).count();
            (cat, count)
        })
        .collect();

    let total_count = tools.len();
    let tools_signal = StoredValue::new(tools);

    // Filtered tools signal
    let filtered_tools = Memo::new(move |_| {
        let query = search_query.get();
        let categories = selected_categories.get();
        let level = selected_level.get();
        let focus = selected_focus.get();
        tools_signal
            .get_value()
            .into_iter()
            .filter(|t| {
                matches_search(t, &query)
                    && matches_category(t, &categories)
                    && matches_level(t, &level)
                    && matches_focus(t, &focus)
            })
            .collect::<Vec<_>>()
    });

    let filtered_count = Signal::derive(move || filtered_tools.get().len());

    // Group filtered tools by category for display
    let grouped_tools = Memo::new(move |_| {
        let filtered = filtered_tools.get();
        let mut groups: std::collections::HashMap<ToolCategory, Vec<ToolItem>> =
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
                    <ToolSearchBar search_query=search_query />
                </div>
                <p class="text-sm text-secondary">
                    {move || {
                        let count = filtered_count.get();
                        if count == total_count {
                            format!("Showing all {} tools", total_count)
                        } else {
                            format!("Showing {} of {} tools", count, total_count)
                        }
                    }}
                </p>
            </div>

            // Category chips
            <ToolCategoryChips
                selected_categories=selected_categories
                category_counts=category_counts
            />

            // Level and Focus filters
            <div class="flex flex-col sm:flex-row gap-4">
                <div class="flex items-center gap-3">
                    <span class="text-sm text-secondary font-medium">"Level:"</span>
                    <ToolLevelFilterChips selected_level=selected_level />
                </div>
                <div class="flex items-center gap-3">
                    <span class="text-sm text-secondary font-medium">"Focus:"</span>
                    <FocusFilterChips selected_focus=selected_focus />
                </div>
            </div>

            // Tool sections
            <div>
                {move || {
                    let groups = grouped_tools.get();
                    let mut index = 0usize;
                    ToolCategory::all()
                        .into_iter()
                        .filter_map(|cat| {
                            let items = groups.get(&cat)?;
                            if items.is_empty() {
                                return None;
                            }
                            let current_index = index;
                            index += 1;
                            Some(view! {
                                <ToolSection category=cat items=items.clone() index=current_index />
                            })
                        })
                        .collect_view()
                }}
            </div>

            // Empty state
            {move || {
                if filtered_tools.get().is_empty() {
                    Some(view! {
                        <div class="text-center py-16">
                            <div class="text-6xl mb-4">"🔍"</div>
                            <h3 class="text-xl font-semibold text-primary mb-2">"No tools found"</h3>
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

/// Tool section component (rendered within the island)
#[component]
fn ToolSection(category: ToolCategory, items: Vec<ToolItem>, index: usize) -> impl IntoView {
    let section_id = category.id();
    let section_title = category.full_label();
    let description = category.description();

    // Different grid layouts for different categories
    let grid_class = match category {
        ToolCategory::ProgrammingLanguages => "grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4",
        ToolCategory::RPackages => "grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4",
        ToolCategory::IDEs => "grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4",
        ToolCategory::VersionControl => "grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4",
        ToolCategory::DataFormats => "grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4",
        ToolCategory::Notebooks => "grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-3 gap-4",
        ToolCategory::Databases => "grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4",
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
                    let large_logo = matches!(category, ToolCategory::Notebooks);
                    view! { <ToolCard item=item large_logo=large_logo /> }
                }).collect_view()}
            </div>
        </section>
    }
}

/// Individual tool card component
#[component]
fn ToolCard(item: ToolItem, #[prop(default = false)] large_logo: bool) -> impl IntoView {
    let logo_url = item.logo.clone().unwrap_or_default();
    let has_logo = !logo_url.is_empty();
    let first_char = item.title.chars().next().unwrap_or('?');
    let is_featured = item.is_featured == Some(true);

    let logo_class = if large_logo {
        "aspect-[4/3] w-full logo-tile logo-tile-lg"
    } else {
        "aspect-[4/3] w-full logo-tile"
    };

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
            <div class=logo_class>
                {if has_logo {
                    view! {
                        <img
                            src=logo_url
                            alt=item.title.clone()
                            class="group-hover:scale-110 transition-transform"
                            loading="lazy"
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="logo-placeholder">
                            <span>{first_char}</span>
                        </div>
                    }.into_any()
                }}
            </div>
            <div class="p-4">
                <ToolBadges
                    level=item.level.clone().unwrap_or_default()
                    is_open_source=item.is_open_source.unwrap_or(false)
                    is_featured=item.is_featured.unwrap_or(false)
                />
                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors text-center mb-2">
                    {item.title}
                </h3>
                <p class="text-sm text-secondary text-center">
                    {item.description}
                </p>
            </div>
        </a>
    }
}
