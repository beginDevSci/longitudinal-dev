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
    #[serde(default)]
    pub logo: Option<String>,
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

/// Card component for a tool with logo image.
#[component]
fn ToolCard(tool: Tool) -> impl IntoView {
    let logo_url = tool.logo.clone().unwrap_or_default();
    let has_logo = !logo_url.is_empty();

    view! {
        <a
            href=tool.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class="group block bg-surface border border-default rounded-2xl overflow-hidden hover:border-accent hover:shadow-lg transition-all duration-300"
        >
            // Logo section
            <div class="h-24 w-full bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 flex items-center justify-center p-4 border-b border-default">
                {if has_logo {
                    view! {
                        <img
                            src=logo_url
                            alt=tool.title.clone()
                            class="h-14 w-auto max-w-[120px] object-contain group-hover:scale-110 transition-transform duration-300"
                            loading="lazy"
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="h-14 w-14 rounded-xl bg-gradient-to-br from-accent/20 to-accent/10 flex items-center justify-center">
                            <span class="text-2xl font-bold text-accent">{tool.title.chars().next().unwrap_or('?')}</span>
                        </div>
                    }.into_any()
                }}
            </div>

            // Content section
            <div class="p-4">
                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors mb-2">
                    {tool.title}
                </h3>
                <p class="text-sm text-secondary line-clamp-2">{tool.blurb}</p>
            </div>
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
                        "Essential tools for data science research: programming languages, development environments, version control, and more."
                    </p>

                    // Anchor navigation pills
                    <nav class="mt-8 flex flex-wrap gap-3">
                        <a href="#languages" class="px-4 py-2 rounded-full bg-surface border border-default text-primary hover:border-accent hover:text-accent hover:bg-accent/5 transition-all duration-200 text-sm font-medium">
                            "Languages"
                        </a>
                        <a href="#ides" class="px-4 py-2 rounded-full bg-surface border border-default text-primary hover:border-accent hover:text-accent hover:bg-accent/5 transition-all duration-200 text-sm font-medium">
                            "IDEs"
                        </a>
                        <a href="#version-control" class="px-4 py-2 rounded-full bg-surface border border-default text-primary hover:border-accent hover:text-accent hover:bg-accent/5 transition-all duration-200 text-sm font-medium">
                            "Version Control"
                        </a>
                        <a href="#data-formats" class="px-4 py-2 rounded-full bg-surface border border-default text-primary hover:border-accent hover:text-accent hover:bg-accent/5 transition-all duration-200 text-sm font-medium">
                            "Data Formats"
                        </a>
                        <a href="#notebooks" class="px-4 py-2 rounded-full bg-surface border border-default text-primary hover:border-accent hover:text-accent hover:bg-accent/5 transition-all duration-200 text-sm font-medium">
                            "Notebooks"
                        </a>
                        <a href="#databases" class="px-4 py-2 rounded-full bg-surface border border-default text-primary hover:border-accent hover:text-accent hover:bg-accent/5 transition-all duration-200 text-sm font-medium">
                            "Databases"
                        </a>
                    </nav>
                </div>
            </section>

            // Programming Languages section
            <section id="languages" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <h2 class="text-2xl font-bold text-primary mb-2">"Programming Languages"</h2>
                <p class="text-secondary mb-6">"Core languages for statistical computing and data analysis."</p>
                <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
                    {tools.programming_languages.iter().map(|tool| {
                        view! { <ToolCard tool=tool.clone() /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // IDEs section
            <section id="ides" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <h2 class="text-2xl font-bold text-primary mb-2">"Development Environments"</h2>
                    <p class="text-secondary mb-6">"Editors and IDEs for writing and debugging code."</p>
                    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4">
                        {tools.ides.iter().map(|tool| {
                            view! { <ToolCard tool=tool.clone() /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>

            // Version Control section
            <section id="version-control" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <h2 class="text-2xl font-bold text-primary mb-2">"Version Control"</h2>
                <p class="text-secondary mb-6">"Tools for tracking changes and collaborating on code."</p>
                <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
                    {tools.version_control.iter().map(|tool| {
                        view! { <ToolCard tool=tool.clone() /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // Data Formats section
            <section id="data-formats" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <h2 class="text-2xl font-bold text-primary mb-2">"Data Formats"</h2>
                    <p class="text-secondary mb-6">"Common file formats for storing and exchanging data."</p>
                    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4">
                        {tools.data_formats.iter().map(|tool| {
                            view! { <ToolCard tool=tool.clone() /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>

            // Notebooks section
            <section id="notebooks" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <h2 class="text-2xl font-bold text-primary mb-2">"Notebooks & Literate Programming"</h2>
                <p class="text-secondary mb-6">"Interactive environments for reproducible research."</p>
                <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-3 gap-4">
                    {tools.notebooks.iter().map(|tool| {
                        view! { <ToolCard tool=tool.clone() /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // Databases section
            <section id="databases" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <h2 class="text-2xl font-bold text-primary mb-2">"Databases"</h2>
                    <p class="text-secondary mb-6">"Systems for storing and querying structured data."</p>
                    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
                        {tools.databases.iter().map(|tool| {
                            view! { <ToolCard tool=tool.clone() /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>
        </main>
    }
}
