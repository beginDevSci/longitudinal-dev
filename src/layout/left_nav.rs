use crate::models::post::Post;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// Icon constants for statistical method families
mod icons {
    pub const LGCM: &str = "📈";   // Latent Growth Curve Model
    pub const LCSM: &str = "📊";   // Latent Change Score Model
    pub const LMM: &str = "🔀";    // Linear Mixed Model
    pub const LM: &str = "📉";     // Linear Model
    pub const GLMM: &str = "🔬";   // Generalized Linear Mixed Model
    pub const GEE: &str = "🧮";    // Generalized Estimating Equations
    pub const GMM: &str = "🎯";    // Growth Mixture Model
    pub const LCGA: &str = "👥";   // Latent Class Growth Analysis
    pub const MLGCM: &str = "🌐";  // Multivariate Latent Growth Curve Model
    pub const DEFAULT: &str = "📄"; // Default icon
}

const HIDDEN_METHOD_FAMILIES: &[&str] = &["blmm"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavItem {
    pub title: String,
    pub slug: String,
    pub icon: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub tutorials: Vec<NavItem>,
}

impl NavItem {
    /// Convert a Post into a NavItem for navigation
    pub fn from_post(post: &Post) -> Self {
        let description = extract_description(post);
        let icon = determine_icon(&post.slug);
        let formatted_title = format_title(&post.title);

        Self {
            title: formatted_title,
            slug: post.slug.to_string(),
            icon,
            description: Some(description),
        }
    }

    /// Convert multiple posts into nav items
    pub fn from_posts(posts: &[Post]) -> Vec<Self> {
        posts.iter().map(Self::from_post).collect()
    }
}

impl NavCategory {
    /// Organize posts into hierarchical categories based on method_family from metadata
    pub fn from_posts(posts: &[Post]) -> Vec<Self> {
        use std::collections::HashMap;

        // Build a map of method_family -> list of posts
        let mut family_map: HashMap<String, Vec<NavItem>> = HashMap::new();

        for post in posts {
            // Get method family from metadata, skip if not available
            let method_family = match &post.metadata {
                Some(metadata) => metadata.method_family.clone(),
                None => continue,
            };

            if HIDDEN_METHOD_FAMILIES
                .iter()
                .any(|family| method_family.eq_ignore_ascii_case(family))
            {
                continue;
            }

            let nav_item = NavItem::from_post(post);

            family_map.entry(method_family).or_default().push(nav_item);
        }

        // Convert to categories
        let mut categories: Vec<NavCategory> = family_map
            .into_iter()
            .map(|(family, mut tutorials)| {
                // Sort tutorials within each category by title
                tutorials.sort_by(|a, b| a.title.cmp(&b.title));

                NavCategory {
                    id: family.to_lowercase(),
                    name: family.clone(),
                    icon: get_family_icon(&family),
                    tutorials,
                }
            })
            .collect();

        // Sort categories alphabetically by name
        categories.sort_by(|a, b| a.name.cmp(&b.name));

        categories
    }
}

/// Extract a short description from the post's overview summary
///
/// Takes the first sentence or first 100 characters, whichever is shorter
fn extract_description(post: &Post) -> String {
    // Get the first summary paragraph
    let summary = post
        .overview
        .summary_paragraphs
        .first()
        .map(|s| s.as_ref())
        .unwrap_or("");

    // Find first sentence (ending with ., !, or ?)
    let first_sentence = summary
        .split_inclusive(&['.', '!', '?'])
        .next()
        .unwrap_or(summary)
        .trim();

    // Truncate to 120 chars if still too long
    if first_sentence.len() > 120 {
        format!("{}...", &first_sentence[..117])
    } else {
        first_sentence.to_string()
    }
}

fn format_title(title: &str) -> String {
    match title.split_once(':') {
        Some((prefix, variant)) => {
            let prefix_trimmed = prefix.trim();
            let variant_trimmed = variant.trim();
            if prefix_trimmed.is_empty() || variant_trimmed.is_empty() {
                title.to_string()
            } else {
                format!("{variant_trimmed} ({prefix_trimmed})")
            }
        }
        None => title.to_string(),
    }
}

/// Determine icon based on post slug/title
///
/// Uses simple pattern matching on common statistical method abbreviations
fn determine_icon(slug: &str) -> String {
    let slug_lower = slug.to_lowercase();

    // Match common statistical method patterns
    if slug_lower.contains("lgcm") || slug_lower.contains("growth-curve") {
        icons::LGCM.to_string()
    } else if slug_lower.contains("lmm") || slug_lower.contains("mixed") {
        icons::LMM.to_string()
    } else if slug_lower.contains("glmm") || slug_lower.contains("generalized") {
        icons::GLMM.to_string()
    } else if slug_lower.contains("gmm") || slug_lower.contains("mixture") {
        icons::GMM.to_string()
    } else if slug_lower.contains("lcga") || slug_lower.contains("class") {
        icons::LCGA.to_string()
    } else if slug_lower.contains("lcsm") || slug_lower.contains("change-score") {
        icons::LCSM.to_string()
    } else if slug_lower.contains("mlgcm") || slug_lower.contains("multivariate") {
        icons::MLGCM.to_string()
    } else if slug_lower.contains("residual") {
        icons::LM.to_string()
    } else {
        icons::DEFAULT.to_string()
    }
}

/// Determine icon based on method family
///
/// Maps method family abbreviations to appropriate icons
fn get_family_icon(family: &str) -> String {
    match family {
        "LGCM" => icons::LGCM.to_string(),
        "LCSM" => icons::LCSM.to_string(),
        "LMM" => icons::LMM.to_string(),
        "LM" => icons::LM.to_string(),
        "GLMM" => icons::GLMM.to_string(),
        "GEE" => icons::GEE.to_string(),
        "GMM" => icons::GMM.to_string(),
        "LCGA" => icons::LCGA.to_string(),
        "MLGCM" => icons::MLGCM.to_string(),
        _ => icons::DEFAULT.to_string(),
    }
}

/// Collapsible category component with localStorage persistence
#[island]
pub fn CategoryItem(
    category: NavCategory,
    current_slug: Option<String>,
    base_path: String,
    search_query: String,
) -> impl IntoView {
    // Store values in StoredValue for shared access across closures
    let current_slug_stored = StoredValue::new(current_slug.clone());
    let base_path_stored = StoredValue::new(base_path.clone());
    let category_id_stored = StoredValue::new(category.id.clone());

    // Check if current post is in this category (auto-expand if so)
    let contains_current = current_slug
        .as_ref()
        .map(|slug| category.tutorials.iter().any(|item| &item.slug == slug))
        .unwrap_or(false);

    // Initialize expanded state - will be loaded from localStorage on client side
    let is_expanded = RwSignal::new(contains_current);

    // Handle localStorage persistence (client-side only via Effect)
    let storage_key = format!("nav-category-{}", category.id);

    // Load initial state from localStorage on client mount
    Effect::new({
        let storage_key = storage_key.clone();
        move |_| {
            // Only run on client side (after hydration)
            if let Ok(Some(storage)) = window().local_storage() {
                if let Ok(Some(stored_value)) = storage.get_item(&storage_key) {
                    let should_expand = stored_value == "true";
                    if should_expand != is_expanded.get_untracked() {
                        is_expanded.set(should_expand);
                    }
                } else if contains_current {
                    // Auto-expand if contains current tutorial and no stored preference
                    is_expanded.set(true);
                }
            }
        }
    });

    // Save to localStorage when state changes
    Effect::new(move |_| {
        let expanded = is_expanded.get();
        if let Ok(Some(storage)) = window().local_storage() {
            let _ = storage.set_item(&storage_key, if expanded { "true" } else { "false" });
        }
    });

    // Filter tutorials based on search (as a memo)
    let search_is_active = !search_query.is_empty();
    let search_query_signal = Signal::derive(move || search_query.clone());
    let all_tutorials = category.tutorials.clone();

    let filtered_tutorials = Memo::new(move |_| {
        let query = search_query_signal.get();
        all_tutorials
            .iter()
            .filter(|item| {
                if query.is_empty() {
                    true
                } else {
                    let query_lower = query.to_lowercase();
                    item.title.to_lowercase().contains(&query_lower)
                        || item
                            .description
                            .as_ref()
                            .is_some_and(|desc| desc.to_lowercase().contains(&query_lower))
                }
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    // Don't render if search filtered all tutorials
    let has_tutorials = !filtered_tutorials.get().is_empty();
    if search_is_active && !has_tutorials {
        return ().into_any();
    }

    let category_name = category.name.clone();

    view! {
        <div class="left-nav-category">
            // Category header (clickable to expand/collapse)
            <button
                class=move || {
                    let base = "left-nav-category-btn";
                    if contains_current {
                        format!("{base} left-nav-category-btn--active")
                    } else {
                        base.to_string()
                    }
                }
                on:click=move |_| is_expanded.update(|exp| *exp = !*exp)
            >
                <div class="flex items-center gap-3 flex-1">
                    // Expand/collapse chevron
                    <svg
                        class=move || {
                            format!(
                                "left-nav-chevron {}",
                                if is_expanded.get() { "left-nav-chevron--expanded" } else { "" }
                            )
                        }
                        fill="currentColor"
                        viewBox="0 0 20 20"
                    >
                        <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"/>
                    </svg>

                    // Category name
                    <span class="left-nav-category-name">
                        {category_name}
                    </span>

                    // Tutorial count
                    <span class="left-nav-category-count">
                        {move || filtered_tutorials.get().len()}
                    </span>
                </div>
            </button>

            // Tutorials list (shown when expanded)
            <Show when=move || is_expanded.get()>
                <div class="left-nav-items">
                    {move || {
                        filtered_tutorials.get().into_iter().map(move |item| {
                            let is_active = current_slug_stored.get_value().as_ref() == Some(&item.slug);
                            let base = base_path_stored.get_value();
                            let family = category_id_stored.get_value();
                            let href = if base == "/" {
                                format!("/abcd/{}/{}/", family, item.slug)
                            } else {
                                format!("{}abcd/{}/{}/", base.trim_end_matches('/'), family, item.slug)
                            };

                            view! {
                                <a
                                    href=href
                                    class=if is_active {
                                        "left-nav-link left-nav-link-active"
                                    } else {
                                        "left-nav-link"
                                    }
                                >
                                    <span class="left-nav-link-title" title={item.title.clone()}>
                                        {item.title.clone()}
                                    </span>
                                </a>
                            }
                        }).collect_view()
                    }}
                </div>
            </Show>
        </div>
    }.into_any()
}

/// Left navigation sidebar showing all blog posts in hierarchical categories
///
/// Features:
/// - Collapsible sidebar (expands to 280px, collapses to 64px)
/// - Hierarchical category organization
/// - Search/filter functionality
/// - Active post highlighting
/// - localStorage persistence for category states
/// - Auto-expand active category
///
/// This is an island component for client-side interactivity
#[island]
pub fn LeftNav(
    /// List of navigation categories with tutorials
    categories: Vec<NavCategory>,
    /// Current post slug for active highlighting
    #[prop(optional)]
    current_slug: Option<String>,
    /// Base path for URLs (e.g., "/" or "/longitudinal-dev/")
    base_path: String,
) -> impl IntoView {
    // State management
    let (is_collapsed, set_is_collapsed) = signal(false);
    let search_query = RwSignal::new(String::new());
    let (_search_focused, set_search_focused) = signal(false);

    // localStorage persistence for collapsed state
    let storage_key = "left-nav-collapsed";

    // Load initial state from localStorage on client mount
    Effect::new({
        let storage_key = storage_key.to_string();
        move |_| {
            if let Ok(Some(storage)) = window().local_storage() {
                if let Ok(Some(stored_value)) = storage.get_item(&storage_key) {
                    let should_collapse = stored_value == "true";
                    if should_collapse != is_collapsed.get_untracked() {
                        set_is_collapsed.set(should_collapse);
                    }
                }
            }
        }
    });

    // Save to localStorage when state changes
    Effect::new({
        let storage_key = storage_key.to_string();
        move |_| {
            let collapsed = is_collapsed.get();
            if let Ok(Some(storage)) = window().local_storage() {
                let _ = storage.set_item(&storage_key, if collapsed { "true" } else { "false" });
            }
        }
    });

    view! {
        <aside class=move || {
            format!(
                "left-nav-container sticky top-16 h-[calc(100vh-4rem)] transition-all duration-200 ease-in-out overflow-hidden {}",
                if is_collapsed.get() { "w-14" } else { "w-[280px]" }
            )
        }>
            <div class="h-full flex flex-col">
                // Search bar with integrated collapse toggle
                <div class="left-nav-search-container flex-shrink-0">
                    <div class="flex items-center gap-2">
                        // Search input (hidden when collapsed)
                        <Show when=move || !is_collapsed.get()>
                            <div class="relative flex-1">
                                <input
                                    type="text"
                                    placeholder="Search..."
                                    class="left-nav-search"
                                    on:input=move |ev| {
                                        search_query.set(event_target_value(&ev));
                                    }
                                    on:focus=move |_| set_search_focused.set(true)
                                    on:blur=move |_| set_search_focused.set(false)
                                />
                                <svg
                                    class="left-nav-search-icon"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
                                </svg>

                                // Clear button
                                <Show when=move || !search_query.get().is_empty()>
                                    <button
                                        class="left-nav-search-clear"
                                        on:click=move |_| search_query.set(String::new())
                                    >
                                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12"></path>
                                        </svg>
                                    </button>
                                </Show>
                            </div>
                        </Show>

                        // Collapse/Expand Toggle (always visible, centered when collapsed)
                        <button
                            class=move || {
                                if is_collapsed.get() {
                                    "left-nav-collapse-btn flex-shrink-0 mx-auto"
                                } else {
                                    "left-nav-collapse-btn flex-shrink-0"
                                }
                            }
                            on:click=move |_| set_is_collapsed.update(|collapsed| *collapsed = !*collapsed)
                            title=move || if is_collapsed.get() { "Expand sidebar" } else { "Collapse sidebar" }
                        >
                            <svg
                                class=move || {
                                    format!(
                                        "w-5 h-5 transform transition-transform duration-200 {}",
                                        if is_collapsed.get() { "rotate-180" } else { "rotate-0" }
                                    )
                                }
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M15 19l-7-7 7-7" />
                            </svg>
                        </button>
                    </div>
                </div>

                // Navigation Categories (scrollable)
                <nav class="flex-1 overflow-y-auto mt-4 px-3 pb-4">
                    <For
                        each=move || categories.clone()
                        key=|cat| cat.id.clone()
                        children=move |category| {
                            let query = search_query.get();
                            let slug_clone = current_slug.clone();
                            let base_clone = base_path.clone();

                            view! {
                                <CategoryItem
                                    category=category
                                    current_slug=slug_clone
                                    base_path=base_clone
                                    search_query=query
                                />
                            }
                        }
                    />
                </nav>
            </div>
        </aside>
    }
}
