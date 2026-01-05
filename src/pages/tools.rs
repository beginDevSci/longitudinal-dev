//! Open Source Tools page component.
//!
//! Displays tools for data science: programming languages, IDEs, version control,
//! data formats, notebooks, and databases.
//! Data is loaded from content/tools.yaml at build time.

use leptos::prelude::*;
use longitudinal_dev::ui::{
    CardContent, CardDescription, CardMedia, CardShell, CardTitle, PageSectionHeader,
};
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
    let title = tool.title.clone();
    let alt_text = tool.title.clone();
    let first_char = tool.title.chars().next().unwrap_or('?');

    view! {
        <CardShell href=tool.url.clone() class="group">
            <CardMedia aspect="logo" class="w-full logo-tile">
                {if has_logo {
                    view! {
                        <img
                            src=logo_url
                            alt=alt_text
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
            </CardMedia>
            <CardContent>
                <CardTitle class="mb-2">{title}</CardTitle>
                <CardDescription>{tool.blurb}</CardDescription>
            </CardContent>
        </CardShell>
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
                <PageSectionHeader
                    title="Programming Languages"
                    description="Core languages for statistical computing and data analysis."
                    id="languages-header"
                />
                <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
                    {tools.programming_languages.iter().map(|tool| {
                        view! { <ToolCard tool=tool.clone() /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // IDEs section
            <section id="ides" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <PageSectionHeader
                        title="Development Environments"
                        description="Editors and IDEs for writing and debugging code."
                        id="ides-header"
                    />
                    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4">
                        {tools.ides.iter().map(|tool| {
                            view! { <ToolCard tool=tool.clone() /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>

            // Version Control section
            <section id="version-control" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <PageSectionHeader
                    title="Version Control"
                    description="Tools for tracking changes and collaborating on code."
                    id="version-control-header"
                />
                <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
                    {tools.version_control.iter().map(|tool| {
                        view! { <ToolCard tool=tool.clone() /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // Data Formats section
            <section id="data-formats" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <PageSectionHeader
                        title="Data Formats"
                        description="Common file formats for storing and exchanging data."
                        id="data-formats-header"
                    />
                    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4">
                        {tools.data_formats.iter().map(|tool| {
                            view! { <ToolCard tool=tool.clone() /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>

            // Notebooks section
            <section id="notebooks" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <PageSectionHeader
                    title="Notebooks & Literate Programming"
                    description="Interactive environments for reproducible research."
                    id="notebooks-header"
                />
                <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-3 gap-4">
                    {tools.notebooks.iter().map(|tool| {
                        view! { <ToolCard tool=tool.clone() /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // Databases section
            <section id="databases" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <PageSectionHeader
                        title="Databases"
                        description="Systems for storing and querying structured data."
                        id="databases-header"
                    />
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
