//! Open Source Tools page component.
//!
//! Displays tools for data science: programming languages, IDEs, version control,
//! data formats, notebooks, and databases.
//! Data is loaded from content/tools.yaml at build time.

use leptos::prelude::*;
use serde::Deserialize;

/// Generic tool item used across all categories.
#[derive(Debug, Clone, Deserialize)]
pub struct Tool {
    pub title: String,
    pub url: String,
    pub blurb: String,
}

/// Container for all tools loaded from YAML.
#[derive(Debug, Clone, Deserialize)]
pub struct Tools {
    pub programming_languages: Vec<Tool>,
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

/// Card component for a tool with category-specific icon.
#[component]
fn ToolCard(tool: Tool, #[prop(into)] icon: String) -> impl IntoView {
    view! {
        <a
            href=tool.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class="block p-5 bg-surface border border-default rounded-xl hover:border-accent hover:shadow-md transition-all duration-200 group"
        >
            <div class="flex items-start gap-3 mb-3">
                <span class="text-2xl" aria-hidden="true">{icon}</span>
                <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">
                        {tool.title}
                    </h3>
                </div>
            </div>
            <p class="text-sm text-secondary line-clamp-3">{tool.blurb}</p>
        </a>
    }
}

/// Main Tools page component.
#[component]
pub fn ToolsPage(tools: Tools) -> impl IntoView {
    view! {
        <main class="min-h-screen bg-surface">
            // Header
            <section class="relative overflow-hidden bg-subtle">
                <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-14 lg:py-20">
                    <h1 class="text-4xl md:text-5xl font-bold text-primary">"Open Source Tools"</h1>
                    <p class="mt-3 text-lg md:text-xl text-secondary max-w-3xl">
                        "Tools for data science research: programming languages, development environments, version control, and more."
                    </p>

                    // Anchor navigation
                    <nav class="mt-8 flex flex-wrap gap-3">
                        <a href="#languages" class="px-4 py-2 rounded-lg bg-surface border border-default text-primary hover:border-accent hover:text-accent transition-colors">
                            "Languages"
                        </a>
                        <a href="#ides" class="px-4 py-2 rounded-lg bg-surface border border-default text-primary hover:border-accent hover:text-accent transition-colors">
                            "IDEs"
                        </a>
                        <a href="#version-control" class="px-4 py-2 rounded-lg bg-surface border border-default text-primary hover:border-accent hover:text-accent transition-colors">
                            "Version Control"
                        </a>
                        <a href="#data-formats" class="px-4 py-2 rounded-lg bg-surface border border-default text-primary hover:border-accent hover:text-accent transition-colors">
                            "Data Formats"
                        </a>
                        <a href="#notebooks" class="px-4 py-2 rounded-lg bg-surface border border-default text-primary hover:border-accent hover:text-accent transition-colors">
                            "Notebooks"
                        </a>
                        <a href="#databases" class="px-4 py-2 rounded-lg bg-surface border border-default text-primary hover:border-accent hover:text-accent transition-colors">
                            "Databases"
                        </a>
                    </nav>
                </div>
            </section>

            // Programming Languages section
            <section id="languages" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <h2 class="text-2xl font-bold text-primary mb-6">"Programming Languages"</h2>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                    {tools.programming_languages.iter().map(|tool| {
                        view! { <ToolCard tool=tool.clone() icon="💻" /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // IDEs section
            <section id="ides" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <h2 class="text-2xl font-bold text-primary mb-6">"Integrated Development Environments (IDEs)"</h2>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                        {tools.ides.iter().map(|tool| {
                            view! { <ToolCard tool=tool.clone() icon="🛠️" /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>

            // Version Control section
            <section id="version-control" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <h2 class="text-2xl font-bold text-primary mb-6">"Version Control"</h2>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {tools.version_control.iter().map(|tool| {
                        view! { <ToolCard tool=tool.clone() icon="🔀" /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // Data Formats section
            <section id="data-formats" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <h2 class="text-2xl font-bold text-primary mb-6">"Data Formats"</h2>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                        {tools.data_formats.iter().map(|tool| {
                            view! { <ToolCard tool=tool.clone() icon="📁" /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>

            // Notebooks section
            <section id="notebooks" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <h2 class="text-2xl font-bold text-primary mb-6">"Notebooks & Literate Programming"</h2>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {tools.notebooks.iter().map(|tool| {
                        view! { <ToolCard tool=tool.clone() icon="📓" /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // Databases section
            <section id="databases" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <h2 class="text-2xl font-bold text-primary mb-6">"Databases"</h2>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                        {tools.databases.iter().map(|tool| {
                            view! { <ToolCard tool=tool.clone() icon="🗄️" /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>
        </main>
    }
}
