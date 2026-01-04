//! R Learning Resources page component.
//!
//! Displays curated R learning materials: books, videos, tutorials, and cheatsheets.
//! Data is loaded from content/resources.yaml at build time.

use leptos::prelude::*;
use serde::Deserialize;

/// Book resource with optional cover image.
#[derive(Debug, Clone, Deserialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub url: String,
    pub blurb: String,
    #[serde(default)]
    pub image: Option<String>,
}

/// Video resource with source attribution.
#[derive(Debug, Clone, Deserialize)]
pub struct Video {
    pub title: String,
    pub source: String,
    pub url: String,
    pub blurb: String,
}

/// Interactive tutorial resource.
#[derive(Debug, Clone, Deserialize)]
pub struct Tutorial {
    pub title: String,
    pub url: String,
    pub blurb: String,
    #[serde(default)]
    pub platform: Option<String>,
}

/// Cheatsheet/quick reference resource.
#[derive(Debug, Clone, Deserialize)]
pub struct Cheatsheet {
    pub title: String,
    pub url: String,
    pub blurb: String,
    #[serde(default)]
    pub format: Option<String>,
}

/// Container for all resources loaded from YAML.
#[derive(Debug, Clone, Deserialize)]
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

/// Card component for a book resource.
#[component]
fn BookCard(book: Book) -> impl IntoView {
    view! {
        <a
            href=book.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class="block p-5 bg-surface border border-default rounded-xl hover:border-accent hover:shadow-md transition-all duration-200 group"
        >
            <div class="flex items-start gap-3 mb-3">
                <span class="text-2xl" aria-hidden="true">"📚"</span>
                <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2">
                        {book.title}
                    </h3>
                    <p class="text-sm text-tertiary mt-0.5">{book.author}</p>
                </div>
            </div>
            <p class="text-sm text-secondary line-clamp-3">{book.blurb}</p>
        </a>
    }
}

/// Card component for a video resource.
#[component]
fn VideoCard(video: Video) -> impl IntoView {
    view! {
        <a
            href=video.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class="block p-5 bg-surface border border-default rounded-xl hover:border-accent hover:shadow-md transition-all duration-200 group"
        >
            <div class="flex items-start gap-3 mb-3">
                <span class="text-2xl" aria-hidden="true">"🎬"</span>
                <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2">
                        {video.title}
                    </h3>
                    <p class="text-sm text-tertiary mt-0.5">{video.source}</p>
                </div>
            </div>
            <p class="text-sm text-secondary line-clamp-3">{video.blurb}</p>
        </a>
    }
}

/// Card component for an interactive tutorial resource.
#[component]
fn TutorialCard(tutorial: Tutorial) -> impl IntoView {
    let platform_display = tutorial.platform.clone().unwrap_or_default();
    let show_platform = !platform_display.is_empty();

    view! {
        <a
            href=tutorial.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class="block p-5 bg-surface border border-default rounded-xl hover:border-accent hover:shadow-md transition-all duration-200 group"
        >
            <div class="flex items-start gap-3 mb-3">
                <span class="text-2xl" aria-hidden="true">"🎓"</span>
                <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2">
                        {tutorial.title}
                    </h3>
                    {if show_platform {
                        view! { <p class="text-sm text-tertiary mt-0.5">{platform_display}</p> }.into_any()
                    } else {
                        view! { }.into_any()
                    }}
                </div>
            </div>
            <p class="text-sm text-secondary line-clamp-3">{tutorial.blurb}</p>
        </a>
    }
}

/// Card component for a cheatsheet resource.
#[component]
fn CheatsheetCard(cheatsheet: Cheatsheet) -> impl IntoView {
    let format_badge = cheatsheet.format.clone().map(|f| f.to_uppercase());

    view! {
        <a
            href=cheatsheet.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class="block p-5 bg-surface border border-default rounded-xl hover:border-accent hover:shadow-md transition-all duration-200 group"
        >
            <div class="flex items-start gap-3 mb-3">
                <span class="text-2xl" aria-hidden="true">"📄"</span>
                <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2">
                        {cheatsheet.title}
                    </h3>
                    {if let Some(badge) = format_badge {
                        view! {
                            <span class="inline-block mt-1 px-2 py-0.5 text-xs font-medium bg-accent/10 text-accent rounded">
                                {badge}
                            </span>
                        }.into_any()
                    } else {
                        view! { }.into_any()
                    }}
                </div>
            </div>
            <p class="text-sm text-secondary line-clamp-3">{cheatsheet.blurb}</p>
        </a>
    }
}

/// Main Resources page component.
#[component]
pub fn ResourcesPage(resources: Resources) -> impl IntoView {
    view! {
        <main class="min-h-screen bg-surface">
            // Header
            <section class="relative overflow-hidden bg-subtle">
                <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-14 lg:py-20">
                    <h1 class="text-4xl md:text-5xl font-bold text-primary">"R Learning Resources"</h1>
                    <p class="mt-3 text-lg md:text-xl text-secondary max-w-3xl">
                        "Curated collection of books, videos, tutorials, and cheatsheets for learning R programming."
                    </p>

                    // Anchor navigation
                    <nav class="mt-8 flex flex-wrap gap-3">
                        <a href="#books" class="px-4 py-2 rounded-lg bg-surface border border-default text-primary hover:border-accent hover:text-accent transition-colors">
                            "Books"
                        </a>
                        <a href="#videos" class="px-4 py-2 rounded-lg bg-surface border border-default text-primary hover:border-accent hover:text-accent transition-colors">
                            "Videos"
                        </a>
                        <a href="#tutorials" class="px-4 py-2 rounded-lg bg-surface border border-default text-primary hover:border-accent hover:text-accent transition-colors">
                            "Tutorials"
                        </a>
                        <a href="#cheatsheets" class="px-4 py-2 rounded-lg bg-surface border border-default text-primary hover:border-accent hover:text-accent transition-colors">
                            "Cheatsheets"
                        </a>
                    </nav>
                </div>
            </section>

            // Books section
            <section id="books" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <h2 class="text-2xl font-bold text-primary mb-6">"Open Source R Books"</h2>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                    {resources.books.iter().map(|book| {
                        view! { <BookCard book=book.clone() /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // Videos section
            <section id="videos" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <h2 class="text-2xl font-bold text-primary mb-6">"Open Source R Videos"</h2>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                        {resources.videos.iter().map(|video| {
                            view! { <VideoCard video=video.clone() /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>

            // Tutorials section
            <section id="tutorials" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <h2 class="text-2xl font-bold text-primary mb-6">"Open Source R Tutorials"</h2>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {resources.tutorials.iter().map(|tutorial| {
                        view! { <TutorialCard tutorial=tutorial.clone() /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // Cheatsheets section
            <section id="cheatsheets" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <h2 class="text-2xl font-bold text-primary mb-6">"Open Source R Cheatsheets"</h2>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                        {resources.cheatsheets.iter().map(|cheatsheet| {
                            view! { <CheatsheetCard cheatsheet=cheatsheet.clone() /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>
        </main>
    }
}
