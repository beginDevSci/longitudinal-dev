//! R Learning Resources page component.
//!
//! Displays curated R learning materials: books, videos, tutorials, and cheatsheets.
//! Data is loaded from content/resources.yaml at build time.

use leptos::prelude::*;
use serde::Deserialize;

/// Book resource with cover image.
#[derive(Debug, Clone, Deserialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub url: String,
    pub blurb: String,
    #[serde(default)]
    pub image: Option<String>,
}

/// Video resource with embed URL.
#[derive(Debug, Clone, Deserialize)]
pub struct Video {
    pub title: String,
    pub source: String,
    pub url: String,
    #[serde(default)]
    pub embed_url: Option<String>,
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
    #[serde(default)]
    pub access: Option<String>,
}

/// Cheatsheet/quick reference resource.
#[derive(Debug, Clone, Deserialize)]
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

/// Card component for a book resource with cover image.
#[component]
fn BookCard(book: Book) -> impl IntoView {
    let has_image = book.image.is_some();
    let image_url = book.image.clone().unwrap_or_default();

    view! {
        <a
            href=book.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class="group block bg-surface border border-default rounded-2xl overflow-hidden hover:border-accent hover:shadow-xl transition-all duration-300 transform hover:-translate-y-1"
        >
            // Cover image
            {if has_image {
                view! {
                    <div class="aspect-[3/4] w-full overflow-hidden bg-subtle">
                        <img
                            src=image_url
                            alt=book.title.clone()
                            class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                            loading="lazy"
                        />
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="aspect-[3/4] w-full bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
                        <span class="text-6xl opacity-50">"📚"</span>
                    </div>
                }.into_any()
            }}

            // Content
            <div class="p-4">
                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2 mb-1">
                    {book.title}
                </h3>
                <p class="text-sm text-tertiary mb-2">{book.author}</p>
                <p class="text-sm text-secondary line-clamp-2">{book.blurb}</p>
            </div>
        </a>
    }
}

/// Card component for a video resource with embedded player.
#[component]
fn VideoCard(video: Video) -> impl IntoView {
    let has_embed = video.embed_url.is_some();
    let embed_url = video.embed_url.clone().unwrap_or_default();

    view! {
        <div class="group bg-surface border border-default rounded-2xl overflow-hidden hover:border-accent hover:shadow-xl transition-all duration-300">
            // Video embed or thumbnail
            {if has_embed {
                view! {
                    <div class="aspect-video w-full bg-black">
                        <iframe
                            src=embed_url
                            title=video.title.clone()
                            class="w-full h-full border-0"
                            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                            allowfullscreen=true
                        />
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="aspect-video w-full bg-gradient-to-br from-red-500/20 to-red-600/10 flex items-center justify-center">
                        <div class="w-16 h-16 rounded-full bg-red-500/80 flex items-center justify-center">
                            <svg class="w-8 h-8 text-white ml-1" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M8 5v14l11-7z"/>
                            </svg>
                        </div>
                    </div>
                }.into_any()
            }}

            // Content
            <div class="p-4">
                <a
                    href=video.url.clone()
                    target="_blank"
                    rel="noopener noreferrer"
                    class="block"
                >
                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2 mb-1">
                        {video.title}
                    </h3>
                    <p class="text-sm text-tertiary mb-2">{video.source}</p>
                    <p class="text-sm text-secondary line-clamp-2">{video.blurb}</p>
                </a>
            </div>
        </div>
    }
}

/// Card component for an interactive tutorial resource.
#[component]
fn TutorialCard(tutorial: Tutorial) -> impl IntoView {
    let platform_display = tutorial.platform.clone().unwrap_or_default();
    let show_platform = !platform_display.is_empty();
    let access_badge = tutorial.access.clone().map(|a| {
        match a.to_lowercase().as_str() {
            "freemium" => ("Freemium", "bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300"),
            _ => ("Open", "bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-300"),
        }
    });

    view! {
        <a
            href=tutorial.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class="group block p-6 bg-surface border border-default rounded-2xl hover:border-accent hover:shadow-xl transition-all duration-300 transform hover:-translate-y-1"
        >
            <div class="flex items-start gap-4 mb-4">
                <div class="w-14 h-14 rounded-xl bg-gradient-to-br from-blue-500/20 to-purple-500/20 flex items-center justify-center flex-shrink-0">
                    <span class="text-2xl">"🎓"</span>
                </div>
                <div class="flex-1 min-w-0">
                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2">
                        {tutorial.title}
                    </h3>
                    <div class="flex items-center gap-2 mt-1">
                        {if show_platform {
                            view! { <span class="text-sm text-tertiary">{platform_display}</span> }.into_any()
                        } else {
                            view! { }.into_any()
                        }}
                        {if let Some((label, classes)) = access_badge {
                            view! {
                                <span class=format!("inline-block px-2 py-0.5 text-xs font-medium rounded {}", classes)>
                                    {label}
                                </span>
                            }.into_any()
                        } else {
                            view! { }.into_any()
                        }}
                    </div>
                </div>
            </div>
            <p class="text-sm text-secondary line-clamp-3">{tutorial.blurb}</p>

            // Call to action
            <div class="mt-4 flex items-center text-accent text-sm font-medium">
                <span>"Open Tutorial"</span>
                <svg class="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                </svg>
            </div>
        </a>
    }
}

/// Get icon SVG for cheatsheet based on icon type.
fn get_cheatsheet_icon(icon: &str) -> &'static str {
    match icon {
        "chart" => r#"<path stroke-linecap="round" stroke-linejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 0 1 3 19.875v-6.75ZM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 0 1-1.125-1.125V8.625ZM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 0 1-1.125-1.125V4.125Z"/>"#,
        "wrench" => r#"<path stroke-linecap="round" stroke-linejoin="round" d="M21.75 6.75a4.5 4.5 0 0 1-4.884 4.484c-1.076-.091-2.264.071-2.95.904l-7.152 8.684a2.548 2.548 0 1 1-3.586-3.586l8.684-7.152c.833-.686.995-1.874.904-2.95a4.5 4.5 0 0 1 6.336-4.486l-3.276 3.276a3.004 3.004 0 0 0 2.25 2.25l3.276-3.276c.256.565.398 1.192.398 1.852Z"/>"#,
        "graph" => r#"<path stroke-linecap="round" stroke-linejoin="round" d="M7.5 14.25v2.25m3-4.5v4.5m3-6.75v6.75m3-9v9M6 20.25h12A2.25 2.25 0 0 0 20.25 18V6A2.25 2.25 0 0 0 18 3.75H6A2.25 2.25 0 0 0 3.75 6v12A2.25 2.25 0 0 0 6 20.25Z"/>"#,
        "document" => r#"<path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z"/>"#,
        "sparkles" => r#"<path stroke-linecap="round" stroke-linejoin="round" d="M9.813 15.904 9 18.75l-.813-2.846a4.5 4.5 0 0 0-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 0 0 3.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 0 0 3.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 0 0-3.09 3.09ZM18.259 8.715 18 9.75l-.259-1.035a3.375 3.375 0 0 0-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 0 0 2.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 0 0 2.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 0 0-2.456 2.456ZM16.894 20.567 16.5 21.75l-.394-1.183a2.25 2.25 0 0 0-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 0 0 1.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 0 0 1.423 1.423l1.183.394-1.183.394a2.25 2.25 0 0 0-1.423 1.423Z"/>"#,
        "text" => r#"<path stroke-linecap="round" stroke-linejoin="round" d="M7.5 8.25h9m-9 3H12m-9.75 1.51c0 1.6 1.123 2.994 2.707 3.227 1.129.166 2.27.293 3.423.379.35.026.67.21.865.501L12 21l2.755-4.133a1.14 1.14 0 0 1 .865-.501 48.172 48.172 0 0 0 3.423-.379c1.584-.233 2.707-1.626 2.707-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0 0 12 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018Z"/>"#,
        _ => r#"<path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z"/>"#,
    }
}

/// Card component for a cheatsheet resource with styled icon.
#[component]
fn CheatsheetCard(cheatsheet: Cheatsheet) -> impl IntoView {
    let format_badge = cheatsheet.format.clone().map(|f| f.to_uppercase());
    let icon_path = get_cheatsheet_icon(&cheatsheet.icon.clone().unwrap_or_default());

    view! {
        <a
            href=cheatsheet.url.clone()
            target="_blank"
            rel="noopener noreferrer"
            class="group block p-6 bg-surface border border-default rounded-2xl hover:border-accent hover:shadow-xl transition-all duration-300 transform hover:-translate-y-1"
        >
            // Icon container
            <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform duration-300">
                <svg class="w-8 h-8 text-accent" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24" inner_html=icon_path />
            </div>

            <h3 class="font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2 mb-2">
                {cheatsheet.title}
            </h3>
            <p class="text-sm text-secondary line-clamp-2 mb-4">{cheatsheet.blurb}</p>

            // Footer with badge and external link indicator
            <div class="flex items-center justify-between">
                {if let Some(badge) = format_badge {
                    view! {
                        <span class="inline-block px-3 py-1 text-xs font-semibold bg-accent/10 text-accent rounded-full">
                            {badge}
                        </span>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
                <svg class="w-5 h-5 text-tertiary group-hover:text-accent group-hover:translate-x-1 group-hover:-translate-y-1 transition-all" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
                </svg>
            </div>
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
                        "A curated, browseable collection of R learning resources. Open-source-first, with clearly labeled freemium resources commonly used in research workflows."
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
                <h2 class="text-2xl font-bold text-primary mb-2">"Books"</h2>
                <p class="text-secondary mb-6">"Includes both modern tidyverse-era resources and classic texts that remain influential."</p>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                    {resources.books.iter().map(|book| {
                        view! { <BookCard book=book.clone() /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // Videos section
            <section id="videos" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <h2 class="text-2xl font-bold text-primary mb-6">"Videos"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {resources.videos.iter().map(|video| {
                            view! { <VideoCard video=video.clone() /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>

            // Tutorials section
            <section id="tutorials" class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <h2 class="text-2xl font-bold text-primary mb-6">"Tutorials"</h2>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    {resources.tutorials.iter().map(|tutorial| {
                        view! { <TutorialCard tutorial=tutorial.clone() /> }
                    }).collect::<Vec<_>>()}
                </div>
            </section>

            // Cheatsheets section
            <section id="cheatsheets" class="bg-subtle">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                    <h2 class="text-2xl font-bold text-primary mb-6">"Cheatsheets"</h2>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
                        {resources.cheatsheets.iter().map(|cheatsheet| {
                            view! { <CheatsheetCard cheatsheet=cheatsheet.clone() /> }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </section>
        </main>
    }
}
