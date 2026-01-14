//! R Learning Resources page component.
//!
//! Displays curated R learning materials: books, videos, tutorials, and cheatsheets.
//! Data is loaded from content/resources.yaml at build time.

use leptos::prelude::*;
use longitudinal_dev::base_path;
use longitudinal_dev::resource_catalog::{ResourceCatalogIsland, ResourceCategory, ResourceItem};
use serde::{Deserialize, Serialize};

/// Learning path for "Start Here" section.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LearningPath {
    pub id: String,
    pub title: String,
    pub audience: String,
    pub summary: String,
    pub level: String,
    pub steps: Vec<String>,
}

/// Book resource with cover image.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub url: String,
    pub blurb: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub is_open_source: Option<bool>,
    #[serde(default)]
    pub is_featured: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Video resource with embed URL.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Video {
    pub title: String,
    pub source: String,
    pub url: String,
    #[serde(default)]
    pub embed_url: Option<String>,
    pub blurb: String,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub is_open_source: Option<bool>,
    #[serde(default)]
    pub is_featured: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Interactive tutorial resource.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tutorial {
    pub title: String,
    pub url: String,
    pub blurb: String,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub access: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub is_open_source: Option<bool>,
    #[serde(default)]
    pub is_featured: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Cheatsheet/quick reference resource.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cheatsheet {
    pub title: String,
    pub url: String,
    pub blurb: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub is_open_source: Option<bool>,
    #[serde(default)]
    pub is_featured: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Container for all resources loaded from YAML.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Resources {
    #[serde(default)]
    pub paths: Vec<LearningPath>,
    pub books: Vec<Book>,
    pub videos: Vec<Video>,
    pub tutorials: Vec<Tutorial>,
    pub cheatsheets: Vec<Cheatsheet>,
}

/// Load resources from YAML file at build time.
pub fn load_resources() -> Resources {
    let yaml_content = include_str!("../../content/resources.yaml");
    serde_yaml::from_str(yaml_content).expect("Failed to parse resources.yaml")
}

/// Convert Resources to a flat Vec<ResourceItem> for the catalog island.
pub fn resources_to_items(resources: &Resources) -> Vec<ResourceItem> {
    let mut items = Vec::new();

    // Books
    for book in &resources.books {
        items.push(ResourceItem {
            title: book.title.clone(),
            description: book.blurb.clone(),
            url: book.url.clone(),
            category: ResourceCategory::Books,
            image: book.image.clone(),
            embed_url: None,
            author: Some(book.author.clone()),
            source: None,
            platform: None,
            access: None,
            format: None,
            icon: None,
            level: book.level.clone(),
            is_open_source: book.is_open_source,
            is_featured: book.is_featured,
            tags: book.tags.clone(),
        });
    }

    // Videos
    for video in &resources.videos {
        items.push(ResourceItem {
            title: video.title.clone(),
            description: video.blurb.clone(),
            url: video.url.clone(),
            category: ResourceCategory::Videos,
            image: None,
            embed_url: video.embed_url.clone(),
            author: None,
            source: Some(video.source.clone()),
            platform: None,
            access: None,
            format: None,
            icon: None,
            level: video.level.clone(),
            is_open_source: video.is_open_source,
            is_featured: video.is_featured,
            tags: video.tags.clone(),
        });
    }

    // Tutorials
    for tutorial in &resources.tutorials {
        items.push(ResourceItem {
            title: tutorial.title.clone(),
            description: tutorial.blurb.clone(),
            url: tutorial.url.clone(),
            category: ResourceCategory::Tutorials,
            image: None,
            embed_url: None,
            author: None,
            source: None,
            platform: tutorial.platform.clone(),
            access: tutorial.access.clone(),
            format: None,
            icon: None,
            level: tutorial.level.clone(),
            is_open_source: tutorial.is_open_source,
            is_featured: tutorial.is_featured,
            tags: tutorial.tags.clone(),
        });
    }

    // Cheatsheets
    for cheatsheet in &resources.cheatsheets {
        items.push(ResourceItem {
            title: cheatsheet.title.clone(),
            description: cheatsheet.blurb.clone(),
            url: cheatsheet.url.clone(),
            category: ResourceCategory::Cheatsheets,
            image: None,
            embed_url: None,
            author: None,
            source: None,
            platform: None,
            access: None,
            format: cheatsheet.format.clone(),
            icon: cheatsheet.icon.clone(),
            level: cheatsheet.level.clone(),
            is_open_source: cheatsheet.is_open_source,
            is_featured: cheatsheet.is_featured,
            tags: cheatsheet.tags.clone(),
        });
    }

    items
}

/// Main Resources page component.
#[component]
pub fn ResourcesPage(resources: Resources) -> impl IntoView {
    let items = resources_to_items(&resources);
    let toolkit_href = base_path::join("toolkit/");
    let paths = resources.paths.clone();

    view! {
        <main class="min-h-screen bg-surface">
            // Header
            <section class="relative overflow-hidden bg-subtle">
                <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 lg:py-10">
                    <a
                        href=toolkit_href
                        class="inline-flex items-center gap-1 text-sm text-secondary hover:text-accent transition-colors mb-4"
                    >
                        <span>"←"</span>
                        <span>"Back to Toolkit"</span>
                    </a>
                    <h1 class="text-4xl md:text-5xl font-bold text-primary">"R Learning Resources"</h1>
                    <p class="mt-3 text-lg md:text-xl text-secondary max-w-3xl">
                        "A curated collection of open-source R learning resources for longitudinal data analysis and research workflows."
                    </p>
                    <p class="mt-2 text-sm text-tertiary">
                        "All resources marked with "
                        <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300">
                            "Open Source"
                        </span>
                        " are freely available."
                    </p>
                    <div class="mt-4 p-3 rounded-lg bg-surface border border-stroke">
                        <p class="text-sm text-secondary">
                            <span class="font-medium text-primary">"Tip: "</span>
                            "Start with a learning path below, or filter by category, level, or topic. Items marked "
                            <span class="inline-flex items-center px-1.5 py-0.5 rounded-full text-xs font-medium bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300">
                                "★"
                            </span>
                            " are particularly good starting points."
                        </p>
                    </div>
                </div>
            </section>

            // Start Here paths section
            {if !paths.is_empty() {
                Some(view! {
                    <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 border-b border-stroke">
                        <div class="mb-6">
                            <h2 class="text-2xl font-bold text-primary">"Start Here"</h2>
                            <p class="text-secondary mt-1">"Choose a learning path based on your experience level and goals."</p>
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                            {paths.into_iter().map(|path| {
                                let level_class = match path.level.as_str() {
                                    "beginner" => "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300",
                                    "intermediate" => "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300",
                                    "advanced" => "bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300",
                                    _ => "bg-slate-100 text-slate-800 dark:bg-slate-800 dark:text-slate-300",
                                };
                                let level_label = match path.level.as_str() {
                                    "beginner" => "Foundational".to_string(),
                                    "intermediate" => "Intermediate".to_string(),
                                    "advanced" => "Advanced".to_string(),
                                    _ => path.level.clone(),
                                };
                                view! {
                                    <div class="resource-card p-6">
                                        <div class="flex items-center gap-2 mb-3">
                                            <span class=format!("px-2 py-0.5 rounded-full text-xs font-medium {}", level_class)>
                                                {level_label}
                                            </span>
                                        </div>
                                        <h3 class="text-lg font-semibold text-primary mb-2">{path.title}</h3>
                                        <p class="text-sm text-tertiary mb-3">"Best for: " {path.audience}</p>
                                        <p class="text-sm text-secondary mb-4">{path.summary}</p>
                                        <div class="space-y-1">
                                            {path.steps.into_iter().enumerate().map(|(i, step)| {
                                                view! {
                                                    <div class="flex items-start gap-2 text-sm">
                                                        <span class="text-accent font-medium">{format!("{}.", i + 1)}</span>
                                                        <span class="text-secondary">{step}</span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </section>
                })
            } else {
                None
            }}

            // Interactive catalog with search and filtering
            <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <ResourceCatalogIsland resources=items />
            </section>
        </main>
    }
}
