//! R Learning Resources page component.
//!
//! Displays curated R learning materials: books, videos, tutorials, and cheatsheets.
//! Data is loaded from content/resources.yaml at build time.

use leptos::prelude::*;
use longitudinal_dev::base_path;
use longitudinal_dev::resource_catalog::{ResourceCatalogIsland, ResourceCategory, ResourceItem};
use serde::{Deserialize, Serialize};

/// Book resource with cover image.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub url: String,
    pub blurb: String,
    #[serde(default)]
    pub image: Option<String>,
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
}

/// Container for all resources loaded from YAML.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Resources {
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
        });
    }

    items
}

/// Main Resources page component.
#[component]
pub fn ResourcesPage(resources: Resources) -> impl IntoView {
    let items = resources_to_items(&resources);
    let toolkit_href = base_path::join("toolkit/");

    view! {
        <main class="min-h-screen bg-surface">
            // Header
            <section class="relative overflow-hidden bg-subtle">
                <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-14 lg:py-20">
                    <a
                        href=toolkit_href
                        class="inline-flex items-center gap-1 text-sm text-secondary hover:text-accent transition-colors mb-4"
                    >
                        <span>"←"</span>
                        <span>"Back to Toolkit"</span>
                    </a>
                    <h1 class="text-4xl md:text-5xl font-bold text-primary">"R Learning Resources"</h1>
                    <p class="mt-3 text-lg md:text-xl text-secondary max-w-3xl">
                        "A curated, browseable collection of open-source R learning resources for research workflows."
                    </p>
                </div>
            </section>

            // Interactive catalog with search and filtering
            <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <ResourceCatalogIsland resources=items />
            </section>
        </main>
    }
}
