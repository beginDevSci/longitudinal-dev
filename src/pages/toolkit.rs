//! Toolkit hub page - enhanced landing page with featured content and category previews.

use leptos::prelude::*;

use super::resources::Resources;
use super::tools::Tools;
use crate::base_path;

/// Toolkit hub page - landing page linking to learning resources and software tools.
#[component]
pub fn ToolkitPage(resources: Resources, tools: Tools) -> impl IntoView {
    let learning_href = base_path::join("toolkit/learning/");
    let software_href = base_path::join("toolkit/software/");

    // Compute stats
    let book_count = resources.books.len();
    let video_count = resources.videos.len();
    let tutorial_count = resources.tutorials.len();
    let cheatsheet_count = resources.cheatsheets.len();
    let total_resources = book_count + video_count + tutorial_count + cheatsheet_count;

    let lang_count = tools.programming_languages.len();
    let ide_count = tools.ides.len();
    let vc_count = tools.version_control.len();
    let format_count = tools.data_formats.len();
    let notebook_count = tools.notebooks.len();
    let db_count = tools.databases.len();
    let total_tools = lang_count + ide_count + vc_count + format_count + notebook_count + db_count;

    // Featured items for Getting Started section
    let featured_book = resources.books.first().cloned();
    let featured_tutorial = resources.tutorials.first().cloned();
    let featured_cheatsheet = resources.cheatsheets.first().cloned();
    let featured_ide = tools.ides.iter().find(|t| t.title == "RStudio").cloned()
        .or_else(|| tools.ides.first().cloned());

    // Preview images for category cards
    let book_previews: Vec<_> = resources.books.iter()
        .filter_map(|b| b.image.clone())
        .take(3)
        .collect();

    let tool_logos: Vec<_> = tools.programming_languages.iter()
        .filter_map(|t| t.logo.clone())
        .take(3)
        .collect();

    let ide_logos: Vec<_> = tools.ides.iter()
        .filter_map(|t| t.logo.clone())
        .take(3)
        .collect();

    let notebook_logos: Vec<_> = tools.notebooks.iter()
        .filter_map(|t| t.logo.clone())
        .take(2)
        .collect();

    view! {
        <main class="min-h-screen bg-surface">
            // Hero section with stats
            <section class="relative overflow-hidden bg-subtle">
                <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10 lg:py-14">
                    <h1 class="text-4xl md:text-5xl font-bold text-primary">"Toolkit"</h1>
                    <p class="mt-3 text-lg md:text-xl text-secondary max-w-3xl">
                        "Curated resources and tools for longitudinal data science research."
                    </p>
                    // Stats badges
                    <div class="mt-6 flex flex-wrap gap-3">
                        <span class="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-accent/10 text-accent font-medium">
                            <span class="text-lg">"📚"</span>
                            <span>{total_resources}" Resources"</span>
                        </span>
                        <span class="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-accent/10 text-accent font-medium">
                            <span class="text-lg">"🔧"</span>
                            <span>{total_tools}" Tools"</span>
                        </span>
                    </div>
                </div>
            </section>

            // Getting Started - Featured Section
            <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
                <div class="rounded-2xl tutorial-gradient-accent p-6 lg:p-8">
                    <div class="flex items-center gap-2 mb-6">
                        <span class="text-2xl">"✨"</span>
                        <h2 class="text-2xl font-bold text-primary">"Getting Started"</h2>
                    </div>
                    <p class="text-secondary mb-6 max-w-2xl">
                        "New to R programming? Start with these essential resources to build a solid foundation."
                    </p>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                        // Featured Book - R for Data Science
                        {featured_book.map(|book| {
                            let image_url = book.image.clone().unwrap_or_default();
                            let has_image = !image_url.is_empty();
                            view! {
                                <a
                                    href=book.url.clone()
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="group block rounded-xl bg-elevated border border-stroke p-4 hover:border-accent/50 hover:shadow-lg transition-all"
                                >
                                    <div class="flex items-center gap-3 mb-3">
                                        <span class="px-2 py-1 rounded-full text-xs font-semibold bg-blue-500/10 text-blue-400 border border-blue-500/20">
                                            "Book"
                                        </span>
                                    </div>
                                    {if has_image {
                                        view! {
                                            <div class="aspect-[3/2] mb-3 rounded-lg overflow-hidden bg-subtle">
                                                <img
                                                    src=image_url
                                                    alt=book.title.clone()
                                                    class="w-full h-full object-contain"
                                                    loading="lazy"
                                                />
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="aspect-[3/2] mb-3 rounded-lg bg-gradient-to-br from-blue-500/20 to-blue-500/5 flex items-center justify-center">
                                                <span class="text-4xl opacity-50">"📖"</span>
                                            </div>
                                        }.into_any()
                                    }}
                                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2">
                                        {book.title}
                                    </h3>
                                    <p class="text-sm text-tertiary mt-1">{book.author}</p>
                                </a>
                            }
                        })}

                        // Featured IDE - RStudio
                        {featured_ide.map(|ide| {
                            let logo_url = ide.logo.clone().unwrap_or_default();
                            let has_logo = !logo_url.is_empty();
                            view! {
                                <a
                                    href=ide.url.clone()
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="group block rounded-xl bg-elevated border border-stroke p-4 hover:border-accent/50 hover:shadow-lg transition-all"
                                >
                                    <div class="flex items-center gap-3 mb-3">
                                        <span class="px-2 py-1 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                                            "IDE"
                                        </span>
                                    </div>
                                    <div class="aspect-[3/2] mb-3 rounded-lg bg-gradient-to-br from-neutral-100 to-neutral-50 dark:from-neutral-800 dark:to-neutral-900 flex items-center justify-center p-4">
                                        {if has_logo {
                                            view! {
                                                <img
                                                    src=logo_url
                                                    alt=ide.title.clone()
                                                    class="max-h-16 max-w-full object-contain"
                                                    loading="lazy"
                                                />
                                            }.into_any()
                                        } else {
                                            view! {
                                                <span class="text-4xl opacity-50">"💻"</span>
                                            }.into_any()
                                        }}
                                    </div>
                                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">
                                        {ide.title}
                                    </h3>
                                    <p class="text-sm text-secondary mt-1 line-clamp-2">{ide.blurb}</p>
                                </a>
                            }
                        })}

                        // Featured Cheatsheet - Base R
                        {featured_cheatsheet.map(|cs| {
                            view! {
                                <a
                                    href=cs.url.clone()
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="group block rounded-xl bg-elevated border border-stroke p-4 hover:border-accent/50 hover:shadow-lg transition-all"
                                >
                                    <div class="flex items-center gap-3 mb-3">
                                        <span class="px-2 py-1 rounded-full text-xs font-semibold bg-purple-500/10 text-purple-400 border border-purple-500/20">
                                            "Cheatsheet"
                                        </span>
                                    </div>
                                    <div class="aspect-[3/2] mb-3 rounded-lg bg-gradient-to-br from-purple-500/20 to-purple-500/5 flex items-center justify-center">
                                        <span class="text-4xl">"📄"</span>
                                    </div>
                                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">
                                        {cs.title}
                                    </h3>
                                    <p class="text-sm text-secondary mt-1 line-clamp-2">{cs.blurb}</p>
                                </a>
                            }
                        })}

                        // Featured Tutorial
                        {featured_tutorial.map(|tutorial| {
                            let platform = tutorial.platform.clone().unwrap_or_default();
                            view! {
                                <a
                                    href=tutorial.url.clone()
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="group block rounded-xl bg-elevated border border-stroke p-4 hover:border-accent/50 hover:shadow-lg transition-all"
                                >
                                    <div class="flex items-center gap-3 mb-3">
                                        <span class="px-2 py-1 rounded-full text-xs font-semibold bg-amber-500/10 text-amber-400 border border-amber-500/20">
                                            "Tutorial"
                                        </span>
                                    </div>
                                    <div class="aspect-[3/2] mb-3 rounded-lg bg-gradient-to-br from-amber-500/20 to-amber-500/5 flex items-center justify-center">
                                        <span class="text-4xl">"🎓"</span>
                                    </div>
                                    <h3 class="font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2">
                                        {tutorial.title}
                                    </h3>
                                    {(!platform.is_empty()).then(|| view! {
                                        <p class="text-sm text-tertiary mt-1">{platform}</p>
                                    })}
                                </a>
                            }
                        })}
                    </div>
                </div>
            </section>

            // Learning Resources Section
            <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                <div class="flex items-center justify-between mb-6">
                    <div>
                        <h2 class="text-2xl font-bold text-primary">"Learning Resources"</h2>
                        <p class="text-secondary mt-1">"Books, videos, tutorials, and quick references for R programming"</p>
                    </div>
                    <a
                        href=learning_href.clone()
                        class="hidden sm:flex items-center gap-2 text-accent font-medium hover:underline"
                    >
                        <span>"View all"</span>
                        <span>"→"</span>
                    </a>
                </div>
                <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                    // Books category
                    <a
                        href=learning_href.clone()
                        class="group block p-5 rounded-xl bg-subtle border border-default hover:border-accent/50 transition-all hover:shadow-md"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <span class="text-2xl">"📚"</span>
                            <div>
                                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">"Books"</h3>
                                <p class="text-sm text-tertiary">{book_count}" titles"</p>
                            </div>
                        </div>
                        // Book cover previews
                        <div class="flex gap-1 h-16 overflow-hidden rounded-lg">
                            {book_previews.into_iter().map(|img| {
                                view! {
                                    <img
                                        src=img
                                        alt="Book cover"
                                        class="h-full w-auto object-contain rounded"
                                        loading="lazy"
                                    />
                                }
                            }).collect_view()}
                        </div>
                    </a>

                    // Videos category
                    <a
                        href=learning_href.clone()
                        class="group block p-5 rounded-xl bg-subtle border border-default hover:border-accent/50 transition-all hover:shadow-md"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <span class="text-2xl">"🎬"</span>
                            <div>
                                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">"Videos"</h3>
                                <p class="text-sm text-tertiary">{video_count}" courses"</p>
                            </div>
                        </div>
                        <div class="h-16 rounded-lg bg-gradient-to-br from-red-500/10 to-red-500/5 flex items-center justify-center">
                            <span class="text-3xl opacity-60">"▶️"</span>
                        </div>
                    </a>

                    // Tutorials category
                    <a
                        href=learning_href.clone()
                        class="group block p-5 rounded-xl bg-subtle border border-default hover:border-accent/50 transition-all hover:shadow-md"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <span class="text-2xl">"💻"</span>
                            <div>
                                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">"Tutorials"</h3>
                                <p class="text-sm text-tertiary">{tutorial_count}" interactive"</p>
                            </div>
                        </div>
                        <div class="h-16 rounded-lg bg-gradient-to-br from-amber-500/10 to-amber-500/5 flex items-center justify-center">
                            <span class="text-3xl opacity-60">"🎓"</span>
                        </div>
                    </a>

                    // Cheatsheets category
                    <a
                        href=learning_href.clone()
                        class="group block p-5 rounded-xl bg-subtle border border-default hover:border-accent/50 transition-all hover:shadow-md"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <span class="text-2xl">"📄"</span>
                            <div>
                                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">"Cheatsheets"</h3>
                                <p class="text-sm text-tertiary">{cheatsheet_count}" references"</p>
                            </div>
                        </div>
                        <div class="h-16 rounded-lg bg-gradient-to-br from-purple-500/10 to-purple-500/5 flex items-center justify-center">
                            <span class="text-3xl opacity-60">"📋"</span>
                        </div>
                    </a>
                </div>
                // Mobile "View all" link
                <a
                    href=learning_href
                    class="sm:hidden mt-4 flex items-center justify-center gap-2 text-accent font-medium hover:underline"
                >
                    <span>"View all resources"</span>
                    <span>"→"</span>
                </a>
            </section>

            // Software & Tools Section
            <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 pb-16">
                <div class="flex items-center justify-between mb-6">
                    <div>
                        <h2 class="text-2xl font-bold text-primary">"Software & Tools"</h2>
                        <p class="text-secondary mt-1">"Languages, IDEs, version control, and more for data science"</p>
                    </div>
                    <a
                        href=software_href.clone()
                        class="hidden sm:flex items-center gap-2 text-accent font-medium hover:underline"
                    >
                        <span>"View all"</span>
                        <span>"→"</span>
                    </a>
                </div>
                <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    // Languages category
                    <a
                        href=software_href.clone()
                        class="group block p-5 rounded-xl bg-subtle border border-default hover:border-accent/50 transition-all hover:shadow-md"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <span class="text-2xl">"🐍"</span>
                            <div>
                                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">"Languages"</h3>
                                <p class="text-sm text-tertiary">{lang_count}" languages"</p>
                            </div>
                        </div>
                        <div class="flex gap-3 h-12 items-center justify-center">
                            {tool_logos.into_iter().map(|logo| {
                                view! {
                                    <img
                                        src=logo
                                        alt="Language logo"
                                        class="h-10 w-auto object-contain"
                                        loading="lazy"
                                    />
                                }
                            }).collect_view()}
                        </div>
                    </a>

                    // IDEs category
                    <a
                        href=software_href.clone()
                        class="group block p-5 rounded-xl bg-subtle border border-default hover:border-accent/50 transition-all hover:shadow-md"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <span class="text-2xl">"⌨️"</span>
                            <div>
                                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">"IDEs & Editors"</h3>
                                <p class="text-sm text-tertiary">{ide_count}" tools"</p>
                            </div>
                        </div>
                        <div class="flex gap-3 h-12 items-center justify-center">
                            {ide_logos.into_iter().map(|logo| {
                                view! {
                                    <img
                                        src=logo
                                        alt="IDE logo"
                                        class="h-10 w-auto object-contain"
                                        loading="lazy"
                                    />
                                }
                            }).collect_view()}
                        </div>
                    </a>

                    // Version Control category
                    <a
                        href=software_href.clone()
                        class="group block p-5 rounded-xl bg-subtle border border-default hover:border-accent/50 transition-all hover:shadow-md"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <span class="text-2xl">"🔀"</span>
                            <div>
                                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">"Version Control"</h3>
                                <p class="text-sm text-tertiary">{vc_count}" tools"</p>
                            </div>
                        </div>
                        <div class="h-12 rounded-lg bg-gradient-to-br from-orange-500/10 to-orange-500/5 flex items-center justify-center">
                            <span class="text-2xl opacity-60">"Git • GitHub • GitLab"</span>
                        </div>
                    </a>

                    // Data Formats category
                    <a
                        href=software_href.clone()
                        class="group block p-5 rounded-xl bg-subtle border border-default hover:border-accent/50 transition-all hover:shadow-md"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <span class="text-2xl">"📊"</span>
                            <div>
                                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">"Data Formats"</h3>
                                <p class="text-sm text-tertiary">{format_count}" formats"</p>
                            </div>
                        </div>
                        <div class="h-12 rounded-lg bg-gradient-to-br from-cyan-500/10 to-cyan-500/5 flex items-center justify-center">
                            <span class="text-sm text-secondary opacity-80">"CSV • JSON • Parquet • Arrow"</span>
                        </div>
                    </a>

                    // Notebooks category
                    <a
                        href=software_href.clone()
                        class="group block p-5 rounded-xl bg-subtle border border-default hover:border-accent/50 transition-all hover:shadow-md"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <span class="text-2xl">"📓"</span>
                            <div>
                                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">"Notebooks"</h3>
                                <p class="text-sm text-tertiary">{notebook_count}" platforms"</p>
                            </div>
                        </div>
                        <div class="flex gap-3 h-12 items-center justify-center">
                            {notebook_logos.into_iter().map(|logo| {
                                view! {
                                    <img
                                        src=logo
                                        alt="Notebook logo"
                                        class="h-10 w-auto object-contain"
                                        loading="lazy"
                                    />
                                }
                            }).collect_view()}
                        </div>
                    </a>

                    // Databases category
                    <a
                        href=software_href.clone()
                        class="group block p-5 rounded-xl bg-subtle border border-default hover:border-accent/50 transition-all hover:shadow-md"
                    >
                        <div class="flex items-center gap-3 mb-3">
                            <span class="text-2xl">"🗄️"</span>
                            <div>
                                <h3 class="font-semibold text-primary group-hover:text-accent transition-colors">"Databases"</h3>
                                <p class="text-sm text-tertiary">{db_count}" systems"</p>
                            </div>
                        </div>
                        <div class="h-12 rounded-lg bg-gradient-to-br from-indigo-500/10 to-indigo-500/5 flex items-center justify-center">
                            <span class="text-sm text-secondary opacity-80">"PostgreSQL • DuckDB • Redis"</span>
                        </div>
                    </a>
                </div>
                // Mobile "View all" link
                <a
                    href=software_href
                    class="sm:hidden mt-4 flex items-center justify-center gap-2 text-accent font-medium hover:underline"
                >
                    <span>"View all tools"</span>
                    <span>"→"</span>
                </a>
            </section>
        </main>
    }
}
