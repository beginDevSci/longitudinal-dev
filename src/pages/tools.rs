//! Open Source Tools page component.
//!
//! Displays tools for data science: programming languages, IDEs, version control,
//! data formats, notebooks, databases, and R packages.
//! Data is loaded from content/tools.yaml at build time.

use leptos::prelude::*;
use longitudinal_dev::base_path;
use longitudinal_dev::tools_catalog::{ToolCategory, ToolItem, ToolsCatalogIsland};
use serde::{Deserialize, Serialize};

/// Generic tool item used across all categories.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tool {
    pub title: String,
    pub url: String,
    pub blurb: String,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub is_open_source: Option<bool>,
    #[serde(default)]
    pub is_featured: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Container for all tools loaded from YAML.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tools {
    pub programming_languages: Vec<Tool>,
    #[serde(default)]
    pub r_packages: Vec<Tool>,
    pub ides: Vec<Tool>,
    pub version_control: Vec<Tool>,
    pub data_formats: Vec<Tool>,
    pub notebooks: Vec<Tool>,
    pub databases: Vec<Tool>,
}

/// Load tools from YAML file at build time.
pub fn load_tools() -> Tools {
    let yaml_content = include_str!("../../content/tools.yaml");
    serde_yaml::from_str(yaml_content).expect("Failed to parse tools.yaml")
}

/// Convert Tools to a flat Vec<ToolItem> for the catalog island.
pub fn tools_to_items(tools: &Tools) -> Vec<ToolItem> {
    let mut items = Vec::new();

    // Programming Languages
    for tool in &tools.programming_languages {
        items.push(ToolItem {
            title: tool.title.clone(),
            description: tool.blurb.clone(),
            url: tool.url.clone(),
            category: ToolCategory::ProgrammingLanguages,
            logo: tool.logo.clone(),
            level: tool.level.clone(),
            is_open_source: tool.is_open_source,
            is_featured: tool.is_featured,
            tags: tool.tags.clone(),
        });
    }

    // R Packages
    for tool in &tools.r_packages {
        items.push(ToolItem {
            title: tool.title.clone(),
            description: tool.blurb.clone(),
            url: tool.url.clone(),
            category: ToolCategory::RPackages,
            logo: tool.logo.clone(),
            level: tool.level.clone(),
            is_open_source: tool.is_open_source,
            is_featured: tool.is_featured,
            tags: tool.tags.clone(),
        });
    }

    // IDEs
    for tool in &tools.ides {
        items.push(ToolItem {
            title: tool.title.clone(),
            description: tool.blurb.clone(),
            url: tool.url.clone(),
            category: ToolCategory::IDEs,
            logo: tool.logo.clone(),
            level: tool.level.clone(),
            is_open_source: tool.is_open_source,
            is_featured: tool.is_featured,
            tags: tool.tags.clone(),
        });
    }

    // Version Control
    for tool in &tools.version_control {
        items.push(ToolItem {
            title: tool.title.clone(),
            description: tool.blurb.clone(),
            url: tool.url.clone(),
            category: ToolCategory::VersionControl,
            logo: tool.logo.clone(),
            level: tool.level.clone(),
            is_open_source: tool.is_open_source,
            is_featured: tool.is_featured,
            tags: tool.tags.clone(),
        });
    }

    // Data Formats
    for tool in &tools.data_formats {
        items.push(ToolItem {
            title: tool.title.clone(),
            description: tool.blurb.clone(),
            url: tool.url.clone(),
            category: ToolCategory::DataFormats,
            logo: tool.logo.clone(),
            level: tool.level.clone(),
            is_open_source: tool.is_open_source,
            is_featured: tool.is_featured,
            tags: tool.tags.clone(),
        });
    }

    // Notebooks
    for tool in &tools.notebooks {
        items.push(ToolItem {
            title: tool.title.clone(),
            description: tool.blurb.clone(),
            url: tool.url.clone(),
            category: ToolCategory::Notebooks,
            logo: tool.logo.clone(),
            level: tool.level.clone(),
            is_open_source: tool.is_open_source,
            is_featured: tool.is_featured,
            tags: tool.tags.clone(),
        });
    }

    // Databases
    for tool in &tools.databases {
        items.push(ToolItem {
            title: tool.title.clone(),
            description: tool.blurb.clone(),
            url: tool.url.clone(),
            category: ToolCategory::Databases,
            logo: tool.logo.clone(),
            level: tool.level.clone(),
            is_open_source: tool.is_open_source,
            is_featured: tool.is_featured,
            tags: tool.tags.clone(),
        });
    }

    items
}

/// Main Tools page component.
#[component]
pub fn ToolsPage(tools: Tools) -> impl IntoView {
    let items = tools_to_items(&tools);
    let toolkit_href = base_path::join("toolkit/");

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
                    <h1 class="text-4xl md:text-5xl font-bold text-primary">"Open Source Tools"</h1>
                    <p class="mt-3 text-lg md:text-xl text-secondary max-w-3xl">
                        "Tools for longitudinal data analysis and reproducible research: programming languages, R packages, development environments, and more."
                    </p>
                    <p class="mt-2 text-sm text-tertiary">
                        "All tools marked with "
                        <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300">
                            "Open Source"
                        </span>
                        " are free to use."
                    </p>
                    <div class="mt-6 p-4 rounded-lg bg-surface border border-stroke">
                        <p class="text-sm text-secondary">
                            <span class="font-medium text-primary">"How to use this page: "</span>
                            "Browse by category, filter by skill level, or use the Focus filters to find tools for specific tasks like reproducibility or mixed models. "
                            "Tools marked "
                            <span class="inline-flex items-center px-1.5 py-0.5 rounded-full text-xs font-medium bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300">
                                "★"
                            </span>
                            " are particularly recommended for longitudinal analysis."
                        </p>
                    </div>
                </div>
            </section>

            // Interactive catalog with search and filtering
            <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <ToolsCatalogIsland tools=items />
            </section>
        </main>
    }
}
