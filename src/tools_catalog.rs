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
            ToolCategory::ProgrammingLanguages => "Core languages for statistical computing and data analysis.",
            ToolCategory::RPackages => "R packages for mixed models, SEM, growth curves, and missing data handling.",
            ToolCategory::IDEs => "Editors and IDEs for writing and debugging code.",
            ToolCategory::VersionControl => "Tools for version control, reproducibility, and workflow management.",
            ToolCategory::DataFormats => "Common file formats for storing and exchanging data.",
            ToolCategory::Notebooks => "Interactive environments for reproducible research.",
            ToolCategory::Databases => "Systems for storing and querying structured data.",
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

/// Main tools catalog island that orchestrates filtering
#[island]
pub fn ToolsCatalogIsland(tools: Vec<ToolItem>) -> impl IntoView {
    let search_query = RwSignal::new(String::new());
    let selected_categories = RwSignal::new(Vec::<ToolCategory>::new());

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
        tools_signal
            .get_value()
            .into_iter()
            .filter(|t| matches_search(t, &query) && matches_category(t, &categories))
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

    // Alternating backgrounds: odd indices get bg-subtle
    let section_class = if index % 2 == 1 {
        "py-12 -mx-4 sm:-mx-6 lg:-mx-8 px-4 sm:px-6 lg:px-8 bg-subtle"
    } else {
        "py-12"
    };

    view! {
        <section id=section_id class=section_class>
            <div class="mb-6">
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

    let logo_class = if large_logo {
        "aspect-[4/3] w-full logo-tile logo-tile-lg"
    } else {
        "aspect-[4/3] w-full logo-tile"
    };

    view! {
        <a
            href=item.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class="resource-card group block"
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
