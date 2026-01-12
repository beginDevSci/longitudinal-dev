use crate::models::post::Post;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

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
        "📈".to_string()
    } else if slug_lower.contains("lmm") || slug_lower.contains("mixed") {
        "🔀".to_string()
    } else if slug_lower.contains("glmm") || slug_lower.contains("generalized") {
        "🔬".to_string()
    } else if slug_lower.contains("gmm") || slug_lower.contains("mixture") {
        "🎯".to_string()
    } else if slug_lower.contains("lcga") || slug_lower.contains("class") {
        "👥".to_string()
    } else if slug_lower.contains("lcsm") || slug_lower.contains("change-score") {
        "📊".to_string()
    } else if slug_lower.contains("mlgcm") || slug_lower.contains("multivariate") {
        "🌐".to_string()
    } else if slug_lower.contains("residual") {
        "📉".to_string()
    } else {
        "📄".to_string() // Default icon
    }
}

/// Determine icon based on method family
///
/// Maps method family abbreviations to appropriate icons
fn get_family_icon(family: &str) -> String {
    match family {
        "LGCM" => "📈".to_string(),  // Latent Growth Curve Model
        "LCSM" => "📊".to_string(),  // Latent Change Score Model
        "LMM" => "🔀".to_string(),   // Linear Mixed Model
        "LM" => "📉".to_string(),    // Linear Model
        "GLMM" => "🔬".to_string(),  // Generalized Linear Mixed Model
        "GEE" => "🧮".to_string(),   // Generalized Estimating Equations
        "GMM" => "🎯".to_string(),   // Growth Mixture Model
        "LCGA" => "👥".to_string(),  // Latent Class Growth Analysis
        "MLGCM" => "🌐".to_string(), // Multivariate Latent Growth Curve Model
        _ => "📄".to_string(),       // Default icon
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
        <div class="mb-4">
            // Category header (clickable to expand/collapse)
            <button
                class=move || {
                    let base = "w-full flex items-center justify-between px-4 py-3.5 rounded-lg transition-all duration-200 border-l-3 border-transparent focus-visible:outline-none focus-visible:shadow-[0_0_0_2px_var(--color-accent-400)]";
                    if contains_current {
                        format!("{base} bg-accent/5 border-l-accent")
                    } else {
                        format!("{base} hover:bg-aside-hover-bg")
                    }
                }
                on:click=move |_| is_expanded.update(|exp| *exp = !*exp)
            >
                <div class="flex items-center gap-3 flex-1">
                    // Expand/collapse arrow
                    <svg
                        class=move || {
                            format!(
                                "w-4 h-4 transition-transform duration-200 flex-shrink-0 {}",
                                if is_expanded.get() { "rotate-90" } else { "" }
                            )
                        }
                        fill="currentColor"
                        viewBox="0 0 20 20"
                    >
                        <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"/>
                    </svg>

                    // Category name and count
                    <div class="flex-1 text-left">
                        <div class="font-bold text-lg" style="color: var(--color-accent-300); text-shadow: 0 0 25px rgba(6, 182, 212, 0.35);">
                            {category_name}
                        </div>
                        <div class="text-xs mt-1" style="color: var(--color-aside-text-muted);">
                            {move || filtered_tutorials.get().len()} " tutorials"
                        </div>
                    </div>
                </div>
            </button>

            // Tutorials list (shown when expanded)
            <Show when=move || is_expanded.get()>
                <div class="mt-3 ml-4 pl-5 border-l-2 border-aside-border space-y-2">
                    {move || {
                        filtered_tutorials.get().into_iter().map(move |item| {
                            let is_active = current_slug_stored.get_value().as_ref() == Some(&item.slug);
                            let base = base_path_stored.get_value();
                            let href = if base == "/" {
                                format!("/posts/{}", item.slug)
                            } else {
                                format!("{}posts/{}", base.trim_end_matches('/'), item.slug)
                            };

                            view! {
                                <a
                                    href=href
                                    class=if is_active {
                                        "left-nav-link left-nav-link-active px-4 py-3"
                                    } else {
                                        "left-nav-link px-4 py-3"
                                    }
                                >
                                    // Content
                                    <div class="flex-1">
                                        <div class="left-nav-link-title" title={item.title.clone()}>
                                            {item.title.clone()}
                                        </div>
                                    </div>
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
                "left-nav-container sticky top-16 h-[calc(100vh-4rem)] transition-all duration-300 ease-in-out overflow-hidden {}",
                if is_collapsed.get() { "w-14" } else { "w-[280px]" }
            )
        }>
            <div class="h-full flex flex-col">
                // Sidebar Header with Branding
                <div class="left-nav-header flex items-center justify-between h-16 px-4 flex-shrink-0">
                    <div class=move || {
                        format!(
                            "flex items-center space-x-3 transition-opacity duration-200 {}",
                            if is_collapsed.get() {
                                "opacity-0 w-0 overflow-hidden"
                            } else {
                                "opacity-100"
                            }
                        )
                    }>
                        <div>
                            <h2 class="left-nav-brand-title">
                                "ABCD Analyses"
                            </h2>
                        </div>
                    </div>

                    // Collapse/Expand Toggle (always visible)
                    <button
                        class="left-nav-collapse-btn flex-shrink-0"
                        on:click=move |_| set_is_collapsed.update(|collapsed| *collapsed = !*collapsed)
                        title=move || if is_collapsed.get() { "Expand sidebar" } else { "Collapse sidebar" }
                    >
                        <svg
                            class=move || {
                                format!(
                                    "w-5 h-5 transform transition-transform duration-300 {}",
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

                // Search bar (when expanded)
                <Show when=move || !is_collapsed.get()>
                    <div class="left-nav-search-container flex-shrink-0">
                        <div class="relative">
                            <input
                                type="text"
                                placeholder="Search tutorials..."
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
                    </div>
                </Show>

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
