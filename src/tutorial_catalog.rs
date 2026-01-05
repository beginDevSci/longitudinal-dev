//! Tutorial catalog with search, filtering, pagination, and sorting.
//!
//! This module provides an interactive catalog view for browsing tutorials with:
//! - Client-side fetch of tutorial index from /api/tutorial_index.json
//! - Paginated results (24 per page by default)
//! - Ranked search with match highlighting
//! - Scalable filter facets with active chips
//! - Backward compatibility with embedded data

use crate::base_path;
use crate::index_generator::TutorialIndexEntry;
use crate::models::post::Post;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Page size for pagination
const PAGE_SIZE: usize = 24;

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
/// Used for backward compatibility with embedded data mode
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
    // Extended fields from index (optional for backward compat)
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub search_text: Option<String>,
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
            summary: summary.clone(),
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
            difficulty: metadata.and_then(|m| m.difficulty.clone()),
            search_text: None, // Not precomputed for embedded mode
        }
    }
}

impl From<TutorialIndexEntry> for TutorialData {
    fn from(entry: TutorialIndexEntry) -> Self {
        Self {
            slug: entry.slug,
            title: entry.title,
            summary: entry.summary,
            method_family: entry.method_family,
            statistical_engine: entry.statistical_engine,
            covariates: entry.covariates,
            outcome_type: entry.outcome_type,
            updated_at: entry.updated_at,
            author: entry.author,
            tags: entry.tags,
            difficulty: entry.difficulty,
            search_text: Some(entry.search_text),
        }
    }
}

/// View mode for tutorial display
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ViewMode {
    Cards,
    Table,
}

/// Data loading state
#[derive(Debug, Clone, PartialEq)]
pub enum LoadState {
    Loading,
    Loaded(Vec<TutorialData>),
    Error(String),
}

/// Active filter representation for chips display
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveFilter {
    pub category: String,
    pub value: String,
}

/// Facet counts type alias
type FacetCounts = Vec<(String, usize)>;

/// Compute facet counts from tutorials
fn compute_facets(tutorials: &[TutorialData]) -> (FacetCounts, FacetCounts, FacetCounts, FacetCounts) {
    let mut families: HashMap<String, usize> = HashMap::new();
    let mut engines: HashMap<String, usize> = HashMap::new();
    let mut covariates: HashMap<String, usize> = HashMap::new();
    let mut difficulties: HashMap<String, usize> = HashMap::new();

    for tutorial in tutorials {
        *families.entry(tutorial.method_family.clone()).or_insert(0) += 1;
        *engines.entry(tutorial.statistical_engine.clone()).or_insert(0) += 1;
        *covariates.entry(tutorial.covariates.clone()).or_insert(0) += 1;
        if let Some(ref diff) = tutorial.difficulty {
            *difficulties.entry(diff.clone()).or_insert(0) += 1;
        }
    }

    let mut families: Vec<_> = families.into_iter().collect();
    let mut engines: Vec<_> = engines.into_iter().collect();
    let mut covariates: Vec<_> = covariates.into_iter().collect();
    let mut difficulties: Vec<_> = difficulties.into_iter().collect();

    families.sort_by(|a, b| a.0.cmp(&b.0));
    engines.sort_by(|a, b| a.0.cmp(&b.0));
    covariates.sort_by(|a, b| a.0.cmp(&b.0));
    difficulties.sort_by(|a, b| a.0.cmp(&b.0));

    (families, engines, covariates, difficulties)
}

/// Rank a tutorial based on search query match quality
/// Returns (score, title_highlighted, summary_highlighted)
fn rank_and_highlight(tutorial: &TutorialData, query: &str) -> (i32, String, String) {
    if query.is_empty() {
        return (0, tutorial.title.clone(), tutorial.summary.clone());
    }

    let query_lower = query.to_lowercase();
    let title_lower = tutorial.title.to_lowercase();
    let summary_lower = tutorial.summary.to_lowercase();

    let mut score = 0i32;
    let mut title_highlighted = tutorial.title.clone();
    let mut summary_highlighted = tutorial.summary.clone();

    // Title exact match: highest score
    if title_lower == query_lower {
        score += 100;
    } else if title_lower.starts_with(&query_lower) {
        score += 80;
    } else if title_lower.contains(&query_lower) {
        score += 60;
    }

    // Highlight title matches (case-insensitive)
    if let Some(pos) = title_lower.find(&query_lower) {
        let end = pos + query.len();
        let before = &tutorial.title[..pos];
        let matched = &tutorial.title[pos..end];
        let after = &tutorial.title[end..];
        title_highlighted = format!("{}<mark class=\"bg-yellow-200 dark:bg-yellow-800\">{}</mark>{}", before, matched, after);
    }

    // Summary match
    if summary_lower.contains(&query_lower) {
        score += 30;
        // Highlight summary matches
        if let Some(pos) = summary_lower.find(&query_lower) {
            let end = pos + query.len();
            let before = &tutorial.summary[..pos];
            let matched = &tutorial.summary[pos..end];
            let after = &tutorial.summary[end..];
            summary_highlighted = format!("{}<mark class=\"bg-yellow-200 dark:bg-yellow-800\">{}</mark>{}", before, matched, after);
        }
    }

    // Tag match
    for tag in &tutorial.tags {
        if tag.to_lowercase().contains(&query_lower) {
            score += 20;
            break;
        }
    }

    // Method family match
    if tutorial.method_family.to_lowercase().contains(&query_lower) {
        score += 15;
    }

    // Use precomputed search_text if available
    if let Some(ref search_text) = tutorial.search_text {
        if search_text.contains(&query_lower) && score == 0 {
            score += 10;
        }
    }

    (score, title_highlighted, summary_highlighted)
}

/// Search bar island component
#[island]
pub fn SearchBar(search_query: RwSignal<String>, current_page: RwSignal<usize>) -> impl IntoView {
    view! {
        <div class="relative">
            <input
                type="text"
                placeholder="Search tutorials by title, tags, or content..."
                class="w-full px-4 py-3 pl-10 rounded-lg border border-stroke bg-surface text-primary placeholder:text-muted focus:outline-none focus:ring-2 focus:ring-accent focus:border-accent transition-colors"
                on:input=move |ev| {
                    search_query.set(event_target_value(&ev));
                    current_page.set(1); // Reset to page 1 on search
                }
                prop:value=move || search_query.get()
            />
            <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
            </svg>
        </div>
    }
}

/// Active filter chips component
#[island]
pub fn ActiveFilterChips(
    selected_families: RwSignal<Vec<String>>,
    selected_engines: RwSignal<Vec<String>>,
    selected_covariates: RwSignal<Vec<String>>,
    current_page: RwSignal<usize>,
) -> impl IntoView {
    let has_filters = move || {
        !selected_families.get().is_empty()
            || !selected_engines.get().is_empty()
            || !selected_covariates.get().is_empty()
    };

    view! {
        <Show when=has_filters>
            <div class="flex flex-wrap items-center gap-2 p-3 bg-subtle rounded-lg border border-stroke">
                <span class="text-sm font-medium text-secondary">"Active filters:"</span>

                // Family chips
                {move || selected_families.get().into_iter().map(|family| {
                    let family_clone = family.clone();
                    view! {
                        <span class="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-accent/10 text-accent border border-accent/20">
                            {family.clone()}
                            <button
                                class="hover:bg-accent/20 rounded-full p-0.5"
                                on:click=move |_| {
                                    let mut current = selected_families.get();
                                    current.retain(|f| f != &family_clone);
                                    selected_families.set(current);
                                    current_page.set(1);
                                }
                            >
                                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                </svg>
                            </button>
                        </span>
                    }
                }).collect_view()}

                // Engine chips
                {move || selected_engines.get().into_iter().map(|engine| {
                    let engine_clone = engine.clone();
                    view! {
                        <span class="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-info-bg text-info border border-info">
                            {engine.clone()}
                            <button
                                class="hover:bg-info/20 rounded-full p-0.5"
                                on:click=move |_| {
                                    let mut current = selected_engines.get();
                                    current.retain(|e| e != &engine_clone);
                                    selected_engines.set(current);
                                    current_page.set(1);
                                }
                            >
                                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                </svg>
                            </button>
                        </span>
                    }
                }).collect_view()}

                // Covariates chips
                {move || selected_covariates.get().into_iter().map(|cov| {
                    let cov_clone = cov.clone();
                    view! {
                        <span class="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-warning-bg text-warning border border-warning">
                            {cov.clone()}
                            <button
                                class="hover:bg-warning/20 rounded-full p-0.5"
                                on:click=move |_| {
                                    let mut current = selected_covariates.get();
                                    current.retain(|c| c != &cov_clone);
                                    selected_covariates.set(current);
                                    current_page.set(1);
                                }
                            >
                                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                </svg>
                            </button>
                        </span>
                    }
                }).collect_view()}

                // Clear all button
                <button
                    class="ml-2 text-xs text-muted hover:text-primary underline"
                    on:click=move |_| {
                        selected_families.set(vec![]);
                        selected_engines.set(vec![]);
                        selected_covariates.set(vec![]);
                        current_page.set(1);
                    }
                >
                    "Clear all"
                </button>
            </div>
        </Show>
    }
}

/// Pagination controls component
#[island]
pub fn PaginationControls(
    current_page: RwSignal<usize>,
    total_count: usize,
) -> impl IntoView {
    let total_pages = total_count.div_ceil(PAGE_SIZE);

    view! {
        <Show when=move || { total_pages > 1 }>
            <div class="flex items-center justify-center gap-2 mt-8">
                // Previous button
                <button
                    class="px-3 py-2 rounded-lg border border-stroke bg-surface text-primary hover:bg-subtle disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    disabled=move || current_page.get() == 1
                    on:click=move |_| {
                        let page = current_page.get();
                        if page > 1 {
                            current_page.set(page - 1);
                        }
                    }
                >
                    "← Previous"
                </button>

                // Page numbers
                <div class="flex items-center gap-1">
                    {move || {
                        let current = current_page.get();

                        // Show up to 5 page numbers centered on current
                        let start = if current <= 3 { 1 } else { current - 2 };
                        let end = std::cmp::min(start + 4, total_pages);
                        let start = if end - start < 4 && end > 4 { end - 4 } else { start };

                        (start..=end).map(|page| {
                            let is_current = page == current;
                            view! {
                                <button
                                    class=move || {
                                        if is_current {
                                            "px-3 py-2 rounded-lg bg-accent text-white font-medium"
                                        } else {
                                            "px-3 py-2 rounded-lg border border-stroke bg-surface text-primary hover:bg-subtle transition-colors"
                                        }
                                    }
                                    on:click=move |_| current_page.set(page)
                                >
                                    {page}
                                </button>
                            }
                        }).collect_view()
                    }}
                </div>

                // Next button
                <button
                    class="px-3 py-2 rounded-lg border border-stroke bg-surface text-primary hover:bg-subtle disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    disabled=move || current_page.get() >= total_pages
                    on:click=move |_| {
                        let page = current_page.get();
                        if page < total_pages {
                            current_page.set(page + 1);
                        }
                    }
                >
                    "Next →"
                </button>
            </div>
        </Show>
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
                        "z-a" => SortOption::TitleZA,
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
                <option value="z-a" selected=move || sort_by.get() == SortOption::TitleZA>
                    "Z-A"
                </option>
            </select>
        </div>
    }
}

/// Collapsible filter section
#[island]
pub fn FilterSection(
    title: String,
    items: Vec<(String, usize)>,
    selected: RwSignal<Vec<String>>,
    current_page: RwSignal<usize>,
) -> impl IntoView {
    let is_collapsed = RwSignal::new(false);

    view! {
        <div class="space-y-3">
            <button
                class="flex items-center justify-between w-full text-sm font-semibold text-primary hover:text-accent"
                on:click=move |_| is_collapsed.update(|v| *v = !*v)
            >
                <span>{title}</span>
                <svg
                    class=move || format!("w-4 h-4 transition-transform {}", if is_collapsed.get() { "" } else { "rotate-180" })
                    fill="none" stroke="currentColor" viewBox="0 0 24 24"
                >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                </svg>
            </button>

            <Show when=move || !is_collapsed.get()>
                <div class="space-y-2">
                    {items.clone().into_iter().map(|(item, count)| {
                        let item_for_checked = item.clone();
                        let item_for_change = item.clone();
                        let item_for_display = item.clone();
                        view! {
                            <label class="flex items-center gap-2 cursor-pointer group">
                                <input
                                    type="checkbox"
                                    class="w-4 h-4 rounded border-stroke text-accent focus:ring-2 focus:ring-accent focus:ring-offset-0 cursor-pointer"
                                    checked=move || selected.get().contains(&item_for_checked)
                                    on:change=move |_| {
                                        let mut current = selected.get();
                                        if current.contains(&item_for_change) {
                                            current.retain(|f| f != &item_for_change);
                                        } else {
                                            current.push(item_for_change.clone());
                                        }
                                        selected.set(current);
                                        current_page.set(1);
                                    }
                                />
                                <span class="text-sm text-secondary group-hover:text-primary transition-colors flex-1">
                                    {item_for_display}
                                </span>
                                <span class="text-xs text-muted">
                                    "(" {count} ")"
                                </span>
                            </label>
                        }
                    }).collect_view()}
                </div>
            </Show>
        </div>
    }
}

/// Sidebar filters component with collapsible sections
#[island]
pub fn SidebarFilters(
    selected_families: RwSignal<Vec<String>>,
    selected_engines: RwSignal<Vec<String>>,
    selected_covariates: RwSignal<Vec<String>>,
    method_families: Vec<(String, usize)>,
    statistical_engines: Vec<(String, usize)>,
    covariates: Vec<(String, usize)>,
    current_page: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="bg-elevated border border-stroke rounded-xl p-6 space-y-6">
            <h2 class="text-lg font-semibold text-primary">"Filters"</h2>

            <FilterSection
                title="Method Family".to_string()
                items=method_families
                selected=selected_families
                current_page=current_page
            />

            <FilterSection
                title="Statistical Engine".to_string()
                items=statistical_engines
                selected=selected_engines
                current_page=current_page
            />

            <FilterSection
                title="Covariates".to_string()
                items=covariates
                selected=selected_covariates
                current_page=current_page
            />
        </div>
    }
}

/// Tutorial card component with optional highlighting
#[component]
pub fn TutorialCard(
    tutorial: TutorialData,
    #[prop(optional)] title_html: Option<String>,
    #[prop(optional)] summary_html: Option<String>,
) -> impl IntoView {
    let family_slug = tutorial.method_family.to_lowercase().replace(' ', "-");
    let href = base_path::join(&format!("tutorials/{}/{}/", family_slug, tutorial.slug));

    let title_display = title_html.unwrap_or_else(|| tutorial.title.clone());
    let summary_display = summary_html.unwrap_or_else(|| tutorial.summary.clone());

    view! {
        <a
            href={href}
            class="group block rounded-xl transition-all duration-200 hover:scale-102 hover:shadow-xl bg-elevated border border-stroke p-6"
        >
            <div class="flex items-center gap-2 mb-2">
                {tutorial.difficulty.as_ref().map(|d| {
                    let (bg, text) = match d.as_str() {
                        "intro" => ("bg-emerald-100 dark:bg-emerald-900/30", "text-emerald-700 dark:text-emerald-400"),
                        "intermediate" => ("bg-amber-100 dark:bg-amber-900/30", "text-amber-700 dark:text-amber-400"),
                        "advanced" => ("bg-rose-100 dark:bg-rose-900/30", "text-rose-700 dark:text-rose-400"),
                        _ => ("bg-gray-100 dark:bg-gray-800", "text-gray-600 dark:text-gray-400"),
                    };
                    view! {
                        <span class=format!("text-xs px-2 py-0.5 rounded-full {} {}", bg, text)>
                            {d.clone()}
                        </span>
                    }
                })}
                <span class="text-xs text-muted">
                    "Updated " {tutorial.updated_at.clone()}
                </span>
            </div>
            <h3
                class="text-lg font-semibold group-hover:underline group-hover:text-accent transition-colors duration-200 text-primary"
                inner_html=title_display
            />
            <p
                class="mt-2 text-sm text-secondary"
                inner_html=summary_display
            />
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
            </div>
        </a>
    }
}

/// Table view component with sortable headers
#[island]
pub fn TutorialTable(tutorials: Vec<TutorialData>, sort_by: RwSignal<SortOption>) -> impl IntoView {
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
                        <th class="px-4 py-3 text-left text-sm font-semibold text-primary">"Engine"</th>
                        <th class="px-4 py-3 text-left text-sm font-semibold text-primary">"Covariates"</th>
                        <th class="px-4 py-3 text-left text-sm font-semibold text-primary">"Updated"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-stroke">
                    {tutorials.into_iter().map(|tutorial| {
                        let family_slug = tutorial.method_family.to_lowercase().replace(' ', "-");
                        let href = base_path::join(&format!("tutorials/{}/{}/", family_slug, tutorial.slug));
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

/// Loading skeleton for catalog
#[component]
fn CatalogSkeleton() -> impl IntoView {
    view! {
        <div class="animate-pulse space-y-6">
            <div class="h-12 bg-subtle rounded-lg"/>
            <div class="flex gap-2">
                <div class="h-10 w-24 bg-subtle rounded-lg"/>
                <div class="h-10 w-24 bg-subtle rounded-lg"/>
                <div class="h-10 w-24 bg-subtle rounded-lg"/>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
                {(0..6).map(|_| view! {
                    <div class="h-48 bg-subtle rounded-xl"/>
                }).collect_view()}
            </div>
        </div>
    }
}

/// Main tutorial catalog island with filtering, search, and pagination
#[island]
pub fn TutorialCatalog(tutorials: Vec<TutorialData>) -> impl IntoView {
    // State
    let search_query = RwSignal::new(String::new());
    let selected_families = RwSignal::new(Vec::<String>::new());
    let selected_engines = RwSignal::new(Vec::<String>::new());
    let selected_covariates = RwSignal::new(Vec::<String>::new());
    let sort_by = RwSignal::new(SortOption::Newest);
    let view_mode = RwSignal::new(ViewMode::Cards);
    let current_page = RwSignal::new(1usize);

    // Compute facets from data
    let (method_families, statistical_engines, covariates, _difficulties) = compute_facets(&tutorials);

    // Computed: filtered, ranked, and sorted tutorials
    let processed_tutorials = Memo::new(move |_| {
        let query = search_query.get();
        let families = selected_families.get();
        let engines = selected_engines.get();
        let covs = selected_covariates.get();
        let sort = sort_by.get();

        // First, rank and filter
        let mut results: Vec<(i32, String, String, TutorialData)> = tutorials
            .iter()
            .filter_map(|tutorial| {
                // Apply filters first
                if !families.is_empty() && !families.contains(&tutorial.method_family) {
                    return None;
                }
                if !engines.is_empty() && !engines.contains(&tutorial.statistical_engine) {
                    return None;
                }
                if !covs.is_empty() && !covs.contains(&tutorial.covariates) {
                    return None;
                }

                // Rank and highlight
                let (score, title_html, summary_html) = rank_and_highlight(tutorial, &query);

                // If searching, only include matches
                if !query.is_empty() && score == 0 {
                    return None;
                }

                Some((score, title_html, summary_html, tutorial.clone()))
            })
            .collect();

        // Sort: if searching, sort by score first, then by selected sort
        if !query.is_empty() {
            results.sort_by(|a, b| b.0.cmp(&a.0)); // Higher score first
        } else {
            match sort {
                SortOption::Newest => results.sort_by(|a, b| b.3.updated_at.cmp(&a.3.updated_at)),
                SortOption::TitleAZ => results.sort_by(|a, b| a.3.title.cmp(&b.3.title)),
                SortOption::TitleZA => results.sort_by(|a, b| b.3.title.cmp(&a.3.title)),
                SortOption::FamilyAZ => results.sort_by(|a, b| a.3.method_family.cmp(&b.3.method_family)),
                SortOption::FamilyZA => results.sort_by(|a, b| b.3.method_family.cmp(&a.3.method_family)),
                SortOption::EngineAZ => results.sort_by(|a, b| a.3.statistical_engine.cmp(&b.3.statistical_engine)),
                SortOption::EngineZA => results.sort_by(|a, b| b.3.statistical_engine.cmp(&a.3.statistical_engine)),
                _ => {}
            }
        }

        results
    });

    // Paginated results for current page
    let page_results = Memo::new(move |_| {
        let all = processed_tutorials.get();
        let page = current_page.get();
        let start = (page - 1) * PAGE_SIZE;
        let end = std::cmp::min(start + PAGE_SIZE, all.len());

        if start >= all.len() {
            vec![]
        } else {
            all[start..end].to_vec()
        }
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
                    current_page=current_page
                />
            </div>

            // Main content area
            <div class="flex-1 space-y-6">
                // Search bar
                <div class="bg-elevated border border-stroke rounded-xl p-4">
                    <SearchBar search_query current_page=current_page />
                </div>

                // Active filter chips
                <ActiveFilterChips
                    selected_families
                    selected_engines
                    selected_covariates
                    current_page=current_page
                />

                // Controls: View toggle and Sort
                <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                    <div class="text-sm text-muted">
                        {move || {
                            let total = processed_tutorials.get().len();
                            let page = current_page.get();
                            let start = (page - 1) * PAGE_SIZE + 1;
                            let end = std::cmp::min(page * PAGE_SIZE, total);
                            if total == 0 {
                                "No tutorials found".to_string()
                            } else {
                                format!("Showing {}-{} of {} tutorials", start, end, total)
                            }
                        }}
                    </div>
                    <div class="flex items-center gap-4">
                        <ViewToggle view_mode />
                        <SortDropdown sort_by />
                    </div>
                </div>

                // Content display (cards or table)
                {move || {
                    let results = page_results.get();
                    let query = search_query.get();

                    if results.is_empty() {
                        view! {
                            <div class="text-center py-12 text-muted">
                                <svg class="w-16 h-16 mx-auto mb-4 text-muted/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                </svg>
                                <p class="text-lg font-medium">"No tutorials found"</p>
                                <p class="text-sm mt-1">"Try adjusting your search or filters"</p>
                            </div>
                        }.into_any()
                    } else {
                        match view_mode.get() {
                            ViewMode::Cards => view! {
                                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                                    {results.into_iter().map(|(_score, title_html, summary_html, tutorial)| {
                                        let has_highlight = !query.is_empty();
                                        if has_highlight {
                                            view! {
                                                <TutorialCard
                                                    tutorial=tutorial
                                                    title_html=title_html
                                                    summary_html=summary_html
                                                />
                                            }.into_any()
                                        } else {
                                            view! {
                                                <TutorialCard tutorial=tutorial />
                                            }.into_any()
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any(),
                            ViewMode::Table => {
                                let tutorials: Vec<_> = results.into_iter().map(|(_, _, _, t)| t).collect();
                                view! {
                                    <TutorialTable tutorials sort_by />
                                }.into_any()
                            }
                        }
                    }
                }}

                // Pagination
                {move || {
                    let total = processed_tutorials.get().len();
                    view! { <PaginationControls current_page total_count=total /> }
                }}
            </div>
        </div>
    }
}

/// Method family tabs component (simplified version)
#[island]
pub fn MethodTabs(
    selected_method: RwSignal<Option<String>>,
    method_families: Vec<(String, usize)>,
) -> impl IntoView {
    let total_count = method_families.iter().map(|(_, count)| count).sum::<usize>();

    view! {
        <div class="flex flex-wrap gap-2">
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
