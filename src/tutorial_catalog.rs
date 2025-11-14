//! Tutorial catalog with search, filtering, and sorting capabilities.
//!
//! This module provides an interactive catalog view for browsing tutorials with:
//! - Real-time search across title, description, and tags
//! - Method family filtering
//! - Sort options (newest, A-Z)
//! - Responsive grid layout

use crate::base_path;
use crate::models::post::Post;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortOption {
    Newest,
    TitleAZ,
    TitleZA,
    FamilyAZ,
    FamilyZA,
    EngineAZ,
    EngineZA,
    CovariatesAZ,
    CovariatesZA,
    OutcomeAZ,
    OutcomeZA,
}

impl SortOption {
    pub fn label(&self) -> &'static str {
        match self {
            SortOption::Newest => "Newest",
            SortOption::TitleAZ => "A-Z",
            SortOption::TitleZA => "Z-A",
            SortOption::FamilyAZ => "Family A-Z",
            SortOption::FamilyZA => "Family Z-A",
            SortOption::EngineAZ => "Engine A-Z",
            SortOption::EngineZA => "Engine Z-A",
            SortOption::CovariatesAZ => "Covariates A-Z",
            SortOption::CovariatesZA => "Covariates Z-A",
            SortOption::OutcomeAZ => "Outcome A-Z",
            SortOption::OutcomeZA => "Outcome Z-A",
        }
    }
}

/// Catalog data for a single tutorial (serializable)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TutorialData {
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub method_family: String,
    pub statistical_engine: String,
    pub covariates: String,
    pub outcome_type: String,
    pub updated_at: String,
    pub author: String,
    pub tags: Vec<String>,
}

impl TutorialData {
    pub fn from_post(post: &Post) -> Self {
        let summary = post
            .overview
            .summary_paragraphs
            .first()
            .map(|s| {
                let text = s.to_string();
                if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text
                }
            })
            .unwrap_or_default();

        let metadata = post.metadata.as_ref();

        Self {
            slug: post.slug.to_string(),
            title: post.title.to_string(),
            summary,
            method_family: metadata
                .map(|m| m.method_family.clone())
                .unwrap_or_default(),
            statistical_engine: metadata
                .map(|m| m.statistical_engine.clone())
                .unwrap_or_default(),
            covariates: metadata.map(|m| m.covariates.clone()).unwrap_or_default(),
            outcome_type: metadata.map(|m| m.outcome_type.clone()).unwrap_or_default(),
            updated_at: metadata.map(|m| m.updated_at.clone()).unwrap_or_default(),
            author: metadata.map(|m| m.author.clone()).unwrap_or_default(),
            tags: metadata.map(|m| m.tags.clone()).unwrap_or_default(),
        }
    }
}

/// View mode for tutorial display
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ViewMode {
    Cards,
    Table,
}

/// Get unique method families from tutorial data
fn get_method_families(tutorials: &[TutorialData]) -> Vec<(String, usize)> {
    let mut families: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for tutorial in tutorials {
        *families.entry(tutorial.method_family.clone()).or_insert(0) += 1;
    }

    let mut result: Vec<_> = families.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Get unique statistical engines from tutorial data
fn get_statistical_engines(tutorials: &[TutorialData]) -> Vec<(String, usize)> {
    let mut engines: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for tutorial in tutorials {
        *engines
            .entry(tutorial.statistical_engine.clone())
            .or_insert(0) += 1;
    }

    let mut result: Vec<_> = engines.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Get unique covariate types from tutorial data
fn get_covariates(tutorials: &[TutorialData]) -> Vec<(String, usize)> {
    let mut covariates: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for tutorial in tutorials {
        *covariates.entry(tutorial.covariates.clone()).or_insert(0) += 1;
    }

    let mut result: Vec<_> = covariates.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Search bar island component
#[island]
pub fn SearchBar(search_query: RwSignal<String>) -> impl IntoView {
    view! {
        <input
            type="text"
            placeholder="Search tutorials..."
            class="w-full px-4 py-3 rounded-lg border border-stroke bg-surface text-primary placeholder:text-muted focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors"
            on:input=move |ev| search_query.set(event_target_value(&ev))
            prop:value=move || search_query.get()
        />
    }
}

/// Method family tabs component
#[island]
pub fn MethodTabs(
    selected_method: RwSignal<Option<String>>,
    method_families: Vec<(String, usize)>,
) -> impl IntoView {
    let total_count = method_families
        .iter()
        .map(|(_, count)| count)
        .sum::<usize>();

    view! {
        <div class="flex flex-wrap gap-2">
            // "All Methods" tab
            <button
                class=move || {
                    let base = "px-4 py-2 rounded-lg font-medium transition-all duration-200";
                    if selected_method.get().is_none() {
                        format!("{base} bg-accent text-white")
                    } else {
                        format!("{base} bg-elevated border border-stroke text-secondary hover:bg-subtle hover:text-primary")
                    }
                }
                on:click=move |_| selected_method.set(None)
            >
                "All Methods (" {total_count} ")"
            </button>

            // Individual method tabs
            {method_families.into_iter().map(|(family, count)| {
                let family_clone = family.clone();
                view! {
                    <button
                        class=move || {
                            let base = "px-4 py-2 rounded-lg font-medium transition-all duration-200";
                            if selected_method.get() == Some(family_clone.clone()) {
                                format!("{base} bg-accent text-white")
                            } else {
                                format!("{base} bg-elevated border border-stroke text-secondary hover:bg-subtle hover:text-primary")
                            }
                        }
                        on:click=move |_| selected_method.set(Some(family.clone()))
                    >
                        {family.clone()} " (" {count} ")"
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

/// View toggle component
#[island]
pub fn ViewToggle(view_mode: RwSignal<ViewMode>) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-secondary">"View:"</span>
            <div class="flex rounded-lg border border-stroke bg-surface overflow-hidden">
                <button
                    class=move || {
                        let base = "px-4 py-2 text-sm font-medium transition-colors";
                        if view_mode.get() == ViewMode::Cards {
                            format!("{base} bg-accent text-white")
                        } else {
                            format!("{base} text-secondary hover:bg-subtle")
                        }
                    }
                    on:click=move |_| view_mode.set(ViewMode::Cards)
                >
                    "Cards"
                </button>
                <button
                    class=move || {
                        let base = "px-4 py-2 text-sm font-medium transition-colors border-l border-stroke";
                        if view_mode.get() == ViewMode::Table {
                            format!("{base} bg-accent text-white")
                        } else {
                            format!("{base} text-secondary hover:bg-subtle")
                        }
                    }
                    on:click=move |_| view_mode.set(ViewMode::Table)
                >
                    "Table"
                </button>
            </div>
        </div>
    }
}

/// Sort dropdown component
#[island]
pub fn SortDropdown(sort_by: RwSignal<SortOption>) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-secondary">"Sort:"</span>
            <select
                class="px-3 py-2 rounded-lg border border-stroke bg-surface text-primary focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors cursor-pointer"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    let option = match value.as_str() {
                        "newest" => SortOption::Newest,
                        "a-z" => SortOption::TitleAZ,
                        _ => SortOption::Newest,
                    };
                    sort_by.set(option);
                }
            >
                <option value="newest" selected=move || sort_by.get() == SortOption::Newest>
                    "Newest"
                </option>
                <option value="a-z" selected=move || sort_by.get() == SortOption::TitleAZ>
                    "A-Z"
                </option>
            </select>
        </div>
    }
}

/// Sidebar filters component
#[island]
pub fn SidebarFilters(
    selected_families: RwSignal<Vec<String>>,
    selected_engines: RwSignal<Vec<String>>,
    selected_covariates: RwSignal<Vec<String>>,
    method_families: Vec<(String, usize)>,
    statistical_engines: Vec<(String, usize)>,
    covariates: Vec<(String, usize)>,
) -> impl IntoView {
    view! {
        <div class="bg-elevated border border-stroke rounded-xl p-6 space-y-6">
            <h2 class="text-lg font-semibold text-primary">"Filters"</h2>

            // Method Family filters
            <div class="space-y-3">
                <h3 class="text-sm font-semibold text-primary">"Method Family"</h3>
                <div class="space-y-2">
                    {method_families.into_iter().map(|(family, count)| {
                        let family_for_checked = family.clone();
                        let family_for_change = family.clone();
                        let family_for_display = family.clone();
                        view! {
                            <label class="flex items-center gap-2 cursor-pointer group">
                                <input
                                    type="checkbox"
                                    class="w-4 h-4 rounded border-stroke text-accent focus:ring-2 focus:ring-accent focus:ring-offset-0 cursor-pointer"
                                    checked=move || selected_families.get().contains(&family_for_checked)
                                    on:change=move |_| {
                                        let mut current = selected_families.get();
                                        if current.contains(&family_for_change) {
                                            current.retain(|f| f != &family_for_change);
                                        } else {
                                            current.push(family_for_change.clone());
                                        }
                                        selected_families.set(current);
                                    }
                                />
                                <span class="text-sm text-secondary group-hover:text-primary transition-colors">
                                    {family_for_display} " (" {count} ")"
                                </span>
                            </label>
                        }
                    }).collect_view()}
                </div>
            </div>

            // Statistical Engine filters
            <div class="space-y-3">
                <h3 class="text-sm font-semibold text-primary">"Statistical Engine"</h3>
                <div class="space-y-2">
                    {statistical_engines.into_iter().map(|(engine, count)| {
                        let engine_for_checked = engine.clone();
                        let engine_for_change = engine.clone();
                        let engine_for_display = engine.clone();
                        view! {
                            <label class="flex items-center gap-2 cursor-pointer group">
                                <input
                                    type="checkbox"
                                    class="w-4 h-4 rounded border-stroke text-accent focus:ring-2 focus:ring-accent focus:ring-offset-0 cursor-pointer"
                                    checked=move || selected_engines.get().contains(&engine_for_checked)
                                    on:change=move |_| {
                                        let mut current = selected_engines.get();
                                        if current.contains(&engine_for_change) {
                                            current.retain(|e| e != &engine_for_change);
                                        } else {
                                            current.push(engine_for_change.clone());
                                        }
                                        selected_engines.set(current);
                                    }
                                />
                                <span class="text-sm text-secondary group-hover:text-primary transition-colors">
                                    {engine_for_display} " (" {count} ")"
                                </span>
                            </label>
                        }
                    }).collect_view()}
                </div>
            </div>

            // Covariates filters
            <div class="space-y-3">
                <h3 class="text-sm font-semibold text-primary">"Covariates"</h3>
                <div class="space-y-2">
                    {covariates.into_iter().map(|(covariate, count)| {
                        let covariate_for_checked = covariate.clone();
                        let covariate_for_change = covariate.clone();
                        let covariate_for_display = covariate.clone();
                        view! {
                            <label class="flex items-center gap-2 cursor-pointer group">
                                <input
                                    type="checkbox"
                                    class="w-4 h-4 rounded border-stroke text-accent focus:ring-2 focus:ring-accent focus:ring-offset-0 cursor-pointer"
                                    checked=move || selected_covariates.get().contains(&covariate_for_checked)
                                    on:change=move |_| {
                                        let mut current = selected_covariates.get();
                                        if current.contains(&covariate_for_change) {
                                            current.retain(|c| c != &covariate_for_change);
                                        } else {
                                            current.push(covariate_for_change.clone());
                                        }
                                        selected_covariates.set(current);
                                    }
                                />
                                <span class="text-sm text-secondary group-hover:text-primary transition-colors">
                                    {covariate_for_display} " (" {count} ")"
                                </span>
                            </label>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

/// Tutorial card component (non-island, server-rendered)
#[component]
pub fn TutorialCard(tutorial: TutorialData) -> impl IntoView {
    let href = base_path::join(&format!("posts/{}/", tutorial.slug));

    view! {
        <a
            href={href}
            class="group block rounded-xl transition-all duration-200 hover:scale-102 hover:shadow-xl bg-elevated border border-stroke p-6"
        >
            <div class="flex items-center gap-2 mb-2">
                <span class="text-xs px-2 py-0.5 rounded-full bg-accent/10 text-accent border border-accent/20">
                    "New"
                </span>
                <span class="text-xs text-muted">
                    "Updated " {tutorial.updated_at.clone()}
                </span>
            </div>
            <h3 class="text-lg font-semibold group-hover:underline group-hover:text-accent transition-colors duration-200 text-primary">
                {tutorial.title.clone()}
            </h3>
            <p class="mt-2 text-sm text-secondary">
                {tutorial.summary.clone()}
            </p>
            <div class="mt-4 flex flex-wrap gap-2">
                <span class="px-3 py-1.5 rounded-full text-xs font-medium bg-accent/5 text-accent border border-accent/20">
                    {tutorial.method_family.clone()}
                </span>
                <span class="px-3 py-1.5 rounded-full text-xs font-medium bg-info-bg text-info border border-info">
                    {tutorial.statistical_engine.clone()}
                </span>
                {if tutorial.covariates != "None" {
                    Some(view! {
                        <span class="px-3 py-1.5 rounded-full text-xs font-medium bg-warning-bg text-warning border border-warning">
                            {tutorial.covariates.clone()}
                        </span>
                    })
                } else {
                    None
                }}
                {if tutorial.outcome_type != "None" {
                    Some(view! {
                        <span class="px-3 py-1.5 rounded-full text-xs font-medium bg-amber-50 text-amber-600 border border-amber-200 dark:bg-amber-900/20 dark:text-amber-400 dark:border-amber-800">
                            {tutorial.outcome_type.clone()}
                        </span>
                    })
                } else {
                    None
                }}
            </div>
        </a>
    }
}

/// Table view component with sortable headers
#[island]
pub fn TutorialTable(tutorials: Vec<TutorialData>, sort_by: RwSignal<SortOption>) -> impl IntoView {
    // Helper function to get sort indicator
    let sort_indicator = move |column_asc: SortOption, column_desc: SortOption| -> &'static str {
        let current = sort_by.get();
        if current == column_asc {
            " ▲"
        } else if current == column_desc {
            " ▼"
        } else {
            ""
        }
    };

    view! {
        <div class="overflow-x-auto rounded-xl border border-stroke bg-elevated">
            <table class="w-full">
                <thead class="bg-subtle border-b border-stroke">
                    <tr>
                        <th class="px-4 py-3 text-left text-sm font-semibold text-primary">
                            <button
                                class="hover:text-accent transition-colors cursor-pointer"
                                on:click=move |_| {
                                    let current = sort_by.get();
                                    if current == SortOption::TitleAZ {
                                        sort_by.set(SortOption::TitleZA);
                                    } else {
                                        sort_by.set(SortOption::TitleAZ);
                                    }
                                }
                            >
                                "Tutorial" {move || sort_indicator(SortOption::TitleAZ, SortOption::TitleZA)}
                            </button>
                        </th>
                        <th class="px-4 py-3 text-left text-sm font-semibold text-primary">
                            <button
                                class="hover:text-accent transition-colors cursor-pointer"
                                on:click=move |_| {
                                    let current = sort_by.get();
                                    if current == SortOption::FamilyAZ {
                                        sort_by.set(SortOption::FamilyZA);
                                    } else {
                                        sort_by.set(SortOption::FamilyAZ);
                                    }
                                }
                            >
                                "Family" {move || sort_indicator(SortOption::FamilyAZ, SortOption::FamilyZA)}
                            </button>
                        </th>
                        <th class="px-4 py-3 text-left text-sm font-semibold text-primary">
                            <button
                                class="hover:text-accent transition-colors cursor-pointer"
                                on:click=move |_| {
                                    let current = sort_by.get();
                                    if current == SortOption::EngineAZ {
                                        sort_by.set(SortOption::EngineZA);
                                    } else {
                                        sort_by.set(SortOption::EngineAZ);
                                    }
                                }
                            >
                                "Engine" {move || sort_indicator(SortOption::EngineAZ, SortOption::EngineZA)}
                            </button>
                        </th>
                        <th class="px-4 py-3 text-left text-sm font-semibold text-primary">
                            <button
                                class="hover:text-accent transition-colors cursor-pointer"
                                on:click=move |_| {
                                    let current = sort_by.get();
                                    if current == SortOption::CovariatesAZ {
                                        sort_by.set(SortOption::CovariatesZA);
                                    } else {
                                        sort_by.set(SortOption::CovariatesAZ);
                                    }
                                }
                            >
                                "Covariates" {move || sort_indicator(SortOption::CovariatesAZ, SortOption::CovariatesZA)}
                            </button>
                        </th>
                        <th class="px-4 py-3 text-left text-sm font-semibold text-primary">
                            <button
                                class="hover:text-accent transition-colors cursor-pointer"
                                on:click=move |_| {
                                    let current = sort_by.get();
                                    if current == SortOption::OutcomeAZ {
                                        sort_by.set(SortOption::OutcomeZA);
                                    } else {
                                        sort_by.set(SortOption::OutcomeAZ);
                                    }
                                }
                            >
                                "Outcome" {move || sort_indicator(SortOption::OutcomeAZ, SortOption::OutcomeZA)}
                            </button>
                        </th>
                        <th class="px-4 py-3 text-left text-sm font-semibold text-primary">
                            <button
                                class="hover:text-accent transition-colors cursor-pointer"
                                on:click=move |_| {
                                    sort_by.set(SortOption::Newest);
                                }
                            >
                                "Updated" {move || if sort_by.get() == SortOption::Newest { " ▼" } else { "" }}
                            </button>
                        </th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-stroke">
                    {tutorials.into_iter().map(|tutorial| {
                        let href = base_path::join(&format!("posts/{}/", tutorial.slug));
                        view! {
                            <tr class="hover:bg-subtle transition-colors">
                                <td class="px-4 py-3">
                                    <a href={href} class="text-primary hover:text-accent font-medium hover:underline">
                                        {tutorial.title}
                                    </a>
                                </td>
                                <td class="px-4 py-3">
                                    <span class="inline-flex px-2 py-1 rounded-full text-xs font-medium bg-accent/10 text-accent border border-accent/20">
                                        {tutorial.method_family}
                                    </span>
                                </td>
                                <td class="px-4 py-3">
                                    <span class="inline-flex px-2 py-1 rounded-full text-xs font-medium bg-info-bg text-info border border-info">
                                        {tutorial.statistical_engine}
                                    </span>
                                </td>
                                <td class="px-4 py-3">
                                    {if tutorial.covariates != "None" {
                                        Some(view! {
                                            <span class="inline-flex px-2 py-1 rounded-full text-xs font-medium bg-warning-bg text-warning border border-warning">
                                                {tutorial.covariates.clone()}
                                            </span>
                                        })
                                    } else {
                                        None
                                    }}
                                </td>
                                <td class="px-4 py-3">
                                    {if tutorial.outcome_type != "None" {
                                        Some(view! {
                                            <span class="inline-flex px-2 py-1 rounded-full text-xs font-medium bg-amber-50 text-amber-600 border border-amber-200 dark:bg-amber-900/20 dark:text-amber-400 dark:border-amber-800">
                                                {tutorial.outcome_type.clone()}
                                            </span>
                                        })
                                    } else {
                                        None
                                    }}
                                </td>
                                <td class="px-4 py-3 text-sm text-secondary">
                                    {tutorial.updated_at}
                                </td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

/// Main tutorial catalog island with filtering logic
#[island]
pub fn TutorialCatalog(tutorials: Vec<TutorialData>) -> impl IntoView {
    // Filter state
    let search_query = RwSignal::new(String::new());
    let selected_method = RwSignal::new(None::<String>);
    let selected_families = RwSignal::new(Vec::<String>::new());
    let selected_engines = RwSignal::new(Vec::<String>::new());
    let selected_covariates = RwSignal::new(Vec::<String>::new());
    let sort_by = RwSignal::new(SortOption::Newest);
    let view_mode = RwSignal::new(ViewMode::Cards);

    // Get unique values for filters
    let method_families = get_method_families(&tutorials);
    let statistical_engines = get_statistical_engines(&tutorials);
    let covariates = get_covariates(&tutorials);

    // Computed: filtered and sorted tutorials
    let filtered_tutorials = Memo::new(move |_| {
        let query = search_query.get().to_lowercase();
        let selected = selected_method.get();
        let families = selected_families.get();
        let engines = selected_engines.get();
        let covs = selected_covariates.get();
        let sort = sort_by.get();

        let mut filtered: Vec<TutorialData> = tutorials
            .iter()
            .filter(|tutorial| {
                // Search filter
                if !query.is_empty() {
                    let title_match = tutorial.title.to_lowercase().contains(&query);
                    let summary_match = tutorial.summary.to_lowercase().contains(&query);
                    let tags_match = tutorial
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query));

                    if !title_match && !summary_match && !tags_match {
                        return false;
                    }
                }

                // Method family tab filter (from horizontal tabs)
                if let Some(method) = &selected {
                    if &tutorial.method_family != method {
                        return false;
                    }
                }

                // Sidebar filters
                if !families.is_empty() && !families.contains(&tutorial.method_family) {
                    return false;
                }

                if !engines.is_empty() && !engines.contains(&tutorial.statistical_engine) {
                    return false;
                }

                if !covs.is_empty() && !covs.contains(&tutorial.covariates) {
                    return false;
                }

                true
            })
            .cloned()
            .collect();

        // Sort
        match sort {
            SortOption::Newest => {
                filtered.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)); // Newest first
            }
            SortOption::TitleAZ => {
                filtered.sort_by(|a, b| a.title.cmp(&b.title));
            }
            SortOption::TitleZA => {
                filtered.sort_by(|a, b| b.title.cmp(&a.title));
            }
            SortOption::FamilyAZ => {
                filtered.sort_by(|a, b| a.method_family.cmp(&b.method_family));
            }
            SortOption::FamilyZA => {
                filtered.sort_by(|a, b| b.method_family.cmp(&a.method_family));
            }
            SortOption::EngineAZ => {
                filtered.sort_by(|a, b| a.statistical_engine.cmp(&b.statistical_engine));
            }
            SortOption::EngineZA => {
                filtered.sort_by(|a, b| b.statistical_engine.cmp(&a.statistical_engine));
            }
            SortOption::CovariatesAZ => {
                filtered.sort_by(|a, b| a.covariates.cmp(&b.covariates));
            }
            SortOption::CovariatesZA => {
                filtered.sort_by(|a, b| b.covariates.cmp(&a.covariates));
            }
            SortOption::OutcomeAZ => {
                filtered.sort_by(|a, b| a.outcome_type.cmp(&b.outcome_type));
            }
            SortOption::OutcomeZA => {
                filtered.sort_by(|a, b| b.outcome_type.cmp(&a.outcome_type));
            }
        }

        filtered
    });

    view! {
        <div class="flex flex-col lg:flex-row gap-6">
            // Left sidebar filters
            <div class="lg:w-64 flex-shrink-0">
                <SidebarFilters
                    selected_families
                    selected_engines
                    selected_covariates
                    method_families=method_families.clone()
                    statistical_engines=statistical_engines.clone()
                    covariates=covariates.clone()
                />
            </div>

            // Main content area
            <div class="flex-1 space-y-6">
                // Search bar
                <div class="bg-elevated border border-stroke rounded-xl p-4">
                    <SearchBar search_query />
                </div>

                // Controls: Method tabs, View toggle, and Sort
                <div class="flex flex-col gap-4">
                    <MethodTabs selected_method method_families />
                    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                        <ViewToggle view_mode />
                        <SortDropdown sort_by />
                    </div>
                </div>

                // Results header
                <div class="flex items-center justify-between">
                    <h2 class="text-2xl font-semibold text-primary">"All Tutorials"</h2>
                    <div class="text-sm text-muted">
                        "Showing " {move || filtered_tutorials.get().len()} " tutorials"
                    </div>
                </div>

                // Content display (cards or table)
                {move || {
                    let tutorials = filtered_tutorials.get();
                    match view_mode.get() {
                        ViewMode::Cards => view! {
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                                {tutorials.into_iter().map(|tutorial| {
                                    view! {
                                        <TutorialCard tutorial />
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any(),
                        ViewMode::Table => view! {
                            <TutorialTable tutorials sort_by />
                        }.into_any(),
                    }
                }}
            </div>
        </div>
    }
}
